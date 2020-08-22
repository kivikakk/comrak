/*!
  In many of these cases the AST will be scanned and then it
  is found there is no match. In many of these cases the scan
  turns up False. It can be see that in the very simplest cases,
  usually by doing a char check at the very begginning of the
  line, we can eliminate these checks without the same allocations
  that are done otherwise and cause the program considerable
  slowdown.

*/

use pest::Parser;
use std::str;
use twoway::find_bytes;

#[cfg(debug_assertions)]
const _LEXER: &str = include_str!("lexer.pest");

#[derive(Parser)]
#[grammar = "lexer.pest"]
struct Lexer;

#[inline(always)]
fn search(rule: Rule, line: &[u8]) -> Option<usize> {
    if let Ok(pairs) = Lexer::parse(rule, unsafe { str::from_utf8_unchecked(line) }) {
        Some(pairs.last().unwrap().as_span().end())
    } else {
        None
    }
}
#[inline(always)]
fn is_match(rule: Rule, line: &[u8]) -> bool {
    Lexer::parse(rule, unsafe { str::from_utf8_unchecked(line) }).is_ok()
}

#[inline(always)]
pub fn atx_heading_start(line: &[u8]) -> Option<usize> {
    if line[0] != b'#' {
        return None;
    }
    search(Rule::atx_heading_start, line)
}

#[inline(always)]
pub fn html_block_end_1(line: &[u8]) -> bool {
    // XXX: should be case-insensitive
    find_bytes(line, b"</script>").is_some()
        || find_bytes(line, b"</pre>").is_some()
        || find_bytes(line, b"</style>").is_some()
}

#[inline(always)]
pub fn html_block_end_2(line: &[u8]) -> bool {
    find_bytes(line, b"-->").is_some()
}

#[inline(always)]
pub fn html_block_end_3(line: &[u8]) -> bool {
    find_bytes(line, b"?>").is_some()
}

#[inline(always)]
pub fn html_block_end_4(line: &[u8]) -> bool {
    line.contains(&b'>')
}

#[inline(always)]
pub fn html_block_end_5(line: &[u8]) -> bool {
    find_bytes(line, b"]]>").is_some()
}

#[inline(always)]
pub fn open_code_fence(line: &[u8]) -> Option<usize> {
    if line[0] != b'`' && line[0] != b'~' {
        return None;
    }
    search(Rule::open_code_fence, line)
}

#[inline(always)]
pub fn close_code_fence(line: &[u8]) -> Option<usize> {
    if line[0] != b'`' && line[0] != b'~' {
        return None;
    }
    search(Rule::close_code_fence, line)
}

#[inline(always)]
pub fn html_block_start(line: &[u8]) -> Option<usize> {
    lazy_static! {
        static ref STR2: &'static [u8] = b"<!--";
        static ref STR3: &'static [u8] = b"<?";
        static ref STR5: &'static [u8] = b"<![CDATA[";
    }

    if !line.starts_with(b"<") {
        return None;
    }

    if is_match(Rule::html_block_start_1, line) {
        Some(1)
    } else if line.starts_with(*STR2) {
        Some(2)
    } else if line.starts_with(*STR3) {
        Some(3)
    } else if is_match(Rule::html_block_start_4, line) {
        Some(4)
    } else if line.starts_with(*STR5) {
        Some(5)
    } else if is_match(Rule::html_block_start_6, line) {
        Some(6)
    } else {
        None
    }
}

#[inline(always)]
pub fn html_block_start_7(line: &[u8]) -> Option<usize> {
    if is_match(Rule::html_block_start_7, line) {
        Some(7)
    } else {
        None
    }
}

pub enum SetextChar {
    Equals,
    Hyphen,
}

#[inline(always)]
pub fn setext_heading_line(line: &[u8]) -> Option<SetextChar> {
    if (line[0] == b'=' || line[0] == b'-') && is_match(Rule::setext_heading_line, line) {
        if line[0] == b'=' {
            Some(SetextChar::Equals)
        } else {
            Some(SetextChar::Hyphen)
        }
    } else {
        None
    }
}

#[inline(always)]
pub fn thematic_break(line: &[u8]) -> Option<usize> {
    if line[0] != b'*' && line[0] != b'-' && line[0] != b'_' {
        return None;
    }
    search(Rule::thematic_break, line)
}

#[inline(always)]
pub fn footnote_definition(line: &[u8]) -> Option<usize> {
    search(Rule::footnote_definition, line)
}

#[inline(always)]
pub fn scheme(line: &[u8]) -> Option<usize> {
    search(Rule::scheme_rule, line)
}

#[inline(always)]
pub fn autolink_uri(line: &[u8]) -> Option<usize> {
    search(Rule::autolink_uri, line)
}

#[inline(always)]
pub fn autolink_email(line: &[u8]) -> Option<usize> {
    search(Rule::autolink_email, line)
}

#[inline(always)]
pub fn html_tag(line: &[u8]) -> Option<usize> {
    search(Rule::html_tag, line)
}

#[inline(always)]
pub fn spacechars(line: &[u8]) -> Option<usize> {
    search(Rule::spacechars, line)
}

#[inline(always)]
pub fn link_title(line: &[u8]) -> Option<usize> {
    search(Rule::link_title, line)
}

#[inline(always)]
pub fn table_start(line: &[u8]) -> Option<usize> {
    search(Rule::table_start, line)
}

#[inline(always)]
pub fn table_cell(line: &[u8]) -> Option<usize> {
    search(Rule::table_cell, line)
}

#[inline(always)]
pub fn table_cell_end(line: &[u8]) -> Option<usize> {
    search(Rule::table_cell_end, line)
}

#[inline(always)]
pub fn table_row_end(line: &[u8]) -> Option<usize> {
    search(Rule::table_row_end, line)
}

#[inline(always)]
pub fn dangerous_url(line: &[u8]) -> Option<usize> {
    search(Rule::dangerous_url, line)
}
