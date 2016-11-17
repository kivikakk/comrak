use std::{str, mem};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let n = ::parse_document(b"My **document**.\n\nIt's mine.\n", 0);
    }
}

pub struct Node {
    
}

pub fn parse_document(buffer: &[u8], options: u32) -> Node {
    let mut parser = Parser::new(options);
    parser.feed(buffer, true);
    parser.finish()
}

struct Parser {
    last_buffer_ended_with_cr: bool,
    linebuf: Vec<u8>,
}

impl Parser {
    fn new(options: u32) -> Parser {
        Parser {
            last_buffer_ended_with_cr: false,
            linebuf: vec![],
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
        println!("process: {}", str::from_utf8(buffer).unwrap())
    }

    fn finish(&mut self) -> Node {
        Node {}
    }
}

fn is_line_end_char(ch: u8) -> bool {
    match ch {
        10 | 13 => true,
        _ => false,
    }
}
