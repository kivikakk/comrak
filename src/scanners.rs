use regex::bytes::Regex;
use twoway::find_bytes;
use pest::Parser;
use std::str;

#[cfg(debug_assertions)]
const _LEXER: &str = include_str!("lexer.pest");

#[derive(Parser)]
#[grammar = "lexer.pest"]
struct Lexer;

fn search(re: &Regex, line: &[u8]) -> Option<usize> {
    re.find(line).map(|m| m.end() - m.start())
}

fn search_(rule: Rule, line: &[u8]) -> Option<usize> {
    if let Ok(pairs) = Lexer::parse(rule, unsafe { str::from_utf8_unchecked(line) }) {
        Some(pairs.last().unwrap().into_span().end())
    } else {
        None
    }
}

fn is_match(re: &Regex, line: &[u8]) -> bool {
    re.is_match(line)
}

fn is_match_(rule: Rule, line: &[u8]) -> bool {
    Lexer::parse(rule, unsafe { str::from_utf8_unchecked(line) }).is_ok()
}

pub fn atx_heading_start(line: &[u8]) -> Option<usize> {
    search_(Rule::atx_heading_start, line)
}

pub fn html_block_end_1(line: &[u8]) -> bool {
    find_bytes(line, b"</script>").is_some() ||
        find_bytes(line, b"</pre>").is_some() ||
        find_bytes(line, b"</style>").is_some()
}

pub fn html_block_end_2(line: &[u8]) -> bool {
    find_bytes(line, b"-->").is_some()
}

pub fn html_block_end_3(line: &[u8]) -> bool {
    find_bytes(line, b"?>").is_some()
}

pub fn html_block_end_4(line: &[u8]) -> bool {
    line.contains(&b'>')
}

pub fn html_block_end_5(line: &[u8]) -> bool {
    find_bytes(line, b"]]>").is_some()
}

pub fn open_code_fence(line: &[u8]) -> Option<usize> {
    search_(Rule::open_code_fence, line)
}

pub fn close_code_fence(line: &[u8]) -> Option<usize> {
    search_(Rule::close_code_fence, line)
}

pub fn html_block_start(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref STR2: &'static [u8] = b"<!--";
        static ref STR3: &'static [u8] = b"<?";
        static ref STR5: &'static [u8] = b"<![CDATA[";
    }

    if !line.starts_with(b"<") {
        return None;
    }

    if is_match_(Rule::html_block_start_1, line) {
        Some(1)
    } else if line.starts_with(*STR2) {
        Some(2)
    } else if line.starts_with(*STR3) {
        Some(3)
    } else if is_match_(Rule::html_block_start_4, line) {
        Some(4)
    } else if line.starts_with(*STR5) {
        Some(5)
    } else if is_match_(Rule::html_block_start_6, line) {
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
        r#"(?:[^"'=<>`\x00 ]+|['][^'\x00]*[']|["][^"\x00]*["])"#;
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

pub fn html_block_start_7(line: &[u8]) -> Option<usize> {
    if is_match_(Rule::html_block_start_7, line) {
        Some(7)
    } else {
        None
    }
}

pub enum SetextChar {
    Equals,
    Hyphen,
}

pub fn setext_heading_line(line: &[u8]) -> Option<SetextChar> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:(=+|-+)[ \t]*[\r\n])").unwrap();
    }

    if is_match(&RE, line) {
        if line[0] == b'=' {
            Some(SetextChar::Equals)
        } else {
            Some(SetextChar::Hyphen)
        }
    } else {
        None
    }
}

pub fn thematic_break(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"\A(?:((\*[ \t]*){3,}|(_[ \t]*){3,}|(-[ \t]*){3,})[ \t]*[\r\n])").unwrap();
    }
    search(&RE, line)
}

pub fn footnote_definition(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A\[\^[^\]\r\n\x00\t]+\]:[ \t]*").unwrap();
    }
    search(&RE, line)
}

lazy_static! {
    static ref SCHEME: &'static str = r"[A-Za-z][A-Za-z0-9.+-]{1,31}";
}

pub fn scheme(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A(?:{}:)", *SCHEME)).unwrap();
    }

    search(&RE, line)
}

pub fn autolink_uri(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A(?:{}:[^\x00-\x20<>]*>)", *SCHEME)).unwrap();
    }

    search(&RE, line)
}

pub fn autolink_email(line: &[u8]) -> Option<usize> {
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

pub fn html_tag(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(&format!(r"\A(?:{})", *HTML_TAG)).unwrap();
    }

    search(&RE, line)
}

pub fn spacechars(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\A(?:[ \t\v\f\r\n]+)").unwrap();
    }

    search(&RE, line)
}

pub fn link_title(line: &[u8]) -> Option<usize> {
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

pub fn table_start(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A\|?{}(\|{})*\|?{}*{}",
            *TABLE_MARKER, *TABLE_MARKER, *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}

pub fn table_cell(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(&format!(r"\A{}", *TABLE_CELL)).unwrap();
    }

    search(&RE, line)
}

pub fn table_cell_end(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A\|{}*{}?", *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}

pub fn table_row_end(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            &format!(r"\A{}*{}", *TABLE_SPACECHAR, *TABLE_NEWLINE)).unwrap();
    }

    search(&RE, line)
}
