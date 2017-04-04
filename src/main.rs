#![allow(unused_variables)]

extern crate unicode_categories;
extern crate typed_arena;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod arena_tree;
mod scanners;
mod html;
mod ctype;
mod node;
mod entity;
mod entity_data;
mod strings;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::cmp::min;
use std::collections::{BTreeSet, HashMap};
use std::io::Read;
use std::mem;

use unicode_categories::UnicodeCategories;
use typed_arena::Arena;

pub use html::format_document;
use arena_tree::Node;
use ctype::{isspace, ispunct, isdigit};
use node::{NodeValue, Ast, AstCell, NodeCodeBlock, NodeHeading, NodeList, ListType, ListDelimType,
           NodeHtmlBlock, NodeLink, make_block};
use strings::*;

fn main() {
    let mut buf = vec![];
    std::io::stdin().read_to_end(&mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let chars: Vec<char> = s.chars().collect::<Vec<_>>();
    let arena = Arena::new();
    let n = parse_document(&arena, &chars, 0);
    print!("{}", format_document(n));
}

pub fn parse_document<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                          buffer: &[char],
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

const TAB_STOP: usize = 4;
const CODE_INDENT: usize = 4;
const MAXBACKTICKS: usize = 80;
const MAX_LINK_LABEL_LENGTH: usize = 1000;

struct Parser<'a> {
    arena: &'a Arena<Node<'a, AstCell>>,
    refmap: HashMap<Vec<char>, Reference>,
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
    linebuf: Vec<char>,
    last_buffer_ended_with_cr: bool,
}

#[derive(Clone)]
struct Reference {
    url: Vec<char>,
    title: Vec<char>,
}

impl<'a> Parser<'a> {
    fn new(arena: &'a Arena<Node<'a, AstCell>>,
           root: &'a Node<'a, AstCell>,
           options: u32)
           -> Parser<'a> {
        Parser {
            arena: arena,
            refmap: HashMap::new(),
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

    fn feed(&mut self, mut buffer: &[char], eof: bool) {
        if self.last_buffer_ended_with_cr && buffer[0] == '\n' {
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
                if buffer[eol] == '\0' {
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
                if eol < buffer.len() && buffer[eol] == '\0' {
                    self.linebuf.extend_from_slice(&buffer[0..eol]);
                    self.linebuf.push('\u{fffd}');
                    eol += 1;
                } else {
                    self.linebuf.extend_from_slice(&buffer[0..eol]);
                }
            }

            buffer = &buffer[eol..];
            if buffer.len() > 0 && buffer[0] == '\r' {
                buffer = &buffer[1..];
                if buffer.len() == 0 {
                    self.last_buffer_ended_with_cr = true;
                }
            }
            if buffer.len() > 0 && buffer[0] == '\n' {
                buffer = &buffer[1..];
            }
        }
    }

    fn find_first_nonspace(&mut self, line: &mut Vec<char>) {
        self.first_nonspace = self.offset;
        self.first_nonspace_column = self.column;
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        while let Some(&c) = line.get(self.first_nonspace) {
            match c {
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
        self.blank = line.get(self.first_nonspace).map_or(false, is_line_end_char);
    }

    fn process_line(&mut self, buffer: &[char]) {
        let mut line: Vec<char> = buffer.into();
        if line.len() == 0 || !is_line_end_char(&line[line.len() - 1]) {
            line.push('\n');
        }

        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        if self.line_number == 0 && line.len() >= 1 && line[0] == '\u{feff}' {
            self.offset += 1;
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
        if self.last_line_length > 0 && line[self.last_line_length - 1] == '\n' {
            self.last_line_length -= 1;
        }
        if self.last_line_length > 0 && line[self.last_line_length - 1] == '\r' {
            self.last_line_length -= 1;
        }
    }

    fn check_open_blocks(&mut self,
                         line: &mut Vec<char>,
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
                    NodeValue::HtmlBlock(ref nhb) => {
                        if !self.parse_html_block_prefix(nhb.block_type) {
                            break 'done;
                        }
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
                       line: &mut Vec<char>,
                       all_matched: bool) {
        let mut matched: usize = 0;
        let mut nl: NodeList = NodeList::default();
        let mut sc: scanners::SetextChar = scanners::SetextChar::Equals;
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

            if !indented && line.get(self.first_nonspace) == Some(&'>') {
                let blockquote_startpos = self.first_nonspace;
                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if line.get(self.offset).map_or(false, is_space_or_tab) {
                    self.advance_offset(line, 1, true);
                }
                *container =
                    self.add_child(*container, NodeValue::BlockQuote, blockquote_startpos + 1);
            } else if !indented &&
                      unwrap_into(scanners::atx_heading_start(&line[self.first_nonspace..]),
                                  &mut matched) {
                let heading_startpos = self.first_nonspace;
                let offset = self.offset;
                self.advance_offset(line, heading_startpos + matched - offset, false);
                *container = self.add_child(*container,
                                            NodeValue::Heading(NodeHeading::default()),
                                            heading_startpos + 1);

                let mut hashpos =
                    line[self.first_nonspace..].iter().position(|&c| c == '#').unwrap() +
                    self.first_nonspace;
                let mut level = 0;
                while line.get(hashpos) == Some(&'#') {
                    level += 1;
                    hashpos += 1;
                }

                container.data.borrow_mut().value = NodeValue::Heading(NodeHeading {
                    level: level,
                    setext: false,
                });

            } else if !indented &&
                      unwrap_into(scanners::open_code_fence(&line[self.first_nonspace..]),
                                  &mut matched) {
                let first_nonspace = self.first_nonspace;
                let offset = self.offset;
                let ncb = NodeCodeBlock {
                    fenced: true,
                    fence_char: *line.get(first_nonspace).unwrap(),
                    fence_length: matched,
                    fence_offset: first_nonspace - offset,
                    info: vec![],
                    literal: vec![],
                };
                *container =
                    self.add_child(*container, NodeValue::CodeBlock(ncb), first_nonspace + 1);
                self.advance_offset(line, first_nonspace + matched - offset, false);
            } else if !indented &&
                      (unwrap_into(scanners::html_block_start(&line[self.first_nonspace..]),
                                   &mut matched) ||
                       match &container.data.borrow().value {
                &NodeValue::Paragraph => false,
                _ => {
                    unwrap_into(scanners::html_block_start_7(&line[self.first_nonspace..]),
                                &mut matched)
                }
            }) {
                let offset = self.first_nonspace + 1;
                let nhb = NodeHtmlBlock {
                    block_type: matched as u8,
                    literal: vec![],
                };

                *container = self.add_child(*container, NodeValue::HtmlBlock(nhb), offset);
            } else if !indented &&
                      match &container.data.borrow().value {
                &NodeValue::Paragraph => {
                    unwrap_into(scanners::setext_heading_line(&line[self.first_nonspace..]),
                                &mut sc)
                }
                _ => false,
            } {
                container.data.borrow_mut().value = NodeValue::Heading(NodeHeading {
                    level: match sc {
                        scanners::SetextChar::Equals => 1,
                        scanners::SetextChar::Hyphen => 2,
                    },
                    setext: true,
                });
                let adv = line.len() - 1 - self.offset;
                self.advance_offset(line, adv, false);
            } else if !indented &&
                      match (&container.data.borrow().value, all_matched) {
                (&NodeValue::Paragraph, false) => false,
                _ => {
                    unwrap_into(scanners::thematic_break(&line[self.first_nonspace..]),
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
                      line.get(self.offset).map_or(false, is_space_or_tab) {
                    self.advance_offset(line, 1, true);
                }

                let i = self.column - save_column;
                if i >= 5 || i < 1 || line.get(self.offset).map_or(false, is_line_end_char) {
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
                    fence_char: '\0',
                    fence_length: 0,
                    fence_offset: 0,
                    info: vec![],
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

    fn advance_offset(&mut self, line: &mut Vec<char>, mut count: usize, columns: bool) {
        while count > 0 {
            match line.get(self.offset) {
                None => break,
                Some(&'\t') => {
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

    fn parse_block_quote_prefix(&mut self, line: &mut Vec<char>) -> bool {
        let indent = self.indent;
        if indent <= 3 && line.get(self.first_nonspace) == Some(&'>') {
            self.advance_offset(line, indent + 1, true);

            if line.get(self.offset).map_or(false, is_space_or_tab) {
                self.advance_offset(line, 1, true);
            }

            return true;
        }

        false
    }

    fn parse_node_item_prefix(&mut self,
                              line: &mut Vec<char>,
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
                               line: &mut Vec<char>,
                               container: &'a Node<'a, AstCell>,
                               ast: &mut Ast,
                               should_continue: &mut bool)
                               -> bool {
        let ncb = match ast.value {
                NodeValue::CodeBlock(ref ncb) => Some(ncb.clone()),
                _ => None,
            }
            .unwrap();

        if !ncb.fenced {
            if self.indent >= CODE_INDENT {
                self.advance_offset(line, CODE_INDENT, true);
                return true;
            } else if self.blank {
                let offset = self.first_nonspace - self.offset;
                self.advance_offset(line, offset, false);
                return true;
            }
            return false;
        }

        let matched = if self.indent <= 3 &&
                         line.get(self.first_nonspace) == Some(&ncb.fence_char) {
            scanners::close_code_fence(&line[self.first_nonspace..]).unwrap_or(0)
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
        while i > 0 && line.get(self.offset).map_or(false, is_space_or_tab) {
            self.advance_offset(line, 1, true);
            i -= 1;
        }
        true
    }

    fn parse_html_block_prefix(&mut self, t: u8) -> bool {
        match t {
            1 | 2 | 3 | 4 | 5 => true,
            6 | 7 => !self.blank,
            _ => {
                assert!(false);
                false
            }
        }
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
                             line: &mut Vec<char>) {
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
                &NodeValue::HtmlBlock(ref nhb) => {
                    self.add_line(container, line);

                    let matches_end_condition = match nhb.block_type {
                        1 => scanners::html_block_end_1(&line[self.first_nonspace..]).is_some(),
                        2 => scanners::html_block_end_2(&line[self.first_nonspace..]).is_some(),
                        3 => scanners::html_block_end_3(&line[self.first_nonspace..]).is_some(),
                        4 => scanners::html_block_end_4(&line[self.first_nonspace..]).is_some(),
                        5 => scanners::html_block_end_5(&line[self.first_nonspace..]).is_some(),
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

    fn add_line(&mut self, node: &'a Node<'a, AstCell>, line: &mut Vec<char>) {
        let mut ast = node.data.borrow_mut();
        assert!(ast.open);
        if self.partially_consumed_tab {
            self.offset += 1;
            let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
            for i in 0..chars_to_tab {
                ast.content.push(' ');
            }
        }
        if self.offset < line.len() {
            ast.content.extend_from_slice(&line[self.offset..]);
        }
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
            self.current = self.finalize(self.current).unwrap();
        }

        self.finalize(self.root);
        self.process_inlines();
    }

    fn finalize(&mut self, node: &'a Node<'a, AstCell>) -> Option<&'a Node<'a, AstCell>> {
        self.finalize_borrowed(node, &mut *node.data.borrow_mut())
    }

    fn finalize_borrowed(&mut self,
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
            if ast.end_column > 0 && self.linebuf[ast.end_column - 1] == '\n' {
                ast.end_column -= 1;
            }
            if ast.end_column > 0 && self.linebuf[ast.end_column - 1] == '\r' {
                ast.end_column -= 1;
            }
        } else {
            ast.end_line = self.line_number - 1;
            ast.end_column = self.last_line_length;
        }

        let content = &mut ast.content;
        let mut pos = 0;

        let parent = node.parent();

        match &mut ast.value {
            &mut NodeValue::Paragraph => {
                while content.get(0) == Some(&'[') &&
                      unwrap_into(parse_reference_inline(self.arena, content, &mut self.refmap),
                                  &mut pos) {
                    for i in 0..pos {
                        content.remove(0);
                    }
                }
                if is_blank(content) {
                    node.detach();
                }
            }
            &mut NodeValue::CodeBlock(ref mut ncb) => {
                if !ncb.fenced {
                    remove_trailing_blank_lines(content);
                    content.push('\n');
                } else {
                    let mut pos = 0;
                    while pos < content.len() {
                        if is_line_end_char(&content[pos]) {
                            break;
                        }
                        pos += 1;
                    }
                    assert!(pos < content.len());

                    let mut tmp = entity::unescape_html(&content[..pos]);
                    trim(&mut tmp);
                    unescape(&mut tmp);
                    ncb.info = tmp;

                    if content.get(pos) == Some(&'\r') {
                        pos += 1;
                    }
                    if content.get(pos) == Some(&'\n') {
                        pos += 1;
                    }

                    for i in 0..pos {
                        content.remove(0);
                    }
                }
                ncb.literal = content.clone();
                content.clear();
            }
            &mut NodeValue::HtmlBlock(ref mut nhb) => {
                nhb.literal = content.clone();
                content.clear();
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

        parent
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
            refmap: &mut self.refmap,
            delimiters: vec![],
            brackets: vec![],
            backticks: [0; MAXBACKTICKS + 1],
            scanned_for_backticks: false,
        };
        rtrim(&mut subj.input);

        while !subj.eof() && subj.parse_inline(node) {}

        subj.process_emphasis(-1);

        while subj.delimiters.len() > 0 {
            // XXX ???
            assert!(false);
            subj.brackets.pop();
        }
        while subj.brackets.len() > 0 {
            subj.brackets.pop();
        }
    }
}

fn parse_reference_inline<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                              content: &[char],
                              refmap: &mut HashMap<Vec<char>, Reference>)
                              -> Option<usize> {
    let mut subj = Subject {
        arena: arena,
        input: content.to_vec(),
        pos: 0,
        refmap: refmap,
        delimiters: vec![],
        brackets: vec![],
        backticks: [0; MAXBACKTICKS + 1],
        scanned_for_backticks: false,
    };

    let mut lab = match subj.link_label() {
            Some(lab) => if lab.len() == 0 { return None } else { lab },
            None => return None,
        }
        .to_vec();

    if subj.peek_char() != Some(&':') {
        return None;
    }

    subj.pos += 1;
    subj.spnl();
    let matchlen = match manual_scan_link_url(&subj.input[subj.pos..]) {
        Some(matchlen) => matchlen,
        None => return None,
    };
    let url = subj.input[subj.pos..subj.pos + matchlen].to_vec();
    subj.pos += matchlen;

    let beforetitle = subj.pos;
    subj.spnl();
    let title = match scanners::link_title(&subj.input[subj.pos..]) {
        Some(matchlen) => {
            let t = &subj.input[subj.pos..subj.pos + matchlen];
            subj.pos += matchlen;
            t.to_vec()
        }
        _ => {
            subj.pos = beforetitle;
            vec![]
        }
    };

    subj.skip_spaces();
    if !subj.skip_line_end() {
        if title.len() > 0 {
            subj.pos = beforetitle;
            subj.skip_spaces();
            if !subj.skip_line_end() {
                return None;
            }
        } else {
            return None;
        }
    }

    lab = normalize_reference_label(&lab);
    subj.refmap.entry(lab).or_insert(Reference {
        url: clean_url(&url),
        title: clean_title(&title),
    });
    Some(subj.pos)
}

struct Subject<'a, 'b> {
    arena: &'a Arena<Node<'a, AstCell>>,
    input: Vec<char>,
    pos: usize,
    refmap: &'b mut HashMap<Vec<char>, Reference>,
    delimiters: Vec<Delimiter<'a>>,
    brackets: Vec<Bracket<'a>>,
    backticks: [usize; MAXBACKTICKS + 1],
    scanned_for_backticks: bool,
}

struct Delimiter<'a> {
    inl: &'a Node<'a, AstCell>,
    delim_char: char,
    can_open: bool,
    can_close: bool,
}

struct Bracket<'a> {
    previous_delimiter: i32,
    inl_text: &'a Node<'a, AstCell>,
    position: usize,
    image: bool,
    active: bool,
    bracket_after: bool,
}

impl<'a, 'b> Subject<'a, 'b> {
    fn parse_inline(&mut self, node: &'a Node<'a, AstCell>) -> bool {
        let new_inl: Option<&'a Node<'a, AstCell>>;
        let c = match self.peek_char() {
            None => return false,
            Some(ch) => *ch as char,
        };

        match c {
            '\0' => return false,
            '\r' | '\n' => new_inl = Some(self.handle_newline()),
            '`' => new_inl = Some(self.handle_backticks()),
            '\\' => new_inl = Some(self.handle_backslash()),
            '&' => new_inl = Some(self.handle_entity()),
            '<' => new_inl = Some(self.handle_pointy_brace()),
            '*' | '_' | '\'' | '"' => new_inl = Some(self.handle_delim(c)),
            // TODO: smart characters. Eh.
            //'-' => new_inl => Some(self.handle_hyphen()),
            //'.' => new_inl => Some(self.handle_period()),
            '[' => {
                self.pos += 1;
                let inl = make_inline(self.arena, NodeValue::Text(vec!['[']));
                new_inl = Some(inl);
                self.push_bracket(false, inl);
            },
            ']' => new_inl = self.handle_close_bracket(),
            '!' => {
                self.pos += 1;
                if self.peek_char() == Some(&'[') {
                    self.pos += 1;
                    let inl = make_inline(self.arena, NodeValue::Text(vec!['!', '[']));
                    new_inl = Some(inl);
                    self.push_bracket(true, inl);
                } else {
                    new_inl = Some(make_inline(self.arena, NodeValue::Text(vec!['!'])));
                }
            },
            _ => {
                let endpos = self.find_special_char();
                let mut contents = self.input[self.pos..endpos].to_vec();
                self.pos = endpos;

                if self.peek_char().map_or(false, is_line_end_char) {
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

    fn process_emphasis(&mut self, stack_bottom: i32) {
        let mut closer = self.delimiters.len() as i32 - 1;
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

        while closer != -1 && (closer as usize) < self.delimiters.len() {
            if self.delimiters[closer as usize].can_close {
                let mut opener = closer - 1;
                let mut opener_found = false;

                while opener != -1 && opener != stack_bottom &&
                      opener !=
                      openers_bottom[self.delimiters[closer as usize]
                    .inl
                    .data
                    .borrow_mut()
                    .value
                    .text()
                    .unwrap()
                    .len() % 3][self.delimiters[closer as usize]
                    .delim_char as usize] {
                    if self.delimiters[opener as usize].can_open &&
                       self.delimiters[opener as usize].delim_char ==
                       self.delimiters[closer as usize].delim_char {
                        let odd_match = (self.delimiters[closer as usize].can_open ||
                                         self.delimiters[opener as usize].can_close) &&
                                        ((self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap()
                            .len() +
                                          self.delimiters[closer as usize]
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

                if self.delimiters[closer as usize].delim_char == '*' ||
                   self.delimiters[closer as usize].delim_char == '_' {
                    if opener_found {
                        closer = self.insert_emph(opener, closer);
                    } else {
                        closer += 1;
                    }
                } else if self.delimiters[closer as usize].delim_char == '\'' {
                    *self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "’".chars().collect();
                    if opener_found {
                        *self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "‘".chars().collect();
                    }
                    closer += 1;
                } else if self.delimiters[closer as usize].delim_char == '"' {
                    *self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "”".chars().collect();
                    if opener_found {
                        *self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "“".chars().collect();
                    }
                    closer += 1;
                }
                if !opener_found {
                    let ix = self.delimiters[old_closer as usize]
                        .inl
                        .data
                        .borrow_mut()
                        .value
                        .text()
                        .unwrap()
                        .len() % 3;
                    openers_bottom[ix][self.delimiters[old_closer as usize].delim_char as usize] =
                        old_closer - 1;
                    if !self.delimiters[old_closer as usize].can_open {
                        self.delimiters.remove(old_closer as usize);
                    }
                }
            } else {
                closer += 1;
            }
        }

        // TODO truncate instead!
        while self.delimiters.len() > (stack_bottom + 1) as usize {
            self.delimiters.pop();
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char<'x>(&'x self) -> Option<&'x char> {
        self.input.get(self.pos).map(|c| {
            assert!(*c > '\0');
            c
        })
    }

    fn find_special_char(&self) -> usize {
        lazy_static! {
            static ref SPECIAL_CHARS: BTreeSet<char> =
                ['\n',
                '\r',
                '_',
                '*',
                '"',
                '`',
                '\\',
                '&',
                '<',
                '[',
                ']',
                '!',
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
        if self.input[self.pos] == '\r' {
            self.pos += 1;
        }
        if self.input[self.pos] == '\n' {
            self.pos += 1;
        }
        self.skip_spaces();
        if nlpos > 1 && self.input[nlpos - 1] == ' ' && self.input[nlpos - 2] == ' ' {
            make_inline(self.arena, NodeValue::LineBreak)
        } else {
            make_inline(self.arena, NodeValue::SoftBreak)
        }
    }

    fn take_while(&mut self, c: char) -> Vec<char> {
        let mut v = vec![];
        while self.peek_char() == Some(&c) {
            v.push(self.input[self.pos]);
            self.pos += 1;
        }
        v
    }

    fn scan_to_closing_backtick(&mut self, openticklength: usize) -> Option<usize> {
        if openticklength > MAXBACKTICKS {
            return None;
        }

        if self.scanned_for_backticks && self.backticks[openticklength] <= self.pos {
            return None;
        }

        loop {
            while self.peek_char().map_or(false, |&c| c != '`') {
                self.pos += 1;
            }
            if self.pos >= self.input.len() {
                self.scanned_for_backticks = true;
                return None;
            }
            let numticks = self.take_while('`').len();
            if numticks <= MAXBACKTICKS {
                self.backticks[numticks] = self.pos - numticks;
            }
            if numticks == openticklength {
                return Some(self.pos);
            }
        }
    }

    fn handle_backticks(&mut self) -> &'a Node<'a, AstCell> {
        let openticks = self.take_while('`');
        let startpos = self.pos;
        let endpos = self.scan_to_closing_backtick(openticks.len());

        match endpos {
            None => {
                self.pos = startpos;
                return make_inline(self.arena, NodeValue::Text(openticks));
            }
            Some(endpos) => {
                let mut buf = self.input[startpos..endpos - openticks.len()].to_vec();
                trim(&mut buf);
                normalize_whitespace(&mut buf);
                make_inline(self.arena, NodeValue::Code(buf))
            }
        }
    }

    fn skip_spaces(&mut self) -> bool {
        let mut skipped = false;
        while self.peek_char().map_or(false, |&c| c == ' ' || c == '\t') {
            self.pos += 1;
            skipped = true;
        }
        skipped
    }

    fn handle_delim(&mut self, c: char) -> &'a Node<'a, AstCell> {
        let (numdelims, can_open, can_close) = self.scan_delims(c);

        let contents = self.input[self.pos - numdelims..self.pos].to_vec();
        let inl = make_inline(self.arena, NodeValue::Text(contents));

        if (can_open || can_close) && c != '\'' && c != '"' {
            self.push_delimiter(c, can_open, can_close, inl);
        }

        inl
    }

    fn scan_delims(&mut self, c: char) -> (usize, bool, bool) {
        let before_char = if self.pos == 0 {
            '\n'
        } else {
            self.input[self.pos - 1]
        };

        let mut numdelims = 0;
        if c == '\'' || c == '"' {
            numdelims += 1;
            self.pos += 1;
        } else {
            while self.peek_char() == Some(&c) {
                numdelims += 1;
                self.pos += 1;
            }
        }

        let after_char = if self.eof() {
            '\n'
        } else {
            self.input[self.pos]
        };

        let left_flanking = numdelims > 0 && !after_char.is_whitespace() &&
                            !(after_char.is_punctuation() && !before_char.is_whitespace() &&
                              !before_char.is_punctuation());
        let right_flanking = numdelims > 0 && !before_char.is_whitespace() &&
                             !(before_char.is_punctuation() && !after_char.is_whitespace() &&
                               !after_char.is_punctuation());

        if c == '_' {
            (numdelims,
             left_flanking && (!right_flanking || before_char.is_punctuation()),
             right_flanking && (!left_flanking || after_char.is_punctuation()))
        } else if c == '\'' || c == '"' {
            (numdelims, left_flanking && !right_flanking, right_flanking)
        } else {
            (numdelims, left_flanking, right_flanking)
        }
    }

    fn push_delimiter(&mut self,
                      c: char,
                      can_open: bool,
                      can_close: bool,
                      inl: &'a Node<'a, AstCell>) {
        self.delimiters.push(Delimiter {
            inl: inl,
            delim_char: c,
            can_open: can_open,
            can_close: can_close,
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
            closer -= 1;
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

    fn handle_backslash(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;
        if self.peek_char().map_or(false, ispunct) {
            self.pos += 1;
            return make_inline(self.arena, NodeValue::Text(vec![self.input[self.pos - 1]]));
        } else if !self.eof() && self.skip_line_end() {
            return make_inline(self.arena, NodeValue::LineBreak);
        } else {
            return make_inline(self.arena, NodeValue::Text(vec!['\\']));
        }
    }

    fn skip_line_end(&mut self) -> bool {
        let mut seen_line_end_char = false;
        if self.peek_char() == Some(&'\r') {
            self.pos += 1;
            seen_line_end_char = true;
        }
        if self.peek_char() == Some(&'\n') {
            self.pos += 1;
            seen_line_end_char = true;
        }
        seen_line_end_char || self.eof()
    }

    fn handle_entity(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;

        match entity::unescape(&self.input[self.pos..]) {
            None => make_inline(self.arena, NodeValue::Text(vec!['&'])),
            Some((entity, len)) => {
                self.pos += len;
                make_inline(self.arena, NodeValue::Text(entity))
            }
        }
    }

    fn handle_pointy_brace(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;

        if let Some(matchlen) = scanners::autolink_uri(&self.input[self.pos..]) {
            let inl = make_autolink(self.arena,
                                    &self.input[self.pos..self.pos + matchlen - 1],
                                    AutolinkType::URI);
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::autolink_email(&self.input[self.pos..]) {
            let inl = make_autolink(self.arena,
                                    &self.input[self.pos..self.pos + matchlen - 1],
                                    AutolinkType::Email);
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::html_tag(&self.input[self.pos..]) {
            let contents = &self.input[self.pos - 1..self.pos + matchlen];
            let inl = make_inline(self.arena, NodeValue::HtmlInline(contents.to_vec()));
            self.pos += matchlen;
            return inl;
        }

        make_inline(self.arena, NodeValue::Text(vec!['<']))
    }

    fn push_bracket(&mut self, image: bool, inl_text: &'a Node<'a, AstCell>) {
        let len = self.brackets.len();
        if len > 0 {
            self.brackets[len - 1].bracket_after = true;
        }
        self.brackets.push(Bracket {
            previous_delimiter: self.delimiters.len() as i32 - 1,
            inl_text: inl_text,
            position: self.pos,
            image: image,
            active: true,
            bracket_after: false,
        });
    }

    fn handle_close_bracket(&mut self) -> Option<&'a Node<'a, AstCell>> {
        self.pos += 1;
        let initial_pos = self.pos;

        let brackets_len = self.brackets.len();
        if brackets_len == 0 {
            return Some(make_inline(self.arena, NodeValue::Text(vec![']'])));
        }

        if !self.brackets[brackets_len - 1].active {
            self.brackets.pop();
            return Some(make_inline(self.arena, NodeValue::Text(vec![']'])));
        }

        let is_image = self.brackets[brackets_len - 1].image;
        let after_link_text_pos = self.pos;

        let mut sps = 0;
        let mut n = 0;
        if self.peek_char() == Some(&'(') &&
           {
            sps = scanners::spacechars(&self.input[self.pos + 1..]).unwrap_or(0);
            unwrap_into(manual_scan_link_url(&self.input[self.pos + 1 + sps..]),
                        &mut n)
        } {
            let starturl = self.pos + 1 + sps;
            let endurl = starturl + n;
            let starttitle = endurl + scanners::spacechars(&self.input[endurl..]).unwrap_or(0);
            let endtitle = if starttitle == endurl {
                starttitle
            } else {
                starttitle + scanners::link_title(&self.input[starttitle..]).unwrap_or(0)
            };
            let endall = endtitle + scanners::spacechars(&self.input[endtitle..]).unwrap_or(0);

            if self.input.get(endall) == Some(&')') {
                self.pos = endall + 1;
                let url = clean_url(&self.input[starturl..endurl]);
                let title = clean_title(&self.input[starttitle..endtitle]);
                self.close_bracket_match(is_image, url, title);
                return None;
            } else {
                self.pos = after_link_text_pos;
            }
        }

        let (mut lab, mut found_label) = match self.link_label() {
            Some(lab) => (lab.to_vec(), true),
            None => (vec![], false),
        };

        if !found_label {
            self.pos = initial_pos;
        }

        if (!found_label || lab.len() == 0) && !self.brackets[brackets_len - 1].bracket_after {
            lab = self.input[self.brackets[brackets_len - 1].position..initial_pos - 1]
                .to_vec();
            found_label = true;
        }

        let reff: Option<Reference> = if found_label {
            lab = normalize_reference_label(&lab);
            self.refmap.get(&lab).map(|c| c.clone())
        } else {
            None
        };

        if let Some(reff) = reff {
            self.close_bracket_match(is_image, reff.url.clone(), reff.title.clone());
            return None;
        }

        self.brackets.pop();
        self.pos = initial_pos;
        Some(make_inline(self.arena, NodeValue::Text(vec![']'])))
    }

    fn close_bracket_match(&mut self, is_image: bool, url: Vec<char>, title: Vec<char>) {
        let nl = NodeLink {
            url: url,
            title: title,
        };
        let inl = make_inline(self.arena,
                              if is_image {
                                  NodeValue::Image(nl)
                              } else {
                                  NodeValue::Link(nl)
                              });

        let mut brackets_len = self.brackets.len();
        self.brackets[brackets_len - 1].inl_text.insert_before(inl);
        let mut tmpch = self.brackets[brackets_len - 1].inl_text.next_sibling();
        while let Some(tmp) = tmpch {
            tmpch = tmp.next_sibling();
            inl.append(tmp);
        }
        self.brackets[brackets_len - 1].inl_text.detach();
        let previous_delimiter = self.brackets[brackets_len - 1].previous_delimiter;
        self.process_emphasis(previous_delimiter);
        self.brackets.pop();
        brackets_len -= 1;

        if !is_image {
            let mut i = brackets_len as i32 - 1;
            while i >= 0 {
                if !self.brackets[i as usize].image {
                    if !self.brackets[i as usize].active {
                        break;
                    } else {
                        self.brackets[i as usize].active = false;
                    }
                }
                i -= 1;
            }
        }
    }

    fn link_label(&mut self) -> Option<&[char]> {
        let startpos = self.pos;

        if self.peek_char() != Some(&'[') {
            return None;
        }

        self.pos += 1;

        let mut length = 0;
        let mut c = '\0';
        while unwrap_into_copy(self.peek_char(), &mut c) && c != '[' && c != ']' {
            if c == '\\' {
                self.pos += 1;
                length += 1;
                if self.peek_char().map_or(false, ispunct) {
                    self.pos += 1;
                    length += 1;
                }
            } else {
                self.pos += 1;
                length += 1;
            }
            if length > MAX_LINK_LABEL_LENGTH {
                self.pos = startpos;
                return None;
            }
        }

        if c == ']' {
            let raw_label = &self.input[startpos + 1..self.pos];
            trim_slice(raw_label);
            self.pos += 1;
            Some(raw_label)
        } else {
            self.pos = startpos;
            None
        }
    }

    fn spnl(&mut self) {
        self.skip_spaces();
        if self.skip_line_end() {
            self.skip_spaces();
        }
    }
}

fn manual_scan_link_url(input: &[char]) -> Option<usize> {
    let len = input.len();
    let mut i = 0;
    let mut nb_p = 0;

    if i < len && input[i] == '<' {
        i += 1;
        while i < len {
            if input[i] == '>' {
                i += 1;
                break;
            } else if input[i] == '\\' {
                i += 2;
            } else if isspace(&input[i]) {
                return None;
            } else {
                i += 1;
            }
        }
    } else {
        while i < len {
            if input[i] == '\\' {
                i += 2;
            } else if input[i] == '(' {
                nb_p += 1;
                i += 1;
            } else if input[i] == ')' {
                if nb_p == 0 {
                    break;
                }
                nb_p -= 1;
                i += 1;
            } else if isspace(&input[i]) {
                break;
            } else {
                i += 1;
            }
        }
    }

    if i >= len { None } else { Some(i) }
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

fn parse_list_marker(line: &mut Vec<char>,
                     mut pos: usize,
                     interrupts_paragraph: bool)
                     -> Option<(usize, NodeList)> {
    let mut c = match line.get(pos) {
        Some(c) => *c,
        _ => return None,
    };
    let startpos = pos;

    if c == '*' || c == '-' || c == '+' {
        pos += 1;
        if !line.get(pos).map_or(false, isspace) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while line.get(i).map_or(false, is_space_or_tab) {
                i += 1;
            }
            if line.get(i) == Some(&'\n') {
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
            start = (10 * start) + (*line.get(pos).unwrap() as u32 - '0' as u32) as usize;
            pos += 1;
            digits += 1;

            if !(digits < 9 && line.get(pos).map_or(false, isdigit)) {
                break;
            }
        }

        if interrupts_paragraph && start != 1 {
            return None;
        }

        c = line.get(pos).map_or('\0', |&c| c);
        if c != '.' && c != ')' {
            return None;
        }

        pos += 1;

        if !line.get(pos).map_or(false, isspace) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while line.get(i).map_or(false, is_space_or_tab) {
                i += 1;
            }
            if line.get(i).map_or(false, is_line_end_char) {
                return None;
            }
        }

        return Some((pos - startpos,
                     NodeList {
                         list_type: ListType::Ordered,
                         marker_offset: 0,
                         padding: 0,
                         start: start,
                         delimiter: if c == '.' {
                             ListDelimType::Period
                         } else {
                             ListDelimType::Paren
                         },
                         bullet_char: '\0',
                         tight: false,
                     }));
    }

    None
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

fn unwrap_into_copy<T: Copy>(t: Option<&T>, out: &mut T) -> bool {
    match t {
        Some(v) => {
            *out = *v;
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

#[derive(PartialEq)]
pub enum AutolinkType {
    URI,
    Email,
}

fn make_autolink<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                     url: &[char],
                     kind: AutolinkType)
                     -> &'a Node<'a, AstCell> {
    let inl = make_inline(arena,
                          NodeValue::Link(NodeLink {
                              url: clean_autolink(url, kind),
                              title: vec![],
                          }));
    inl.append(make_inline(arena, NodeValue::Text(entity::unescape_html(url))));
    inl
}
