use regex::bytes::Regex;

pub fn atx_heading_start(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref ATX: Regex = Regex::new(r"#{1,6}([ \t]+|[\r\n])").unwrap();
    }

    ATX.find(&line[from..]).map(|m| m.end() - m.start())
}
