mod autolink;
mod inlines;
pub mod options;
#[cfg(feature = "shortcodes")]
pub mod shortcodes;
mod table;

use std::borrow::Cow;
use std::cmp::{min, Ordering};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::mem;
use std::str;

use crate::ctype::{isdigit, isspace};
use crate::entity;
use crate::node_matches;
use crate::nodes::{
    self, AlertType, Ast, ListDelimType, ListType, Node, NodeAlert, NodeCodeBlock,
    NodeDescriptionItem, NodeFootnoteDefinition, NodeHeading, NodeHtmlBlock, NodeList,
    NodeMultilineBlockQuote, NodeValue, Sourcepos,
};
use crate::parser::inlines::RefMap;
pub use crate::parser::options::Options;
use crate::scanners;
use crate::strings::{self, split_off_front_matter, Case};
use crate::Arena;

const TAB_STOP: usize = 4;
const CODE_INDENT: usize = 4;

// Very deeply nested lists can cause quadratic performance issues.
// This constant is used in open_new_blocks() to limit the nesting
// depth. It is unlikely that a non-contrived markdown document will
// be nested this deeply.
const MAX_LIST_DEPTH: usize = 100;

/// Parse a Markdown document to an AST.
///
/// See the documentation of the crate root for an example.
pub fn parse_document(arena: &mut Arena, md: &str, options: &Options) -> Node {
    let root = arena
        .alloc(
            Ast {
                value: NodeValue::Document,
                content: String::new(),
                sourcepos: (1, 1, 1, 1).into(),
                open: true,
                last_line_blank: false,
                table_visited: false,
                line_offsets: Vec::new(),
            }
            .into(),
        )
        .into();
    let mut parser = Parser::new(arena, root, options);
    let linebuf = parser.feed(String::new(), md, true);
    parser.finish(linebuf)
}

pub struct Parser<'a, 'o, 'c> {
    arena: &'a mut Arena,
    refmap: RefMap,
    footnote_defs: inlines::FootnoteDefs,
    root: Node,
    current: Node,
    line_number: usize,
    offset: usize,
    column: usize,
    thematic_break_kill_pos: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    partially_consumed_tab: bool,
    curline_len: usize,
    curline_end_col: usize,
    last_line_length: usize,
    last_buffer_ended_with_cr: bool,
    total_size: usize,
    options: &'o Options<'c>,
}

/// A reference link's resolved details.
#[derive(Clone, Debug)]
pub struct ResolvedReference {
    /// The destination URL of the reference link.
    pub url: String,

    /// The text of the link.
    pub title: String,
}

struct FootnoteDefinition {
    ix: Option<u32>,
    node: Node,
    name: String,
    total_references: u32,
}

impl<'a, 'o, 'c> Parser<'a, 'o, 'c>
where
    'c: 'o,
{
    fn new(arena: &'a mut Arena, root: Node, options: &'o Options<'c>) -> Self {
        Parser {
            arena,
            refmap: RefMap::new(),
            footnote_defs: inlines::FootnoteDefs::new(),
            root,
            current: root,
            line_number: 0,
            offset: 0,
            column: 0,
            thematic_break_kill_pos: 0,
            first_nonspace: 0,
            first_nonspace_column: 0,
            indent: 0,
            blank: false,
            partially_consumed_tab: false,
            curline_len: 0,
            curline_end_col: 0,
            last_line_length: 0,
            last_buffer_ended_with_cr: false,
            total_size: 0,
            options,
        }
    }

    fn feed(&mut self, mut linebuf: String, mut s: &str, eof: bool) -> String {
        if let (0, Some(delimiter)) = (
            self.total_size,
            &self.options.extension.front_matter_delimiter,
        ) {
            if let Some((front_matter, rest)) = split_off_front_matter(s, delimiter) {
                self.handle_front_matter(front_matter, delimiter);
                s = rest;
            }
        }

        let s = s;
        let sb = s.as_bytes();

        if s.len() > usize::MAX - self.total_size {
            self.total_size = usize::MAX;
        } else {
            self.total_size += s.len();
        }

        let mut buffer = 0;
        if self.last_buffer_ended_with_cr && !s.is_empty() && sb[0] == b'\n' {
            buffer += 1;
        }
        self.last_buffer_ended_with_cr = false;

        let end = s.len();

        while buffer < end {
            let mut process = false;
            let mut eol = buffer;
            let mut ate_line_end = false;
            while eol < end {
                if strings::is_line_end_char(sb[eol]) {
                    process = true;
                    ate_line_end = true;
                    eol += 1;
                    break;
                }
                if sb[eol] == 0 {
                    break;
                }
                eol += 1;
            }

            if eol >= end && eof {
                process = true;
            }

            if process {
                if !linebuf.is_empty() {
                    linebuf.push_str(&s[buffer..eol]);
                    let line = mem::take(&mut linebuf);
                    self.process_line(line.into());
                } else {
                    self.process_line(s[buffer..eol].into());
                }
            } else if eol < end && sb[eol] == b'\0' {
                linebuf.push_str(&s[buffer..eol]);
                linebuf.push('\u{fffd}');
            } else {
                linebuf.push_str(&s[buffer..eol]);
            }

            buffer = eol;
            if buffer < end {
                if sb[buffer] == b'\0' {
                    buffer += 1;
                } else {
                    if ate_line_end {
                        buffer -= 1;
                    }
                    if sb[buffer] == b'\r' {
                        buffer += 1;
                        if buffer == end {
                            self.last_buffer_ended_with_cr = true;
                        }
                    }
                    if buffer < end && sb[buffer] == b'\n' {
                        buffer += 1;
                    }
                }
            }
        }

        linebuf
    }

    fn handle_front_matter(&mut self, front_matter: &str, delimiter: &str) {
        let lines = front_matter
            .as_bytes()
            .iter()
            .filter(|b| **b == b'\n')
            .count();

        let stripped_front_matter = strings::remove_trailing_blank_lines_slice(front_matter);
        let stripped_lines = stripped_front_matter
            .as_bytes()
            .iter()
            .filter(|b| **b == b'\n')
            .count();

        let node = self.add_child(
            self.root,
            NodeValue::FrontMatter(front_matter.to_string()),
            1,
        );
        self.finalize(node).unwrap();

        node.data_mut(self.arena).sourcepos = Sourcepos {
            start: nodes::LineColumn { line: 1, column: 1 },
            end: nodes::LineColumn {
                line: 1 + stripped_lines,
                column: delimiter.len(),
            },
        };
        self.line_number += lines;
    }

    fn process_line(&mut self, mut line: Cow<str>) {
        let last_byte = line.as_bytes().last();
        if last_byte.map_or(true, |&b| !strings::is_line_end_char(b)) {
            line.to_mut().push('\n');
        } else if last_byte == Some(&b'\r') {
            let line_mut = line.to_mut();
            line_mut.pop();
            line_mut.push('\n');
        };
        let line = line.as_ref();
        let bytes = line.as_bytes();

        self.curline_len = line.len();
        self.curline_end_col = line.len();
        if self.curline_end_col > 0 && bytes[self.curline_end_col - 1] == b'\n' {
            self.curline_end_col -= 1;
        }
        if self.curline_end_col > 0 && bytes[self.curline_end_col - 1] == b'\r' {
            self.curline_end_col -= 1;
        }

        self.offset = 0;
        self.column = 0;
        self.first_nonspace = 0;
        self.first_nonspace_column = 0;
        self.indent = 0;
        self.thematic_break_kill_pos = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        if self.line_number == 0 && line.len() >= 3 && line.starts_with('\u{feff}') {
            self.offset += 3;
        }

        self.line_number += 1;

        if let Some((last_matched_container, all_matched)) = self.check_open_blocks(line) {
            let mut container = last_matched_container;
            let current = self.current;
            self.open_new_blocks(&mut container, line, all_matched);

            if current == self.current {
                self.add_text_to_container(container, last_matched_container, line);
            }
        }

        self.last_line_length = self.curline_end_col;

        self.curline_len = 0;
        self.curline_end_col = 0;
    }

    ///////////////////////
    // Check open blocks //
    ///////////////////////

    fn check_open_blocks(&mut self, line: &str) -> Option<(Node, bool)> {
        let (all_matched, mut container) = self.check_open_blocks_inner(self.root, line)?;

        if !all_matched {
            container = container.parent(self.arena).unwrap();
        }

        Some((container, all_matched))
    }

    fn check_open_blocks_inner(&mut self, mut container: Node, line: &str) -> Option<(bool, Node)> {
        let mut all_matched = false;

        loop {
            if !container.last_child_is_open(self.arena) {
                all_matched = true;
                break;
            }
            container = container.last_child(self.arena).unwrap();

            self.find_first_nonspace(line);

            match container.data(self.arena).value {
                NodeValue::BlockQuote => {
                    if !self.parse_block_quote_prefix(line) {
                        break;
                    }
                }
                NodeValue::Item(nl) => {
                    if !self.parse_node_item_prefix(line, container, &nl) {
                        break;
                    }
                }
                NodeValue::DescriptionItem(di) => {
                    if !self.parse_description_item_prefix(line, container, &di) {
                        break;
                    }
                }
                NodeValue::CodeBlock(..) => {
                    if !self.parse_code_block_prefix(line, container)? {
                        break;
                    }
                }
                NodeValue::HtmlBlock(ref nhb) => {
                    if !self.parse_html_block_prefix(nhb.block_type) {
                        break;
                    }
                }
                NodeValue::Paragraph => {
                    if self.blank {
                        break;
                    }
                }
                NodeValue::Table(..) => {
                    if !table::matches(&line[self.first_nonspace..], self.options.extension.spoiler)
                    {
                        break;
                    }
                }
                NodeValue::Heading(..)
                | NodeValue::TableRow(..)
                | NodeValue::TableCell
                | NodeValue::Subtext => {
                    break;
                }
                NodeValue::FootnoteDefinition(..) => {
                    if !self.parse_footnote_definition_block_prefix(line) {
                        break;
                    }
                }
                NodeValue::MultilineBlockQuote(..) => {
                    self.parse_multiline_block_quote_prefix(line, container)?;
                }
                NodeValue::Alert(ref alert) => {
                    if alert.multiline {
                        self.parse_multiline_block_quote_prefix(line, container)?;
                    } else if !self.parse_block_quote_prefix(line) {
                        break;
                    }
                }
                _ => {}
            }
        }

        Some((all_matched, container))
    }

    fn find_first_nonspace(&mut self, line: &str) {
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
        let bytes = line.as_bytes();

        if self.first_nonspace <= self.offset {
            self.first_nonspace = self.offset;
            self.first_nonspace_column = self.column;

            loop {
                if self.first_nonspace >= line.len() {
                    break;
                }
                match bytes[self.first_nonspace] {
                    32 => {
                        self.first_nonspace += 1;
                        self.first_nonspace_column += 1;
                        chars_to_tab -= 1;
                        if chars_to_tab == 0 {
                            chars_to_tab = TAB_STOP;
                        }
                    }
                    9 => {
                        self.first_nonspace += 1;
                        self.first_nonspace_column += chars_to_tab;
                        chars_to_tab = TAB_STOP;
                    }
                    _ => break,
                }
            }
        }

        self.indent = self.first_nonspace_column - self.column;
        self.blank = self.first_nonspace < line.len()
            && strings::is_line_end_char(bytes[self.first_nonspace]);
    }

    fn parse_block_quote_prefix(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let indent = self.indent;
        if indent <= 3 && bytes[self.first_nonspace] == b'>' && self.is_not_greentext(line) {
            self.advance_offset(line, indent + 1, true);

            if strings::is_space_or_tab(bytes[self.offset]) {
                self.advance_offset(line, 1, true);
            }

            return true;
        }

        false
    }

    fn is_not_greentext(&self, line: &str) -> bool {
        !self.options.extension.greentext
            || strings::is_space_or_tab(line.as_bytes()[self.first_nonspace + 1])
    }

    fn parse_node_item_prefix(&mut self, line: &str, container: Node, nl: &NodeList) -> bool {
        if self.indent >= nl.marker_offset + nl.padding {
            self.advance_offset(line, nl.marker_offset + nl.padding, true);
            true
        } else if self.blank && container.first_child(self.arena).is_some() {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            true
        } else {
            false
        }
    }

    fn parse_description_item_prefix(
        &mut self,
        line: &str,
        container: Node,
        di: &NodeDescriptionItem,
    ) -> bool {
        if self.indent >= di.marker_offset + di.padding {
            self.advance_offset(line, di.marker_offset + di.padding, true);
            true
        } else if self.blank && container.first_child(self.arena).is_some() {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            true
        } else {
            false
        }
    }

    fn parse_code_block_prefix(&mut self, line: &str, container: Node) -> Option<bool> {
        let (fenced, fence_char, fence_length, fence_offset) =
            match container.data(self.arena).value {
                NodeValue::CodeBlock(ref ncb) => (
                    ncb.fenced,
                    ncb.fence_char,
                    ncb.fence_length,
                    ncb.fence_offset,
                ),
                _ => unreachable!(),
            };

        if !fenced {
            if self.indent >= CODE_INDENT {
                self.advance_offset(line, CODE_INDENT, true);
                return Some(true);
            } else if self.blank {
                let offset = self.first_nonspace - self.offset;
                self.advance_offset(line, offset, false);
                return Some(true);
            }
            return Some(false);
        }

        let bytes = line.as_bytes();
        let matched = if self.indent <= 3 && bytes[self.first_nonspace] == fence_char {
            scanners::close_code_fence(&line[self.first_nonspace..]).unwrap_or(0)
        } else {
            0
        };

        if matched >= fence_length {
            self.advance_offset(line, matched, false);
            self.current = self.finalize(container).unwrap();
            return None;
        }

        let mut i = fence_offset;
        while i > 0 && strings::is_space_or_tab(bytes[self.offset]) {
            self.advance_offset(line, 1, true);
            i -= 1;
        }
        Some(true)
    }

    fn parse_html_block_prefix(&self, t: u8) -> bool {
        match t {
            1..=5 => true,
            6 | 7 => !self.blank,
            _ => unreachable!(),
        }
    }

    fn parse_footnote_definition_block_prefix(&mut self, line: &str) -> bool {
        if self.indent >= 4 {
            self.advance_offset(line, 4, true);
            true
        } else {
            line == "\n" || line == "\r\n"
        }
    }

    fn parse_multiline_block_quote_prefix(&mut self, line: &str, container: Node) -> Option<()> {
        // XXX: refactoring revealed that, unlike parse_code_block_prefix, this
        // function never fails to match without signalling 'should_continue'
        // (which is a `Some(false)` in that function). Is that odd?

        let (fence_length, fence_offset) = match container.data(self.arena).value {
            NodeValue::MultilineBlockQuote(ref node_value) => {
                (node_value.fence_length, node_value.fence_offset)
            }
            NodeValue::Alert(ref node_value) => (node_value.fence_length, node_value.fence_offset),
            _ => unreachable!(),
        };

        let bytes = line.as_bytes();
        let matched = if self.indent <= 3 && bytes[self.first_nonspace] == b'>' {
            scanners::close_multiline_block_quote_fence(&line[self.first_nonspace..]).unwrap_or(0)
        } else {
            0
        };

        if matched >= fence_length {
            self.advance_offset(line, matched, false);

            // The last child, like an indented codeblock, could be left open.
            // Make sure it's finalized.
            if container.last_child_is_open(self.arena) {
                let child = container.last_child(self.arena).unwrap();
                self.finalize(child).unwrap();
            }

            self.current = self.finalize(container).unwrap();
            return None;
        }

        let mut i = fence_offset;
        while i > 0 && strings::is_space_or_tab(bytes[self.offset]) {
            self.advance_offset(line, 1, true);
            i -= 1;
        }
        Some(())
    }

    /////////////////////
    // Open new blocks //
    /////////////////////

    fn open_new_blocks(&mut self, container: &mut Node, line: &str, all_matched: bool) {
        let mut maybe_lazy = node_matches!(self.arena, self.current, NodeValue::Paragraph);
        let mut depth = 0;

        while !node_matches!(
            self.arena,
            container,
            NodeValue::CodeBlock(..) | NodeValue::HtmlBlock(..)
        ) {
            depth += 1;
            self.find_first_nonspace(line);
            let indented = self.indent >= CODE_INDENT;

            if !((!indented
                && (self.handle_alert(container, line)
                    || self.handle_multiline_blockquote(container, line)
                    || self.handle_blockquote(container, line)
                    || self.handle_atx_heading(container, line)
                    || self.handle_atx_subtext(container, line)
                    || self.handle_code_fence(container, line)
                    || self.handle_html_block(container, line)
                    || self.handle_setext_heading(container, line)
                    || self.handle_thematic_break(container, line, all_matched)
                    || self.handle_footnote(container, line, depth)
                    || self.handle_description_list(container, line)))
                || self.handle_list(container, line, indented, depth)
                || self.handle_code_block(container, line, indented, maybe_lazy)
                || self.handle_table(container, line, indented))
            {
                break;
            }

            if container.data(self.arena).value.accepts_lines() {
                break;
            }

            maybe_lazy = false;
        }
    }

    fn handle_alert(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(alert_type) = self.detect_alert(line) else {
            return false;
        };

        let alert_startpos = self.first_nonspace;
        let mut title_startpos = self.first_nonspace;
        let mut fence_length = 0;

        let bytes = line.as_bytes();
        while bytes[title_startpos] != b']' {
            if bytes[title_startpos] == b'>' {
                fence_length += 1
            }
            title_startpos += 1;
        }
        title_startpos += 1;

        if fence_length == 2
            || (fence_length >= 3 && !self.options.extension.multiline_block_quotes)
        {
            return false;
        }

        // anything remaining on this line is considered an alert title
        let mut title = entity::unescape_html(&line[title_startpos..]).into_owned();
        strings::trim(&mut title);
        strings::unescape(&mut title);

        let na = NodeAlert {
            alert_type,
            multiline: fence_length >= 3,
            fence_length,
            fence_offset: self.first_nonspace - self.offset,
            title: if title.is_empty() { None } else { Some(title) },
        };

        let offset = self.curline_len - self.offset - 1;
        self.advance_offset(line, offset, false);

        *container = self.add_child(
            *container,
            NodeValue::Alert(Box::new(na)),
            alert_startpos + 1,
        );

        true
    }

    fn detect_alert(&self, line: &str) -> Option<AlertType> {
        if self.options.extension.alerts && line.as_bytes()[self.first_nonspace] == b'>' {
            scanners::alert_start(&line[self.first_nonspace..])
        } else {
            None
        }
    }

    fn handle_multiline_blockquote(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_multiline_blockquote(line) else {
            return false;
        };

        let first_nonspace = self.first_nonspace;
        let offset = self.offset;
        let nmbc = NodeMultilineBlockQuote {
            fence_length: matched,
            fence_offset: first_nonspace - offset,
        };

        *container = self.add_child(
            *container,
            NodeValue::MultilineBlockQuote(nmbc),
            self.first_nonspace + 1,
        );

        self.advance_offset(line, first_nonspace + matched - offset, false);

        true
    }

    fn detect_multiline_blockquote(&self, line: &str) -> Option<usize> {
        if self.options.extension.multiline_block_quotes {
            scanners::open_multiline_block_quote_fence(&line[self.first_nonspace..])
        } else {
            None
        }
    }

    fn handle_blockquote(&mut self, container: &mut Node, line: &str) -> bool {
        if !self.detect_blockquote(line) {
            return false;
        }

        let blockquote_startpos = self.first_nonspace;

        let offset = self.first_nonspace + 1 - self.offset;
        self.advance_offset(line, offset, false);
        if strings::is_space_or_tab(line.as_bytes()[self.offset]) {
            self.advance_offset(line, 1, true);
        }
        *container = self.add_child(*container, NodeValue::BlockQuote, blockquote_startpos + 1);

        true
    }

    fn detect_blockquote(&self, line: &str) -> bool {
        line.as_bytes()[self.first_nonspace] == b'>' && self.is_not_greentext(line)
    }

    fn handle_atx_heading(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_atx_heading(line) else {
            return false;
        };

        let heading_startpos = self.first_nonspace;
        let offset = self.offset;
        self.advance_offset(line, heading_startpos + matched - offset, false);
        *container = self.add_child(
            *container,
            NodeValue::Heading(NodeHeading::default()),
            heading_startpos + 1,
        );

        let bytes = line.as_bytes();
        let mut hashpos = bytes[self.first_nonspace..]
            .iter()
            .position(|&c| c == b'#')
            .unwrap()
            + self.first_nonspace;
        let mut level = 0;
        while bytes[hashpos] == b'#' {
            level += 1;
            hashpos += 1;
        }

        let container_ast = &mut container.data_mut(self.arena);
        container_ast.value = NodeValue::Heading(NodeHeading {
            level,
            setext: false,
        });

        true
    }

    fn detect_atx_heading(&self, line: &str) -> Option<usize> {
        scanners::atx_heading_start(&line[self.first_nonspace..])
    }

    fn handle_atx_subtext(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_atx_subtext(line) else {
            return false;
        };

        let heading_startpos = self.first_nonspace;
        let offset = self.offset;
        self.advance_offset(line, heading_startpos + matched - offset, false);
        *container = self.add_child(*container, NodeValue::Subtext, heading_startpos + 1);

        let container_ast = &mut container.data_mut(self.arena);
        container_ast.value = NodeValue::Subtext;

        true
    }

    fn detect_atx_subtext(&self, line: &str) -> Option<usize> {
        if self.options.extension.subtext {
            scanners::atx_subtext_start(&line[self.first_nonspace..])
        } else {
            None
        }
    }

    fn handle_code_fence(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_code_fence(line) else {
            return false;
        };

        let first_nonspace = self.first_nonspace;
        let offset = self.offset;
        let ncb = NodeCodeBlock {
            fenced: true,
            fence_char: line.as_bytes()[first_nonspace],
            fence_length: matched,
            fence_offset: first_nonspace - offset,
            info: String::with_capacity(10),
            literal: String::new(),
        };
        *container = self.add_child(
            *container,
            NodeValue::CodeBlock(Box::new(ncb)),
            self.first_nonspace + 1,
        );
        self.advance_offset(line, first_nonspace + matched - offset, false);

        true
    }

    fn detect_code_fence(&self, line: &str) -> Option<usize> {
        scanners::open_code_fence(&line[self.first_nonspace..])
    }

    fn handle_html_block(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_html_block(*container, line) else {
            return false;
        };

        let nhb = NodeHtmlBlock {
            block_type: matched as u8,
            literal: String::new(),
        };

        *container = self.add_child(
            *container,
            NodeValue::HtmlBlock(nhb),
            self.first_nonspace + 1,
        );

        true
    }

    fn detect_html_block(&self, container: Node, line: &str) -> Option<usize> {
        scanners::html_block_start(&line[self.first_nonspace..]).or_else(|| {
            if !node_matches!(self.arena, container, NodeValue::Paragraph) {
                scanners::html_block_start_7(&line[self.first_nonspace..])
            } else {
                None
            }
        })
    }

    fn handle_setext_heading(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(sc) = self.detect_setext_heading(*container, line) else {
            return false;
        };

        let has_content = self.resolve_reference_link_definitions(*container);
        if has_content {
            container.data_mut(self.arena).value = NodeValue::Heading(NodeHeading {
                level: match sc {
                    scanners::SetextChar::Equals => 1,
                    scanners::SetextChar::Hyphen => 2,
                },
                setext: true,
            });
            let adv = line.len() - 1 - self.offset;
            self.advance_offset(line, adv, false);
        }

        true
    }

    fn detect_setext_heading(&self, container: Node, line: &str) -> Option<scanners::SetextChar> {
        if node_matches!(self.arena, container, NodeValue::Paragraph)
            && !self.options.parse.ignore_setext
        {
            scanners::setext_heading_line(&line[self.first_nonspace..])
        } else {
            None
        }
    }

    fn handle_thematic_break(
        &mut self,
        container: &mut Node,
        line: &str,
        all_matched: bool,
    ) -> bool {
        let Some(_matched) = self.detect_thematic_break(*container, line, all_matched) else {
            return false;
        };

        *container = self.add_child(
            *container,
            NodeValue::ThematicBreak,
            self.first_nonspace + 1,
        );

        let adv = line.len() - 1 - self.offset;
        container.data_mut(self.arena).sourcepos.end = (self.line_number, adv).into();
        self.advance_offset(line, adv, false);

        true
    }

    fn detect_thematic_break(
        &mut self,
        container: Node,
        line: &str,
        all_matched: bool,
    ) -> Option<usize> {
        if !matches!(
            (&container.data(self.arena,).value, all_matched),
            (&NodeValue::Paragraph, false)
        ) && self.thematic_break_kill_pos <= self.first_nonspace
        {
            let (offset, found) = self.scan_thematic_break_inner(line);
            if !found {
                self.thematic_break_kill_pos = offset;
                None
            } else {
                Some(offset)
            }
        } else {
            None
        }
    }

    fn scan_thematic_break_inner(&self, line: &str) -> (usize, bool) {
        let mut i = self.first_nonspace;

        if i >= line.len() {
            return (i, false);
        }

        let bytes = line.as_bytes();
        let c = bytes[i];
        if c != b'*' && c != b'_' && c != b'-' {
            return (i, false);
        }

        let mut count = 1;
        let mut nextc;
        loop {
            i += 1;
            if i >= line.len() {
                return (i, false);
            }
            nextc = bytes[i];

            if nextc == c {
                count += 1;
            } else if nextc != b' ' && nextc != b'\t' {
                break;
            }
        }

        if count >= 3 && (nextc == b'\r' || nextc == b'\n') {
            ((i - self.first_nonspace) + 1, true)
        } else {
            (i, false)
        }
    }

    fn handle_footnote(&mut self, container: &mut Node, line: &str, depth: usize) -> bool {
        let Some(matched) = self.detect_footnote(line, depth) else {
            return false;
        };

        let mut c = &line[self.first_nonspace + 2..self.first_nonspace + matched];
        c = c.split(']').next().unwrap();
        let offset = self.first_nonspace + matched - self.offset;
        self.advance_offset(line, offset, false);
        *container = self.add_child(
            *container,
            NodeValue::FootnoteDefinition(NodeFootnoteDefinition {
                name: c.to_string(),
                total_references: 0,
            }),
            self.first_nonspace + 1,
        );

        true
    }

    fn detect_footnote(&self, line: &str, depth: usize) -> Option<usize> {
        if self.options.extension.footnotes && depth < MAX_LIST_DEPTH {
            scanners::footnote_definition(&line[self.first_nonspace..])
        } else {
            None
        }
    }

    fn handle_description_list(&mut self, container: &mut Node, line: &str) -> bool {
        let Some(matched) = self.detect_description_list(container, line) else {
            return false;
        };

        let offset = self.first_nonspace + matched - self.offset;
        self.advance_offset(line, offset, false);
        if strings::is_space_or_tab(line.as_bytes()[self.offset]) {
            self.advance_offset(line, 1, true);
        }

        true
    }

    fn detect_description_list(&mut self, container: &mut Node, line: &str) -> Option<usize> {
        if self.options.extension.description_lists {
            if let Some(matched) = scanners::description_item_start(&line[self.first_nonspace..]) {
                if self.parse_desc_list_details(container, matched) {
                    return Some(matched);
                }
            }
        }
        None
    }

    fn parse_desc_list_details(&mut self, container: &mut Node, matched: usize) -> bool {
        let mut tight = false;
        let last_child = match container.last_child(self.arena) {
            Some(lc) => lc,
            None => {
                // Happens when the detail line is directly after the term,
                // without a blank line between.
                if !node_matches!(self.arena, container, NodeValue::Paragraph) {
                    // If the container is not a paragraph, then this can't
                    // be a description list item.
                    return false;
                }

                let parent = container.parent(self.arena);
                if parent.is_none() {
                    return false;
                }

                tight = true;
                *container = parent.unwrap();
                container.last_child(self.arena).unwrap()
            }
        };

        if node_matches!(self.arena, last_child, NodeValue::Paragraph) {
            // We have found the details after the paragraph for the term.
            //
            // This paragraph is moved as a child of a new DescriptionTerm node.
            //
            // If the node before the paragraph is a description list, the item
            // is added to it. If not, create a new list.

            last_child.detach(self.arena);
            let last_child_sourcepos = last_child.data(self.arena).sourcepos;

            // TODO: description list sourcepos has issues.
            //
            // DescriptionItem:
            //   For all but the last, the end line/col is wrong.
            //   Where it should be l:c, it gives (l+1):0.
            //
            // DescriptionTerm:
            //   All are incorrect; they all give the start line/col of
            //   the DescriptionDetails, and the end line/col is completely off.
            //
            // DescriptionDetails:
            //   Same as the DescriptionItem.  All but last, the end line/col
            //   is (l+1):0.
            //
            // See crate::tests::description_lists::sourcepos.
            let list = match container.last_child(self.arena) {
                Some(lc) if node_matches!(self.arena, lc, NodeValue::DescriptionList) => {
                    reopen_ast_nodes(self.arena, lc);
                    lc
                }
                _ => {
                    let list = self.add_child(
                        *container,
                        NodeValue::DescriptionList,
                        self.first_nonspace + 1,
                    );
                    list.data_mut(self.arena).sourcepos.start = last_child_sourcepos.start;
                    list
                }
            };

            let metadata = NodeDescriptionItem {
                marker_offset: self.indent,
                padding: matched,
                tight,
            };

            let item = self.add_child(
                list,
                NodeValue::DescriptionItem(metadata),
                self.first_nonspace + 1,
            );
            item.data_mut(self.arena).sourcepos.start = last_child_sourcepos.start;
            let term = self.add_child(item, NodeValue::DescriptionTerm, self.first_nonspace + 1);
            let details =
                self.add_child(item, NodeValue::DescriptionDetails, self.first_nonspace + 1);

            term.append(self.arena, last_child);

            *container = details;

            true
        } else if node_matches!(self.arena, last_child, NodeValue::DescriptionItem(..)) {
            let parent = last_child.parent(self.arena).unwrap();
            let tight = match last_child.data(self.arena).value {
                NodeValue::DescriptionItem(ref ndi) => ndi.tight,
                _ => false,
            };

            let metadata = NodeDescriptionItem {
                marker_offset: self.indent,
                padding: matched,
                tight,
            };

            let item = self.add_child(
                parent,
                NodeValue::DescriptionItem(metadata),
                self.first_nonspace + 1,
            );

            let details =
                self.add_child(item, NodeValue::DescriptionDetails, self.first_nonspace + 1);

            *container = details;

            true
        } else {
            false
        }
    }

    fn handle_list(
        &mut self,
        container: &mut Node,
        line: &str,
        indented: bool,
        depth: usize,
    ) -> bool {
        let Some((matched, mut nl)) = self.detect_list(*container, line, indented, depth) else {
            return false;
        };

        let offset = self.first_nonspace + matched - self.offset;
        self.advance_offset(line, offset, false);
        let (save_partially_consumed_tab, save_offset, save_column) =
            (self.partially_consumed_tab, self.offset, self.column);

        let bytes = line.as_bytes();
        while self.column - save_column <= 5 && strings::is_space_or_tab(bytes[self.offset]) {
            self.advance_offset(line, 1, true);
        }

        let i = self.column - save_column;
        if !(1..5).contains(&i) || strings::is_line_end_char(bytes[self.offset]) {
            nl.padding = matched + 1;
            self.offset = save_offset;
            self.column = save_column;
            self.partially_consumed_tab = save_partially_consumed_tab;
            if i > 0 {
                self.advance_offset(line, 1, true);
            }
        } else {
            nl.padding = matched + i;
        }

        nl.marker_offset = self.indent;

        if match container.data(self.arena).value {
            NodeValue::List(ref mnl) => !lists_match(&nl, mnl),
            _ => true,
        } {
            *container = self.add_child(*container, NodeValue::List(nl), self.first_nonspace + 1);
        }

        *container = self.add_child(*container, NodeValue::Item(nl), self.first_nonspace + 1);

        true
    }

    fn detect_list(
        &self,
        container: Node,
        line: &str,
        indented: bool,
        depth: usize,
    ) -> Option<(usize, NodeList)> {
        if (!indented || node_matches!(self.arena, container, NodeValue::List(..)))
            && self.indent < 4
            && depth < MAX_LIST_DEPTH
        {
            parse_list_marker(
                line,
                self.first_nonspace,
                node_matches!(self.arena, container, NodeValue::Paragraph),
            )
        } else {
            None
        }
    }

    fn handle_code_block(
        &mut self,
        container: &mut Node,
        line: &str,
        indented: bool,
        maybe_lazy: bool,
    ) -> bool {
        if !self.detect_code_block(indented, maybe_lazy) {
            return false;
        }

        self.advance_offset(line, CODE_INDENT, true);
        let ncb = NodeCodeBlock {
            fenced: false,
            fence_char: 0,
            fence_length: 0,
            fence_offset: 0,
            info: String::new(),
            literal: String::new(),
        };
        *container = self.add_child(
            *container,
            NodeValue::CodeBlock(Box::new(ncb)),
            self.offset + 1,
        );

        true
    }

    fn detect_code_block(&self, indented: bool, maybe_lazy: bool) -> bool {
        indented && !maybe_lazy && !self.blank
    }

    fn handle_table(&mut self, container: &mut Node, line: &str, indented: bool) -> bool {
        let Some((new_container, replace, mark_visited)) =
            self.detect_table(*container, line, indented)
        else {
            return false;
        };

        if replace {
            container.insert_after(self.arena, new_container);
            container.detach(self.arena);
            *container = new_container;
        } else {
            *container = new_container;
        }
        if mark_visited {
            container.data_mut(self.arena).table_visited = true;
        }

        true
    }

    fn detect_table(
        &mut self,
        container: Node,
        line: &str,
        indented: bool,
    ) -> Option<(Node, bool, bool)> {
        if !indented && self.options.extension.table {
            table::try_opening_block(self, container, line)
        } else {
            None
        }
    }

    //////////
    // Core //
    //////////

    fn advance_offset(&mut self, line: &str, mut count: usize, columns: bool) {
        let bytes = line.as_bytes();
        while count > 0 {
            match bytes[self.offset] {
                9 => {
                    let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
                    if columns {
                        self.partially_consumed_tab = chars_to_tab > count;
                        let chars_to_advance = min(count, chars_to_tab);
                        self.column += chars_to_advance;
                        self.offset += if self.partially_consumed_tab { 0 } else { 1 };
                        count -= chars_to_advance;
                    } else {
                        self.partially_consumed_tab = false;
                        self.column += chars_to_tab;
                        self.offset += 1;
                        count -= 1;
                    }
                }
                _ => {
                    self.partially_consumed_tab = false;
                    self.offset += 1;
                    self.column += 1;
                    count -= 1;
                }
            }
        }
    }

    fn add_child(&mut self, mut parent: Node, value: NodeValue, start_column: usize) -> Node {
        while !self.arena[parent.0].can_contain_type(&value) {
            parent = self.finalize(parent).unwrap();
        }

        assert!(start_column > 0);

        let child = Ast::new(value, (self.line_number, start_column).into());
        let node = self.arena.alloc(child.into()).into();
        parent.append(self.arena, node);
        node
    }

    fn add_text_to_container(
        &mut self,
        mut container: Node,
        last_matched_container: Node,
        line: &str,
    ) {
        self.find_first_nonspace(line);

        if self.blank {
            if let Some(last_child) = container.last_child(self.arena) {
                last_child.data_mut(self.arena).last_line_blank = true;
            }
        }

        container.data_mut(self.arena).last_line_blank = self.blank
            && match container.data(self.arena).value {
                NodeValue::BlockQuote
                | NodeValue::Heading(..)
                | NodeValue::ThematicBreak
                | NodeValue::Subtext => false,
                NodeValue::CodeBlock(ref ncb) => !ncb.fenced,
                NodeValue::Item(..) => {
                    container.first_child(self.arena).is_some()
                        || container.data(self.arena).sourcepos.start.line != self.line_number
                }
                NodeValue::MultilineBlockQuote(..) => false,
                NodeValue::Alert(..) => false,
                _ => true,
            };

        let mut tmp = container;
        while let Some(parent) = tmp.parent(self.arena) {
            parent.data_mut(self.arena).last_line_blank = false;
            tmp = parent;
        }

        if self.current != last_matched_container
            && container == last_matched_container
            && !self.blank
            && (!self.options.extension.greentext
                || !node_matches!(
                    self.arena,
                    container,
                    NodeValue::BlockQuote | NodeValue::Document
                ))
            && node_matches!(self.arena, self.current, NodeValue::Paragraph)
        {
            self.add_line(self.current, line);
        } else {
            while self.current != last_matched_container {
                self.current = self.finalize(self.current).unwrap();
            }

            let add_text_result = match container.data(self.arena).value {
                NodeValue::CodeBlock(..) => AddTextResult::LiteralText,
                NodeValue::HtmlBlock(ref nhb) => AddTextResult::HtmlBlock(nhb.block_type),
                _ => AddTextResult::Otherwise,
            };

            match add_text_result {
                AddTextResult::LiteralText => {
                    self.add_line(container, line);
                }
                AddTextResult::HtmlBlock(block_type) => {
                    self.add_line(container, line);

                    let matches_end_condition = match block_type {
                        1 => scanners::html_block_end_1(&line[self.first_nonspace..]),
                        2 => scanners::html_block_end_2(&line[self.first_nonspace..]),
                        3 => scanners::html_block_end_3(&line[self.first_nonspace..]),
                        4 => scanners::html_block_end_4(&line[self.first_nonspace..]),
                        5 => scanners::html_block_end_5(&line[self.first_nonspace..]),
                        _ => false,
                    };

                    if matches_end_condition {
                        container = self.finalize(container).unwrap();
                    }
                }
                _ => {
                    if self.blank {
                        // do nothing
                    } else if container.data(self.arena).value.accepts_lines() {
                        let mut line = line;
                        if let NodeValue::Heading(ref nh) = container.data(self.arena).value {
                            if !nh.setext {
                                line = strings::chop_trailing_hashes(line);
                            }
                        };
                        let count = self.first_nonspace - self.offset;

                        // In a rare case the above `chop` operation can leave
                        // the line shorter than the recorded `first_nonspace`
                        // This happens with ATX headers containing no header
                        // text, multiple spaces and trailing hashes, e.g
                        //
                        // ###     ###
                        //
                        // In this case `first_nonspace` indexes into the second
                        // set of hashes, while `chop_trailing_hashtags` truncates
                        // `line` to just `###` (the first three hashes).
                        // In this case there's no text to add, and no further
                        // processing to be done.
                        let have_line_text = self.first_nonspace <= line.len();

                        if have_line_text {
                            self.advance_offset(line, count, false);
                            self.add_line(container, line);
                        }
                    } else {
                        container = self.add_child(
                            container,
                            NodeValue::Paragraph,
                            self.first_nonspace + 1,
                        );
                        let count = self.first_nonspace - self.offset;
                        self.advance_offset(line, count, false);
                        self.add_line(container, line);
                    }
                }
            }

            self.current = container;
        }
    }

    fn add_line(&mut self, node: Node, line: &str) {
        let ast = node.data_mut(self.arena);
        assert!(ast.open);
        if self.partially_consumed_tab {
            self.offset += 1;
            let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
            ast.content.reserve(chars_to_tab);
            for _ in 0..chars_to_tab {
                ast.content.push(' ');
            }
        }
        if self.offset < line.len() {
            // since whitespace is stripped off the beginning of lines, we need to keep
            // track of how much was stripped off. This allows us to properly calculate
            // inline sourcepos during inline processing.
            ast.line_offsets.push(self.offset);

            ast.content.push_str(&line[self.offset..]);
        }
    }

    fn finish(&mut self, remaining: String) -> Node {
        if !remaining.is_empty() {
            self.process_line(remaining.into());
        }

        self.finalize_document();
        self.postprocess_text_nodes(self.root);
        self.root
    }

    fn finalize_document(&mut self) {
        while self.current != self.root {
            self.current = self.finalize(self.current).unwrap();
        }

        self.finalize(self.root);

        self.refmap.max_ref_size = if self.total_size > 100000 {
            self.total_size
        } else {
            100000
        };

        self.process_inlines();

        // Append auto-generated inline footnote definitions
        if self.options.extension.footnotes && self.options.extension.inline_footnotes {
            let inline_defs = self.footnote_defs.definitions();
            for def in inline_defs.iter() {
                self.root.append(self.arena, *def);
            }
        }

        if self.options.extension.footnotes {
            self.process_footnotes();
        }
    }

    fn resolve_reference_link_definitions(&mut self, node: Node) -> bool {
        let mut seeked = 0;
        let mut rrs_to_add = vec![];

        {
            let content = &node.data(self.arena).content;
            let bytes = content.as_bytes();
            while seeked < content.len() && bytes[seeked] == b'[' {
                if let Some((offset, rr)) = self.parse_reference_inline(&content[seeked..]) {
                    seeked += offset;
                    if let Some(rr) = rr {
                        rrs_to_add.push(rr);
                    }
                } else {
                    break;
                }
            }
        }

        for (lab, rr) in rrs_to_add {
            self.refmap.map.insert(lab, rr);
        }

        let content = &mut node.data_mut(self.arena).content;
        if seeked != 0 {
            strings::remove_from_start(content, seeked);
        }

        !strings::is_blank(content)
    }

    fn finalize(&mut self, node: Node) -> Option<Node> {
        let parent = node.parent(self.arena);
        let ast = node.data_mut(self.arena);

        assert!(ast.open);
        ast.open = false;

        let content = &mut ast.content;

        if self.curline_len == 0 {
            ast.sourcepos.end = (self.line_number, self.last_line_length).into();
        } else if match ast.value {
            NodeValue::Document => true,
            NodeValue::CodeBlock(ref ncb) => ncb.fenced,
            NodeValue::MultilineBlockQuote(..) => true,
            _ => false,
        } {
            ast.sourcepos.end = (self.line_number, self.curline_end_col).into();
        } else if matches!(ast.value, NodeValue::ThematicBreak) {
            // sourcepos.end set during opening.
        } else {
            ast.sourcepos.end = (self.line_number - 1, self.last_line_length).into();
        }

        match ast.value {
            NodeValue::Paragraph => {
                let has_content = self.resolve_reference_link_definitions(node);
                if !has_content {
                    node.detach(self.arena);
                }
            }
            NodeValue::CodeBlock(ref mut ncb) => {
                if !ncb.fenced {
                    strings::remove_trailing_blank_lines(content);
                    content.push('\n');
                } else {
                    let mut pos = 0;
                    while pos < content.len() {
                        if strings::is_line_end_char(content.as_bytes()[pos]) {
                            break;
                        }
                        pos += 1;
                    }
                    assert!(pos < content.len());

                    let mut info = entity::unescape_html(&content[..pos]).into();
                    strings::trim(&mut info);
                    strings::unescape(&mut info);
                    if info.is_empty() {
                        ncb.info = self
                            .options
                            .parse
                            .default_info_string
                            .as_ref()
                            .map_or(info, |s| s.clone());
                    } else {
                        ncb.info = info;
                    }

                    if content.as_bytes()[pos] == b'\r' {
                        pos += 1;
                    }
                    if content.as_bytes()[pos] == b'\n' {
                        pos += 1;
                    }

                    content.drain(..pos);
                }
                mem::swap(&mut ncb.literal, content);
            }
            NodeValue::HtmlBlock(ref mut nhb) => {
                mem::swap(&mut nhb.literal, content);
            }
            NodeValue::List(_) => {
                let tight = self.determine_list_tight(node);
                let NodeValue::List(ref mut nl) = node.data_mut(self.arena).value else {
                    unreachable!();
                };
                nl.tight = tight;
            }
            _ => (),
        }

        parent
    }

    fn determine_list_tight(&self, node: Node) -> bool {
        let mut ch = node.first_child(self.arena);

        while let Some(item) = ch {
            if item.data(self.arena).last_line_blank && item.next_sibling(self.arena).is_some() {
                return false;
            }

            let mut subch = item.first_child(self.arena);
            while let Some(subitem) = subch {
                if (item.next_sibling(self.arena).is_some()
                    || subitem.next_sibling(self.arena).is_some())
                    && subitem.ends_with_blank_line(self.arena)
                {
                    return false;
                }
                subch = subitem.next_sibling(self.arena);
            }

            ch = item.next_sibling(self.arena);
        }

        true
    }

    fn process_inlines(&mut self) {
        self.process_inlines_node(self.root);
    }

    fn process_inlines_node(&mut self, node: Node) {
        let mut it = node.descendants_free();
        while let Some(node) = it.next(self.arena) {
            if node.data(self.arena).value.contains_inlines() {
                self.parse_inlines(node);
            }
        }
    }

    fn parse_inlines(&mut self, node: Node) {
        let mut delimiter_arena = id_arena::Arena::new();
        let node_data = node.data_mut(self.arena);
        let mut content = mem::take(&mut node_data.content);
        let line = node_data.sourcepos.start.line;

        strings::rtrim(&mut content);

        let mut subj = inlines::Subject::new(
            self.arena,
            self.options,
            content,
            line,
            &mut self.refmap,
            &self.footnote_defs,
            &mut delimiter_arena,
        );

        while subj.parse_inline(node) {}
        subj.process_emphasis(0);
        subj.clear_brackets();
    }

    fn process_footnotes(&mut self) {
        let mut fd_map = HashMap::new();
        self.find_footnote_definitions(self.root, &mut fd_map);

        let mut next_ix = 0;
        self.find_footnote_references(self.root, &mut fd_map, &mut next_ix);

        let mut fds = fd_map.into_values().collect::<Vec<_>>();
        fds.sort_unstable_by(|a, b| a.ix.cmp(&b.ix));
        for fd in fds {
            if fd.ix.is_some() {
                let NodeValue::FootnoteDefinition(ref mut nfd) = fd.node.data_mut(self.arena).value
                else {
                    unreachable!()
                };
                nfd.name = fd.name.to_string();
                nfd.total_references = fd.total_references;
                self.root.append(self.arena, fd.node);
            } else {
                fd.node.detach(self.arena);
            }
        }
    }

    fn find_footnote_definitions(&self, node: Node, map: &mut HashMap<String, FootnoteDefinition>) {
        match node.data(self.arena).value {
            NodeValue::FootnoteDefinition(ref nfd) => {
                map.insert(
                    strings::normalize_label(&nfd.name, Case::Fold),
                    FootnoteDefinition {
                        ix: None,
                        node,
                        name: strings::normalize_label(&nfd.name, Case::Preserve),
                        total_references: 0,
                    },
                );
            }
            _ => {
                for n in node.children(self.arena) {
                    self.find_footnote_definitions(n, map);
                }
            }
        }
    }

    fn find_footnote_references(
        &mut self,
        node: Node,
        map: &mut HashMap<String, FootnoteDefinition>,
        ixp: &mut u32,
    ) {
        let ast = node.data_mut(self.arena);
        match ast.value {
            NodeValue::FootnoteReference(ref mut nfr) => {
                let normalized = strings::normalize_label(&nfr.name, Case::Fold);
                if let Some(ref mut footnote) = map.get_mut(&normalized) {
                    let ix = match footnote.ix {
                        Some(ix) => ix,
                        None => {
                            *ixp += 1;
                            footnote.ix = Some(*ixp);
                            *ixp
                        }
                    };
                    footnote.total_references += 1;
                    nfr.ref_num = footnote.total_references;
                    nfr.ix = ix;
                    nfr.name = strings::normalize_label(&footnote.name, Case::Preserve);
                } else {
                    ast.value = NodeValue::Text(format!("[^{}]", nfr.name).into());
                }
            }
            _ => {
                let mut it = node.children_free(self.arena);
                while let Some(n) = it.next(self.arena) {
                    self.find_footnote_references(n, map, ixp);
                }
            }
        }
    }

    fn postprocess_text_nodes(&mut self, root: Node) {
        let mut stack = vec![(root, false)];
        let mut children = vec![];

        while let Some((parent, in_bracket_context)) = stack.pop() {
            let mut it = parent.first_child(self.arena);

            while let Some(node) = it {
                let mut child_in_bracket_context = in_bracket_context;
                let mut emptied = false;
                let ast = node.data_mut(self.arena);
                let sourcepos = ast.sourcepos;
                match ast.value {
                    NodeValue::Text(ref mut text) => {
                        let mut subject = mem::take(text);
                        let sourcepos = self.postprocess_text_node_with_context(
                            node,
                            sourcepos,
                            subject.to_mut(),
                            in_bracket_context,
                        );
                        let ast = node.data_mut(self.arena);
                        ast.sourcepos = sourcepos;

                        let NodeValue::Text(ref mut text) = ast.value else {
                            unreachable!()
                        };
                        mem::swap(&mut subject, text);
                        emptied = text.is_empty();
                    }
                    NodeValue::Link(..) | NodeValue::Image(..) | NodeValue::WikiLink(..) => {
                        // Recurse into links, images, and wikilinks to join adjacent text nodes,
                        // but mark the context so autolinks won't be generated within them.
                        child_in_bracket_context = true;
                    }
                    _ => {}
                }

                if !emptied {
                    children.push((node, child_in_bracket_context));
                }

                it = node.next_sibling(self.arena);

                if emptied {
                    node.detach(self.arena);
                }
            }

            // Push children onto work stack in reverse order so they are
            // traversed in order
            stack.extend(children.drain(..).rev());
        }
    }

    fn postprocess_text_node_with_context(
        &mut self,
        node: Node,
        mut sourcepos: Sourcepos,
        root: &mut String,
        in_bracket_context: bool,
    ) -> Sourcepos {
        // Join adjacent text nodes together, then post-process.
        // Record the original list of sourcepos and bytecounts
        // for the post-processing step.

        let mut spxv = VecDeque::new();
        spxv.push_back((sourcepos, root.len()));
        while let Some(ns) = node.next_sibling(self.arena) {
            match ns.data(self.arena).value {
                NodeValue::Text(ref adj) => {
                    root.push_str(adj);
                    let sp = ns.data(self.arena).sourcepos;
                    spxv.push_back((sp, adj.len()));
                    sourcepos.end.column = sp.end.column;
                    ns.detach(self.arena);
                }
                _ => break,
            }
        }

        self.postprocess_text_node_with_context_inner(
            node,
            root,
            &mut sourcepos,
            spxv,
            in_bracket_context,
        );

        sourcepos
    }

    fn postprocess_text_node_with_context_inner(
        &mut self,
        node: Node,
        text: &mut String,
        sourcepos: &mut Sourcepos,
        spxv: VecDeque<(Sourcepos, usize)>,
        in_bracket_context: bool,
    ) {
        let mut spx = Spx(spxv);
        if self.options.extension.tasklist {
            self.process_tasklist(node, text, sourcepos, &mut spx);
        }

        if self.options.extension.autolink && !in_bracket_context {
            autolink::process_email_autolinks(
                self.arena,
                node,
                text,
                self.options.parse.relaxed_autolinks,
                sourcepos,
                &mut spx,
            );
        }
    }

    // Processes tasklist items in a text node.  This function
    // must not detach `node`, as we iterate through siblings in
    // `postprocess_text_nodes` and may end up relying on it
    // remaining in place.
    //
    // `text` is the mutably borrowed textual content of `node`.  If it is empty
    // after the call to `process_tasklist`, it will be properly cleaned up.
    fn process_tasklist(
        &mut self,
        node: Node,
        text: &mut String,
        sourcepos: &mut Sourcepos,
        spx: &mut Spx,
    ) {
        let (end, symbol) = match scanners::tasklist(text) {
            Some(p) => p,
            None => return,
        };

        let symbol = symbol as char;

        if !self.options.parse.relaxed_tasklist_matching && !matches!(symbol, ' ' | 'x' | 'X') {
            return;
        }

        let parent = node.parent(self.arena).unwrap();

        if node_matches!(self.arena, parent, NodeValue::TableCell) {
            if !self.options.parse.tasklist_in_table {
                return;
            }

            if node.previous_sibling(self.arena).is_some()
                || node.next_sibling(self.arena).is_some()
            {
                return;
            }

            // For now, require the task item is the only content of the table cell.
            // If we want to relax this later, we can.
            if end != text.len() {
                return;
            }

            text.drain(..end);
            let sym = self
                .arena
                .alloc(
                    Ast::new_with_sourcepos(
                        NodeValue::TaskItem(if symbol == ' ' { None } else { Some(symbol) }),
                        *sourcepos,
                    )
                    .into(),
                )
                .into();
            parent.prepend(self.arena, sym);
        } else if node_matches!(self.arena, parent, NodeValue::Paragraph) {
            if node.previous_sibling(self.arena).is_some()
                || parent.previous_sibling(self.arena).is_some()
            {
                return;
            }

            let grandparent = parent.parent(self.arena).unwrap();
            if !node_matches!(self.arena, grandparent, NodeValue::Item(..)) {
                return;
            }

            let great_grandparent = grandparent.parent(self.arena).unwrap();
            if !node_matches!(self.arena, great_grandparent, NodeValue::List(..)) {
                return;
            }

            // These are sound only because the exact text that we've matched and
            // the count thereof (i.e. "end") will precisely map to characters in
            // the source document.
            text.drain(..end);

            let adjust = spx.consume(end) + 1;
            assert_eq!(
                sourcepos.start.column,
                parent.data(self.arena,).sourcepos.start.column
            );

            // See tests::fuzz::echaw9. The paragraph doesn't exist in the source,
            // so we remove it.
            if sourcepos.end.column < adjust && node.next_sibling(self.arena).is_none() {
                parent.detach(self.arena);
            } else {
                sourcepos.start.column = adjust;
                parent.data_mut(self.arena).sourcepos.start.column = adjust;
            }

            grandparent.data_mut(self.arena).value =
                NodeValue::TaskItem(if symbol == ' ' { None } else { Some(symbol) });

            if let NodeValue::List(ref mut list) = &mut great_grandparent.data_mut(self.arena).value
            {
                list.is_task_list = true;
            }
        }
    }

    fn parse_reference_inline(
        &self,
        content: &str,
    ) -> Option<(usize, Option<(String, ResolvedReference)>)> {
        let mut scanner = inlines::Scanner::new();

        let mut lab: String = match scanner.link_label(content) {
            Some(lab) if !lab.is_empty() => lab.to_string(),
            _ => return None,
        };

        if scanner.peek_byte(content) != Some(b':') {
            return None;
        }

        scanner.pos += 1;
        scanner.spnl(content);
        let (url, matchlen) = match inlines::manual_scan_link_url(&content[scanner.pos..]) {
            Some((url, matchlen)) => (url.to_string(), matchlen),
            None => return None,
        };
        scanner.pos += matchlen;

        let beforetitle = scanner.pos;
        scanner.spnl(content);
        let title_search = if scanner.pos == beforetitle {
            None
        } else {
            scanners::link_title(&content[scanner.pos..])
        };
        let title = match title_search {
            Some(matchlen) => {
                let t = &content[scanner.pos..scanner.pos + matchlen];
                scanner.pos += matchlen;
                t.to_string()
            }
            _ => {
                scanner.pos = beforetitle;
                String::new()
            }
        };

        scanner.skip_spaces(content);
        if !scanner.skip_line_end(content) {
            if !title.is_empty() {
                scanner.pos = beforetitle;
                scanner.skip_spaces(content);
                if !scanner.skip_line_end(content) {
                    return None;
                }
            } else {
                return None;
            }
        }

        lab = strings::normalize_label(&lab, Case::Fold);
        let mut rr = None;
        if !lab.is_empty() {
            if !self.refmap.map.contains_key(&lab) {
                rr = Some((
                    lab,
                    ResolvedReference {
                        url: strings::clean_url(&url).into(),
                        title: strings::clean_title(&title).into(),
                    },
                ));
            }
        }

        Some((scanner.pos, rr))
    }
}

enum AddTextResult {
    LiteralText,
    HtmlBlock(u8),
    Otherwise,
}

fn parse_list_marker(
    line: &str,
    mut pos: usize,
    interrupts_paragraph: bool,
) -> Option<(usize, NodeList)> {
    let bytes = line.as_bytes();
    let mut c = bytes[pos];
    let startpos = pos;

    if c == b'*' || c == b'-' || c == b'+' {
        pos += 1;
        if !isspace(bytes[pos]) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while strings::is_space_or_tab(bytes[i]) {
                i += 1;
            }
            if bytes[i] == b'\n' {
                return None;
            }
        }

        return Some((
            pos - startpos,
            NodeList {
                list_type: ListType::Bullet,
                marker_offset: 0,
                padding: 0,
                start: 1,
                delimiter: ListDelimType::Period,
                bullet_char: c,
                tight: false,
                is_task_list: false,
            },
        ));
    } else if isdigit(c) {
        let mut start: usize = 0;
        let mut digits = 0;

        loop {
            start = (10 * start) + (bytes[pos] - b'0') as usize;
            pos += 1;
            digits += 1;

            if !(digits < 9 && isdigit(bytes[pos])) {
                break;
            }
        }

        if interrupts_paragraph && start != 1 {
            return None;
        }

        c = bytes[pos];
        if c != b'.' && c != b')' {
            return None;
        }

        pos += 1;

        if !isspace(bytes[pos]) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while strings::is_space_or_tab(bytes[i]) {
                i += 1;
            }
            if strings::is_line_end_char(bytes[i]) {
                return None;
            }
        }

        return Some((
            pos - startpos,
            NodeList {
                list_type: ListType::Ordered,
                marker_offset: 0,
                padding: 0,
                start,
                delimiter: if c == b'.' {
                    ListDelimType::Period
                } else {
                    ListDelimType::Paren
                },
                bullet_char: 0,
                tight: false,
                is_task_list: false,
            },
        ));
    }

    None
}

fn lists_match(list_data: &NodeList, item_data: &NodeList) -> bool {
    list_data.list_type == item_data.list_type
        && list_data.delimiter == item_data.delimiter
        && list_data.bullet_char == item_data.bullet_char
}

fn reopen_ast_nodes<'a>(arena: &'a mut Arena, mut ast: Node) {
    loop {
        ast.data_mut(arena).open = true;
        ast = match ast.parent(arena) {
            Some(p) => p,
            None => return,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutolinkType {
    Uri,
    Email,
}

pub(crate) struct Spx(VecDeque<(Sourcepos, usize)>);

impl Spx {
    // Sourcepos end column `e` of a node determined by advancing through `spx`
    // until `i` bytes of input are seen.
    //
    // For each element `(sp, x)` in `spx`:
    // - if remaining `i` is greater than the byte count `x`,
    //     set `i -= x` and continue.
    // - if remaining `i` is equal to the byte count `x`,
    //     set `e = sp.end.column` and finish.
    // - if remaining `i` is less than the byte count `x`,
    //     assert `sp.end.column - sp.start.column + 1 == x || i == 0` (1),
    //     set `e = sp.start.column + i - 1` and finish.
    //
    // (1) If `x` doesn't equal the range covered between the start and end column,
    //     there's no way to determine sourcepos within the range. This is a bug if
    //     it happens; it suggests we've matched an email autolink with some smart
    //     punctuation in it, or worse.
    //
    //     The one exception is if `i == 0`. Given nothing to consume, we can
    //     happily restore what we popped, returning `sp.start.column - 1` for the
    //     end column of the original node.
    pub(crate) fn consume(&mut self, mut rem: usize) -> usize {
        while let Some((sp, x)) = self.0.pop_front() {
            match rem.cmp(&x) {
                Ordering::Greater => rem -= x,
                Ordering::Equal => return sp.end.column,
                Ordering::Less => {
                    assert!((sp.end.column - sp.start.column + 1 == x) || rem == 0);
                    self.0.push_front((
                        (
                            sp.start.line,
                            sp.start.column + rem,
                            sp.end.line,
                            sp.end.column,
                        )
                            .into(),
                        x - rem,
                    ));
                    return sp.start.column + rem - 1;
                }
            }
        }
        unreachable!();
    }
}
