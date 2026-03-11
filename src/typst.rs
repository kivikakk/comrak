use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write};

use crate::nodes::{
    AlertType, ListDelimType, ListType, Node, NodeAlert, NodeCode, NodeCodeBlock,
    NodeDescriptionItem, NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading, NodeHtmlBlock,
    NodeLink, NodeList, NodeMath, NodeTaskItem, NodeValue, NodeWikiLink, TableAlignment,
};
use crate::parser::options::{Options, Plugins};
use crate::{Anchorizer, node_matches};

/// Formats an AST as Typst, modified by the given options.
pub fn format_document(root: Node<'_>, options: &Options, output: &mut dyn Write) -> fmt::Result {
    #[cfg(debug_assertions)]
    root.validate().unwrap_or_else(|e| {
        panic!("The document to format is ill-formed: {:?}", e);
    });

    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Formats an AST as Typst, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins(
    root: Node<'_>,
    options: &Options,
    output: &mut dyn Write,
    _plugins: &Plugins,
) -> fmt::Result {
    let mut formatter = TypstFormatter::new(root, options);
    let rendered = formatter.render_subtree(root);
    let mut rendered = formatter.expand_footnotes(rendered);

    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }

    output.write_str(&rendered)
}

#[derive(Clone, Copy)]
struct FootnoteEntry<'a> {
    label: usize,
    node: Node<'a>,
}

struct TypstFormatter<'a, 'o, 'c> {
    options: &'o Options<'c>,
    heading_labels: HashMap<usize, String>,
    footnotes: HashMap<String, FootnoteEntry<'a>>,
    footnotes_by_label: HashMap<usize, FootnoteEntry<'a>>,
    emitted_footnotes: HashSet<usize>,
}

impl<'a, 'o, 'c> TypstFormatter<'a, 'o, 'c> {
    fn new(root: Node<'a>, options: &'o Options<'c>) -> Self {
        let mut footnotes = HashMap::new();
        let mut footnotes_by_label = HashMap::new();
        let mut next_footnote = 1usize;

        for node in root.descendants() {
            let NodeValue::FootnoteDefinition(NodeFootnoteDefinition { ref name, .. }) =
                node.data().value
            else {
                continue;
            };

            footnotes.entry(name.clone()).or_insert_with(|| {
                let entry = FootnoteEntry {
                    label: next_footnote,
                    node,
                };
                footnotes_by_label.insert(next_footnote, entry);
                next_footnote += 1;
                entry
            });
        }

        let heading_labels = compute_heading_labels(root, options, &footnotes);

        TypstFormatter {
            options,
            heading_labels,
            footnotes,
            footnotes_by_label,
            emitted_footnotes: HashSet::new(),
        }
    }

    fn render_subtree(&mut self, root: Node<'a>) -> String {
        enum Phase {
            Pre,
            Post,
        }

        let mut rendered = HashMap::new();
        let mut stack = vec![(root, Phase::Pre)];

        while let Some((node, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    stack.push((node, Phase::Post));
                    if !matches!(node.data().value, NodeValue::FootnoteDefinition(_)) {
                        for child in node.reverse_children() {
                            stack.push((child, Phase::Pre));
                        }
                    }
                }
                Phase::Post => {
                    let child_outputs = take_child_outputs(node, &mut rendered);
                    let output = self.render_node(node, child_outputs);
                    rendered.insert(node_key(node), output);
                }
            }
        }

        rendered.remove(&node_key(root)).unwrap_or_default()
    }

    fn expand_footnotes(&mut self, input: String) -> String {
        enum ExpansionKind {
            Root,
            Footnote { label: usize },
        }

        struct ExpansionFrame {
            chars: Vec<char>,
            pos: usize,
            out: String,
            kind: ExpansionKind,
        }

        impl ExpansionFrame {
            fn new(text: String, kind: ExpansionKind) -> Self {
                Self {
                    chars: text.chars().collect(),
                    pos: 0,
                    out: String::new(),
                    kind,
                }
            }
        }

        let mut stack = vec![ExpansionFrame::new(input, ExpansionKind::Root)];

        while let Some(frame) = stack.last_mut() {
            if frame.pos >= frame.chars.len() {
                let completed = stack.pop().unwrap();
                let rendered = match completed.kind {
                    ExpansionKind::Root => completed.out,
                    ExpansionKind::Footnote { label } => {
                        format!(
                            "#footnote{} <footnote-{label}>",
                            content_block(&completed.out, 0)
                        )
                    }
                };

                if let Some(parent) = stack.last_mut() {
                    parent.out.push_str(&rendered);
                    continue;
                }

                return rendered;
            }

            match frame.chars[frame.pos] {
                FOOTNOTE_PLACEHOLDER_START => {
                    let mut label_end = frame.pos + 1;
                    while matches!(frame.chars.get(label_end), Some(ch) if ch.is_ascii_digit()) {
                        label_end += 1;
                    }

                    if label_end == frame.pos + 1
                        || frame.chars.get(label_end) != Some(&FOOTNOTE_PLACEHOLDER_END)
                    {
                        frame.out.push(FOOTNOTE_PLACEHOLDER_START);
                        frame.pos += 1;
                        continue;
                    }

                    let label = frame.chars[frame.pos + 1..label_end]
                        .iter()
                        .collect::<String>()
                        .parse()
                        .unwrap();
                    frame.pos = label_end + 1;

                    if !self.emitted_footnotes.insert(label) {
                        frame
                            .out
                            .push_str(&format!("#footnote(<footnote-{label}>)"));
                        continue;
                    }

                    let Some(entry) = self.footnotes_by_label.get(&label).copied() else {
                        frame.out.push_str(&format!("[^{}]", label));
                        continue;
                    };

                    let body = self.render_footnote_body(entry.node);
                    stack.push(ExpansionFrame::new(body, ExpansionKind::Footnote { label }));
                }
                ch => {
                    frame.out.push(ch);
                    frame.pos += 1;
                }
            }
        }

        String::new()
    }

    fn render_node(&mut self, node: Node<'a>, child_outputs: Vec<String>) -> String {
        match node.data().value {
            NodeValue::Document => join_block_children(&child_outputs),
            NodeValue::FrontMatter(_) => String::new(),
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                self.render_quote(join_block_children(&child_outputs))
            }
            NodeValue::List(ref list) => self.render_list(node, *list, child_outputs),
            NodeValue::Item(_) | NodeValue::TaskItem(_) => render_list_item(&child_outputs),
            NodeValue::DescriptionList => render_description_list(&child_outputs),
            NodeValue::DescriptionItem(ref meta) => {
                self.render_description_item(node, *meta, child_outputs)
            }
            NodeValue::DescriptionTerm | NodeValue::DescriptionDetails => {
                join_block_children(&child_outputs)
            }
            NodeValue::CodeBlock(ref code) => self.render_code_block(code),
            NodeValue::HtmlBlock(ref block) => self.render_raw_block(&block.literal, None),
            #[cfg(feature = "phoenix_heex")]
            NodeValue::HeexBlock(ref block) => self.render_raw_block(&block.literal, None),
            NodeValue::Paragraph
            | NodeValue::TableCell
            | NodeValue::SpoileredText
            | NodeValue::Escaped => child_outputs.concat(),
            NodeValue::Heading(ref heading) => {
                self.render_heading(node, *heading, child_outputs.concat())
            }
            NodeValue::ThematicBreak => "#line(length: 100%)".to_string(),
            NodeValue::FootnoteDefinition(_) => String::new(),
            NodeValue::Table(ref table) => self.render_table(table, child_outputs),
            NodeValue::TableRow(is_header) => self.render_table_row(node, is_header, child_outputs),
            NodeValue::Alert(ref alert) => {
                self.render_alert(alert, join_block_children(&child_outputs))
            }
            NodeValue::Subtext => render_inline_wrapper("sub", &child_outputs.concat()),
            NodeValue::Text(ref text) => escape_text(text),
            NodeValue::SoftBreak => {
                if self.options.render.hardbreaks {
                    "\\\n".to_string()
                } else {
                    " ".to_string()
                }
            }
            NodeValue::LineBreak => "\\\n".to_string(),
            NodeValue::Code(NodeCode { ref literal, .. }) => raw_inline(literal),
            NodeValue::HtmlInline(ref literal) => render_html_inline(literal),
            NodeValue::Raw(ref literal) => literal.clone(),
            #[cfg(feature = "phoenix_heex")]
            NodeValue::HeexInline(ref literal) => raw_inline(literal),
            NodeValue::Emph => format!("_{}_", child_outputs.concat()),
            NodeValue::Strong => format!("*{}*", child_outputs.concat()),
            NodeValue::Strikethrough => render_inline_wrapper("strike", &child_outputs.concat()),
            NodeValue::Highlight => render_inline_wrapper("highlight", &child_outputs.concat()),
            NodeValue::Superscript => render_inline_wrapper("super", &child_outputs.concat()),
            NodeValue::Underline => render_inline_wrapper("underline", &child_outputs.concat()),
            NodeValue::Subscript => render_inline_wrapper("sub", &child_outputs.concat()),
            NodeValue::Insert => render_inline_wrapper("underline", &child_outputs.concat()),
            NodeValue::Link(ref link) => self.render_link(node, link, child_outputs.concat()),
            NodeValue::Image(ref link) => self.render_image(node, link),
            NodeValue::FootnoteReference(ref reference) => {
                self.render_footnote_reference(reference)
            }
            #[cfg(feature = "shortcodes")]
            NodeValue::ShortCode(ref shortcode) => shortcode.emoji.clone(),
            NodeValue::Math(ref math) => self.render_math(math),
            NodeValue::WikiLink(ref link) => self.render_wikilink(link, child_outputs.concat()),
            NodeValue::EscapedTag(tag) => escape_text(tag),
        }
    }

    fn render_heading(&mut self, node: Node<'a>, heading: NodeHeading, mut body: String) -> String {
        if let Some(label) = self.render_heading_label(node) {
            if !body.is_empty() && !body.ends_with(char::is_whitespace) {
                body.push(' ');
            }
            body.push_str(&label);
        }

        format!("{} {}", "=".repeat(heading.level as usize), body.trim_end())
    }

    fn render_quote(&mut self, body: String) -> String {
        format!("#quote(block: true){}", content_block(&body, 0))
    }

    fn render_alert(&mut self, alert: &NodeAlert, body: String) -> String {
        let title = alert
            .title
            .as_deref()
            .unwrap_or(alert_title(alert.alert_type));
        let mut out = String::from("#quote(\n");
        out.push_str("  block: true,\n");
        out.push_str("  attribution: ");
        out.push_str(&content_block(&escape_text(title), 2));
        out.push_str(",\n");
        out.push(')');
        out.push_str(&content_block(&body, 0));
        out
    }

    fn render_list(&mut self, node: Node<'a>, list: NodeList, item_outputs: Vec<String>) -> String {
        if list.is_task_list {
            return self.render_task_list(node, list, item_outputs);
        }

        let mut out = String::new();
        let func = match list.list_type {
            ListType::Bullet => "list",
            ListType::Ordered => "enum",
        };

        out.push('#');
        out.push_str(func);
        out.push_str("(\n");

        if !list.tight {
            out.push_str("  tight: false,\n");
        }

        if matches!(list.list_type, ListType::Ordered) && list.start != 1 {
            out.push_str(&format!("  start: {},\n", list.start));
        }

        if matches!(list.list_type, ListType::Ordered)
            && matches!(list.delimiter, ListDelimType::Paren)
        {
            out.push_str("  numbering: \"1)\",\n");
        }

        for rendered in item_outputs {
            let trimmed = rendered.trim_matches('\n');
            if trimmed.is_empty() {
                continue;
            }

            out.push_str("  ");
            out.push_str(&content_block(trimmed, 2));
            out.push_str(",\n");
        }

        out.push(')');
        out
    }

    fn render_task_list(
        &mut self,
        node: Node<'a>,
        list: NodeList,
        item_outputs: Vec<String>,
    ) -> String {
        let mut ordinal = list.start;
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();

        for (child, rendered) in node.children().zip(item_outputs) {
            let trimmed = rendered.trim_matches('\n');
            let marker = list_item_marker(child, list, ordinal);

            if matches!(list.list_type, ListType::Ordered) {
                ordinal += 1;
            }

            if trimmed.is_empty() {
                continue;
            }

            if let Some((last_marker, items)) = groups.last_mut() {
                if *last_marker == marker {
                    items.push(trimmed.to_string());
                    continue;
                }
            }

            groups.push((marker, vec![trimmed.to_string()]));
        }

        groups
            .into_iter()
            .map(|(marker, items)| render_custom_marker_list(&marker, list.tight, &items))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_description_item(
        &mut self,
        node: Node<'a>,
        _meta: NodeDescriptionItem,
        child_outputs: Vec<String>,
    ) -> String {
        let mut term = String::new();
        let mut details = String::new();

        for (child, rendered) in node.children().zip(child_outputs) {
            match child.data().value {
                NodeValue::DescriptionTerm => term = rendered,
                NodeValue::DescriptionDetails => details = rendered,
                _ => {}
            }
        }

        format!(
            "terms.item({}, {})",
            content_block(&term, 4),
            content_block(&details, 4)
        )
    }

    fn render_footnote_body(&mut self, node: Node<'a>) -> String {
        let mut parts = Vec::new();

        for child in node.children() {
            let rendered = self.render_subtree(child);
            let trimmed = rendered.trim_matches('\n');
            if !trimmed.is_empty() {
                parts.push(trimmed.to_string());
            }
        }

        parts.join("\n\n")
    }

    fn render_code_block(&mut self, code: &NodeCodeBlock) -> String {
        let lang = code
            .info
            .split_whitespace()
            .next()
            .filter(|token| !token.is_empty());

        if matches!(lang, Some("typst" | "typ")) {
            return code.literal.trim_end_matches('\n').to_string();
        }

        self.render_raw_block(&code.literal, lang)
    }

    fn render_raw_block(&self, literal: &str, lang: Option<&str>) -> String {
        let mut out = format!("#raw(\"{}\", block: true", escape_string(literal));

        if let Some(lang) = lang {
            out.push_str(&format!(", lang: \"{}\"", escape_string(lang)));
        }

        out.push(')');
        out
    }

    fn render_table(
        &mut self,
        table: &crate::nodes::NodeTable,
        row_outputs: Vec<String>,
    ) -> String {
        let mut out = String::new();
        out.push_str("#table(\n");
        out.push_str(&format!("  columns: {},\n", table.num_columns));

        for row in row_outputs {
            let trimmed = row.trim_matches('\n');
            if trimmed.is_empty() {
                continue;
            }

            out.push_str(trimmed);
            out.push('\n');
        }

        out.push(')');
        out
    }

    fn render_table_row(
        &mut self,
        node: Node<'a>,
        is_header: bool,
        cell_outputs: Vec<String>,
    ) -> String {
        let mut out = String::new();

        if is_header {
            out.push_str("  table.header(\n");
            for (idx, content) in cell_outputs.into_iter().enumerate() {
                let align = node.parent().and_then(|parent| match parent.data().value {
                    NodeValue::Table(ref table) => table.alignments.get(idx).copied(),
                    _ => None,
                });
                out.push_str("    ");
                out.push_str(&render_table_cell(&content, align));
                out.push_str(",\n");
            }
            out.push_str("  ),");
            return out;
        }

        for (idx, content) in cell_outputs.into_iter().enumerate() {
            let align = node.parent().and_then(|parent| match parent.data().value {
                NodeValue::Table(ref table) => table.alignments.get(idx).copied(),
                _ => None,
            });
            out.push_str("  ");
            out.push_str(&render_table_cell(&content, align));
            out.push_str(",\n");
        }

        out.trim_end_matches('\n').to_string()
    }

    fn render_link(&mut self, node: Node<'a>, link: &NodeLink, body: String) -> String {
        if let Some(label) = render_explicit_typst_label(node, link) {
            return label;
        }

        if let Some(label) = typst_link_target(&link.url) {
            if body.is_empty() {
                return format!("#link(<{}>)", label);
            }

            return format!("#link(<{}>){}", label, content_block(&body, 0));
        }

        let url = escape_string(&link.url);

        if body.is_empty() {
            format!("#link(\"{}\")", url)
        } else {
            format!("#link(\"{}\"){}", url, content_block(&body, 0))
        }
    }

    fn render_image(&mut self, node: Node<'a>, link: &NodeLink) -> String {
        let alt = plain_text(node);
        let image = image_expr(&link.url, if alt.is_empty() { None } else { Some(&alt) });

        let standalone = node
            .parent()
            .is_some_and(|parent| is_typst_standalone_image_paragraph(parent));

        if standalone && self.options.render.figure_with_caption && !link.title.is_empty() {
            let mut out = String::new();
            out.push_str("#figure(\n");
            out.push_str("  ");
            out.push_str(&image);
            out.push_str(",\n");
            out.push_str("  caption: ");
            out.push_str(&content_block(&escape_text(&link.title), 2));
            out.push_str(",\n");
            out.push(')');
            return out;
        }

        if standalone {
            return format!("#{}", image);
        }

        format!("#box({})", image)
    }

    fn render_footnote_reference(&mut self, reference: &NodeFootnoteReference) -> String {
        let Some(entry) = self.footnotes.get(&reference.name).copied() else {
            return escape_text(&format!("[^{}]", reference.name));
        };

        footnote_placeholder(entry.label)
    }

    fn render_math(&self, math: &NodeMath) -> String {
        let literal = translate_math_literal(&math.literal);

        if math.display_math {
            format!("$\n{}\n$", literal.trim_matches('\n'))
        } else {
            format!("${}$", literal)
        }
    }

    fn render_wikilink(&mut self, link: &NodeWikiLink, label: String) -> String {
        let url = escape_string(&link.url);
        let label = if label.is_empty() {
            escape_text(&link.url)
        } else {
            label
        };
        format!("#link(\"{}\"){}", url, content_block(&label, 0))
    }

    fn render_heading_label(&self, node: Node<'a>) -> Option<String> {
        self.heading_labels.get(&node_key(node)).cloned()
    }
}

fn compute_heading_labels<'a>(
    root: Node<'a>,
    options: &Options<'_>,
    footnotes: &HashMap<String, FootnoteEntry<'a>>,
) -> HashMap<usize, String> {
    let Some(prefix) = options.extension.header_ids.as_ref() else {
        return HashMap::new();
    };

    enum Phase<'a> {
        Enter(Node<'a>),
        AssignHeading(Node<'a>),
    }

    let mut labels = HashMap::new();
    let mut anchorizer = Anchorizer::new();
    let mut emitted_footnotes = HashSet::new();
    let mut stack = vec![Phase::Enter(root)];

    while let Some(phase) = stack.pop() {
        match phase {
            Phase::Enter(node) => match node.data().value {
                NodeValue::FootnoteDefinition(_) => {}
                NodeValue::FootnoteReference(ref reference) => {
                    let Some(entry) = footnotes.get(&reference.name).copied() else {
                        continue;
                    };

                    if emitted_footnotes.insert(reference.name.clone()) {
                        for child in entry.node.reverse_children() {
                            stack.push(Phase::Enter(child));
                        }
                    }
                }
                NodeValue::Heading(_) => {
                    if !node.children().any(is_explicit_typst_label_node) {
                        stack.push(Phase::AssignHeading(node));
                    }

                    for child in node.reverse_children() {
                        stack.push(Phase::Enter(child));
                    }
                }
                _ => {
                    for child in node.reverse_children() {
                        stack.push(Phase::Enter(child));
                    }
                }
            },
            Phase::AssignHeading(node) => {
                let anchor = anchorizer.anchorize(&collect_text_iter(node));
                let label = format!("{prefix}{anchor}");

                if is_typst_label_name(&label) {
                    labels.insert(node_key(node), format!("<{}>", label));
                }
            }
        }
    }

    labels
}

fn alert_title(alert_type: AlertType) -> &'static str {
    match alert_type {
        AlertType::Note => "Note",
        AlertType::Tip => "Tip",
        AlertType::Important => "Important",
        AlertType::Warning => "Warning",
        AlertType::Caution => "Caution",
    }
}

fn list_item_marker(node: Node<'_>, list: NodeList, ordinal: usize) -> String {
    let checkbox = match node.data().value {
        NodeValue::TaskItem(task) => Some(task_marker(task)),
        _ => None,
    };

    match list.list_type {
        ListType::Bullet => checkbox
            .map(str::to_string)
            .unwrap_or_else(|| char::from(list.bullet_char).to_string()),
        ListType::Ordered => {
            let delimiter = match list.delimiter {
                ListDelimType::Period => '.',
                ListDelimType::Paren => ')',
            };

            match checkbox {
                Some(checkbox) => format!("{ordinal}{delimiter} {checkbox}"),
                None => format!("{ordinal}{delimiter}"),
            }
        }
    }
}

fn task_marker(task: NodeTaskItem) -> &'static str {
    if task.symbol.is_some() { "☒" } else { "☐" }
}

fn render_custom_marker_list(marker: &str, tight: bool, items: &[String]) -> String {
    let mut out = String::from("#list(\n");
    out.push_str("  marker: ");
    out.push_str(&content_block(&escape_text(marker), 2));
    out.push_str(",\n");
    if !tight {
        out.push_str("  tight: false,\n");
    }

    for item in items {
        out.push_str("  ");
        out.push_str(&content_block(item, 2));
        out.push_str(",\n");
    }

    out.push(')');
    out
}

fn render_inline_wrapper(function: &str, body: &str) -> String {
    format!("#{function}{}", content_block(body, 0))
}

fn render_list_item(blocks: &[String]) -> String {
    join_block_children(blocks)
}

fn render_description_list(items: &[String]) -> String {
    let mut out = String::from("#terms(\n");

    for rendered in items {
        let trimmed = rendered.trim_matches('\n');
        if trimmed.is_empty() {
            continue;
        }

        out.push_str("  ");
        out.push_str(trimmed);
        out.push_str(",\n");
    }

    out.push(')');
    out
}

fn render_table_cell(content: &str, align: Option<TableAlignment>) -> String {
    let block = content_block(content, 0);

    match align.unwrap_or(TableAlignment::None) {
        TableAlignment::None => block,
        TableAlignment::Left => format!("table.cell(align: left){}", block),
        TableAlignment::Center => format!("table.cell(align: center){}", block),
        TableAlignment::Right => format!("table.cell(align: right){}", block),
    }
}

fn is_typst_standalone_image_paragraph(node: Node<'_>) -> bool {
    if !node_matches!(node, NodeValue::Paragraph) {
        return false;
    }

    let mut saw_image = false;

    for child in node.children() {
        match &child.data().value {
            NodeValue::Image(_) if !saw_image => saw_image = true,
            NodeValue::Text(text) if text.trim().is_empty() => {}
            NodeValue::Link(link) if render_explicit_typst_label(child, link).is_some() => {}
            _ => return false,
        }
    }

    saw_image
}

fn is_explicit_typst_label_node(node: Node<'_>) -> bool {
    let NodeValue::Link(ref link) = node.data().value else {
        return false;
    };

    render_explicit_typst_label(node, link).is_some()
}

fn render_explicit_typst_label(node: Node<'_>, link: &NodeLink) -> Option<String> {
    if !link.title.is_empty() || !is_typst_autolink_label(&link.url) {
        return None;
    }

    (plain_text(node) == link.url).then(|| format!("<{}>", link.url))
}

fn typst_link_target(url: &str) -> Option<&str> {
    let label = url.strip_prefix('#')?;
    is_typst_label_name(label).then_some(label)
}

fn image_expr(url: &str, alt: Option<&str>) -> String {
    let mut out = format!("image(\"{}\"", escape_string(url));

    if let Some(alt) = alt {
        out.push_str(&format!(", alt: \"{}\"", escape_string(alt)));
    }

    out.push(')');
    out
}

fn plain_text(node: Node<'_>) -> String {
    let mut out = String::new();
    let mut stack = vec![node];

    while let Some(node) = stack.pop() {
        match node.data().value {
            NodeValue::Text(ref text) => out.push_str(text),
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
            NodeValue::Code(NodeCode { ref literal, .. })
            | NodeValue::Raw(ref literal)
            | NodeValue::FrontMatter(ref literal) => out.push_str(literal),
            NodeValue::HtmlInline(ref literal) => {
                if let Some(text) = plain_text_html_inline(literal) {
                    out.push_str(&text);
                } else {
                    out.push_str(literal);
                }
            }
            #[cfg(feature = "phoenix_heex")]
            NodeValue::HeexInline(ref literal) => out.push_str(literal),
            #[cfg(feature = "phoenix_heex")]
            NodeValue::HeexBlock(ref block) => out.push_str(&block.literal),
            NodeValue::CodeBlock(ref block) => out.push_str(&block.literal),
            NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. }) => out.push_str(literal),
            NodeValue::Math(NodeMath { ref literal, .. }) => out.push_str(literal),
            NodeValue::FootnoteReference(ref reference) => {
                out.push_str(&format!("[^{}]", reference.name));
            }
            NodeValue::EscapedTag(tag) => out.push_str(tag),
            #[cfg(feature = "shortcodes")]
            NodeValue::ShortCode(ref shortcode) => out.push_str(&shortcode.emoji),
            _ => {
                for child in node.reverse_children() {
                    stack.push(child);
                }
            }
        }
    }

    out
}

fn collect_text_iter(node: Node<'_>) -> String {
    let mut text = String::with_capacity(20);
    let mut stack = vec![node];

    while let Some(node) = stack.pop() {
        match node.data().value {
            NodeValue::Text(ref literal) => text.push_str(literal),
            NodeValue::Code(NodeCode { ref literal, .. }) => text.push_str(literal),
            NodeValue::LineBreak | NodeValue::SoftBreak => text.push(' '),
            NodeValue::Math(NodeMath { ref literal, .. }) => text.push_str(literal),
            _ => {
                for child in node.reverse_children() {
                    stack.push(child);
                }
            }
        }
    }

    text
}

fn node_key(node: Node<'_>) -> usize {
    let ptr: *const _ = node;
    ptr as usize
}

const FOOTNOTE_PLACEHOLDER_START: char = '\u{FDD0}';
const FOOTNOTE_PLACEHOLDER_END: char = '\u{FDD1}';

fn footnote_placeholder(label: usize) -> String {
    format!("{FOOTNOTE_PLACEHOLDER_START}{label}{FOOTNOTE_PLACEHOLDER_END}")
}

fn take_child_outputs(node: Node<'_>, rendered: &mut HashMap<usize, String>) -> Vec<String> {
    node.children()
        .map(|child| rendered.remove(&node_key(child)).unwrap_or_default())
        .collect()
}

fn join_block_children(children: &[String]) -> String {
    let mut parts = Vec::new();

    for rendered in children {
        let trimmed = rendered.trim_matches('\n');
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }

    parts.join("\n\n")
}

fn content_block(content: &str, indent: usize) -> String {
    let trimmed = content.trim_matches('\n');
    if trimmed.is_empty() {
        return "[]".to_string();
    }

    if !trimmed.contains('\n') {
        return format!("[{}]", trimmed);
    }

    let pad = " ".repeat(indent);
    format!("[\n{}\n{}]", indent_lines(trimmed, indent + 2), pad)
}

fn indent_lines(input: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    input
        .lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn raw_inline(literal: &str) -> String {
    format!("#raw(\"{}\")", escape_string(literal))
}

fn render_html_inline(literal: &str) -> String {
    html_inline_translation(literal)
        .map(str::to_string)
        .unwrap_or_else(|| raw_inline(literal))
}

fn plain_text_html_inline(literal: &str) -> Option<String> {
    match html_inline_translation(literal)? {
        "\\\n" => Some(" ".to_string()),
        _ => Some(String::new()),
    }
}

fn html_inline_translation(literal: &str) -> Option<&'static str> {
    let tag = literal.trim();

    if tag.eq_ignore_ascii_case("<br>")
        || tag.eq_ignore_ascii_case("<br/>")
        || tag.eq_ignore_ascii_case("<br />")
    {
        return Some("\\\n");
    }

    match tag {
        _ if tag.eq_ignore_ascii_case("<sub>") => Some("#sub["),
        _ if tag.eq_ignore_ascii_case("</sub>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<sup>") => Some("#super["),
        _ if tag.eq_ignore_ascii_case("</sup>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<u>") => Some("#underline["),
        _ if tag.eq_ignore_ascii_case("</u>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<mark>") => Some("#highlight["),
        _ if tag.eq_ignore_ascii_case("</mark>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<ins>") => Some("#underline["),
        _ if tag.eq_ignore_ascii_case("</ins>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<del>") || tag.eq_ignore_ascii_case("<s>") => {
            Some("#strike[")
        }
        _ if tag.eq_ignore_ascii_case("</del>") || tag.eq_ignore_ascii_case("</s>") => Some("]"),
        _ if tag.eq_ignore_ascii_case("<em>") || tag.eq_ignore_ascii_case("<i>") => Some("_"),
        _ if tag.eq_ignore_ascii_case("</em>") || tag.eq_ignore_ascii_case("</i>") => Some("_"),
        _ if tag.eq_ignore_ascii_case("<strong>") || tag.eq_ignore_ascii_case("<b>") => Some("*"),
        _ if tag.eq_ignore_ascii_case("</strong>") || tag.eq_ignore_ascii_case("</b>") => Some("*"),
        _ => None,
    }
}

fn is_typst_autolink_label(input: &str) -> bool {
    input.contains(':') && is_typst_label_name(input)
}

fn is_typst_label_name(input: &str) -> bool {
    let mut segments = input.split(':').peekable();

    while let Some(segment) = segments.next() {
        if !is_typst_label_segment(segment) {
            return false;
        }

        if segments.peek().is_none() {
            break;
        }
    }

    !input.is_empty()
}

fn is_typst_label_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if first.is_ascii_digit() || !(first.is_ascii_alphanumeric() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn translate_math_literal(input: &str) -> String {
    MathTranslator::new(input).translate()
}

struct MathTranslator {
    chars: Vec<char>,
    pos: usize,
}

impl MathTranslator {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn translate(mut self) -> String {
        self.translate_until(None)
    }

    fn translate_until(&mut self, end: Option<char>) -> String {
        let mut out = String::new();

        while let Some(ch) = self.peek() {
            if Some(ch) == end {
                self.pos += 1;
                break;
            }

            match ch {
                '\\' => {
                    self.pos += 1;
                    out.push_str(&self.translate_command());
                }
                '_' | '^' => {
                    let op = self.next().unwrap();
                    out.push(op);
                    out.push_str(&self.translate_attachment_target());
                }
                _ => out.push(self.next().unwrap()),
            }
        }

        out
    }

    fn translate_attachment_target(&mut self) -> String {
        if self.peek() == Some('{') {
            self.pos += 1;
            let inner = self.translate_until(Some('}'));
            format!("({inner})")
        } else if self.peek() == Some('\\') {
            self.pos += 1;
            self.translate_command()
        } else {
            self.next().map(|ch| ch.to_string()).unwrap_or_default()
        }
    }

    fn translate_command(&mut self) -> String {
        let command = self.read_command_name();

        match command.as_str() {
            "" => "\\".to_string(),
            "frac" => {
                let numerator = self.parse_required_math_group();
                let denominator = self.parse_required_math_group();
                format!("frac({numerator}, {denominator})")
            }
            "sqrt" => {
                let degree = self.parse_optional_group('[', ']');
                let radicand = self.parse_required_math_group();

                match degree {
                    Some(degree) => format!("root({degree}, {radicand})"),
                    None => format!("sqrt({radicand})"),
                }
            }
            "text" | "mathrm" | "textrm" => {
                let text = self.parse_required_text_group();
                format!("\"{}\"", escape_string(&text))
            }
            "operatorname" => {
                let text = self.parse_required_text_group();
                format!("op(\"{}\")", escape_string(&text))
            }
            "left" | "right" | "big" | "Big" | "bigl" | "bigr" | "Bigl" | "Bigr" | "middle"
            | "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => String::new(),
            "," | ";" | ":" | "!" | "quad" | "qquad" => " ".to_string(),
            "\\" => " \\\n".to_string(),
            "{" | "}" | "[" | "]" | "(" | ")" | "_" | "^" | "%" | "$" | "&" | "#" | "~" => command,
            "cdot" => "dot".to_string(),
            "times" => "times".to_string(),
            "to" | "rightarrow" => "->".to_string(),
            "leftarrow" => "<-".to_string(),
            "Rightarrow" => "=>".to_string(),
            "Leftarrow" => "<=".to_string(),
            "mapsto" => "|->".to_string(),
            "le" | "leq" | "leqslant" => "<=".to_string(),
            "ge" | "geq" | "geqslant" => ">=".to_string(),
            "ne" | "neq" => "!=".to_string(),
            "infty" | "infinity" => "oo".to_string(),
            "dots" | "ldots" | "cdots" => "...".to_string(),
            _ => command,
        }
    }

    fn parse_required_math_group(&mut self) -> String {
        self.skip_whitespace();

        if self.peek() == Some('{') {
            self.pos += 1;
            self.translate_until(Some('}'))
        } else if self.peek() == Some('\\') {
            self.pos += 1;
            self.translate_command()
        } else {
            self.next().map(|ch| ch.to_string()).unwrap_or_default()
        }
    }

    fn parse_required_text_group(&mut self) -> String {
        self.skip_whitespace();

        if self.peek() != Some('{') {
            return String::new();
        }

        self.pos += 1;
        self.collect_text_until('}')
    }

    fn parse_optional_group(&mut self, open: char, close: char) -> Option<String> {
        self.skip_whitespace();

        if self.peek() != Some(open) {
            return None;
        }

        self.pos += 1;
        Some(self.translate_until(Some(close)))
    }

    fn collect_text_until(&mut self, end: char) -> String {
        let mut out = String::new();

        while let Some(ch) = self.next() {
            match ch {
                '\\' => {
                    let escaped = self.read_command_name();
                    match escaped.as_str() {
                        "" => out.push('\\'),
                        "{" | "}" | "[" | "]" | "(" | ")" | "_" | "^" | "%" | "$" | "&" | "#"
                        | "~" => out.push_str(&escaped),
                        " " | "," | ";" | ":" | "!" | "quad" | "qquad" => out.push(' '),
                        _ => {
                            out.push('\\');
                            out.push_str(&escaped);
                        }
                    }
                }
                c if c == end => break,
                '{' => {
                    out.push('{');
                    out.push_str(&self.collect_text_until('}'));
                    out.push('}');
                }
                _ => out.push(ch),
            }
        }

        out
    }

    fn read_command_name(&mut self) -> String {
        let mut out = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                out.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        if out.is_empty() {
            if let Some(ch) = self.next() {
                out.push(ch);
            }
        }

        out
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }
}

fn escape_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }

    out
}

fn escape_text(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut at_line_start = true;
    let mut index = 0usize;

    while let Some(&ch) = chars.get(index) {
        if ch == '@' {
            if let Some((reference, next_index)) = consume_typst_reference(&chars, index) {
                out.push_str(&reference);
                at_line_start = reference.ends_with('\n');
                index = next_index;
                continue;
            }
        }

        if ch == '/' {
            match chars.get(index + 1).copied() {
                Some('/') | Some('*') => {
                    out.push('/');
                    out.push('\\');
                    out.push(chars[index + 1]);
                    at_line_start = false;
                    index += 2;
                    continue;
                }
                _ => {}
            }
        }

        if at_line_start && matches!(ch, '=' | '-' | '+' | '/') {
            out.push('\\');
        }

        match ch {
            '\\' | '#' | '[' | ']' | '$' | '`' | '@' | '<' | '>' | '{' | '}' | '*' | '_' | '~' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => {
                out.push('\n');
                at_line_start = true;
                index += 1;
                continue;
            }
            _ => out.push(ch),
        }

        at_line_start = false;
        index += 1;
    }

    out
}

fn consume_typst_reference(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'@') {
        return None;
    }

    if start > 0 && is_typst_reference_word_char(chars[start - 1]) {
        return None;
    }

    let mut index = start + 1;
    let &first = chars.get(index)?;

    if !is_typst_reference_word_char(first) {
        return None;
    }

    let mut token = String::from("@");
    token.push(first);
    index += 1;

    while let Some(&ch) = chars.get(index) {
        if is_typst_reference_word_char(ch) {
            token.push(ch);
            index += 1;
            continue;
        }

        if is_typst_reference_separator(ch)
            && chars
                .get(index + 1)
                .copied()
                .is_some_and(is_typst_reference_word_char)
        {
            token.push(ch);
            index += 1;
            continue;
        }

        break;
    }

    if chars.get(index) == Some(&'[') {
        if let Some((supplement, next_index)) = consume_balanced(chars, index, '[', ']') {
            token.push_str(&supplement);
            index = next_index;
        }
    }

    Some((token, index))
}

fn consume_balanced(
    chars: &[char],
    start: usize,
    open: char,
    close: char,
) -> Option<(String, usize)> {
    let mut depth = 0usize;
    let mut index = start;
    let mut out = String::new();

    while let Some(&ch) = chars.get(index) {
        out.push(ch);

        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((out, index + 1));
            }
        }

        index += 1;
    }

    None
}

fn is_typst_reference_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_typst_reference_separator(ch: char) -> bool {
    matches!(ch, '-' | ':' | '.' | '/')
}
