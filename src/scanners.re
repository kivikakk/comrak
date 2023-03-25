use memchr::memmem;
use std::str;
use pest::Parser;
use pest_derive::Parser;

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

// TODO: consider dropping all the #[inline(always)], we probably don't know
// better than rustc.

/*!re2c
    re2c:define:YYCTYPE      = u8;
    re2c:define:YYPEEK       = "*s.get_unchecked(cursor)";
    re2c:define:YYSKIP       = "cursor += 1;";
    re2c:define:YYBACKUP     = "marker = cursor;";
    re2c:define:YYRESTORE    = "cursor = marker;";
    re2c:define:YYBACKUPCTX  = "ctxmarker = cursor;";
    re2c:define:YYRESTORECTX = "cursor = ctxmarker;";
    re2c:yyfill:enable       = 0;
*/

#[inline(always)]
pub fn atx_heading_start(s: &[u8]) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [#]{1,6} ([ \t]+|[\r\n])  { return Some(cursor); }
    * { return None; }
*/
}

#[inline(always)]
pub fn html_block_end_1(s: &[u8]) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [^\n\x00]* [<] [/] ('script'|'pre'|'textarea'|'style') [>] { return true; }
    * { return false; }
*/
}

#[inline(always)]
pub fn html_block_end_2(s: &[u8]) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [^\n\x00]* '-->' { return true; }
    * { return false; }
*/
}

#[inline(always)]
pub fn html_block_end_3(s: &[u8]) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [^\n\x00]* '?>' { return true; }
    * { return false; }
*/
}

#[inline(always)]
pub fn html_block_end_4(s: &[u8]) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [^\n\x00]* '>' { return true; }
    * { return false; }
*/
}

#[inline(always)]
pub fn html_block_end_5(s: &[u8]) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
/*!re2c
    [^\n\x00]* ']]>' { return true; }
    * { return false; }
*/
}

#[inline(always)]
pub fn open_code_fence(s: &[u8]) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
/*!re2c
    [`]{3,} / [^`\r\n\x00]*[\r\n] { return Some(cursor); }
    [~]{3,} / [^\r\n\x00]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

#[inline(always)]
pub fn close_code_fence(s: &[u8]) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
/*!re2c
    [`]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    [~]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

#[inline(always)]
pub fn html_block_start(line: &[u8]) -> Option<usize> {
    const STR2: &'static [u8] = b"<!--";
    const STR3: &'static [u8] = b"<?";
    const STR5: &'static [u8] = b"<![CDATA[";

    if !line.starts_with(b"<") {
        return None;
    }

    if is_match(Rule::html_block_start_1, line) {
        Some(1)
    } else if line.starts_with(STR2) {
        Some(2)
    } else if line.starts_with(STR3) {
        Some(3)
    } else if is_match(Rule::html_block_start_4, line) {
        Some(4)
    } else if line.starts_with(STR5) {
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
pub fn html_comment(line: &[u8]) -> Option<usize> {
    search(Rule::html_comment, line)
}

#[inline(always)]
pub fn html_processing_instruction(line: &[u8]) -> Option<usize> {
    search(Rule::html_processing_instruction, line)
}

#[inline(always)]
pub fn html_declaration(line: &[u8]) -> Option<usize> {
    search(Rule::html_declaration, line)
}

#[inline(always)]
pub fn html_cdata(line: &[u8]) -> Option<usize> {
    search(Rule::html_cdata, line)
}

#[inline(always)]
pub fn spacechars(line: &[u8]) -> Option<usize> {
    search(Rule::spacechars, line)
}

#[inline(always)]
pub fn link_title(line: &[u8]) -> Option<usize> {
    search(Rule::link_title, line)
}

#[cfg(feature = "shortcodes")]
#[inline(always)]
pub fn shortcode(line: &[u8]) -> Option<usize> {
    search(Rule::shortcode_rule, line)
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

// vim: set ft=rust:
