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

pub fn open_code_fence(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(```+|~~~+)[^`\r\n\x00]*[\r\n]").unwrap();
    }

    let c = match RE.captures(&line[from..]) {
        Some(c) => c,
        None => return None,
    };

    c.get(1).map(|m| m.end() - m.start())
}

pub fn close_code_fence(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(```+|~~~+)[ \t]*[\r\n]").unwrap();
    }

    let c = match RE.captures(&line[from..]) {
        Some(c) => c,
        None => return None,
    };

    c.get(1).map(|m| m.end() - m.start())
}

pub fn html_block_start(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    // TODO
    None
}

pub fn html_block_start_7(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    // TODO
    None
}

pub fn setext_heading_line(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    // TODO
    None
}

pub fn thematic_break(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    // TODO
    None
}
