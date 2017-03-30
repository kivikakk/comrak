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

lazy_static! {
    static ref BLOCK_TAG_NAMES: Vec<&'static str> = vec![
      "address", "article", "aside", "base", "basefont", "blockquote", "body", "caption", "center",
      "col", "colgroup", "dd", "details", "dialog", "dir", "div", "dl", "dt", "fieldset",
      "figcaption", "figure", "footer", "form", "frame", "frameset", "h1", "h2", "h3", "h4", "h5",
      "h6", "head", "header", "hr", "html", "iframe", "legend", "li", "link", "main", "menu",
      "menuitem", "meta", "nav", "noframes", "ol", "optgroup", "option", "p", "param", "section",
      "source", "title", "summary", "table", "tbody", "td", "tfoot", "th", "thead", "title", "tr",
      "track", "ul",
    ];

    static ref BLOCK_TAG_NAMES_PIPED: String = BLOCK_TAG_NAMES.join("|");
}

pub fn html_block_start(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"<(script|pre|style)([ \t\v\f\r\n]|>)").unwrap();
        static ref RE2: Regex = Regex::new(r"<!--").unwrap();
        static ref RE3: Regex = Regex::new(r"<\?").unwrap();
        static ref RE4: Regex = Regex::new(r"<![A-Z]").unwrap();
        static ref RE5: Regex = Regex::new(r"<!\[CDATA\[").unwrap();
        static ref RE6: Regex = Regex::new(
            &format!(r"</?({})([ \t\v\f\r\n]|/?>)", *BLOCK_TAG_NAMES_PIPED)).unwrap();
    }

    if RE1.is_match(&line[..]) {
        Some(1)
    } else if RE2.is_match(&line[..]) {
        Some(2)
    } else if RE3.is_match(&line[..]) {
        Some(3)
    } else if RE4.is_match(&line[..]) {
        Some(4)
    } else if RE5.is_match(&line[..]) {
        Some(5)
    } else if RE6.is_match(&line[..]) {
        Some(6)
    } else {
        None
    }
}

pub fn html_block_start_7(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    // TODO
    None
}

pub enum SetextChar {
    Equals,
    Hyphen,
}

pub fn setext_heading_line(line: &mut Vec<u8>, from: usize) -> Option<SetextChar> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(=+|-+)[ \t]*[\r\n]").unwrap();
    }

    RE.find(&line[from..]).map(|m| if line[from] == '=' as u8 {
        SetextChar::Equals
    } else {
        SetextChar::Hyphen
    })
}

pub fn thematic_break(line: &mut Vec<u8>, from: usize) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"((\*[ \t]*){3,}|(_[ \t]*){3,}|(-[ \t]*){3,})[ \t]*[\r\n]").unwrap();
    }

    RE.find(&line[from..]).map(|m| m.end() - m.start())
}
