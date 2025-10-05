use indextree::Arena;
use std::cmp::max;
use std::fmt;
use std::io::{self, Write};
use std::str;

use crate::ctype::{isalpha, isdigit, ispunct, isspace};
use crate::nodes::{
    Ast, AstNode, ListDelimType, ListType, NodeAlert, NodeCodeBlock, NodeHeading, NodeHtmlBlock,
    NodeLink, NodeMath, NodeTable, NodeValue, NodeWikiLink,
};
use crate::nodes::{NodeList, TableAlignment};
#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;
use crate::parser::{Options, WikiLinksMode};
use crate::scanners;
use crate::strings::trim_start_match;
use crate::Plugins;

/// Formats an AST as CommonMark, modified by the given options.
pub fn format_document<'a>(
    arena: &'a Arena<Ast>,
    root: AstNode,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    // Formatting an ill-formed AST might lead to invalid output. However, we don't want to pay for
    // validation in normal workflow. As a middleground, we validate the AST in debug builds. See
    // https://github.com/kivikakk/comrak/issues/371.
    #[cfg(debug_assertions)]
    root.validate(arena).unwrap_or_else(|e| {
        panic!("The document to format is ill-formed: {:?}", e);
    });

    format_document_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Formats an AST as CommonMark, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    arena: &'a Arena<Ast>,
    root: AstNode,
    options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins,
) -> fmt::Result {
    let mut f = CommonMarkFormatter::new(arena, root, options);
    f.format(root);
    let mut result = f.v;
    if !result.is_empty() && result[result.len() - 1] != b'\n' {
        result.push(b'\n');
    }

    // TODO: redo all of comrak::cm internally with str instead of [u8], so we
    // don't need the String::from_utf8 here.

    let mut s = String::from_utf8(result).unwrap();
    if options.render.experimental_minimize_commonmark {
        minimize_commonmark(&mut s, options);
    }
    output.write_str(&s)
}

struct CommonMarkFormatter<'a, 'o, 'c> {
    arena: &'a Arena<Ast>,
    node: AstNode,
    options: &'o Options<'c>,
    v: Vec<u8>,
    prefix: Vec<u8>,
    column: usize,
    need_cr: u8,
    last_breakable: usize,
    begin_line: bool,
    begin_content: bool,
    no_linebreaks: bool,
    in_tight_list_item: bool,
    custom_escape: Option<for<'i> fn(&'i Arena<Ast>, AstNode, u8) -> bool>,
    footnote_ix: u32,
    ol_stack: Vec<usize>,
}

#[derive(PartialEq, Clone, Copy)]
enum Escaping {
    Literal,
    Normal,
    Url,
    Title,
}

impl<'a, 'o, 'c> Write for CommonMarkFormatter<'a, 'o, 'c> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output(buf, false, Escaping::Literal);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'o, 'c> CommonMarkFormatter<'a, 'o, 'c> {
    fn new(arena: &'a Arena<Ast>, node: AstNode, options: &'o Options<'c>) -> Self {
        CommonMarkFormatter {
            arena,
            node,
            options,
            v: vec![],
            prefix: vec![],
            column: 0,
            need_cr: 0,
            last_breakable: 0,
            begin_line: true,
            begin_content: true,
            no_linebreaks: false,
            in_tight_list_item: false,
            custom_escape: None,
            footnote_ix: 0,
            ol_stack: vec![],
        }
    }

    fn output(&mut self, buf: &[u8], wrap: bool, escaping: Escaping) {
        let wrap = wrap && !self.no_linebreaks;

        if self.in_tight_list_item && self.need_cr > 1 {
            self.need_cr = 1;
        }

        let mut k = self.v.len() as i32 - 1;
        while self.need_cr > 0 {
            if k < 0 || self.v[k as usize] == b'\n' {
                k -= 1;
            } else {
                self.v.push(b'\n');
                if self.need_cr > 1 {
                    self.v.extend(&self.prefix);
                }
            }
            self.column = 0;
            self.last_breakable = 0;
            self.begin_line = true;
            self.begin_content = true;
            self.need_cr -= 1;
        }

        let mut i = 0;
        while i < buf.len() {
            if self.begin_line {
                self.v.extend(&self.prefix);
                self.column = self.prefix.len();
            }

            if self.custom_escape.is_some()
                && self.custom_escape.unwrap()(self.arena, self.node, buf[i])
            {
                self.v.push(b'\\');
            }

            let nextc = buf.get(i + 1);
            if buf[i] == b' ' && wrap {
                if !self.begin_line {
                    let last_nonspace = self.v.len();
                    self.v.push(b' ');
                    self.column += 1;
                    self.begin_line = false;
                    self.begin_content = false;
                    while buf.get(i + 1) == Some(&(b' ')) {
                        i += 1;
                    }
                    if !buf.get(i + 1).map_or(false, |&c| isdigit(c)) {
                        self.last_breakable = last_nonspace;
                    }
                }
            } else if escaping == Escaping::Literal {
                if buf[i] == b'\n' {
                    self.v.push(b'\n');
                    self.column = 0;
                    self.begin_line = true;
                    self.begin_content = true;
                    self.last_breakable = 0;
                } else {
                    self.v.push(buf[i]);
                    self.column += 1;
                    self.begin_line = false;
                    self.begin_content = self.begin_content && isdigit(buf[i]);
                }
            } else {
                self.outc(buf[i], escaping, nextc);
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(buf[i]);
            }

            if self.options.render.width > 0
                && self.column > self.options.render.width
                && !self.begin_line
                && self.last_breakable > 0
            {
                let remainder = self.v[self.last_breakable + 1..].to_vec();
                self.v.truncate(self.last_breakable);
                self.v.push(b'\n');
                self.v.extend(&self.prefix);
                self.v.extend(&remainder);
                self.column = self.prefix.len() + remainder.len();
                self.last_breakable = 0;
                self.begin_line = false;
                self.begin_content = false;
            }

            i += 1;
        }
    }

    fn outc(&mut self, c: u8, escaping: Escaping, nextc: Option<&u8>) {
        let follows_digit = !self.v.is_empty() && isdigit(self.v[self.v.len() - 1]);

        let nextc = nextc.map_or(0, |&c| c);

        let needs_escaping = c < 0x80
            && escaping != Escaping::Literal
            && ((escaping == Escaping::Normal
                && (c < 0x20
                    || c == b'*'
                    || c == b'_'
                    || c == b'['
                    || c == b']'
                    || c == b'#'
                    || c == b'<'
                    || c == b'>'
                    || c == b'\\'
                    || c == b'`'
                    || c == b'!'
                    || (c == b'&' && isalpha(nextc))
                    || (c == b'!' && nextc == 0x5b)
                    || (self.begin_content
                        && (c == b'-' || c == b'+' || c == b'=')
                        && !follows_digit)
                    || (self.begin_content
                        && (c == b'.' || c == b')')
                        && follows_digit
                        && (nextc == 0 || isspace(nextc)))))
                || (escaping == Escaping::Url
                    && (c == b'`'
                        || c == b'<'
                        || c == b'>'
                        || isspace(c)
                        || c == b'\\'
                        || c == b')'
                        || c == b'('))
                || (escaping == Escaping::Title
                    && (c == b'`' || c == b'<' || c == b'>' || c == b'"' || c == b'\\')));

        if needs_escaping {
            if escaping == Escaping::Url && isspace(c) {
                write!(self.v, "%{:2X}", c).unwrap();
                self.column += 3;
            } else if ispunct(c) {
                write!(self.v, "\\{}", c as char).unwrap();
                self.column += 2;
            } else {
                let s = format!("&#{};", c);
                self.write_all(s.as_bytes()).unwrap();
                self.column += s.len();
            }
        } else {
            self.v.push(c);
            self.column += 1;
        }
    }

    fn cr(&mut self) {
        self.need_cr = max(self.need_cr, 1);
    }

    fn blankline(&mut self) {
        self.need_cr = max(self.need_cr, 2);
    }

    fn format(&mut self, node: AstNode) {
        enum Phase {
            Pre,
            Post,
        }
        let mut stack = vec![(node, Phase::Pre)];

        while let Some((node, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    if self.format_node(node, true) {
                        stack.push((node, Phase::Post));
                        for ch in node.children(self.arena).rev() {
                            stack.push((ch, Phase::Pre));
                        }
                    }
                }
                Phase::Post => {
                    self.format_node(node, false);
                }
            }
        }
    }

    fn get_in_tight_list_item(&self, node: AstNode) -> bool {
        let tmp = match node.containing_block(self.arena) {
            Some(tmp) => tmp,
            None => return false,
        };

        match tmp.get(self.arena).value {
            NodeValue::Item(..) | NodeValue::TaskItem(..) => {
                if let NodeValue::List(ref nl) =
                    tmp.parent(self.arena).unwrap().get(self.arena).value
                {
                    return nl.tight;
                }
                return false;
            }
            _ => {}
        }

        let parent = match tmp.parent(self.arena) {
            Some(parent) => parent,
            None => return false,
        };

        match parent.get(self.arena).value {
            NodeValue::Item(..) | NodeValue::TaskItem(..) => {
                if let NodeValue::List(ref nl) =
                    parent.parent(self.arena).unwrap().get(self.arena).value
                {
                    return nl.tight;
                }
            }
            _ => {}
        }

        false
    }

    fn format_node(&mut self, node: AstNode, entering: bool) -> bool {
        self.node = node;
        let allow_wrap = self.options.render.width > 0 && !self.options.render.hardbreaks;

        let parent_node = node.parent(self.arena);
        if entering {
            if parent_node.is_some()
                && matches!(
                    parent_node.unwrap().get(self.arena).value,
                    NodeValue::Item(..) | NodeValue::TaskItem(..)
                )
            {
                self.in_tight_list_item = self.get_in_tight_list_item(node);
            }
        } else if matches!(node.get(self.arena).value, NodeValue::List(..)) {
            self.in_tight_list_item = parent_node.is_some()
                && matches!(
                    parent_node.unwrap().get(self.arena).value,
                    NodeValue::Item(..) | NodeValue::TaskItem(..)
                )
                && self.get_in_tight_list_item(node);
        }
        let next_is_block = node
            .next_sibling(self.arena)
            .map_or(true, |next| next.get(self.arena).value.block());

        match node.get(self.arena).value {
            NodeValue::Document => (),
            NodeValue::FrontMatter(ref fm) => self.format_front_matter(fm.as_bytes(), entering),
            NodeValue::BlockQuote => self.format_block_quote(entering),
            NodeValue::List(..) => self.format_list(node, entering),
            NodeValue::Item(..) => self.format_item(node, entering),
            NodeValue::DescriptionList => (),
            NodeValue::DescriptionItem(..) => (),
            NodeValue::DescriptionTerm => (),
            NodeValue::DescriptionDetails => self.format_description_details(entering),
            NodeValue::Heading(ref nh) => self.format_heading(nh, entering),
            NodeValue::CodeBlock(ref ncb) => self.format_code_block(node, ncb, entering),
            NodeValue::HtmlBlock(ref nhb) => self.format_html_block(nhb, entering),
            NodeValue::ThematicBreak => self.format_thematic_break(entering),
            NodeValue::Paragraph => self.format_paragraph(entering),
            NodeValue::Text(ref literal) => {
                self.format_text(literal.as_bytes(), allow_wrap, entering)
            }
            NodeValue::LineBreak => self.format_line_break(entering, next_is_block),
            NodeValue::SoftBreak => self.format_soft_break(allow_wrap, entering),
            NodeValue::Code(ref code) => {
                self.format_code(code.literal.as_bytes(), allow_wrap, entering)
            }
            NodeValue::HtmlInline(ref literal) => {
                self.format_html_inline(literal.as_bytes(), entering)
            }
            NodeValue::Raw(ref literal) => self.format_raw(literal.as_bytes(), entering),
            NodeValue::Strong => {
                if parent_node.is_none()
                    || !matches!(
                        parent_node.unwrap().get(self.arena).value,
                        NodeValue::Strong
                    )
                {
                    self.format_strong();
                }
            }
            NodeValue::Emph => self.format_emph(node),
            NodeValue::TaskItem(symbol) => self.format_task_item(symbol, node, entering),
            NodeValue::Strikethrough => self.format_strikethrough(),
            NodeValue::Superscript => self.format_superscript(),
            NodeValue::Link(ref nl) => return self.format_link(node, nl, entering),
            NodeValue::Image(ref nl) => self.format_image(nl, allow_wrap, entering),
            #[cfg(feature = "shortcodes")]
            NodeValue::ShortCode(ref ne) => self.format_shortcode(ne, entering),
            NodeValue::Table(..) => self.format_table(entering),
            NodeValue::TableRow(..) => self.format_table_row(entering),
            NodeValue::TableCell => self.format_table_cell(node, entering),
            NodeValue::FootnoteDefinition(ref nfd) => {
                self.format_footnote_definition(&nfd.name, entering)
            }
            NodeValue::FootnoteReference(ref nfr) => {
                self.format_footnote_reference(nfr.name.as_bytes(), entering)
            }
            NodeValue::MultilineBlockQuote(..) => self.format_block_quote(entering),
            NodeValue::Escaped => {
                // noop - automatic escaping is already being done
            }
            NodeValue::Math(ref math) => self.format_math(math, allow_wrap, entering),
            NodeValue::WikiLink(ref nl) => return self.format_wikilink(nl, entering),
            NodeValue::Underline => self.format_underline(),
            NodeValue::Subscript => self.format_subscript(),
            NodeValue::SpoileredText => self.format_spoiler(),
            NodeValue::EscapedTag(ref net) => self.format_escaped_tag(net),
            NodeValue::Alert(ref alert) => self.format_alert(alert, entering),
        };
        true
    }

    fn format_front_matter(&mut self, front_matter: &[u8], entering: bool) {
        if entering {
            self.output(front_matter, false, Escaping::Literal);
        }
    }

    fn format_block_quote(&mut self, entering: bool) {
        if entering {
            write!(self, "> ").unwrap();
            self.begin_content = true;
            write!(self.prefix, "> ").unwrap();
        } else {
            let new_len = self.prefix.len() - 2;
            self.prefix.truncate(new_len);
            self.blankline();
        }
    }

    fn format_list(&mut self, node: AstNode, entering: bool) {
        let ol_start = match node.get(self.arena).value {
            NodeValue::List(NodeList {
                list_type: ListType::Ordered,
                start,
                ..
            }) => Some(start),
            _ => None,
        };

        if entering {
            if let Some(start) = ol_start {
                self.ol_stack.push(start);
            }
        } else {
            if ol_start.is_some() {
                self.ol_stack.pop();
            }

            if match node.next_sibling(self.arena) {
                Some(next_sibling) => matches!(
                    next_sibling.get(self.arena).value,
                    NodeValue::CodeBlock(..) | NodeValue::List(..)
                ),
                _ => false,
            } {
                self.cr();
                write!(self, "<!-- end list -->").unwrap();
                self.blankline();
            }
        }
    }

    fn format_item(&mut self, node: AstNode, entering: bool) {
        let parent = match node.parent(self.arena).unwrap().get(self.arena).value {
            NodeValue::List(ref nl) => *nl,
            _ => unreachable!(),
        };

        let mut listmarker = vec![];

        let marker_width = if parent.list_type == ListType::Bullet {
            2
        } else {
            let list_number = if let Some(last_stack) = self.ol_stack.last_mut() {
                let list_number = *last_stack;
                if entering {
                    *last_stack += 1;
                };
                list_number
            } else {
                match node.get(self.arena).value {
                    NodeValue::Item(ref ni) => ni.start,
                    NodeValue::TaskItem(_) => parent.start,
                    _ => unreachable!(),
                }
            };
            let list_delim = parent.delimiter;
            write!(
                listmarker,
                "{}{} ",
                list_number,
                if list_delim == ListDelimType::Paren {
                    ")"
                } else {
                    "."
                }
            )
            .unwrap();
            let mut current_len = listmarker.len();

            while current_len < self.options.render.ol_width {
                write!(listmarker, " ").unwrap();
                current_len += 1;
            }

            listmarker.len()
        };

        if entering {
            if parent.list_type == ListType::Bullet {
                let bullet = char::from(self.options.render.list_style as u8);
                write!(self, "{} ", bullet).unwrap();
            } else {
                self.write_all(&listmarker).unwrap();
            }
            self.begin_content = true;
            for _ in 0..marker_width {
                write!(self.prefix, " ").unwrap();
            }
        } else {
            let new_len = if self.prefix.len() > marker_width {
                self.prefix.len() - marker_width
            } else {
                0
            };
            self.prefix.truncate(new_len);
            self.cr();
        }
    }

    fn format_description_details(&mut self, entering: bool) {
        if entering {
            write!(self, ": ").unwrap()
        }
    }

    fn format_heading(&mut self, nh: &NodeHeading, entering: bool) {
        if entering {
            for _ in 0..nh.level {
                write!(self, "#").unwrap();
            }
            write!(self, " ").unwrap();
            self.begin_content = true;
            self.no_linebreaks = true;
        } else {
            self.no_linebreaks = false;
            self.blankline();
        }
    }

    fn format_code_block(&mut self, node: AstNode, ncb: &NodeCodeBlock, entering: bool) {
        if entering {
            let first_in_list_item = node.previous_sibling(self.arena).is_none()
                && match node.parent(self.arena) {
                    Some(parent) => {
                        matches!(
                            parent.get(self.arena).value,
                            NodeValue::Item(..) | NodeValue::TaskItem(..)
                        )
                    }
                    _ => false,
                };

            if !first_in_list_item {
                self.blankline();
            }

            let info = ncb.info.as_bytes();
            let literal = ncb.literal.as_bytes();

            #[allow(clippy::len_zero)]
            if !(info.len() > 0
                || literal.len() <= 2
                || isspace(literal[0])
                || first_in_list_item
                || self.options.render.prefer_fenced
                || isspace(literal[literal.len() - 1]) && isspace(literal[literal.len() - 2]))
            {
                write!(self, "    ").unwrap();
                write!(self.prefix, "    ").unwrap();
                self.write_all(literal).unwrap();
                let new_len = self.prefix.len() - 4;
                self.prefix.truncate(new_len);
            } else {
                let fence_char = if info.contains(&b'`') { b'~' } else { b'`' };
                let numticks = max(3, longest_char_sequence(literal, fence_char) + 1);
                for _ in 0..numticks {
                    write!(self, "{}", fence_char as char).unwrap();
                }
                if !info.is_empty() {
                    write!(self, " ").unwrap();
                    self.write_all(info).unwrap();
                }
                self.cr();
                self.write_all(literal).unwrap();
                self.cr();
                for _ in 0..numticks {
                    write!(self, "{}", fence_char as char).unwrap();
                }
            }
            self.blankline();
        }
    }

    fn format_html_block(&mut self, nhb: &NodeHtmlBlock, entering: bool) {
        if entering {
            self.blankline();
            self.write_all(nhb.literal.as_bytes()).unwrap();
            self.blankline();
        }
    }

    fn format_thematic_break(&mut self, entering: bool) {
        if entering {
            self.blankline();
            write!(self, "-----").unwrap();
            self.blankline();
        }
    }

    fn format_paragraph(&mut self, entering: bool) {
        if !entering {
            self.blankline();
        }
    }

    fn format_text(&mut self, literal: &[u8], allow_wrap: bool, entering: bool) {
        if entering {
            self.output(literal, allow_wrap, Escaping::Normal);
        }
    }

    fn format_line_break(&mut self, entering: bool, next_is_block: bool) {
        if entering {
            if !self.options.render.hardbreaks && !next_is_block {
                // If the next element is a block, a backslash means a
                // literal backslash instead of a line break. In this case
                // we can just skip the line break since it's meaningless
                // before a block.
                write!(self, "\\").unwrap();
            }
            self.cr();
        }
    }

    fn format_soft_break(&mut self, allow_wrap: bool, entering: bool) {
        if entering {
            if !self.no_linebreaks
                && self.options.render.width == 0
                && !self.options.render.hardbreaks
            {
                self.cr();
            } else if self.options.render.hardbreaks {
                self.output(b"\n", allow_wrap, Escaping::Literal);
            } else {
                self.output(b" ", allow_wrap, Escaping::Literal);
            }
        }
    }

    fn format_code(&mut self, literal: &[u8], allow_wrap: bool, entering: bool) {
        if entering {
            let numticks = shortest_unused_sequence(literal, b'`');
            for _ in 0..numticks {
                write!(self, "`").unwrap();
            }

            let all_space = literal
                .iter()
                .all(|&c| c == b' ' || c == b'\r' || c == b'\n');
            let has_edge_space = literal[0] == b' ' || literal[literal.len() - 1] == b' ';
            let has_edge_backtick = literal[0] == b'`' || literal[literal.len() - 1] == b'`';

            let pad = literal.is_empty() || has_edge_backtick || (!all_space && has_edge_space);
            if pad {
                write!(self, " ").unwrap();
            }
            self.output(literal, allow_wrap, Escaping::Literal);
            if pad {
                write!(self, " ").unwrap();
            }
            for _ in 0..numticks {
                write!(self, "`").unwrap();
            }
        }
    }

    fn format_html_inline(&mut self, literal: &[u8], entering: bool) {
        if entering {
            self.write_all(literal).unwrap();
        }
    }

    fn format_raw(&mut self, literal: &[u8], entering: bool) {
        if entering {
            self.write_all(literal).unwrap();
        }
    }

    fn format_strong(&mut self) {
        write!(self, "**").unwrap();
    }

    fn format_emph(&mut self, node: AstNode) {
        let emph_delim = if match node.parent(self.arena) {
            Some(parent) => matches!(parent.get(self.arena).value, NodeValue::Emph),
            _ => false,
        } && node.next_sibling(self.arena).is_none()
            && node.previous_sibling(self.arena).is_none()
        {
            b'_'
        } else {
            b'*'
        };

        self.write_all(&[emph_delim]).unwrap();
    }

    fn format_task_item(&mut self, symbol: Option<char>, node: AstNode, entering: bool) {
        self.format_item(node, entering);
        if entering {
            write!(self, "[{}] ", symbol.unwrap_or(' ')).unwrap();
        }
    }

    fn format_strikethrough(&mut self) {
        write!(self, "~~").unwrap();
    }

    fn format_superscript(&mut self) {
        write!(self, "^").unwrap();
    }

    fn format_underline(&mut self) {
        write!(self, "__").unwrap();
    }

    fn format_subscript(&mut self) {
        write!(self, "~").unwrap();
    }

    fn format_spoiler(&mut self) {
        write!(self, "||").unwrap();
    }

    fn format_escaped_tag(&mut self, net: &String) {
        self.output(net.as_bytes(), false, Escaping::Literal);
    }

    fn format_link(&mut self, node: AstNode, nl: &NodeLink, entering: bool) -> bool {
        if is_autolink(self.arena, node, nl) {
            if entering {
                write!(self, "<{}>", trim_start_match(&nl.url, "mailto:")).unwrap();
                return false;
            }
        } else if entering {
            write!(self, "[").unwrap();
        } else {
            write!(self, "](").unwrap();
            self.output(nl.url.as_bytes(), false, Escaping::Url);
            if !nl.title.is_empty() {
                write!(self, " \"").unwrap();
                self.output(nl.title.as_bytes(), false, Escaping::Title);
                write!(self, "\"").unwrap();
            }
            write!(self, ")").unwrap();
        }

        true
    }

    fn format_wikilink(&mut self, nl: &NodeWikiLink, entering: bool) -> bool {
        if entering {
            write!(self, "[[").unwrap();
            if self.options.extension.wikilinks() == Some(WikiLinksMode::UrlFirst) {
                self.output(nl.url.as_bytes(), false, Escaping::Url);
                write!(self, "|").unwrap();
            }
        } else {
            if self.options.extension.wikilinks() == Some(WikiLinksMode::TitleFirst) {
                write!(self, "|").unwrap();
                self.output(nl.url.as_bytes(), false, Escaping::Url);
            }
            write!(self, "]]").unwrap();
        }

        true
    }

    fn format_image(&mut self, nl: &NodeLink, allow_wrap: bool, entering: bool) {
        if entering {
            write!(self, "![").unwrap();
        } else {
            write!(self, "](").unwrap();
            self.output(nl.url.as_bytes(), false, Escaping::Url);
            if !nl.title.is_empty() {
                self.output(b" \"", allow_wrap, Escaping::Literal);
                self.output(nl.title.as_bytes(), false, Escaping::Title);
                write!(self, "\"").unwrap();
            }
            write!(self, ")").unwrap();
        }
    }

    #[cfg(feature = "shortcodes")]
    fn format_shortcode(&mut self, ne: &NodeShortCode, entering: bool) {
        if entering {
            write!(self, ":").unwrap();
            self.output(ne.code.as_bytes(), false, Escaping::Literal);
            write!(self, ":").unwrap();
        }
    }

    fn format_table(&mut self, entering: bool) {
        if entering {
            self.custom_escape = Some(table_escape);
        } else {
            self.custom_escape = None;
        }
        self.blankline();
    }

    fn format_table_row(&mut self, entering: bool) {
        if entering {
            self.cr();
            write!(self, "|").unwrap();
        }
    }

    fn format_table_cell(&mut self, node: AstNode, entering: bool) {
        if entering {
            write!(self, " ").unwrap();
        } else {
            write!(self, " |").unwrap();

            let row = &node.parent(self.arena).unwrap().get(self.arena).value;
            let in_header = match *row {
                NodeValue::TableRow(header) => header,
                _ => panic!(),
            };

            if in_header && node.next_sibling(self.arena).is_none() {
                let table = &node
                    .parent(self.arena)
                    .unwrap()
                    .parent(self.arena)
                    .unwrap()
                    .get(self.arena)
                    .value;
                let alignments = match *table {
                    NodeValue::Table(NodeTable { ref alignments, .. }) => alignments,
                    _ => panic!(),
                };

                self.cr();
                write!(self, "|").unwrap();
                for a in alignments {
                    write!(
                        self,
                        " {} |",
                        match *a {
                            TableAlignment::Left => ":--",
                            TableAlignment::Center => ":-:",
                            TableAlignment::Right => "--:",
                            TableAlignment::None => "---",
                        }
                    )
                    .unwrap();
                }
                self.cr();
            }
        }
    }
    fn format_footnote_definition(&mut self, name: &str, entering: bool) {
        if entering {
            self.footnote_ix += 1;
            writeln!(self, "[^{}]:", name).unwrap();
            write!(self.prefix, "    ").unwrap();
        } else {
            let new_len = self.prefix.len() - 4;
            self.prefix.truncate(new_len);
        }
    }

    fn format_footnote_reference(&mut self, r: &[u8], entering: bool) {
        if entering {
            self.write_all(b"[^").unwrap();
            self.write_all(r).unwrap();
            self.write_all(b"]").unwrap();
        }
    }

    fn format_math(&mut self, math: &NodeMath, allow_wrap: bool, entering: bool) {
        if entering {
            let literal = math.literal.as_bytes();
            let start_fence = if math.dollar_math {
                if math.display_math {
                    "$$"
                } else {
                    "$"
                }
            } else {
                "$`"
            };

            let end_fence = if start_fence == "$`" {
                "`$"
            } else {
                start_fence
            };

            self.output(start_fence.as_bytes(), false, Escaping::Literal);
            self.output(literal, allow_wrap, Escaping::Literal);
            self.output(end_fence.as_bytes(), false, Escaping::Literal);
        }
    }

    fn format_alert(&mut self, alert: &NodeAlert, entering: bool) {
        if entering {
            write!(
                self,
                "> [!{}]",
                alert.alert_type.default_title().to_uppercase()
            )
            .unwrap();
            if alert.title.is_some() {
                let title = alert.title.as_ref().unwrap();
                write!(self, " {}", title).unwrap();
            }
            writeln!(self).unwrap();
            write!(self, "> ").unwrap();
            self.begin_content = true;
            write!(self.prefix, "> ").unwrap();
        } else {
            let new_len = self.prefix.len() - 2;
            self.prefix.truncate(new_len);
            self.blankline();
        }
    }
}

fn longest_char_sequence(literal: &[u8], ch: u8) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for c in literal {
        if *c == ch {
            current += 1;
        } else {
            if current > longest {
                longest = current;
            }
            current = 0;
        }
    }
    if current > longest {
        longest = current;
    }
    longest
}

fn shortest_unused_sequence(literal: &[u8], f: u8) -> usize {
    let mut used = 1;
    let mut current = 0;
    for c in literal {
        if *c == f {
            current += 1;
        } else {
            if current > 0 {
                used |= 1 << current;
            }
            current = 0;
        }
    }

    if current > 0 {
        used |= 1 << current;
    }

    let mut i = 0;
    while used & 1 != 0 {
        used >>= 1;
        i += 1;
    }
    i
}

fn is_autolink<'a>(arena: &'a Arena<Ast>, node: AstNode, nl: &NodeLink) -> bool {
    if nl.url.is_empty() || scanners::scheme(nl.url.as_bytes()).is_none() {
        return false;
    }

    if !nl.title.is_empty() {
        return false;
    }

    let link_text = match node.first_child(arena) {
        None => return false,
        Some(child) => match child.get(arena).value {
            NodeValue::Text(ref t) => t.clone(),
            _ => return false,
        },
    };

    trim_start_match(&nl.url, "mailto:") == link_text
}

fn table_escape<'a>(arena: &'a Arena<Ast>, node: AstNode, c: u8) -> bool {
    match node.get(arena).value {
        NodeValue::Table(..) | NodeValue::TableRow(..) | NodeValue::TableCell => false,
        _ => c == b'|',
    }
}

fn minimize_commonmark(text: &mut String, original_options: &Options) {
    let mut options_without = original_options.clone();
    options_without.render.experimental_minimize_commonmark = false;

    let ixs: Vec<usize> = text
        .as_bytes()
        .iter()
        .enumerate()
        .filter_map(|(ix, &c)| if c == b'\\' { Some(ix) } else { None })
        .collect();
    let original = text.clone();

    let mut adjust = 0;
    for ix in ixs {
        text.remove(ix - adjust);

        let mut arena = Arena::new();
        let root = crate::parse_document(&mut arena, text, &options_without);

        let mut out = String::new();
        format_document(&arena, root, &options_without, &mut out).unwrap();

        if original == out {
            // Removed character is guaranteed to be 1 byte wide, since it's
            // always '\\'.
            adjust += 1;
        } else {
            text.insert(ix - adjust, '\\');
        }
    }
}

/// Escapes the input, rendering it suitable for inclusion in a CommonMark
/// document in a place where regular inline parsing is occurring. Note that
/// this is not minimal --- there will be more escaping backslashes in the
/// output than is strictly necessary. The rendering will not be affected,
/// however.
pub fn escape_inline(text: &str) -> String {
    use std::fmt::Write;

    let mut result = String::with_capacity(text.len() * 3 / 2);

    for c in text.chars() {
        if c < '\x20'
            || c == '*'
            || c == '_'
            || c == '['
            || c == ']'
            || c == '#'
            || c == '<'
            || c == '>'
            || c == '\\'
            || c == '`'
            || c == '!'
            || c == '&'
            || c == '!'
            || c == '-'
            || c == '+'
            || c == '='
            || c == '.'
            || c == '('
            || c == ')'
            || c == '"'
        {
            if ispunct(c as u8) {
                write!(&mut result, "\\{}", c).unwrap();
            } else {
                write!(&mut result, "&#{};", c as u8).unwrap();
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Escapes the input URL, rendering it suitable for inclusion as a [link
/// destination] per the CommonMark spec.
///
/// [link destination]: https://spec.commonmark.org/0.31.2/#link-destination
pub fn escape_link_destination(url: &str) -> String {
    let mut result = String::with_capacity(url.len() * 3 / 2);

    result.push('<');
    for c in url.chars() {
        match c {
            '<' | '>' => {
                result.push('\\');
                result.push(c);
            }
            '\n' => result.push_str("%0A"),
            '\r' => result.push_str("%0D"),
            _ => result.push(c),
        }
    }
    result.push('>');

    result
}
