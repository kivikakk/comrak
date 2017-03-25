use regex::bytes::Regex;

pub fn atx_heading_start(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"#{1,6}([ \t]+|[\r\n])").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}

pub fn html_block_end_1(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*</(script|pre|style)>").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}

pub fn html_block_end_2(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*-->").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}

pub fn html_block_end_3(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*\?>").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}

pub fn html_block_end_4(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*>").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}

pub fn html_block_end_5(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*\]\]\>").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}
