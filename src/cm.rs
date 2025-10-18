use std::cmp::max;
use std::fmt::{self, Write};
use std::str;
pub use typed_arena::Arena;

use crate::ctype::{isalpha, isdigit, ispunct, ispunct_char, isspace, isspace_char};
use crate::nodes::{
    AstNode, ListDelimType, ListType, NodeAlert, NodeCodeBlock, NodeHeading, NodeHtmlBlock,
    NodeLink, NodeList, NodeMath, NodeTable, NodeValue, NodeWikiLink, TableAlignment,
};
#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;
use crate::parser::{Options, WikiLinksMode};
use crate::scanners;
use crate::strings::trim_start_match;
use crate::{node_matches, Plugins};

/// Formats an AST as CommonMark, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
) -> fmt::Result {
    // Formatting an ill-formed AST might lead to invalid output. However, we don't want to pay for
    // validation in normal workflow. As a middleground, we validate the AST in debug builds. See
    // https://github.com/kivikakk/comrak/issues/371.
    #[cfg(debug_assertions)]
    root.validate().unwrap_or_else(|e| {
        panic!("The document to format is ill-formed: {:?}", e);
    });

    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Formats an AST as CommonMark, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
    _plugins: &Plugins,
) -> fmt::Result {
    let mut f = CommonMarkFormatter::new(root, options);
    f.format(root)?;

    // TODO:
    // If we're not using experimental_minimize_commonmark, and
    // options.render.width == 0, we can theoretically output directly to
    // output.
    // Ideally, we separate the width splitting behaviour into a layer between
    // CommonMarkFormatter and the output, which flushes lines as possible.
    // Then we have CommonMarkFormatter output directly to Write, which we
    // also output to a String when using experimental_minimize_commonmark.

    let mut result = f.output;
    if !result.is_empty() && result.as_bytes()[result.len() - 1] != b'\n' {
        result.push('\n');
    }

    if options.render.experimental_minimize_commonmark {
        minimize_commonmark(&mut result, options);
    }

    output.write_str(&result)
}

struct CommonMarkFormatter<'a, 'o, 'c> {
    node: &'a AstNode<'a>,
    options: &'o Options<'c>,
    output: String,
    prefix: String,
    column: usize,
    need_cr: u8,
    last_breakable: usize,
    begin_line: bool,
    begin_content: bool,
    no_linebreaks: bool,
    in_tight_list_item: bool,
    custom_escape: Option<fn(&'a AstNode<'a>, char) -> bool>,
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
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.output(s, false, Escaping::Literal)
    }
}

impl<'a, 'o, 'c> CommonMarkFormatter<'a, 'o, 'c> {
    fn new(node: &'a AstNode<'a>, options: &'o Options<'c>) -> Self {
        CommonMarkFormatter {
            node,
            options,
            output: String::new(),
            prefix: String::new(),
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

    fn output(&mut self, s: &str, wrap: bool, escaping: Escaping) -> fmt::Result {
        let bytes = s.as_bytes();
        let wrap = wrap && !self.no_linebreaks;

        if self.in_tight_list_item && self.need_cr > 1 {
            self.need_cr = 1;
        }

        let mut k = self.output.len() as i32 - 1;
        while self.need_cr > 0 {
            if k < 0 || self.output.as_bytes()[k as usize] == b'\n' {
                k -= 1;
            } else {
                self.output.push('\n');
                if self.need_cr > 1 {
                    self.output.push_str(&self.prefix);
                }
            }
            self.column = 0;
            self.last_breakable = 0;
            self.begin_line = true;
            self.begin_content = true;
            self.need_cr -= 1;
        }

        let mut it = s.char_indices();

        while let Some((mut i, c)) = it.next() {
            // for (i, c) in s.char_indices() {
            if self.begin_line {
                self.output.push_str(&self.prefix);
                self.column = self.prefix.len();
            }

            if self.custom_escape.map_or(false, |f| f(self.node, c)) {
                self.output.push('\\');
            }

            let nextb = bytes.get(i + 1);
            if c == ' ' && wrap {
                if !self.begin_line {
                    let last_nonspace = self.output.len();
                    self.output.push(' ');
                    self.column += 1;
                    self.begin_line = false;
                    self.begin_content = false;
                    while bytes.get(i + 1) == Some(&(b' ')) {
                        (i, _) = it.next().unwrap();
                    }
                    if !bytes.get(i + 1).map_or(false, |&c| isdigit(c)) {
                        self.last_breakable = last_nonspace;
                    }
                }
            } else if escaping == Escaping::Literal {
                if bytes[i] == b'\n' {
                    self.output.push('\n');
                    self.column = 0;
                    self.begin_line = true;
                    self.begin_content = true;
                    self.last_breakable = 0;
                } else {
                    let len_before = self.output.len();
                    self.output.push(c);
                    self.column += self.output.len() - len_before;
                    self.begin_line = false;
                    self.begin_content = self.begin_content && isdigit(bytes[i]);
                }
            } else {
                self.outc(c, escaping, nextb)?;
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(bytes[i]);
            }

            if self.options.render.width > 0
                && self.column > self.options.render.width
                && !self.begin_line
                && self.last_breakable > 0
            {
                let remainder = self.output[self.last_breakable + 1..].to_string();
                self.output.truncate(self.last_breakable);
                self.output.push('\n');
                self.output.push_str(&self.prefix);
                self.output.push_str(&remainder);
                self.column = self.prefix.len() + remainder.len();
                self.last_breakable = 0;
                self.begin_line = false;
                self.begin_content = false;
            }
        }
        Ok(())
    }

    fn outc(&mut self, c: char, escaping: Escaping, nextb: Option<&u8>) -> fmt::Result {
        // NOTE: nextb contains the byte *immediately following the first byte of c*.
        // If c is a multibyte character, nextb contains the second byte of c!
        // We rely on it here *only* where we know c to be a single byte
        // character, in which case it faithfully represents the first byte of the
        // following character (or is None if the string ends).
        // Any use of nextb must be conditional on asserting c is a single byte character.
        let follows_digit =
            !self.output.is_empty() && isdigit(self.output.as_bytes()[self.output.len() - 1]);

        let nextb = nextb.map_or(0, |&c| c);

        let needs_escaping = (c as u64) < 0x80
            && escaping != Escaping::Literal
            && ((escaping == Escaping::Normal
                && ((c as u64) < 0x20
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
                    || (c == '&' && isalpha(nextb))
                    || (c == '!' && nextb == 0x5b)
                    || (self.begin_content
                        && (c == '-' || c == '+' || c == '=')
                        && !follows_digit)
                    || (self.begin_content
                        && (c == '.' || c == ')')
                        && follows_digit
                        && (nextb == 0 || isspace(nextb)))))
                || (escaping == Escaping::Url
                    && (c == '`'
                        || c == '<'
                        || c == '>'
                        || isspace_char(c)
                        || c == '\\'
                        || c == ')'
                        || c == '('))
                || (escaping == Escaping::Title
                    && (c == '`' || c == '<' || c == '>' || c == '"' || c == '\\')));

        if needs_escaping {
            // (c as u64) < 256 is implied.
            let len_before = self.output.len();
            if escaping == Escaping::Url && isspace_char(c) {
                write!(self.output, "%{:2X}", c as u8)?;
            } else if ispunct_char(c) {
                write!(self.output, "\\{}", c)?;
            } else {
                write!(self, "&#{};", c as u8)?;
            }
            self.column += self.output.len() - len_before;
        } else {
            let len_before = self.output.len();
            self.output.push(c);
            self.column += self.output.len() - len_before;
        }

        Ok(())
    }

    fn cr(&mut self) {
        self.need_cr = max(self.need_cr, 1);
    }

    fn blankline(&mut self) {
        self.need_cr = max(self.need_cr, 2);
    }

    fn format(&mut self, node: &'a AstNode<'a>) -> fmt::Result {
        enum Phase {
            Pre,
            Post,
        }
        let mut stack = vec![(node, Phase::Pre)];

        while let Some((node, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    if self.format_node(node, true)? {
                        stack.push((node, Phase::Post));
                        for ch in node.reverse_children() {
                            stack.push((ch, Phase::Pre));
                        }
                    }
                }
                Phase::Post => {
                    self.format_node(node, false)?;
                }
            }
        }
        Ok(())
    }

    fn get_in_tight_list_item(&self, node: &'a AstNode<'a>) -> bool {
        let tmp = match node.containing_block() {
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

    fn format_node(&mut self, node: &'a AstNode<'a>, entering: bool) -> Result<bool, fmt::Error> {
        self.node = node;
        let allow_wrap = self.options.render.width > 0 && !self.options.render.hardbreaks;

        let parent_node = node.parent();
        if entering {
            if parent_node.is_some()
                && matches!(
                    parent_node.unwrap().data.borrow().value,
                    NodeValue::Item(..) | NodeValue::TaskItem(..)
                )
            {
                self.in_tight_list_item = self.get_in_tight_list_item(node);
            }
        } else if matches!(node.data.borrow().value, NodeValue::List(..)) {
            self.in_tight_list_item = parent_node.is_some()
                && matches!(
                    parent_node.unwrap().data.borrow().value,
                    NodeValue::Item(..) | NodeValue::TaskItem(..)
                )
                && self.get_in_tight_list_item(node);
        }
        let next_is_block = node
            .next_sibling()
            .map_or(true, |next| next.data.borrow().value.block());

        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::FrontMatter(ref fm) => self.format_front_matter(fm, entering)?,
            NodeValue::BlockQuote => self.format_block_quote(entering)?,
            NodeValue::List(..) => self.format_list(node, entering)?,
            NodeValue::Item(..) => self.format_item(node, entering)?,
            NodeValue::DescriptionList => (),
            NodeValue::DescriptionItem(..) => (),
            NodeValue::DescriptionTerm => (),
            NodeValue::DescriptionDetails => self.format_description_details(entering)?,
            NodeValue::Heading(ref nh) => self.format_heading(nh, entering)?,
            NodeValue::CodeBlock(ref ncb) => self.format_code_block(node, ncb, entering)?,
            NodeValue::HtmlBlock(ref nhb) => self.format_html_block(nhb, entering)?,
            NodeValue::ThematicBreak => self.format_thematic_break(entering)?,
            NodeValue::Paragraph => self.format_paragraph(entering),
            NodeValue::Text(ref literal) => self.format_text(literal, allow_wrap, entering)?,
            NodeValue::LineBreak => self.format_line_break(entering, next_is_block)?,
            NodeValue::SoftBreak => self.format_soft_break(allow_wrap, entering)?,
            NodeValue::Code(ref code) => self.format_code(&code.literal, allow_wrap, entering)?,
            NodeValue::HtmlInline(ref literal) => self.format_html_inline(literal, entering)?,
            NodeValue::Raw(ref literal) => self.format_raw(literal, entering)?,
            NodeValue::Strong => {
                if parent_node.is_none()
                    || !matches!(parent_node.unwrap().data.borrow().value, NodeValue::Strong)
                {
                    self.format_strong()?;
                }
            }
            NodeValue::Emph => self.format_emph(node)?,
            NodeValue::TaskItem(symbol) => self.format_task_item(symbol, node, entering)?,
            NodeValue::Strikethrough => self.format_strikethrough()?,
            NodeValue::Superscript => self.format_superscript()?,
            NodeValue::Link(ref nl) => return self.format_link(node, nl, entering),
            NodeValue::Image(ref nl) => self.format_image(nl, allow_wrap, entering)?,
            #[cfg(feature = "shortcodes")]
            NodeValue::ShortCode(ref ne) => self.format_shortcode(ne, entering)?,
            NodeValue::Table(..) => self.format_table(entering),
            NodeValue::TableRow(..) => self.format_table_row(entering)?,
            NodeValue::TableCell => self.format_table_cell(node, entering)?,
            NodeValue::FootnoteDefinition(ref nfd) => {
                self.format_footnote_definition(&nfd.name, entering)?
            }
            NodeValue::FootnoteReference(ref nfr) => {
                self.format_footnote_reference(&nfr.name, entering)?
            }
            NodeValue::MultilineBlockQuote(..) => self.format_block_quote(entering)?,
            NodeValue::Escaped => {
                // Noop - the character gets escaped as usual, this is just an
                // AST marker created by escaped_char_spans.
            }
            NodeValue::Math(ref math) => self.format_math(math, allow_wrap, entering)?,
            NodeValue::WikiLink(ref nl) => self.format_wikilink(nl, entering)?,
            NodeValue::Underline => self.format_underline()?,
            NodeValue::Subscript => self.format_subscript()?,
            NodeValue::SpoileredText => self.format_spoiler()?,
            NodeValue::EscapedTag(ref net) => self.format_escaped_tag(net)?,
            NodeValue::Alert(ref alert) => self.format_alert(alert, entering)?,
        };
        Ok(true)
    }

    fn format_front_matter(&mut self, front_matter: &str, entering: bool) -> fmt::Result {
        if entering {
            self.output(front_matter, false, Escaping::Literal)?;
        }
        Ok(())
    }

    fn format_block_quote(&mut self, entering: bool) -> fmt::Result {
        if entering {
            write!(self, "> ")?;
            self.begin_content = true;
            write!(self.prefix, "> ")?;
        } else {
            let new_len = self.prefix.len() - 2;
            self.prefix.truncate(new_len);
            self.blankline();
        }
        Ok(())
    }

    fn format_list(&mut self, node: &'a AstNode<'a>, entering: bool) -> fmt::Result {
        let ol_start = match node.data.borrow().value {
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

            if match node.next_sibling() {
                Some(next_sibling) => matches!(
                    next_sibling.data.borrow().value,
                    NodeValue::CodeBlock(..) | NodeValue::List(..)
                ),
                _ => false,
            } {
                self.cr();
                write!(self, "<!-- end list -->")?;
                self.blankline();
            }
        }
        Ok(())
    }

    fn format_item(&mut self, node: &'a AstNode<'a>, entering: bool) -> fmt::Result {
        let parent = match node.parent().unwrap().data.borrow().value {
            NodeValue::List(ref nl) => *nl,
            _ => unreachable!(),
        };

        let mut listmarker = String::new();

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
                match node.data.borrow().value {
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
            )?;
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
                self.write_str(&listmarker)?;
            }
            self.begin_content = true;
            for _ in 0..marker_width {
                write!(self.prefix, " ")?;
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

        Ok(())
    }

    fn format_description_details(&mut self, entering: bool) -> fmt::Result {
        if entering {
            write!(self, ": ")?
        }
        Ok(())
    }

    fn format_heading(&mut self, nh: &NodeHeading, entering: bool) -> fmt::Result {
        if entering {
            for _ in 0..nh.level {
                write!(self, "#")?;
            }
            write!(self, " ")?;
            self.begin_content = true;
            self.no_linebreaks = true;
        } else {
            self.no_linebreaks = false;
            self.blankline();
        }
        Ok(())
    }

    fn format_code_block(
        &mut self,
        node: &'a AstNode<'a>,
        ncb: &NodeCodeBlock,
        entering: bool,
    ) -> fmt::Result {
        if !entering {
            return Ok(());
        }

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

        #[allow(clippy::len_zero)]
        if !(info.len() > 0
            || literal.len() <= 2
            || isspace(literal[0])
            || first_in_list_item
            || self.options.render.prefer_fenced
            || isspace(literal[literal.len() - 1]) && isspace(literal[literal.len() - 2]))
        {
            write!(self, "    ")?;
            write!(self.prefix, "    ")?;
            self.write_str(&ncb.literal)?;
            let new_len = self.prefix.len() - 4;
            self.prefix.truncate(new_len);
        } else {
            let fence_byte = if info.contains(&b'`') { b'~' } else { b'`' };
            let numticks = max(3, longest_byte_sequence(literal, fence_byte) + 1);
            for _ in 0..numticks {
                write!(self, "{}", fence_byte as char)?;
            }
            if !info.is_empty() {
                write!(self, " ")?;
                self.write_str(&ncb.info)?;
            }
            self.cr();
            self.write_str(&ncb.literal)?;
            self.cr();
            for _ in 0..numticks {
                write!(self, "{}", fence_byte as char)?;
            }
        }
        self.blankline();

        Ok(())
    }

    fn format_html_block(&mut self, nhb: &NodeHtmlBlock, entering: bool) -> fmt::Result {
        if entering {
            self.blankline();
            self.write_str(&nhb.literal)?;
            self.blankline();
        }
        Ok(())
    }

    fn format_thematic_break(&mut self, entering: bool) -> fmt::Result {
        if entering {
            self.blankline();
            write!(self, "-----")?;
            self.blankline();
        }
        Ok(())
    }

    fn format_paragraph(&mut self, entering: bool) {
        if !entering {
            self.blankline();
        }
    }

    fn format_text(&mut self, literal: &str, allow_wrap: bool, entering: bool) -> fmt::Result {
        if entering {
            self.output(literal, allow_wrap, Escaping::Normal)?;
        }
        Ok(())
    }

    fn format_line_break(&mut self, entering: bool, next_is_block: bool) -> fmt::Result {
        if entering {
            if !self.options.render.hardbreaks && !next_is_block {
                // If the next element is a block, a backslash means a
                // literal backslash instead of a line break. In this case
                // we can just skip the line break since it's meaningless
                // before a block.
                write!(self, "\\")?;
            }
            self.cr();
        }
        Ok(())
    }

    fn format_soft_break(&mut self, allow_wrap: bool, entering: bool) -> fmt::Result {
        if entering {
            if !self.no_linebreaks
                && self.options.render.width == 0
                && !self.options.render.hardbreaks
            {
                self.cr();
            } else if self.options.render.hardbreaks {
                self.output("\n", allow_wrap, Escaping::Literal)?;
            } else {
                self.output(" ", allow_wrap, Escaping::Literal)?;
            }
        }
        Ok(())
    }

    fn format_code(&mut self, literal: &str, allow_wrap: bool, entering: bool) -> fmt::Result {
        if entering {
            let literal_bytes = literal.as_bytes();
            let numticks = shortest_unused_sequence(literal_bytes, b'`');
            for _ in 0..numticks {
                write!(self, "`")?;
            }

            let all_space = literal_bytes
                .iter()
                .all(|&c| c == b' ' || c == b'\r' || c == b'\n');
            let has_edge_space =
                literal_bytes[0] == b' ' || literal_bytes[literal_bytes.len() - 1] == b' ';
            let has_edge_backtick =
                literal_bytes[0] == b'`' || literal_bytes[literal_bytes.len() - 1] == b'`';

            let pad = literal.is_empty() || has_edge_backtick || (!all_space && has_edge_space);
            if pad {
                write!(self, " ")?
            }
            self.output(literal, allow_wrap, Escaping::Literal)?;
            if pad {
                write!(self, " ")?;
            }
            for _ in 0..numticks {
                write!(self, "`")?;
            }
        }

        Ok(())
    }

    fn format_html_inline(&mut self, literal: &str, entering: bool) -> fmt::Result {
        if entering {
            self.write_str(literal)?;
        }
        Ok(())
    }

    fn format_raw(&mut self, literal: &str, entering: bool) -> fmt::Result {
        if entering {
            self.write_str(literal)?;
        }
        Ok(())
    }

    fn format_strong(&mut self) -> fmt::Result {
        write!(self, "**")
    }

    fn format_emph(&mut self, node: &'a AstNode<'a>) -> fmt::Result {
        let emph_delim = if match node.parent() {
            Some(parent) => matches!(parent.data.borrow().value, NodeValue::Emph),
            _ => false,
        } && node.next_sibling().is_none()
            && node.previous_sibling().is_none()
        {
            "_"
        } else {
            "*"
        };

        self.write_str(emph_delim)?;
        Ok(())
    }

    fn format_task_item(
        &mut self,
        symbol: Option<char>,
        node: &'a AstNode<'a>,
        entering: bool,
    ) -> fmt::Result {
        if node
            .parent()
            .map(|p| node_matches!(p, NodeValue::List(_)))
            .unwrap_or_default()
        {
            self.format_item(node, entering)?;
        }
        if entering {
            write!(self, "[{}] ", symbol.unwrap_or(' '))?;
        }
        Ok(())
    }

    fn format_strikethrough(&mut self) -> fmt::Result {
        write!(self, "~~")?;
        Ok(())
    }

    fn format_superscript(&mut self) -> fmt::Result {
        write!(self, "^")?;
        Ok(())
    }

    fn format_underline(&mut self) -> fmt::Result {
        write!(self, "__")?;
        Ok(())
    }

    fn format_subscript(&mut self) -> fmt::Result {
        write!(self, "~")?;
        Ok(())
    }

    fn format_spoiler(&mut self) -> fmt::Result {
        write!(self, "||")?;
        Ok(())
    }

    fn format_escaped_tag(&mut self, net: &String) -> fmt::Result {
        self.output(net, false, Escaping::Literal)
    }

    fn format_link(
        &mut self,
        node: &'a AstNode<'a>,
        nl: &NodeLink,
        entering: bool,
    ) -> Result<bool, fmt::Error> {
        if is_autolink(node, nl) {
            if entering {
                write!(self, "<{}>", trim_start_match(&nl.url, "mailto:"))?;
                return Ok(false);
            }
        } else if entering {
            write!(self, "[")?;
        } else {
            write!(self, "](")?;
            self.output(&nl.url, false, Escaping::Url)?;
            if !nl.title.is_empty() {
                write!(self, " \"")?;
                self.output(&nl.title, false, Escaping::Title)?;
                write!(self, "\"")?;
            }
            write!(self, ")")?;
        }

        Ok(true)
    }

    fn format_wikilink(&mut self, nl: &NodeWikiLink, entering: bool) -> fmt::Result {
        if entering {
            write!(self, "[[")?;
            if self.options.extension.wikilinks() == Some(WikiLinksMode::UrlFirst) {
                self.output(&nl.url, false, Escaping::Url)?;
                write!(self, "|").unwrap();
            }
        } else {
            if self.options.extension.wikilinks() == Some(WikiLinksMode::TitleFirst) {
                write!(self, "|")?;
                self.output(&nl.url, false, Escaping::Url)?;
            }
            write!(self, "]]")?;
        }

        Ok(())
    }

    fn format_image(&mut self, nl: &NodeLink, allow_wrap: bool, entering: bool) -> fmt::Result {
        if entering {
            write!(self, "![")?;
        } else {
            write!(self, "](")?;
            self.output(&nl.url, false, Escaping::Url)?;
            if !nl.title.is_empty() {
                self.output(" \"", allow_wrap, Escaping::Literal)?;
                self.output(&nl.title, false, Escaping::Title)?;
                write!(self, "\"")?;
            }
            write!(self, ")")?;
        }
        Ok(())
    }

    #[cfg(feature = "shortcodes")]
    fn format_shortcode(&mut self, ne: &NodeShortCode, entering: bool) -> fmt::Result {
        if entering {
            write!(self, ":")?;
            self.output(&ne.code, false, Escaping::Literal)?;
            write!(self, ":")?;
        }
        Ok(())
    }

    fn format_table(&mut self, entering: bool) {
        if entering {
            self.custom_escape = Some(table_escape);
        } else {
            self.custom_escape = None;
        }
        self.blankline();
    }

    fn format_table_row(&mut self, entering: bool) -> fmt::Result {
        if entering {
            self.cr();
            write!(self, "|")?;
        }
        Ok(())
    }

    fn format_table_cell(&mut self, node: &'a AstNode<'a>, entering: bool) -> fmt::Result {
        if entering {
            write!(self, " ")?;
        } else {
            write!(self, " |")?;

            let row = &node.parent().unwrap().data.borrow().value;
            let in_header = match *row {
                NodeValue::TableRow(header) => header,
                _ => panic!(),
            };

            if in_header && node.next_sibling().is_none() {
                let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
                let alignments = match *table {
                    NodeValue::Table(NodeTable { ref alignments, .. }) => alignments,
                    _ => panic!(),
                };

                self.cr();
                write!(self, "|")?;
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
                    )?;
                }
                self.cr();
            }
        }
        Ok(())
    }

    fn format_footnote_definition(&mut self, name: &str, entering: bool) -> fmt::Result {
        if entering {
            self.footnote_ix += 1;
            writeln!(self, "[^{}]:", name)?;
            write!(self.prefix, "    ")?;
        } else {
            let new_len = self.prefix.len() - 4;
            self.prefix.truncate(new_len);
        }
        Ok(())
    }

    fn format_footnote_reference(&mut self, r: &str, entering: bool) -> fmt::Result {
        if entering {
            self.write_str("[^")?;
            self.write_str(r)?;
            self.write_str("]")?;
        }
        Ok(())
    }

    fn format_math(&mut self, math: &NodeMath, allow_wrap: bool, entering: bool) -> fmt::Result {
        if entering {
            let literal = &math.literal;
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

            self.output(start_fence, false, Escaping::Literal)?;
            self.output(literal, allow_wrap, Escaping::Literal)?;
            self.output(end_fence, false, Escaping::Literal)?;
        }
        Ok(())
    }

    fn format_alert(&mut self, alert: &NodeAlert, entering: bool) -> fmt::Result {
        if entering {
            write!(
                self,
                "> [!{}]",
                alert.alert_type.default_title().to_uppercase()
            )?;
            if let Some(ref title) = alert.title {
                write!(self, " {title}")?;
            }
            writeln!(self)?;
            write!(self, "> ")?;
            self.begin_content = true;
            write!(self.prefix, "> ")?;
        } else {
            let new_len = self.prefix.len() - 2;
            self.prefix.truncate(new_len);
            self.blankline();
        }
        Ok(())
    }
}

fn longest_byte_sequence(buffer: &[u8], ch: u8) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for c in buffer {
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

fn shortest_unused_sequence(buffer: &[u8], f: u8) -> usize {
    let mut used = 1;
    let mut current = 0;
    for c in buffer {
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
    if nl.url.is_empty() || scanners::scheme(&nl.url).is_none() {
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

fn table_escape<'a>(node: &'a AstNode<'a>, c: char) -> bool {
    match node.data.borrow().value {
        NodeValue::Table(..) | NodeValue::TableRow(..) | NodeValue::TableCell => false,
        _ => c == '|',
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

        let arena = Arena::new();
        let root = crate::parse_document(&arena, text, &options_without);

        let mut out = String::new();
        format_document(root, &options_without, &mut out).unwrap();

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
