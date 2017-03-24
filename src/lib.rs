#![feature(move_cell)]
#![allow(unused_variables)]

extern crate typed_arena;
extern crate regex;
#[macro_use] extern crate lazy_static;

mod arena_tree;
mod scanners;

use std::cell::RefCell;
use std::cmp::min;
use std::fmt::{Debug, Formatter, Result};
use std::mem;

use arena_tree::Node;

use typed_arena::Arena;

#[cfg(test)]
mod tests {
    use typed_arena::Arena;
    #[test]
    fn it_works() {
        let arena = Arena::new();
        let n = ::parse_document(
            &arena,
            b"My **document**.\n\nIt's mine.\n\n> Yes.\n\n## Hi!\n\nOkay.",
            0);
        println!("got: {:?}", n);
        let m = ::format_document(n);
        assert_eq!(m, "<p>My <strong>document</strong>.</p>\n<p>It's mine.</p>\n");
    }
}

pub fn parse_document<'a>(arena: &'a Arena<Node<'a, N>>, buffer: &[u8], options: u32) -> &'a Node<'a, N> {
    let root: &'a Node<'a, N> = arena.alloc(Node::new(RefCell::new(NI {
        typ: NodeType::Document,
        start_line: 0,
        start_column: 0,
        end_line: 0,
        open: true,
        last_line_blank: false,
    })));
    let mut parser = Parser::new(arena, root, options);
    parser.feed(buffer, true);
    parser.finish()
}

pub fn format_document<'a>(root: &'a Node<'a, N>) -> String {
    match root.data.borrow().typ {
        NodeType::Document => {
            root.children().map(format_document).collect::<Vec<_>>().concat()
        },
        _ => { "".to_string() }
    }
}

const CODE_INDENT: usize = 4;

#[derive(PartialEq, Debug)]
pub enum NodeType {
    Document,
    BlockQuote,
    List,
    Item,
    CodeBlock,
    HtmlBlock,
    CustomBlock,
    Paragraph,
    Heading,
    ThematicBreak,

    Text,
    SoftBreak,
    LineBreak,
    Code,
    HtmlInline,
    CustomInline,
    Emph,
    Strong,
    Link,
    Image,
}

impl NodeType {
    fn block(&self) -> bool {
        match self {
            &NodeType::Document | &NodeType::BlockQuote | &NodeType::List | &NodeType::Item |
            &NodeType::CodeBlock | &NodeType::HtmlBlock | &NodeType::CustomBlock |
            &NodeType::Paragraph | &NodeType::Heading | &NodeType::ThematicBreak => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct NI {
    typ: NodeType,
    start_line: u32,
    start_column: usize,
    end_line: u32,
    open: bool,
    last_line_blank: bool,
}

fn make_block(typ: NodeType, start_line: u32, start_column: usize) -> NI {
    NI {
        typ: typ,
        start_line: start_line,
        start_column: start_column,
        end_line: start_line,
        open: true,
        last_line_blank: false,
    }
}

type N = RefCell<NI>;

impl<'a> Node<'a, N> {
    fn last_child_is_open(&self) -> bool {
        self.last_child().map_or(false, |n| n.data.borrow().open)
    }

    fn can_contain_type(&self, child: &NodeType) -> bool {
        if child == &NodeType::Document {
            return false;
        }

        match self.data.borrow().typ {
            NodeType::Document | NodeType::BlockQuote | NodeType::Item =>
                child.block() && child != &NodeType::Item,

            NodeType::List =>
                child == &NodeType::Item,

            NodeType::CustomBlock => true,

            NodeType::Paragraph | NodeType::Heading | NodeType::Emph | NodeType::Strong |
            NodeType::Link | NodeType::Image | NodeType::CustomInline =>
                !child.block(),

            _ => false,
        }
    }
}

impl<'a, T: Debug> Debug for Node<'a, RefCell<T>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut ch = vec![];
        let mut c = self.first_child();
        while let Some(e) = c {
            ch.push(e);
            c = e.next_sibling();
        }
        write!(f, "[({:?}) {} children: {{", self.data.borrow(), ch.len())?;
        let mut first = true;
        for e in &ch {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", e)?;
        }
        write!(f, "}}]")?;
        Ok(())
    }
}

struct Parser<'a> {
    arena: &'a Arena<Node<'a, N>>,
    root: &'a Node<'a, N>,
    current: &'a Node<'a, N>,
    line_number: u32,
    offset: usize,
    column: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    partially_consumed_tab: bool,
    last_line_length: u32,
    linebuf: Vec<u8>,
    last_buffer_ended_with_cr: bool,
}

impl<'a> Parser<'a> {
    fn new(arena: &'a Arena<Node<'a, N>>, root: &'a Node<'a, N>, options: u32) -> Parser<'a> {
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
                    self.linebuf.clear();
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
        let mut chars_to_tab = 8 - (self.column % 8);

        while let Some(&c) = peek_at(line, self.first_nonspace) {
            match c as char {
                ' ' => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += 1;
                    chars_to_tab -= 1;
                    if chars_to_tab == 0 {
                        chars_to_tab = 8;
                    }
                },
                '\t' => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += chars_to_tab;
                    chars_to_tab = 8;
                },
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

        println!("process: [{}]", String::from_utf8(line.clone()).unwrap());

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

        self.last_line_length = line.len() as u32;
        if self.last_line_length > 0 && line[(self.last_line_length - 1) as usize] == '\n' as u8 {
            self.last_line_length -= 1;
        }
        if self.last_line_length > 0 && line[(self.last_line_length - 1) as usize] == '\r' as u8 {
            self.last_line_length -= 1;
        }
    }

    fn check_open_blocks(&mut self, line: &mut Vec<u8>, all_matched: &mut bool) -> Option<&'a Node<'a, N>> {
        let mut should_continue = true;
        *all_matched = false;
        let mut container = self.root;

        'done: loop {
            while container.last_child_is_open() {
                container = container.last_child().unwrap();
                let cont_type = &container.data.borrow().typ;

                self.find_first_nonspace(line);

                match cont_type {
                    &NodeType::BlockQuote => {
                        if !self.parse_block_quote_prefix(line) {
                            break 'done;
                        }
                    },
                    &NodeType::Item => {
                        assert!(false);
                        // if !self.parse_node_item_prefix(line, container) {
                        //     break 'done;
                        // }
                    },
                    &NodeType::CodeBlock => {
                        assert!(false);
                        // if !self.parse_code_block_prefix(line, container, &mut should_continue) {
                        //     break 'done;
                        // }
                    },
                    &NodeType::Heading => {
                        break 'done;
                    },
                    &NodeType::HtmlBlock => {
                        assert!(false);
                        // if !self.parse_html_block_prefix(container) {
                        //     break 'done;
                        // }
                    },
                    &NodeType::Paragraph => {
                        if self.blank {
                            break 'done;
                        }
                    },
                    _ => { },
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

    fn open_new_blocks(&mut self, container: &mut &'a Node<'a, N>, line: &mut Vec<u8>, all_matched: bool) {
        let mut cont_type = &container.data.borrow().typ;

        while cont_type != &NodeType::CodeBlock && cont_type != &NodeType::HtmlBlock {
            self.find_first_nonspace(line);
            let indented = self.indent >= CODE_INDENT;

            if !indented && peek_at(line, self.first_nonspace) == Some(&('>' as u8)) {
                let blockquote_startpos = self.first_nonspace;
                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if peek_at(line, self.offset).map_or(false, is_space_or_tab) {
                    self.advance_offset(line, 1, true);
                }
                *container = self.add_child(*container, NodeType::BlockQuote, blockquote_startpos + 1);
            } else if !indented {
                if let Some(matched) = scanners::atx_heading_start(line, self.first_nonspace) {
                    let heading_startpos = self.first_nonspace;
                    let offset = self.offset;
                    self.advance_offset(line, heading_startpos + matched - offset, false);
                    *container = self.add_child(*container, NodeType::Heading, heading_startpos + 1);

                    let mut hashpos = line[self.first_nonspace..].iter().position(|&c| c == '#' as u8).unwrap() + self.first_nonspace;
                    let mut level = 0;
                    while peek_at(line, hashpos) == Some(&('#' as u8)) {
                        level += 1;
                        hashpos += 1;
                    }

                    //container.as.heading.level = level;
                    //container.as.heading.setext = false;
                    //TODO
                }
            } // TODO

            break;
        }
    }

    fn advance_offset(&mut self, line: &mut Vec<u8>, mut count: usize, columns: bool) {
        while count > 0 {
            match peek_at(line, self.offset) {
                None => break,
                Some(&9) => {
                    let chars_to_tab = 8 - (self.column % 8);
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
                },
                Some(_) => {
                    self.partially_consumed_tab = false;
                    self.offset += 1;
                    self.column += 1;
                    count -= 1;
                },
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

    fn add_child(&mut self, mut parent: &'a Node<'a, N>, typ: NodeType, start_column: usize) -> &'a Node<'a, N> {
        while !parent.can_contain_type(&typ) {
            parent = self.finalize(parent);
        }

        let child = make_block(typ, self.line_number, start_column);
        let node = self.arena.alloc(Node::new(RefCell::new(child)));
        parent.append(node);
        node
    }

    fn add_text_to_container(&mut self, container: &'a Node<'a, N>, last_matched_container: &'a Node<'a, N>, line: &mut Vec<u8>) {
    }

    fn finish(&mut self) -> &'a Node<'a, N> {
        while self.current.parent().is_some() {
            self.current = self.finalize(&self.current);
        }
        self.current
    }

    fn finalize(&self, node: &'a Node<'a, N>) -> &'a Node<'a, N> {
        node.parent().unwrap()
    }
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
