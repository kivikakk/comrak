/*!re2c
    re2c:case-insensitive    = 1;
    re2c:encoding:utf8       = 1;
    re2c:encoding-policy     = substitute;

    re2c:define:YYCTYPE      = u8;
    re2c:define:YYPEEK       = "if cursor < len { *s.as_bytes().get_unchecked(cursor) } else { 0 }";
    re2c:define:YYSKIP       = "cursor += 1;";
    re2c:define:YYBACKUP     = "marker = cursor;";
    re2c:define:YYRESTORE    = "cursor = marker;";
    re2c:define:YYBACKUPCTX  = "ctxmarker = cursor;";
    re2c:define:YYRESTORECTX = "cursor = ctxmarker;";
    re2c:yyfill:enable       = 0;
    re2c:indent:string       = '    ';
    re2c:indent:top          = 1;

    wordchar = [^\x00-\x20];

    spacechar = [ \t\v\f\r\n];

    reg_char     = [^\\()\x00-\x20];

    escaped_char = [\\][!"#$%&'()*+,./:;<=>?@[\\\]^_`{|}~-];

    tagname = [A-Za-z][A-Za-z0-9-]*;

    blocktagname = 'address'|'article'|'aside'|'base'|'basefont'|'blockquote'|'body'|'caption'|'center'|'col'|'colgroup'|'dd'|'details'|'dialog'|'dir'|'div'|'dl'|'dt'|'fieldset'|'figcaption'|'figure'|'footer'|'form'|'frame'|'frameset'|'h1'|'h2'|'h3'|'h4'|'h5'|'h6'|'head'|'header'|'hr'|'html'|'iframe'|'legend'|'li'|'link'|'main'|'menu'|'menuitem'|'nav'|'noframes'|'ol'|'optgroup'|'option'|'p'|'param'|'search'|'section'|'title'|'summary'|'table'|'tbody'|'td'|'tfoot'|'th'|'thead'|'title'|'tr'|'track'|'ul';

    attributename = [a-zA-Z_:][a-zA-Z0-9:._-]*;

    unquotedvalue = [^ \t\r\n\v\f"'=<>`\x00]+;
    singlequotedvalue = ['][^'\x00]*['];
    doublequotedvalue = ["][^"\x00]*["];

    attributevalue = unquotedvalue | singlequotedvalue | doublequotedvalue;

    attributevaluespec = spacechar* [=] spacechar* attributevalue;

    attribute = spacechar+ attributename attributevaluespec?;

    opentag = tagname attribute* spacechar* [/]? [>];
    closetag = [/] tagname spacechar* [>];

    htmlcomment = "--" ([^\x00-]+ | "-" [^\x00-] | "--" [^\x00>])* "-->";

    processinginstruction = ([^?>\x00]+ | [?][^>\x00] | [>])+;

    declaration = [A-Z]+ spacechar+ [^>\x00]*;

    cdata = "CDATA[" ([^\]\x00]+ | "]" [^\]\x00] | "]]" [^>\x00])*;

    htmltag = opentag | closetag;

    in_parens_nosp   = [(] (reg_char|escaped_char|[\\])* [)];

    in_double_quotes = ["] (escaped_char|[^"\x00])* ["];
    in_single_quotes = ['] (escaped_char|[^'\x00])* ['];
    in_parens        = [(] (escaped_char|[^)\x00])* [)];

    scheme           = [A-Za-z][A-Za-z0-9.+-]{1,31};
*/

use crate::parser::alert::AlertType;

pub fn atx_heading_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [#]{1,6} ([ \t]+|[\r\n])  { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_block_end_1(s: &str) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [^\n\x00]* [<] [/] ('script'|'pre'|'textarea'|'style') [>] { return true; }
    * { return false; }
*/
}

pub fn html_block_end_2(s: &str) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [^\n\x00]* '-->' { return true; }
    * { return false; }
*/
}

pub fn html_block_end_3(s: &str) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [^\n\x00]* '?>' { return true; }
    * { return false; }
*/
}

pub fn html_block_end_4(s: &str) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [^\n\x00]* '>' { return true; }
    * { return false; }
*/
}

pub fn html_block_end_5(s: &str) -> bool {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [^\n\x00]* ']]>' { return true; }
    * { return false; }
*/
}

pub fn alert_start(s: &str) -> Option<AlertType> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();

/*!re2c
    [>]{1,} ' [!note]' { return Some(AlertType::Note); }
    [>]{1,} ' [!tip]' { return Some(AlertType::Tip); }
    [>]{1,} ' [!important]' { return Some(AlertType::Important); }
    [>]{1,} ' [!warning]' { return Some(AlertType::Warning); }
    [>]{1,} ' [!caution]' { return Some(AlertType::Caution); }
    * { return None; }
*/
}

pub fn open_code_fence(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
    let len = s.len();
/*!re2c
    [`]{3,} / [^`\r\n\x00]*[\r\n] { return Some(cursor); }
    [~]{3,} / [^\r\n\x00]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

pub fn close_code_fence(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
    let len = s.len();
/*!re2c
    [`]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    [~]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_block_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [<] ('script'|'pre'|'textarea'|'style') (spacechar | [>]) { return Some(1); }
    '<!--' { return Some(2); }
    '<?' { return Some(3); }
    '<!' [A-Z] { return Some(4); }
    '<![CDATA[' { return Some(5); }
    [<] [/]? blocktagname (spacechar | [/]? [>])  { return Some(6); }
    * { return None; }
*/
}

pub fn html_block_start_7(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [<] (opentag | closetag) [\t\n\f ]* [\r\n] { return Some(7); }
    * { return None; }
*/
}

pub enum SetextChar {
    Equals,
    Hyphen,
}

pub fn setext_heading_line(s: &str) -> Option<SetextChar> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [=]+ [ \t]* [\r\n] { return Some(SetextChar::Equals); }
    [-]+ [ \t]* [\r\n] { return Some(SetextChar::Hyphen); }
    * { return None; }
*/
}

pub fn footnote_definition(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    '[^' ([^\] \r\n\x00\t]+) ']:' [ \t]* { return Some(cursor); }
    * { return None; }
*/
}

pub fn scheme(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    scheme [:] { return Some(cursor); }
    * { return None; }
*/
}

pub fn autolink_uri(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    scheme [:][^\x00-\x20<>]*[>]  { return Some(cursor); }
    * { return None; }
*/
}

pub fn autolink_email(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+
        [@]
        [a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?
        ([.][a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
        [>] { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_tag(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    htmltag { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_comment(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    htmlcomment { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_processing_instruction(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    processinginstruction { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_declaration(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    declaration { return Some(cursor); }
    * { return None; }
*/
}

pub fn html_cdata(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    cdata { return Some(cursor); }
    * { return None; }
*/
}

pub fn spacechars(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let len = s.len();
/*!re2c
    [ \t\v\f\r\n]+ { return Some(cursor); }
    * { return None; }
*/
}

pub fn link_title(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    ["] (escaped_char|[^"\x00])* ["]   { return Some(cursor); }
    ['] (escaped_char|[^'\x00])* ['] { return Some(cursor); }
    [(] (escaped_char|[^()\x00])* [)]  { return Some(cursor); }
    * { return None; }
*/
}

pub fn dangerous_url(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    'data:image/' ('png'|'gif'|'jpeg'|'webp') { return None; }
    'javascript:' | 'vbscript:' | 'file:' | 'data:' { return Some(cursor); }
    * { return None; }
*/
}

pub fn ipv6_url_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    'http' [s]? '://[' [0-9a-fA-F:]+ ('%25' [a-zA-Z0-9]+)? ']' { return Some(cursor); }
    * { return None; }
*/
}

pub fn ipv6_relaxed_url_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [a-z]+ '://[' [0-9a-fA-F:]+ ('%25' [a-zA-Z0-9]+)? ']' { return Some(cursor); }
    * { return None; }
*/
}

/*!re2c

    table_spoiler = ['|']['|'];
    table_spacechar = [ \t\v\f];
    table_newline = [\r]?[\n];

    table_delimiter = (table_spacechar*[:]?[-]+[:]?table_spacechar*);
    table_cell = (escaped_char|[^\x00|\r\n])+;
    table_cell_spoiler = (escaped_char|table_spoiler|[^\x00|\r\n])+;

*/

pub fn table_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [|]? table_delimiter ([|] table_delimiter)* [|]? table_spacechar* table_newline {
        return Some(cursor);
    }
    * { return None; }
*/
}

pub fn table_cell(s: &str, spoiler: bool) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();

    // In fact, `table_cell` matches non-empty table cells only. The empty
    // string is also a valid table cell, but is handled by the default rule.
    // This approach prevents re2c's match-empty-string warning.
    if spoiler {
/*!re2c
    table_cell_spoiler { return Some(cursor); }
    * { return None; }
*/
    } else {
/*!re2c
    table_cell { return Some(cursor); }
    * { return None; }
*/
    }
}

pub fn table_cell_end(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let len = s.len();
/*!re2c
    [|] table_spacechar* { return Some(cursor); }
    * { return None; }
*/
}

pub fn table_row_end(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    table_spacechar* table_newline { return Some(cursor); }
    * { return None; }
*/
}

#[cfg(feature = "shortcodes")]
pub fn shortcode(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();
/*!re2c
    [A-Za-z0-9+_-]+ [:] { return Some(cursor); }
    * { return None; }
*/
}

pub fn open_multiline_block_quote_fence(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
    let len = s.len();
/*!re2c
    [>]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

pub fn close_multiline_block_quote_fence(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut marker = 0;
    let mut ctxmarker = 0;
    let len = s.len();
/*!re2c
    [>]{3,} / [ \t]*[\r\n] { return Some(cursor); }
    * { return None; }
*/
}

// Returns both the length of the match, and the tasklist character.
pub fn tasklist(s: &str) -> Option<(usize, u8)> {
    let mut cursor = 0;
    let mut marker = 0;
    let len = s.len();

    let t1;
/*!stags:re2c format = 'let mut @@{tag} = 0;'; */

/*!local:re2c
    re2c:define:YYSTAGP = "@@{tag} = cursor;";
    re2c:tags = 1;

    spacechar* [[] @t1 [^\x00\r\n] [\]] (spacechar | [\x00]) {
        if cursor == len + 1 {
            cursor -= 1;
        }
        return Some((cursor, s.as_bytes()[t1]));
    }
    * { return None; }
*/
}

pub fn description_item_start(s: &str) -> Option<usize> {
    let mut cursor = 0;
    let len = s.len();
/*!re2c
    [:~] ([ \t]+) { return Some(cursor); }
    * { return None; }
*/
}
