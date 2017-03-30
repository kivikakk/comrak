#![allow(unused_variables)]

extern crate typed_arena;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod arena_tree;
mod scanners;
mod html;
mod ctype;
mod node;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::cmp::min;
use std::collections::BTreeSet;
use std::io::Read;
use std::mem;

use typed_arena::Arena;

pub use html::format_document;
use arena_tree::Node;
use ctype::{isspace, ispunct, isdigit};
use node::{NodeValue, Ast, AstCell, NodeCodeBlock, NodeHeading, NodeList, ListType, ListDelimType,
           make_block};

fn main() {
    let mut buf = vec![];
    std::io::stdin().read_to_end(&mut buf).unwrap();
    let arena = Arena::new();
    let n = parse_document(&arena, &buf, 0);
    print!("{}", format_document(n));
}

pub fn parse_document<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                          buffer: &[u8],
                          options: u32)
                          -> &'a Node<'a, AstCell> {
    let root: &'a Node<'a, AstCell> = arena.alloc(Node::new(RefCell::new(Ast {
        value: NodeValue::Document,
        content: vec![],
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        open: true,
        last_line_blank: false,
    })));
    let mut parser = Parser::new(arena, root, options);
    parser.feed(buffer, true);
    parser.finish()
}

const TAB_STOP: usize = 8;
const CODE_INDENT: usize = 4;

struct Parser<'a> {
    arena: &'a Arena<Node<'a, AstCell>>,
    root: &'a Node<'a, AstCell>,
    current: &'a Node<'a, AstCell>,
    line_number: u32,
    offset: usize,
    column: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    partially_consumed_tab: bool,
    last_line_length: usize,
    linebuf: Vec<u8>,
    last_buffer_ended_with_cr: bool,
}

impl<'a> Parser<'a> {
    fn new(arena: &'a Arena<Node<'a, AstCell>>,
           root: &'a Node<'a, AstCell>,
           options: u32)
           -> Parser<'a> {
        Parser {
            arena: arena,
            root: root,
            current: root,
            line_number: 0,
            offset: 0,
            column: 0,
            first_nonspace: 0,
            first_nonspace_column: 0,
            indent: 0,
            blank: false,
            partially_consumed_tab: false,
            last_line_length: 0,
            linebuf: vec![],
            last_buffer_ended_with_cr: false,
        }
    }

    fn feed(&mut self, mut buffer: &[u8], eof: bool) {
        if self.last_buffer_ended_with_cr && buffer[0] == 10 {
            buffer = &buffer[1..];
        }
        self.last_buffer_ended_with_cr = false;

        while buffer.len() > 0 {
            let mut process = false;
            let mut eol = 0;
            while eol < buffer.len() {
                if is_line_end_char(&buffer[eol]) {
                    process = true;
                    break;
                }
                if buffer[eol] == 0 {
                    break;
                }
                eol += 1;
            }

            if eol >= buffer.len() && eof {
                process = true;
            }

            if process {
                if self.linebuf.len() > 0 {
                    self.linebuf.extend_from_slice(&buffer[0..eol]);
                    let linebuf = mem::replace(&mut self.linebuf, vec![]);
                    self.process_line(&linebuf);
                } else {
                    self.process_line(&buffer[0..eol]);
                }
            } else {
                if eol < buffer.len() && buffer[eol] == 0 {
                    self.linebuf.extend_from_slice(&buffer[0..eol]);
                    self.linebuf.extend_from_slice(&[239, 191, 189]);
                    eol += 1;
                } else {
                    self.linebuf.extend_from_slice(&buffer[0..eol]);
                }
            }

            buffer = &buffer[eol..];
            if buffer.len() > 0 && buffer[0] == 13 {
                buffer = &buffer[1..];
                if buffer.len() == 0 {
                    self.last_buffer_ended_with_cr = true;
                }
            }
            if buffer.len() > 0 && buffer[0] == 10 {
                buffer = &buffer[1..];
            }
        }
    }

    fn find_first_nonspace(&mut self, line: &mut Vec<u8>) {
        self.first_nonspace = self.offset;
        self.first_nonspace_column = self.column;
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        while let Some(&c) = peek_at(line, self.first_nonspace) {
            match c as char {
                ' ' => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += 1;
                    chars_to_tab -= 1;
                    if chars_to_tab == 0 {
                        chars_to_tab = TAB_STOP;
                    }
                }
                '\t' => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += chars_to_tab;
                    chars_to_tab = TAB_STOP;
                }
                _ => break,
            }

        }

        self.indent = self.first_nonspace_column - self.column;
        self.blank = peek_at(line, self.first_nonspace).map_or(false, is_line_end_char);
    }

    fn process_line(&mut self, buffer: &[u8]) {
        let mut line: Vec<u8> = buffer.into();
        if line.len() == 0 || !is_line_end_char(&line[line.len() - 1]) {
            line.push(10);
        }

        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        if self.line_number == 0 && line.len() >= 3 && &line[0..3] == &[0xef, 0xbb, 0xbf] {
            self.offset += 3;
        }

        self.line_number += 1;

        let mut all_matched = true;
        if let Some(last_matched_container) = self.check_open_blocks(&mut line, &mut all_matched) {
            let mut container = last_matched_container;
            let current = self.current;
            self.open_new_blocks(&mut container, &mut line, all_matched);

            if current.same_node(self.current) {
                self.add_text_to_container(container, last_matched_container, &mut line);
            }
        }

        self.last_line_length = line.len();
        if self.last_line_length > 0 && line[self.last_line_length - 1] == '\n' as u8 {
            self.last_line_length -= 1;
        }
        if self.last_line_length > 0 && line[self.last_line_length - 1] == '\r' as u8 {
            self.last_line_length -= 1;
        }
    }

    fn check_open_blocks(&mut self,
                         line: &mut Vec<u8>,
                         all_matched: &mut bool)
                         -> Option<&'a Node<'a, AstCell>> {
        let mut should_continue = true;
        *all_matched = false;
        let mut container = self.root;

        'done: loop {
            while container.last_child_is_open() {
                container = container.last_child().unwrap();
                let ast = &mut *container.data.borrow_mut();

                self.find_first_nonspace(line);

                match ast.value {
                    NodeValue::BlockQuote => {
                        if !self.parse_block_quote_prefix(line) {
                            break 'done;
                        }
                    }
                    NodeValue::Item(ref nl) => {
                        if !self.parse_node_item_prefix(line, container, nl) {
                            break 'done;
                        }
                    }
                    NodeValue::CodeBlock(..) => {
                        if !self.parse_code_block_prefix(
                            line, container, ast, &mut should_continue) {
                            break 'done;
                        }
                    }
                    NodeValue::Heading(..) => {
                        break 'done;
                    }
                    NodeValue::HtmlBlock(..) => {
                        // TODO
                        assert!(false);
                        // if !self.parse_html_block_prefix(container) {
                        //     break 'done;
                        // }
                    }
                    NodeValue::Paragraph => {
                        if self.blank {
                            break 'done;
                        }
                    }
                    _ => {}
                }
            }

            *all_matched = true;
            break 'done;
        }

        if !*all_matched {
            container = container.parent().unwrap();
        }

        if !should_continue {
            None
        } else {
            Some(container)
        }
    }

    fn open_new_blocks(&mut self,
                       container: &mut &'a Node<'a, AstCell>,
                       line: &mut Vec<u8>,
                       all_matched: bool) {
        let mut matched: usize = 0;
        let mut nl: NodeList = NodeList::default();
        let mut maybe_lazy = match &self.current.data.borrow().value {
            &NodeValue::Paragraph => true,
            _ => false,
        };

        while match &container.data.borrow().value {
            &NodeValue::CodeBlock(..) |
            &NodeValue::HtmlBlock(..) => false,
            _ => true,
        } {
            self.find_first_nonspace(line);
            let indented = self.indent >= CODE_INDENT;

            if !indented && peek_at(line, self.first_nonspace) == Some(&('>' as u8)) {
                let blockquote_startpos = self.first_nonspace;
                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if peek_at(line, self.offset).map_or(false, is_space_or_tab) {
                    self.advance_offset(line, 1, true);
                }
                *container =
                    self.add_child(*container, NodeValue::BlockQuote, blockquote_startpos + 1);
            } else if !indented &&
                      unwrap_into(scanners::atx_heading_start(line, self.first_nonspace),
                                  &mut matched) {
                let heading_startpos = self.first_nonspace;
                let offset = self.offset;
                self.advance_offset(line, heading_startpos + matched - offset, false);
                *container = self.add_child(*container,
                                            NodeValue::Heading(NodeHeading::default()),
                                            heading_startpos + 1);

                let mut hashpos =
                    line[self.first_nonspace..].iter().position(|&c| c == '#' as u8).unwrap() +
                    self.first_nonspace;
                let mut level = 0;
                while peek_at(line, hashpos) == Some(&('#' as u8)) {
                    level += 1;
                    hashpos += 1;
                }

                container.data.borrow_mut().value = NodeValue::Heading(NodeHeading {
                    level: level,
                    setext: false,
                });

            } else if !indented &&
                      unwrap_into(scanners::open_code_fence(line, self.first_nonspace),
                                  &mut matched) {
                let first_nonspace = self.first_nonspace;
                let offset = self.offset;
                let ncb = NodeCodeBlock {
                    fenced: true,
                    fence_char: *peek_at(line, first_nonspace).unwrap(),
                    fence_length: matched,
                    fence_offset: first_nonspace - offset,
                    info: String::new(),
                    literal: vec![],
                };
                *container =
                    self.add_child(*container, NodeValue::CodeBlock(ncb), first_nonspace + 1);
                self.advance_offset(line, first_nonspace + matched - offset, false);
            } else if !indented &&
                      unwrap_into(scanners::html_block_start(line, self.first_nonspace),
                                  &mut matched) ||
                      match &container.data.borrow().value {
                &NodeValue::Paragraph => false,
                _ => {
                    unwrap_into(scanners::html_block_start_7(line, self.first_nonspace),
                                &mut matched)
                }
            } {
                // TODO

            } else if !indented &&
                      match &container.data.borrow().value {
                &NodeValue::Paragraph => {
                    unwrap_into(scanners::setext_heading_line(line, self.first_nonspace),
                                &mut matched)
                }
                _ => false,
            } {
                // TODO

            } else if !indented &&
                      match (&container.data.borrow().value, all_matched) {
                (&NodeValue::Paragraph, false) => false,
                _ => {
                    unwrap_into(scanners::thematic_break(line, self.first_nonspace),
                                &mut matched)
                }
            } {
                let offset = self.first_nonspace + 1;
                *container = self.add_child(*container, NodeValue::ThematicBreak, offset);
                let adv = line.len() - 1 - self.offset;
                self.advance_offset(line, adv, false);
            } else if (!indented ||
                       match &container.data.borrow().value {
                &NodeValue::List(..) => true,
                _ => false,
            }) &&
                      unwrap_into_2(parse_list_marker(line,
                                                      self.first_nonspace,
                                                      match &container.data.borrow().value {
                                                          &NodeValue::Paragraph => true,
                                                          _ => false,
                                                      }),
                                    &mut matched,
                                    &mut nl) {
                let offset = self.first_nonspace + matched - self.offset;
                self.advance_offset(line, offset, false);
                let (save_partially_consumed_tab, save_offset, save_column) =
                    (self.partially_consumed_tab, self.offset, self.column);

                while self.column - save_column <= 5 &&
                      peek_at(line, self.offset).map_or(false, is_space_or_tab) {
                    self.advance_offset(line, 1, true);
                }

                let i = self.column - save_column;
                if i >= 5 || i < 1 || peek_at(line, self.offset).map_or(false, is_line_end_char) {
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

                let offset = self.first_nonspace + 1;
                if match &container.data.borrow().value {
                    &NodeValue::List(ref mnl) => !lists_match(&nl, mnl),
                    _ => true,
                } {
                    *container = self.add_child(*container, NodeValue::List(nl), offset);
                }

                let offset = self.first_nonspace + 1;
                *container = self.add_child(*container, NodeValue::Item(nl), offset);
            } else if indented && !maybe_lazy && !self.blank {
                self.advance_offset(line, CODE_INDENT, true);
                let ncb = NodeCodeBlock {
                    fenced: false,
                    fence_char: 0,
                    fence_length: 0,
                    fence_offset: 0,
                    info: String::new(),
                    literal: vec![],
                };
                let offset = self.offset + 1;
                *container = self.add_child(*container, NodeValue::CodeBlock(ncb), offset);
            } else {
                break;
            }

            if container.data.borrow().value.accepts_lines() {
                break;
            }

            maybe_lazy = false;
        }
    }

    fn advance_offset(&mut self, line: &mut Vec<u8>, mut count: usize, columns: bool) {
        while count > 0 {
            match peek_at(line, self.offset) {
                None => break,
                Some(&9) => {
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
                Some(_) => {
                    self.partially_consumed_tab = false;
                    self.offset += 1;
                    self.column += 1;
                    count -= 1;
                }
            }
        }
    }

    fn parse_block_quote_prefix(&mut self, line: &mut Vec<u8>) -> bool {
        let indent = self.indent;
        if indent <= 3 && peek_at(line, self.first_nonspace) == Some(&('>' as u8)) {
            self.advance_offset(line, indent + 1, true);

            if peek_at(line, self.offset).map_or(false, is_space_or_tab) {
                self.advance_offset(line, 1, true);
            }

            return true;
        }

        false
    }

    fn parse_node_item_prefix(&mut self,
                              line: &mut Vec<u8>,
                              container: &'a Node<'a, AstCell>,
                              nl: &NodeList)
                              -> bool {
        if self.indent >= nl.marker_offset + nl.padding {
            self.advance_offset(line, nl.marker_offset + nl.padding, true);
            true
        } else if self.blank && container.first_child().is_some() {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            true
        } else {
            false
        }
    }

    fn parse_code_block_prefix(&mut self,
                               line: &mut Vec<u8>,
                               container: &'a Node<'a, AstCell>,
                               ast: &mut Ast,
                               should_continue: &mut bool)
                               -> bool {
        let ncb = match ast.value {
                NodeValue::CodeBlock(ref ncb) => Some(ncb.clone()),
                _ => None,
            }
            .unwrap();

        if !ncb.fenced && self.indent >= CODE_INDENT {
            self.advance_offset(line, CODE_INDENT, true);
            return true;
        } else if !ncb.fenced && self.blank {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            return true;
        }

        let matched = if self.indent <= 3 &&
                         peek_at(line, self.first_nonspace)
            .map_or(false, |&c| c == ncb.fence_char) {
            scanners::close_code_fence(line, self.first_nonspace).unwrap_or(0)
        } else {
            0
        };

        if matched >= ncb.fence_length {
            *should_continue = false;
            self.advance_offset(line, matched, false);
            self.current = self.finalize_borrowed(container, ast).unwrap();
            return false;

        }

        let mut i = ncb.fence_offset;
        while i > 0 && peek_at(line, self.offset).map_or(false, is_space_or_tab) {
            self.advance_offset(line, 1, true);
            i -= 1;
        }
        true
    }

    fn add_child(&mut self,
                 mut parent: &'a Node<'a, AstCell>,
                 value: NodeValue,
                 start_column: usize)
                 -> &'a Node<'a, AstCell> {
        while !parent.can_contain_type(&value) {
            parent = self.finalize(parent).unwrap();
        }

        let child = make_block(value, self.line_number, start_column);
        let node = self.arena.alloc(Node::new(RefCell::new(child)));
        parent.append(node);
        node
    }

    fn add_text_to_container(&mut self,
                             mut container: &'a Node<'a, AstCell>,
                             last_matched_container: &'a Node<'a, AstCell>,
                             line: &mut Vec<u8>) {
        self.find_first_nonspace(line);

        if self.blank {
            if let Some(last_child) = container.last_child() {
                last_child.data.borrow_mut().last_line_blank = true;
            }
        }

        container.data.borrow_mut().last_line_blank = self.blank &&
                                                      match &container.data.borrow().value {
            &NodeValue::BlockQuote |
            &NodeValue::Heading(..) |
            &NodeValue::ThematicBreak => false,
            &NodeValue::CodeBlock(ref ncb) => !ncb.fenced,
            &NodeValue::Item(..) => {
                container.first_child().is_some() ||
                container.data.borrow().start_line != self.line_number
            }
            _ => true,
        };

        let mut tmp = container;
        while let Some(parent) = tmp.parent() {
            parent.data.borrow_mut().last_line_blank = false;
            tmp = parent;
        }

        if !self.current.same_node(last_matched_container) &&
           container.same_node(last_matched_container) && !self.blank &&
           match &self.current.data.borrow().value {
            &NodeValue::Paragraph => true,
            _ => false,
        } {
            self.add_line(self.current, line);
        } else {
            while !self.current.same_node(last_matched_container) {
                self.current = self.finalize(self.current).unwrap();
            }

            // TODO: remove this awful clone
            let node_type = container.data.borrow().value.clone();
            match &node_type {
                &NodeValue::CodeBlock(..) => {
                    self.add_line(container, line);
                }
                &NodeValue::HtmlBlock(html_block_type) => {
                    self.add_line(container, line);

                    let matches_end_condition = match html_block_type {
                        1 => scanners::html_block_end_1(line, self.first_nonspace).is_some(),
                        2 => scanners::html_block_end_2(line, self.first_nonspace).is_some(),
                        3 => scanners::html_block_end_3(line, self.first_nonspace).is_some(),
                        4 => scanners::html_block_end_4(line, self.first_nonspace).is_some(),
                        5 => scanners::html_block_end_5(line, self.first_nonspace).is_some(),
                        _ => false,
                    };

                    if matches_end_condition {
                        container = self.finalize(container).unwrap();
                    }
                }
                _ => {
                    if self.blank {
                        // do nothing
                    } else if container.data.borrow().value.accepts_lines() {
                        match &container.data.borrow().value {
                            &NodeValue::Heading(ref nh) => {
                                if !nh.setext {
                                    chop_trailing_hashtags(line);
                                }
                            }
                            _ => (),
                        };
                        let count = self.first_nonspace - self.offset;
                        self.advance_offset(line, count, false);
                        self.add_line(container, line);
                    } else {
                        let start_column = self.first_nonspace + 1;
                        container = self.add_child(container, NodeValue::Paragraph, start_column);
                        let count = self.first_nonspace - self.offset;
                        self.advance_offset(line, count, false);
                        self.add_line(container, line);
                    }
                }
            }

            self.current = container;
        }
    }

    fn add_line(&mut self, node: &'a Node<'a, AstCell>, line: &mut Vec<u8>) {
        let mut ast = node.data.borrow_mut();
        assert!(ast.open);
        if self.partially_consumed_tab {
            self.offset += 1;
            let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
            for i in 0..chars_to_tab {
                ast.content.push(' ' as u8);
            }
        }
        ast.content.extend_from_slice(&line[self.offset..]);
    }

    fn finish(&mut self) -> &'a Node<'a, AstCell> {
        if self.linebuf.len() > 0 {
            let linebuf = mem::replace(&mut self.linebuf, vec![]);
            self.process_line(&linebuf);
        }

        self.finalize_document();

        self.root
    }

    fn finalize_document(&mut self) {
        while !self.current.same_node(self.root) {
            self.current = self.finalize(&self.current).unwrap();
        }

        self.finalize(self.root);
        self.process_inlines();
    }

    fn finalize(&self, node: &'a Node<'a, AstCell>) -> Option<&'a Node<'a, AstCell>> {
        self.finalize_borrowed(node, &mut *node.data.borrow_mut())
    }

    fn finalize_borrowed(&self,
                         node: &'a Node<'a, AstCell>,
                         ast: &mut Ast)
                         -> Option<&'a Node<'a, AstCell>> {
        assert!(ast.open);
        ast.open = false;

        if self.linebuf.len() == 0 {
            ast.end_line = self.line_number;
            ast.end_column = self.last_line_length;
        } else if match &ast.value {
            &NodeValue::Document => true,
            &NodeValue::CodeBlock(ref ncb) => ncb.fenced,
            &NodeValue::Heading(ref nh) => nh.setext,
            _ => false,
        } {
            ast.end_line = self.line_number;
            ast.end_column = self.linebuf.len();
            if ast.end_column > 0 && self.linebuf[ast.end_column - 1] == '\n' as u8 {
                ast.end_column -= 1;
            }
            if ast.end_column > 0 && self.linebuf[ast.end_column - 1] == '\r' as u8 {
                ast.end_column -= 1;
            }
        } else {
            ast.end_line = self.line_number - 1;
            ast.end_column = self.last_line_length;
        }

        let content = &mut ast.content;

        match &mut ast.value {
            &mut NodeValue::Paragraph => {
                // TODO: remove reference links
                /*
                    while (cmark_strbuf_at(node_content, 0) == '[' &&
                           (pos = cmark_parse_reference_inline(parser->mem, node_content,
                                                               parser->refmap))) {

                      cmark_strbuf_drop(node_content, pos);
                    }
                    if (is_blank(node_content, 0)) {
                      // remove blank node (former reference def)
                      cmark_node_free(b);
                    }
                */
            }
            &mut NodeValue::CodeBlock(ref mut ncb) => {
                if !ncb.fenced {
                    remove_trailing_blank_lines(content);
                    content.push('\n' as u8);
                } else {
                    let mut pos = 0;
                    while pos < content.len() {
                        if is_line_end_char(&content[pos]) {
                            break;
                        }
                        pos += 1;
                    }
                    assert!(pos < content.len());

                    // TODO: unescape HTML, etc.
                    let mut tmp = content[0..pos].to_vec();
                    trim(&mut tmp);
                    ncb.info = String::from_utf8(tmp).unwrap();

                    // TODO: boundscheck lol
                    if content[pos] == '\r' as u8 {
                        pos += 1;
                    }
                    if content[pos] == '\n' as u8 {
                        pos += 1;
                    }

                    for i in 0..pos {
                        content.remove(0);
                    }
                }
                ncb.literal = content.clone();
                content.clear();
            }
            &mut NodeValue::HtmlBlock(..) => {
                assert!(false)
                // TODO
            }
            &mut NodeValue::List(ref mut nl) => {
                nl.tight = true;
                let mut ch = node.first_child();

                while let Some(item) = ch {
                    if item.data.borrow().last_line_blank && item.next_sibling().is_some() {
                        nl.tight = false;
                        break;
                    }

                    let mut subch = item.first_child();
                    while let Some(subitem) = subch {
                        if subitem.ends_with_blank_line() &&
                           (item.next_sibling().is_some() || subitem.next_sibling().is_some()) {
                            nl.tight = false;
                            break;
                        }
                        subch = subitem.next_sibling();
                    }

                    if !nl.tight {
                        break;
                    }

                    ch = item.next_sibling();
                }
            }
            _ => (),
        }

        node.parent()
    }

    fn process_inlines(&mut self) {
        self.process_inlines_node(self.root);
    }

    fn process_inlines_node(&mut self, node: &'a Node<'a, AstCell>) {
        if node.data.borrow().value.contains_inlines() {
            self.parse_inlines(node);
        }

        for n in node.children() {
            self.process_inlines_node(n);
        }
    }

    fn parse_inlines(&mut self, node: &'a Node<'a, AstCell>) {
        let mut subj = Subject {
            arena: self.arena,
            input: node.data.borrow().content.clone(),
            pos: 0,
            delimiters: vec![],
        };
        rtrim(&mut subj.input);

        while !subj.eof() && self.parse_inline(&mut subj, node) {}

        self.process_emphasis(&mut subj, -1);
        // TODO
        // while subj.last_delim { subj.pop_bracket() }
        // while subj.last_bracket { subj.pop_bracket() }
    }

    fn process_emphasis(&mut self, subj: &mut Subject<'a>, stack_bottom: i32) {
        let mut closer = subj.delimiters.len() as i32 - 1;
        let mut openers_bottom: Vec<[i32; 128]> = vec![];
        for i in 0..3 {
            let mut a = [-1; 128];
            a['*' as usize] = stack_bottom;
            a['_' as usize] = stack_bottom;
            a['\'' as usize] = stack_bottom;
            a['"' as usize] = stack_bottom;
            openers_bottom.push(a)
        }

        while closer != -1 && closer - 1 > stack_bottom {
            closer -= 1;
        }

        while closer != -1 && (closer as usize) < subj.delimiters.len() {
            if subj.delimiters[closer as usize].can_close {
                let mut opener = closer - 1;
                let mut opener_found = false;

                while opener != -1 && opener != stack_bottom &&
                      opener !=
                      openers_bottom[subj.delimiters[closer as usize]
                    .inl
                    .data
                    .borrow_mut()
                    .value
                    .text()
                    .unwrap()
                    .len() % 3][subj.delimiters[closer as usize]
                    .delim_char as usize] {
                    if subj.delimiters[opener as usize].can_open &&
                       subj.delimiters[opener as usize].delim_char ==
                       subj.delimiters[closer as usize].delim_char {
                        let odd_match = (subj.delimiters[closer as usize].can_open ||
                                         subj.delimiters[opener as usize].can_close) &&
                                        ((subj.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap()
                            .len() +
                                          subj.delimiters[closer as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap()
                            .len()) % 3 == 0);
                        if !odd_match {
                            opener_found = true;
                            break;
                        }
                    }
                    opener -= 1;
                }
                let old_closer = closer;

                if subj.delimiters[closer as usize].delim_char == '*' as u8 ||
                   subj.delimiters[closer as usize].delim_char == '_' as u8 {
                    if opener_found {
                        closer = subj.insert_emph(opener, closer);
                    } else {
                        closer += 1;
                    }
                } else if subj.delimiters[closer as usize].delim_char == '\'' as u8 {
                    *subj.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "’".as_bytes().to_vec();
                    if opener_found {
                        *subj.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "‘".as_bytes().to_vec();
                    }
                    closer += 1;
                } else if subj.delimiters[closer as usize].delim_char == '"' as u8 {
                    *subj.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "”".as_bytes().to_vec();
                    if opener_found {
                        *subj.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "“".as_bytes().to_vec();
                    }
                    closer += 1;
                }
                if !opener_found {
                    let ix = subj.delimiters[old_closer as usize]
                        .inl
                        .data
                        .borrow_mut()
                        .value
                        .text()
                        .unwrap()
                        .len() % 3;
                    openers_bottom[ix][subj.delimiters[old_closer as usize].delim_char as usize] =
                        old_closer - 1;
                    if !subj.delimiters[old_closer as usize].can_open {
                        subj.delimiters.remove(old_closer as usize);
                    }
                }
            } else {
                closer += 1;
            }
        }

        // TODO truncate instead!
        while subj.delimiters.len() > (stack_bottom + 1) as usize {
            subj.delimiters.pop();
        }
    }

    fn parse_inline(&mut self, subj: &mut Subject<'a>, node: &'a Node<'a, AstCell>) -> bool {
        let new_inl: Option<&'a Node<'a, AstCell>>;
        let c = match subj.peek_char() {
            None => return false,
            Some(ch) => *ch as char,
        };

        match c {
            '\0' => return false,
            '\r' | '\n' => new_inl = Some(subj.handle_newline()),
            '*' | '_' | '"' => new_inl = Some(subj.handle_delim(c as u8)),
            // TODO
            _ => {
                let endpos = subj.find_special_char();
                let mut contents = subj.input[subj.pos..endpos].to_vec();
                subj.pos = endpos;

                if subj.peek_char().map_or(false, is_line_end_char) {
                    rtrim(&mut contents);
                }

                new_inl = Some(make_inline(self.arena, NodeValue::Text(contents)));
            }
        }

        if let Some(inl) = new_inl {
            node.append(inl);
        }

        true
    }
}

// typedef struct subject{
// cmark_mem *mem;
// cmark_chunk input;
// bufsize_t pos;
// cmark_reference_map *refmap;
// delimiter *last_delim;
// bracket *last_bracket;
// bufsize_t backticks[MAXBACKTICKS + 1];
// bool scanned_for_backticks;
// } subject;
//
// typedef struct bracket {
// struct bracket *previous;
// struct delimiter *previous_delimiter;
// cmark_node *inl_text;
// bufsize_t position;
// bool image;
// bool active;
// bool bracket_after;
// } bracket;
//
// typedef struct delimiter {
// struct delimiter *previous;
// struct delimiter *next;
// cmark_node *inl_text;
// bufsize_t length;
// int position;
// unsigned char delim_char;
// int can_open;
// int can_close;
// int active;
// } delimiter;
//

struct Subject<'a> {
    arena: &'a Arena<Node<'a, AstCell>>,
    input: Vec<u8>,
    pos: usize,
    delimiters: Vec<Delimiter<'a>>,
}

impl<'a> Subject<'a> {
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char<'b>(&'b self) -> Option<&'b u8> {
        if self.eof() {
            return None;
        }

        let ref c = self.input[self.pos];
        assert!(*c > 0);
        Some(c)
    }

    fn find_special_char(&self) -> usize {
        lazy_static! {
            static ref SPECIAL_CHARS: BTreeSet<u8> =
                ['\n' as u8,
                '\r' as u8,
                '_' as u8,
                '*' as u8,
                '"' as u8,
                /* TODO
                '\\' as u8,
                '`' as u8,
                '&' as u8,
                '[' as u8,
                ']' as u8,
                '<' as u8,
                '!' as u8,
                */
                ].iter().cloned().collect();
        }

        for n in self.pos..self.input.len() {
            if SPECIAL_CHARS.contains(&self.input[n]) {
                return n;
            }
        }

        self.input.len()
    }

    fn handle_newline(&mut self) -> &'a Node<'a, AstCell> {
        let nlpos = self.pos;
        if self.input[self.pos] == '\r' as u8 {
            self.pos += 1;
        }
        if self.input[self.pos] == '\n' as u8 {
            self.pos += 1;
        }
        self.skip_spaces();
        if nlpos > 1 && self.input[nlpos - 1] == ' ' as u8 && self.input[nlpos - 2] == ' ' as u8 {
            make_inline(self.arena, NodeValue::LineBreak)
        } else {
            make_inline(self.arena, NodeValue::SoftBreak)
        }
    }

    fn skip_spaces(&mut self) -> bool {
        let mut skipped = false;
        while self.peek_char().map_or(false, |&c| c == ' ' as u8 || c == '\t' as u8) {
            self.pos += 1;
            skipped = true;
        }
        skipped
    }

    fn handle_delim(&mut self, c: u8) -> &'a Node<'a, AstCell> {
        let (numdelims, can_open, can_close) = self.scan_delims(c);

        let contents = self.input[self.pos - numdelims..self.pos].to_vec();
        let inl = make_inline(self.arena, NodeValue::Text(contents));

        if (can_open || can_close) && c != '\'' as u8 && c != '"' as u8 {
            self.push_delimiter(c, can_open, can_close, inl);
        }

        inl
    }

    fn scan_delims(&mut self, c: u8) -> (usize, bool, bool) {
        // Elided: a bunch of UTF-8 processing stuff.
        let before_char = if self.pos == 0 {
            10
        } else {
            self.input[self.pos - 1]
        };

        let mut numdelims = 0;
        if c == '\'' as u8 || c == '"' as u8 {
            numdelims += 1;
            self.pos += 1;
        } else {
            while self.peek_char().map_or(false, |&x| x == c) {
                numdelims += 1;
                self.pos += 1;
            }
        }

        let after_char = if self.eof() { 10 } else { self.input[self.pos] };

        let left_flanking = numdelims > 0 && !isspace(&after_char) &&
                            !(ispunct(&after_char) && !isspace(&before_char) &&
                              !ispunct(&before_char));
        let right_flanking = numdelims > 0 && !isspace(&before_char) &&
                             !(ispunct(&before_char) && !isspace(&after_char) &&
                               !ispunct(&after_char));

        if c == '_' as u8 {
            (numdelims,
             left_flanking && (!right_flanking || ispunct(&before_char)),
             right_flanking && (!left_flanking || ispunct(&after_char)))
        } else if c == '\'' as u8 || c == '"' as u8 {
            (numdelims, left_flanking && !right_flanking, right_flanking)
        } else {
            (numdelims, left_flanking, right_flanking)
        }
    }

    fn push_delimiter(&mut self,
                      c: u8,
                      can_open: bool,
                      can_close: bool,
                      inl: &'a Node<'a, AstCell>) {
        self.delimiters.push(Delimiter {
            inl: inl,
            position: 0,
            delim_char: c,
            can_open: can_open,
            can_close: can_close,
            active: false,
        });
    }

    fn insert_emph(&mut self, opener: i32, mut closer: i32) -> i32 {
        let mut opener_num_chars =
            self.delimiters[opener as usize].inl.data.borrow_mut().value.text().unwrap().len();
        let mut closer_num_chars =
            self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap().len();
        let use_delims = if closer_num_chars >= 2 && opener_num_chars >= 2 {
            2
        } else {
            1
        };

        opener_num_chars -= use_delims;
        closer_num_chars -= use_delims;
        self.delimiters[opener as usize]
            .inl
            .data
            .borrow_mut()
            .value
            .text()
            .unwrap()
            .truncate(opener_num_chars);
        self.delimiters[closer as usize]
            .inl
            .data
            .borrow_mut()
            .value
            .text()
            .unwrap()
            .truncate(closer_num_chars);

        // TODO: just remove the range directly
        let mut delim = closer - 1;
        while delim != -1 && delim != opener {
            self.delimiters.remove(delim as usize);
            delim -= 1;
        }

        let emph = make_inline(self.arena,
                               if use_delims == 1 {
                                   NodeValue::Emph
                               } else {
                                   NodeValue::Strong
                               });

        let mut tmp = self.delimiters[opener as usize].inl.next_sibling().unwrap();
        while !tmp.same_node(self.delimiters[closer as usize].inl) {
            let next = tmp.next_sibling();
            emph.append(tmp);
            if let Some(n) = next {
                tmp = n;
            } else {
                break;
            }
        }
        self.delimiters[opener as usize].inl.insert_after(emph);

        if opener_num_chars == 0 {
            self.delimiters[opener as usize].inl.detach();
            self.delimiters.remove(opener as usize);
            closer -= 1;
        }

        if closer_num_chars == 0 {
            self.delimiters[closer as usize].inl.detach();
            self.delimiters.remove(closer as usize);
        }

        if closer == -1 || (closer as usize) < self.delimiters.len() {
            closer
        } else {
            -1
        }
    }
}

struct Delimiter<'a> {
    inl: &'a Node<'a, AstCell>,
    position: usize,
    delim_char: u8,
    can_open: bool,
    can_close: bool,
    active: bool,
}

fn is_line_end_char(ch: &u8) -> bool {
    match ch {
        &10 | &13 => true,
        _ => false,
    }
}

fn is_space_or_tab(ch: &u8) -> bool {
    match ch {
        &9 | &32 => true,
        _ => false,
    }
}

fn peek_at(line: &mut Vec<u8>, i: usize) -> Option<&u8> {
    line.get(i)
}

fn chop_trailing_hashtags(line: &mut Vec<u8>) {
    rtrim(line);

    let orig_n = line.len() - 1;
    let mut n = orig_n;

    while line[n] == '#' as u8 {
        n -= 1;
        if n == 0 {
            return;
        }
    }

    if n != orig_n && is_space_or_tab(&line[n]) {
        line.truncate(n);
        rtrim(line);
    }
}

fn rtrim(line: &mut Vec<u8>) {
    let mut len = line.len();
    while len > 0 && isspace(&line[len - 1]) {
        line.pop();
        len -= 1;
    }
}

fn ltrim(line: &mut Vec<u8>) {
    let mut len = line.len();
    while len > 0 && isspace(&line[0]) {
        line.remove(0);
        len -= 1;
    }
}

fn trim(line: &mut Vec<u8>) {
    ltrim(line);
    rtrim(line);
}

fn make_inline<'a>(arena: &'a Arena<Node<'a, AstCell>>, value: NodeValue) -> &'a Node<'a, AstCell> {
    let ast = Ast {
        value: value,
        content: vec![],
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        open: false,
        last_line_blank: false,
    };
    arena.alloc(Node::new(RefCell::new(ast)))
}

fn parse_list_marker(line: &mut Vec<u8>,
                     mut pos: usize,
                     interrupts_paragraph: bool)
                     -> Option<(usize, NodeList)> {
    let mut c = match peek_at(line, pos) {
        Some(c) => *c,
        _ => return None,
    };
    let startpos = pos;

    if c == '*' as u8 || c == '-' as u8 || c == '+' as u8 {
        pos += 1;
        if !peek_at(line, pos).map_or(false, isspace) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while peek_at(line, i).map_or(false, is_space_or_tab) {
                i += 1;
            }
            if peek_at(line, i) == Some(&('\n' as u8)) {
                return None;
            }
        }

        return Some((pos - startpos,
                     NodeList {
                         list_type: ListType::Bullet,
                         marker_offset: 0,
                         padding: 0,
                         start: 1,
                         delimiter: ListDelimType::Period,
                         bullet_char: c,
                         tight: false,
                     }));
    } else if isdigit(&c) {
        let mut start: usize = 0;
        let mut digits = 0;

        loop {
            start = (10 * start) + (peek_at(line, pos).unwrap() - '0' as u8) as usize;
            pos += 1;
            digits += 1;

            if !(digits < 9 && peek_at(line, pos).map_or(false, isdigit)) {
                break;
            }
        }

        if interrupts_paragraph && start != 1 {
            return None;
        }

        c = peek_at(line, pos).map_or(0, |&c| c);
        if c != '.' as u8 && c != ')' as u8 {
            return None;
        }

        pos += 1;

        if !peek_at(line, pos).map_or(false, isspace) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while peek_at(line, i).map_or(false, is_space_or_tab) {
                i += 1;
            }
            if peek_at(line, i).map_or(false, is_line_end_char) {
                return None;
            }
        }

        return Some((pos - startpos,
                     NodeList {
                         list_type: ListType::Ordered,
                         marker_offset: 0,
                         padding: 0,
                         start: start,
                         delimiter: if c == '.' as u8 {
                             ListDelimType::Period
                         } else {
                             ListDelimType::Paren
                         },
                         bullet_char: 0,
                         tight: false,
                     }));
    }

    None
}

fn remove_trailing_blank_lines(line: &mut Vec<u8>) {
    let mut i = line.len() - 1;
    loop {
        let c = line[i];

        if c != ' ' as u8 && c != '\t' as u8 && !is_line_end_char(&c) {
            break;
        }

        if i == 0 {
            line.clear();
            return;
        }

        i -= 1;
    }

    for i in i..line.len() {
        let c = line[i];

        if !is_line_end_char(&c) {
            continue;
        }

        line.truncate(i);
        break;
    }
}

fn unwrap_into<T>(t: Option<T>, out: &mut T) -> bool {
    match t {
        Some(v) => {
            *out = v;
            true
        }
        _ => false,
    }
}

fn unwrap_into_2<T, U>(tu: Option<(T, U)>, out_t: &mut T, out_u: &mut U) -> bool {
    match tu {
        Some((t, u)) => {
            *out_t = t;
            *out_u = u;
            true
        }
        _ => false,
    }
}

fn lists_match(list_data: &NodeList, item_data: &NodeList) -> bool {
    list_data.list_type == item_data.list_type && list_data.delimiter == item_data.delimiter &&
    list_data.bullet_char == item_data.bullet_char
}
