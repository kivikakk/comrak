#![feature(move_cell)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate typed_arena;

mod arena_tree;

use std::cell::Cell;
use std::mem;
use typed_arena::Arena;
use arena_tree::Node;

#[cfg(test)]
mod tests {
    use typed_arena::Arena;
    use arena_tree::Node;
    use std::cell::Cell;
    #[test]
    fn it_works() {
        let arena = Arena::new();
        let root = arena.alloc(Node::new(Cell::new(::NI {})));
        let n = ::parse_document(&arena, b"My **document**.\n\nIt's mine.\n", 0);
    }
}

pub fn parse_document<'a>(arena: &'a Arena<Node<'a, N>>, buffer: &[u8], options: u32) -> &'a mut Node<'a, N> {
    let root: &'a mut Node<'a, N> = arena.alloc(Node::new(Cell::new(NI {})));
    let mut parser = Parser::new(root, options);
    parser.feed(buffer, true);
    parser.finish()
}

pub struct NI {}
type N = Cell<NI>;

struct Parser<'a> {
    last_buffer_ended_with_cr: bool,
    linebuf: Vec<u8>,
    line_number: u32,
    current: &'a mut Node<'a, N>,
}

impl<'a> Parser<'a> {
    fn new(root: &'a mut Node<'a, N>, options: u32) -> Parser<'a> {
        let mut p = Parser {
            last_buffer_ended_with_cr: false,
            linebuf: vec![],
            line_number: 0,
            current: root,
        };
        p
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
                if is_line_end_char(buffer[eol]) {
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

    fn process_line(&mut self, buffer: &[u8]) {
        let mut line: Vec<u8> = buffer.into();
        if line.len() == 0 || !is_line_end_char(line[line.len() - 1]) {
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
        //        while self.root.
        None
    }

    fn finish(self) -> &'a mut Node<'a, N> {
        /*
        while self.current.parent().is_some() {
            let ref mut c = self.current;
            let r = self.finalize(c);
            self.current = r;
        }
        */
        self.current
    }

    fn finalize(&'a mut self, node: &'a mut Node<'a, N>) -> &'a mut Node<'a, N> {
        node
    }
}

fn is_line_end_char(ch: u8) -> bool {
    match ch {
        10 | 13 => true,
        _ => false,
    }
}
