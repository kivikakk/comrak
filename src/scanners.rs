use regex::Regex;

fn search(re: &Regex, line: &[char]) -> Option<usize> {
    let s: String = line.iter().collect();
    re.find(&s).map(|m| m.as_str().chars().count())
}

fn captures(re: &Regex, line: &[char], ix: usize) -> Option<usize> {
    let s: String = line.iter().collect();
    let c = match re.captures(&s) {
        Some(c) => c,
        None => return None,
    };
    c.get(ix).map(|m| m.as_str().chars().count())
}

fn is_match(re: &Regex, line: &[char]) -> bool {
    let s: String = line.iter().collect();
    re.is_match(&s)
}

pub fn atx_heading_start(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:#{1,6}([ \t]+|[\r\n]))").unwrap();
    }
    search(&RE, line)
}

pub fn html_block_end_1(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:.*</(script|pre|style)>)").unwrap();
    }
    search(&RE, line)
}

pub fn html_block_end_2(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:.*-->)").unwrap();
    }
    search(&RE, line)
}

pub fn html_block_end_3(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:.*\?>)").unwrap();
    }
    search(&RE, line)
}

pub fn html_block_end_4(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:.*>)").unwrap();
    }
    search(&RE, line)
}

pub fn html_block_end_5(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:.*\]\]>)").unwrap();
    }
    search(&RE, line)
}

pub fn open_code_fence(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:(```+|~~~+)[^`\r\n\x00]*[\r\n])").unwrap();
    }
    captures(&RE, line, 1)
}

pub fn close_code_fence(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:(```+|~~~+)[ \t]*[\r\n])").unwrap();
    }
    captures(&RE, line, 1)
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

pub fn html_block_start(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"\A(?:<(script|pre|style)([ \t\v\f\r\n]|>))").unwrap();
        static ref RE2: Regex = Regex::new(r"\A(?:<!--)").unwrap();
        static ref RE3: Regex = Regex::new(r"\A(?:<\?)").unwrap();
        static ref RE4: Regex = Regex::new(r"\A(?:<![A-Z])").unwrap();
        static ref RE5: Regex = Regex::new(r"\A(?:<!\[CDATA\[)").unwrap();
        static ref RE6: Regex = Regex::new(
            &format!(r"\A(?:</?({})([ \t\v\f\r\n]|/?>))", *BLOCK_TAG_NAMES_PIPED)).unwrap();
    }

    if is_match(&RE1, line) {
        Some(1)
    } else if is_match(&RE2, line) {
        Some(2)
    } else if is_match(&RE3, line) {
        Some(3)
    } else if is_match(&RE4, line) {
        Some(4)
    } else if is_match(&RE5, line) {
        Some(5)
    } else if is_match(&RE6, line) {
        Some(6)
    } else {
        None
    }
}

lazy_static! {
    static ref SPACE_CHAR: &'static str = r"(?:[ \t\v\f\r\n])";
    static ref TAG_NAME: &'static str = r"(?:[A-Za-z][A-Za-z0-9-]*)";
    static ref CLOSE_TAG: String = format!(r"(?:/{}{}*>)", *TAG_NAME, *SPACE_CHAR);
    static ref ATTRIBUTE_NAME: &'static str = r"(?:[a-zA-Z_:][a-zA-Z0-9:._-]*)";
    static ref ATTRIBUTE_VALUE: &'static str =
        r#"(?:[^"'=<>`\x00]+|['][^'\x00]*[']|["][^"\x00]*["])"#;
    static ref ATTRIBUTE_VALUE_SPEC: String = format!(
        r"(?:{}*={}*{})", *SPACE_CHAR, *SPACE_CHAR, *ATTRIBUTE_VALUE);
    static ref ATTRIBUTE: String = format!(
        r"(?:{}+{}{}?)", *SPACE_CHAR, *ATTRIBUTE_NAME, *ATTRIBUTE_VALUE_SPEC);
    static ref OPEN_TAG: String = format!(r"(?:{}{}*{}*/?>)", *TAG_NAME, *ATTRIBUTE, *SPACE_CHAR);
    static ref HTML_COMMENT: &'static str = r"(?:!---->|!---?[^\x00>-](-?[^\x00-])*-->)";
    static ref PROCESSING_INSTRUCTION: &'static str = r"\?([^?>\x00]+|\?[^>\x00]|>)*\?>";
    static ref DECLARATION: String = format!(r"![A-Z]+{}+[^>\x00]*>", *SPACE_CHAR);
    static ref CDATA: &'static str = r"!\[CDATA\[([^\]\x00]+|\][^\]\x00]|\]\][^>\x00])*\]\]>";
    static ref HTML_TAG: String = format!(
        r"(?:{}|{}|{}|{}|{}|{})", *OPEN_TAG, *CLOSE_TAG, *HTML_COMMENT,
        *PROCESSING_INSTRUCTION, *DECLARATION, *CDATA);
}

pub fn html_block_start_7(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A(?:<({}|{})[\t\n\f ]*[\r\n])", *OPEN_TAG, *CLOSE_TAG)).unwrap();
    }

    if is_match(&RE, line) { Some(7) } else { None }
}

pub enum SetextChar {
    Equals,
    Hyphen,
}

pub fn setext_heading_line(line: &[char]) -> Option<SetextChar> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:(=+|-+)[ \t]*[\r\n])").unwrap();
    }

    if is_match(&RE, line) {
        if line[0] == '=' {
            Some(SetextChar::Equals)
        } else {
            Some(SetextChar::Hyphen)
        }
    } else {
        None
    }
}

pub fn thematic_break(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"\A(?:((\*[ \t]*){3,}|(_[ \t]*){3,}|(-[ \t]*){3,})[ \t]*[\r\n])").unwrap();
    }
    search(&RE, line)
}

lazy_static! {
    static ref SCHEME: &'static str = r"[A-Za-z][A-Za-z0-9.+-]{1,31}";
}

pub fn scheme(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A(?:{}:)", *SCHEME)).unwrap();
    }

    search(&RE, line)
}

pub fn autolink_uri(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A(?:{}:[^\x00-\x20<>]*>)", *SCHEME)).unwrap();
    }

    search(&RE, line)
}

pub fn autolink_email(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            concat!(
            r"\A(?:",
            "[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+",
            r"@",
            r"[a-zA-Z0-9]",
            r"([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?",
            r"(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*",
            r">",
            r")")).unwrap();
    }

    search(&RE, line)
}

pub fn html_tag(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(&format!(r"\A(?:{})", *HTML_TAG)).unwrap();
    }

    search(&RE, line)
}

pub fn spacechars(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:[ \t\v\f\r\n]+)").unwrap();
    }

    search(&RE, line)
}

pub fn link_title(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r#"\A(?:"({}|[^"\x00])*"|'({}|[^'\x00])*'|\(({}|[^)\x00]*)*\))"#,
            *ESCAPED_CHAR, *ESCAPED_CHAR, *ESCAPED_CHAR)).unwrap();
    }

    search(&RE, line)
}

lazy_static! {
    static ref ESCAPED_CHAR: &'static str = r##"(?:\\[!"#$%&'()*+,./:;<=>?@\[\\\]^_`{|}~-])"##;
    static ref TABLE_SPACECHAR: &'static str = r"(?:[ \t\v\f])";
    static ref TABLE_NEWLINE: &'static str = r"(?:\r?\n)";
    static ref TABLE_MARKER: String = format!(r"(?:{}*:?-+:?{}*)",
    *TABLE_SPACECHAR, *TABLE_SPACECHAR);
    static ref TABLE_CELL: String = format!(r"(?:({}|[^|\r\n])*)", *ESCAPED_CHAR);
}

pub fn table_start(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A\|?{}(\|{})*\|?{}*{}",
            *TABLE_MARKER, *TABLE_MARKER, *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}

pub fn table_cell(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(&format!(r"\A{}", *TABLE_CELL)).unwrap();
    }

    search(&RE, line)
}

pub fn table_cell_end(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A\|{}*{}?", *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}

pub fn table_row_end(line: &[char]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A{}*{}", *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}
