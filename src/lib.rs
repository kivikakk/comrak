#![feature(move_cell)]
#![allow(unused_variables)]

extern crate typed_arena;

mod arena_tree;

use std::cell::RefCell;
use std::mem;
use typed_arena::Arena;
use arena_tree::Node;

#[cfg(test)]
mod tests {
    use typed_arena::Arena;
    #[test]
    fn it_works() {
        let arena = Arena::new();
        let n = ::parse_document(&arena, b"My **document**.\n\nIt's mine.\n", 0);
        let m = ::format_document(n);
        assert_eq!(m, "<p>My <strong>document</strong>.</p>\n<p>It's mine.</p>\n");
    }
}

pub fn parse_document<'a>(arena: &'a Arena<Node<'a, N>>, buffer: &[u8], options: u32) -> &'a Node<'a, N> {
    let root: &'a Node<'a, N> = arena.alloc(Node::new(RefCell::new(NI {
        typ: 0,
    })));
    let mut parser = Parser::new(root, options);
    parser.feed(buffer, true);
    parser.finish()
}

pub fn format_document(root: &Node<N>) -> String {
    return "".to_string();
}

pub struct NI {
    typ: u32,
}

impl NI {
    fn last_child_is_open(&self) -> bool {
        false
    }
}

type N = RefCell<NI>;

struct Parser<'a> {
    root: &'a Node<'a, N>,
    current: &'a Node<'a, N>,
    line_number: u32,
    offset: u32,
    column: u32,
    first_nonspace: u32,
    first_nonspace_column: u32,
    indent: u32,
    blank: bool,
    partially_consumed_tab: bool,
    linebuf: Vec<u8>,
    last_buffer_ended_with_cr: bool,
}

impl<'a> Parser<'a> {
    fn new(root: &'a Node<'a, N>, options: u32) -> Parser<'a> {
        Parser {
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

        //self.offset = 0;
        //self.column = 0;
        //self.blank = false;
        //self.partially_consumed_tab = false;
        self.line_number += 1;

        let mut all_matched = true;
        let last_matched_container = self.check_open_blocks(&mut line, &mut all_matched);

        /*

        if (!last_matched_container)
        goto finished;

        container = last_matched_container;

        current = parser->current;

        open_new_blocks(parser, &container, &input, all_matched);

        /* parser->current might have changed if feed_reentrant was called */
        if (current == parser->current)
        add_text_to_container(parser, container, last_matched_container, &input);

        finished:
        parser->last_line_length = input.len;
        if (parser->last_line_length &&
        input.data[parser->last_line_length - 1] == '\n')
        parser->last_line_length -= 1;
        if (parser->last_line_length &&
        input.data[parser->last_line_length - 1] == '\r')
        parser->last_line_length -= 1;
        */
    }

    fn check_open_blocks(&mut self, line: &mut Vec<u8>, all_matched: &mut bool) -> Option<Node<'a, N>> {
        let mut should_continue = true;
        *all_matched = false;
        let mut container = self.root;

        while container.data.borrow().last_child_is_open() {
            container = container.last_child().unwrap();
            let cont_type = container.data.borrow().typ;

            self.find_first_nonspace(line);
        }

        None
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
    match *ch {
        10 | 13 => true,
        _ => false,
    }
}

fn peek_at(line: &mut Vec<u8>, i: u32) -> Option<&u8> {
    line.get(i as usize)
}
