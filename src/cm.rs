use crate::ctype::{isalpha, isdigit, ispunct, isspace};
use crate::nodes::TableAlignment;
use crate::nodes::{
    AstNode, ListDelimType, ListType, NodeCodeBlock, NodeHeading, NodeHtmlBlock, NodeLink,
    NodeValue,
};
#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;
use crate::parser::ComrakOptions;
use crate::scanners;
use crate::strings::trim_start_match;
use crate::{nodes, ComrakPlugins};

use std::cmp::max;
use std::io::{self, Write};

/// Formats an AST as CommonMark, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut dyn Write,
) -> io::Result<()> {
    format_document_with_plugins(root, options, output, &ComrakPlugins::default())
}

/// Formats an AST as CommonMark, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut dyn Write,
    _plugins: &ComrakPlugins,
) -> io::Result<()> {
    let mut f = CommonMarkFormatter::new(root, options);
    f.format(root);
    if !f.v.is_empty() && f.v[f.v.len() - 1] != b'\n' {
        f.v.push(b'\n');
    }
    output.write_all(&f.v)?;
    Ok(())
}

struct CommonMarkFormatter<'a, 'o> {
    node: &'a AstNode<'a>,
    options: &'o ComrakOptions,
    v: Vec<u8>,
    prefix: Vec<u8>,
    column: usize,
    need_cr: u8,
    last_breakable: usize,
    begin_line: bool,
    begin_content: bool,
    no_linebreaks: bool,
    in_tight_list_item: bool,
    custom_escape: Option<fn(&'a AstNode<'a>, u8) -> bool>,
    footnote_ix: u32,
}

#[derive(PartialEq, Clone, Copy)]
enum Escaping {
    Literal,
    Normal,
    Url,
    Title,
}

impl<'a, 'o> Write for CommonMarkFormatter<'a, 'o> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output(buf, false, Escaping::Literal);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a, 'o> CommonMarkFormatter<'a, 'o> {
    fn new(node: &'a AstNode<'a>, options: &'o ComrakOptions) -> Self {
        CommonMarkFormatter {
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

            if self.custom_escape.is_some() && self.custom_escape.unwrap()(self.node, buf[i]) {
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

    fn format(&mut self, node: &'a AstNode<'a>) {
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
                        for ch in node.reverse_children() {
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

    fn get_in_tight_list_item(&self, node: &'a AstNode<'a>) -> bool {
        let tmp = match nodes::containing_block(node) {
            Some(tmp) => tmp,
            None => return false,
        };

        match tmp.data.borrow().value {
            NodeValue::Item(..) | NodeValue::TaskItem(..) => {
                if let NodeValue::List(ref nl) = tmp.parent().unwrap().data.borrow().value {
                    return nl.tight;
                }
                return false;
            }
            _ => {}
        }

        let parent = match tmp.parent() {
            Some(parent) => parent,
            None => return false,
        };

        match parent.data.borrow().value {
            NodeValue::Item(..) | NodeValue::TaskItem(..) => {
                if let NodeValue::List(ref nl) = parent.parent().unwrap().data.borrow().value {
                    return nl.tight;
                }
            }
            _ => {}
        }

        false
    }

    fn format_node(&mut self, node: &'a AstNode<'a>, entering: bool) -> bool {
        self.node = node;
        let allow_wrap = self.options.render.width > 0 && !self.options.render.hardbreaks;

        if !(matches!(
            node.data.borrow().value,
            NodeValue::Item(..) | NodeValue::TaskItem(..)
        ) && node.previous_sibling().is_none()
            && entering)
        {
            self.in_tight_list_item = self.get_in_tight_list_item(node);
        }

        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::FrontMatter(ref fm) => self.format_front_matter(fm.as_bytes(), entering),
            NodeValue::BlockQuote => self.format_block_quote(entering),
            NodeValue::List(..) => self.format_list(node, entering),
            NodeValue::Item(..) => self.format_item(node, entering),
            NodeValue::DescriptionList => (),
            NodeValue::DescriptionItem(..) => (),
            NodeValue::DescriptionTerm => (),
            NodeValue::DescriptionDetails => self.format_description_details(entering),
            NodeValue::Heading(ref nch) => self.format_heading(nch, entering),
            NodeValue::CodeBlock(ref ncb) => self.format_code_block(node, ncb, entering),
            NodeValue::HtmlBlock(ref nhb) => self.format_html_block(nhb, entering),
            NodeValue::ThematicBreak => self.format_thematic_break(entering),
            NodeValue::Paragraph => self.format_paragraph(entering),
            NodeValue::Text(ref literal) => {
                self.format_text(literal.as_bytes(), allow_wrap, entering)
            }
            NodeValue::LineBreak => self.format_line_break(entering),
            NodeValue::SoftBreak => self.format_soft_break(allow_wrap, entering),
            NodeValue::Code(ref code) => {
                self.format_code(code.literal.as_bytes(), allow_wrap, entering)
            }
            NodeValue::HtmlInline(ref literal) => {
                self.format_html_inline(literal.as_bytes(), entering)
            }
            NodeValue::Strong => self.format_strong(),
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
            NodeValue::FootnoteDefinition(_) => self.format_footnote_definition(entering),
            NodeValue::FootnoteReference(ref r) => {
                self.format_footnote_reference(r.as_bytes(), entering)
            }
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

    fn format_list(&mut self, node: &'a AstNode<'a>, entering: bool) {
        if !entering
            && match node.next_sibling() {
                Some(next_sibling) => matches!(
                    next_sibling.data.borrow().value,
                    NodeValue::CodeBlock(..) | NodeValue::List(..)
                ),
                _ => false,
            }
        {
            self.cr();
            write!(self, "<!-- end list -->").unwrap();
            self.blankline();
        }
    }

    fn format_item(&mut self, node: &'a AstNode<'a>, entering: bool) {
        let parent = match node.parent().unwrap().data.borrow().value {
            NodeValue::List(ref nl) => *nl,
            _ => unreachable!(),
        };

        let mut listmarker = vec![];

        let marker_width = if parent.list_type == ListType::Bullet {
            2
        } else {
            let mut list_number = parent.start;
            let list_delim = parent.delimiter;
            let mut tmpch = node;
            while let Some(tmp) = tmpch.previous_sibling() {
                tmpch = tmp;
                list_number += 1;
            }
            write!(
                listmarker,
                "{}{}{}",
                list_number,
                if list_delim == ListDelimType::Paren {
                    ")"
                } else {
                    "."
                },
                if list_number < 10 { "  " } else { " " }
            )
            .unwrap();
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
            let new_len = self.prefix.len() - marker_width;
            self.prefix.truncate(new_len);
            self.cr();
        }
    }

    fn format_description_details(&mut self, entering: bool) {
        if entering {
            write!(self, ": ").unwrap()
        }
    }

    fn format_heading(&mut self, nch: &NodeHeading, entering: bool) {
        if entering {
            for _ in 0..nch.level {
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

    fn format_code_block(&mut self, node: &'a AstNode<'a>, ncb: &NodeCodeBlock, entering: bool) {
        if entering {
            let first_in_list_item = node.previous_sibling().is_none()
                && match node.parent() {
                    Some(parent) => {
                        matches!(
                            parent.data.borrow().value,
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

            if info.is_empty()
                && (literal.len() > 2
                    && !isspace(literal[0])
                    && !(isspace(literal[literal.len() - 1])
                        && isspace(literal[literal.len() - 2])))
                && !first_in_list_item
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

    fn format_line_break(&mut self, entering: bool) {
        if entering {
            if !self.options.render.hardbreaks {
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
            } else {
                self.output(&[b' '], allow_wrap, Escaping::Literal);
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

    fn format_strong(&mut self) {
        write!(self, "**").unwrap();
    }

    fn format_emph(&mut self, node: &'a AstNode<'a>) {
        let emph_delim = if match node.parent() {
            Some(parent) => matches!(parent.data.borrow().value, NodeValue::Emph),
            _ => false,
        } && node.next_sibling().is_none()
            && node.previous_sibling().is_none()
        {
            b'_'
        } else {
            b'*'
        };

        self.write_all(&[emph_delim]).unwrap();
    }

    fn format_task_item(&mut self, symbol: Option<char>, node: &'a AstNode<'a>, entering: bool) {
        self.format_item(node, entering);
        if entering {
            write!(self, "[{}] ", symbol.unwrap_or(' ')).unwrap();
        }
    }

    fn format_strikethrough(&mut self) {
        write!(self, "~").unwrap();
    }

    fn format_superscript(&mut self) {
        write!(self, "^").unwrap();
    }

    fn format_link(&mut self, node: &'a AstNode<'a>, nl: &NodeLink, entering: bool) -> bool {
        if is_autolink(node, nl) {
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

    fn format_image(&mut self, nl: &NodeLink, allow_wrap: bool, entering: bool) {
        if entering {
            write!(self, "![").unwrap();
        } else {
            write!(self, "](").unwrap();
            self.output(nl.url.as_bytes(), false, Escaping::Url);
            if !nl.title.is_empty() {
                self.output(&[b' ', b'"'], allow_wrap, Escaping::Literal);
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
            self.output(ne.shortcode().as_bytes(), false, Escaping::Literal);
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

    fn format_table_cell(&mut self, node: &'a AstNode<'a>, entering: bool) {
        if entering {
            write!(self, " ").unwrap();
        } else {
            write!(self, " |").unwrap();

            let row = &node.parent().unwrap().data.borrow().value;
            let in_header = match *row {
                NodeValue::TableRow(header) => header,
                _ => panic!(),
            };

            if in_header && node.next_sibling().is_none() {
                let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
                let alignments = match *table {
                    NodeValue::Table(ref alignments) => alignments,
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
    fn format_footnote_definition(&mut self, entering: bool) {
        if entering {
            self.footnote_ix += 1;
            let footnote_ix = self.footnote_ix;
            writeln!(self, "[^{}]:", footnote_ix).unwrap();
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

fn is_autolink<'a>(node: &'a AstNode<'a>, nl: &NodeLink) -> bool {
    if nl.url.is_empty() || scanners::scheme(nl.url.as_bytes()).is_none() {
        return false;
    }

    if !nl.title.is_empty() {
        return false;
    }

    let link_text = match node.first_child() {
        None => return false,
        Some(child) => match child.data.borrow().value {
            NodeValue::Text(ref t) => t.clone(),
            _ => return false,
        },
    };

    trim_start_match(&nl.url, "mailto:") == link_text
}

fn table_escape<'a>(node: &'a AstNode<'a>, c: u8) -> bool {
    match node.data.borrow().value {
        NodeValue::Table(..) | NodeValue::TableRow(..) | NodeValue::TableCell => false,
        _ => c == b'|',
    }
}
