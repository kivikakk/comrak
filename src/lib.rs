#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let n = ::parse_document(b"My **document**.", 0);
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
    
}

impl Parser {
    fn new(options: u32) -> Parser {
        Parser {}
    }

    fn feed(&mut self, buffer: &[u8], x: bool) {
        println!("bu {}", buffer.len())
    }

    fn finish(&mut self) -> Node {
        Node {}
    }
}
