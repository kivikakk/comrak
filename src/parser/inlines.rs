mod cjk;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;
use std::str;

use crate::ctype::{isdigit, ispunct, isspace};
use crate::entity;
use crate::nodes::{
    Ast, Node, NodeCode, NodeFootnoteDefinition, NodeFootnoteReference, NodeLink, NodeMath,
    NodeValue, NodeWikiLink, Sourcepos,
};
use crate::parser::inlines::cjk::FlankingCheckHelper;
use crate::parser::options::{BrokenLinkReference, WikiLinksMode};
#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;
use crate::parser::{autolink, AutolinkType, Options, ResolvedReference};
use crate::scanners;
use crate::strings::{self, is_blank, Case};
use crate::Arena;

const MAXBACKTICKS: usize = 80;
const MAX_LINK_LABEL_LENGTH: usize = 1000;
const MAX_MATH_DOLLARS: usize = 2;

pub struct Subject<'a: 'd, 'r, 'o, 'd, 'c, 'p> {
    pub arena: &'a mut Arena,
    pub options: &'o Options<'c>,
    pub input: String,
    line: usize,
    pub scanner: Scanner,
    column_offset: isize,
    line_offset: usize,
    flags: HtmlSkipFlags,
    pub refmap: &'r mut RefMap,
    footnote_defs: &'p FootnoteDefs,
    delimiters: &'d mut DelimiterArena,
    last_delimiter: Option<DelimiterId>,
    brackets: Vec<Bracket>,
    within_brackets: bool,
    pub backticks: [usize; MAXBACKTICKS + 1],
    pub scanned_for_backticks: bool,
    no_link_openers: bool,
    special_char_bytes: [bool; 256],
    skip_char_bytes: [bool; 256],
    smart_char_bytes: [bool; 256],
    emph_delim_bytes: [bool; 256],
}

#[derive(Default)]
struct HtmlSkipFlags {
    cdata: bool,
    declaration: bool,
    pi: bool,
    comment: bool,
}

impl<'a, 'r, 'o, 'd, 'c, 'p> Subject<'a, 'r, 'o, 'd, 'c, 'p> {
    pub fn new(
        arena: &'a mut Arena,
        options: &'o Options<'c>,
        input: String,
        line: usize,
        refmap: &'r mut RefMap,
        footnote_defs: &'p FootnoteDefs,
        delimiter_arena: &'d mut DelimiterArena,
    ) -> Self {
        let mut s = Subject {
            arena,
            options,
            input,
            line,
            scanner: Scanner::new(),
            column_offset: 0,
            line_offset: 0,
            flags: HtmlSkipFlags::default(),
            refmap,
            footnote_defs,
            delimiters: delimiter_arena,
            last_delimiter: None,
            brackets: vec![],
            within_brackets: false,
            backticks: [0; MAXBACKTICKS + 1],
            scanned_for_backticks: false,
            no_link_openers: true,
            special_char_bytes: [false; 256],
            skip_char_bytes: [false; 256],
            smart_char_bytes: [false; 256],
            emph_delim_bytes: [false; 256],
        };
        for &b in b"\n\r_*\"`\\&<[]!$" {
            s.special_char_bytes[b as usize] = true;
        }
        if options.extension.autolink {
            s.special_char_bytes[b':' as usize] = true;
            s.special_char_bytes[b'w' as usize] = true;
        }
        if options.extension.strikethrough || options.extension.subscript {
            s.special_char_bytes[b'~' as usize] = true;
            s.skip_char_bytes[b'~' as usize] = true;
            s.emph_delim_bytes[b'~' as usize] = true;
        }
        if options.extension.superscript || options.extension.inline_footnotes {
            s.special_char_bytes[b'^' as usize] = true;
        }
        if options.extension.superscript {
            s.emph_delim_bytes[b'^' as usize] = true;
        }
        #[cfg(feature = "shortcodes")]
        if options.extension.shortcodes {
            s.special_char_bytes[b':' as usize] = true;
        }
        if options.extension.underline {
            s.special_char_bytes[b'_' as usize] = true;
        }
        if options.extension.spoiler {
            s.special_char_bytes[b'|' as usize] = true;
            s.emph_delim_bytes[b'|' as usize] = true;
        }
        for &b in b"\"'.-" {
            s.smart_char_bytes[b as usize] = true;
        }
        for &b in b"*_" {
            s.emph_delim_bytes[b as usize] = true;
        }
        s
    }

    //////////////////
    // Constructors //
    //////////////////

    fn make_inline(&mut self, value: NodeValue, start_column: usize, end_column: usize) -> Node {
        let start_column =
            start_column as isize + 1 + self.column_offset + self.line_offset as isize;
        let end_column = end_column as isize + 1 + self.column_offset + self.line_offset as isize;

        let ast = Ast {
            value,
            content: String::new(),
            sourcepos: (
                self.line,
                usize::try_from(start_column).unwrap(),
                self.line,
                usize::try_from(end_column).unwrap(),
            )
                .into(),
            open: false,
            last_line_blank: false,
            table_visited: false,
            line_offsets: Vec::new(),
        };
        self.arena.alloc(ast.into()).into()
    }

    fn make_autolink(
        &mut self,
        url: std::ops::Range<usize>,
        kind: AutolinkType,
        start_column: usize,
        end_column: usize,
    ) -> Node {
        let input = &self.input[url];
        let url = strings::clean_autolink(input, kind).into();
        let text = entity::unescape_html(input).into_owned().into();

        let inl = self.make_inline(
            NodeValue::Link(Box::new(NodeLink {
                url,
                title: String::new(),
            })),
            start_column,
            end_column,
        );
        let text = self.make_inline(NodeValue::Text(text), start_column + 1, end_column - 1);
        inl.append(self.arena, text);
        inl
    }

    /////////////
    // Parsers //
    /////////////

    pub fn parse_inline(&mut self, node: Node) -> bool {
        let Some(b) = self.peek_byte() else {
            return false;
        };

        let node_ast = node.data_mut(self.arena);
        let adjusted_line = self.line - node_ast.sourcepos.start.line;
        let mut line_offsets = mem::take(&mut node_ast.line_offsets);
        self.line_offset = line_offsets[adjusted_line];

        let new_inl: Option<Node> = match b {
            b'\0' => {
                let node_ast = node.data_mut(self.arena);
                mem::swap(&mut node_ast.line_offsets, &mut line_offsets);
                return false;
            }
            b'\r' | b'\n' => Some(self.handle_newline()),
            b'`' => Some(self.handle_backticks(&line_offsets)),
            b'\\' => Some(self.handle_backslash()),
            b'&' => Some(self.handle_entity()),
            b'<' => Some(self.handle_pointy_brace(&line_offsets)),
            b':' => {
                let mut res = None;

                if self.options.extension.autolink {
                    res = self.handle_autolink_with(node, autolink::url_match);
                }

                #[cfg(feature = "shortcodes")]
                if res.is_none() && self.options.extension.shortcodes {
                    res = self.handle_shortcodes_colon();
                }

                if res.is_none() {
                    self.scanner.pos += 1;
                    res = Some(self.make_inline(
                        NodeValue::Text(":".into()),
                        self.scanner.pos - 1,
                        self.scanner.pos - 1,
                    ));
                }

                res
            }
            b'w' if self.options.extension.autolink => {
                match self.handle_autolink_with(node, autolink::www_match) {
                    Some(inl) => Some(inl),
                    None => {
                        self.scanner.pos += 1;
                        Some(self.make_inline(
                            NodeValue::Text("w".into()),
                            self.scanner.pos - 1,
                            self.scanner.pos - 1,
                        ))
                    }
                }
            }
            b'*' | b'_' | b'\'' | b'"' => Some(self.handle_delim(b)),
            b'-' => Some(self.handle_hyphen()),
            b'.' => Some(self.handle_period()),
            b'[' => {
                self.scanner.pos += 1;

                let mut wikilink_inl = None;

                if self.options.extension.wikilinks().is_some()
                    && !self.within_brackets
                    && self.peek_byte() == Some(b'[')
                {
                    wikilink_inl = self.handle_wikilink();
                }

                if wikilink_inl.is_none() {
                    let inl = self.make_inline(
                        NodeValue::Text("[".into()),
                        self.scanner.pos - 1,
                        self.scanner.pos - 1,
                    );
                    self.push_bracket(false, inl);
                    self.within_brackets = true;

                    Some(inl)
                } else {
                    wikilink_inl
                }
            }
            b']' => {
                self.within_brackets = false;
                self.handle_close_bracket()
            }
            b'!' => {
                self.scanner.pos += 1;
                if self.peek_byte() == Some(b'[') && self.peek_byte_n(1) != Some(b'^') {
                    self.scanner.pos += 1;
                    let inl = self.make_inline(
                        NodeValue::Text("![".into()),
                        self.scanner.pos - 2,
                        self.scanner.pos - 1,
                    );
                    self.push_bracket(true, inl);
                    self.within_brackets = true;
                    Some(inl)
                } else {
                    Some(self.make_inline(
                        NodeValue::Text("!".into()),
                        self.scanner.pos - 1,
                        self.scanner.pos - 1,
                    ))
                }
            }
            b'~' if self.options.extension.strikethrough || self.options.extension.subscript => {
                Some(self.handle_delim(b'~'))
            }
            b'^' => {
                // Check for inline footnote first
                if self.options.extension.footnotes
                    && self.options.extension.inline_footnotes
                    && self.peek_byte_n(1) == Some(b'[')
                {
                    self.handle_inline_footnote()
                } else if self.options.extension.superscript && !self.within_brackets {
                    Some(self.handle_delim(b'^'))
                } else {
                    // Just regular text
                    self.scanner.pos += 1;
                    Some(self.make_inline(
                        NodeValue::Text("^".into()),
                        self.scanner.pos - 1,
                        self.scanner.pos - 1,
                    ))
                }
            }
            b'$' => Some(self.handle_dollars(&line_offsets)),
            b'|' if self.options.extension.spoiler => Some(self.handle_delim(b'|')),
            _ => {
                let mut endpos = self.find_special_char();
                let startpos = self.scanner.pos;
                self.scanner.pos = endpos;

                let mut contents: Cow<str> = if endpos == self.input.len() {
                    let mut contents = mem::take(&mut self.input);
                    strings::remove_from_start(&mut contents, startpos);
                    contents.into()
                } else {
                    self.input[startpos..endpos].into()
                };

                if self.peek_byte().map_or(false, strings::is_line_end_char) {
                    let size_before = contents.len();
                    strings::rtrim_cow(&mut contents);
                    endpos -= size_before - contents.len();
                }

                // Don't create empty text nodes - this can happen after trimming trailing
                // whitespace, is useless, and would cause sourcepos underflow in endpos - 1.
                if !contents.is_empty() {
                    Some(self.make_inline(
                        NodeValue::Text(contents.into_owned().into()),
                        startpos,
                        endpos - 1,
                    ))
                } else {
                    None
                }
            }
        };

        if let Some(inl) = new_inl {
            node.append(self.arena, inl);
        }

        let node_ast = node.data_mut(self.arena);
        mem::swap(&mut node_ast.line_offsets, &mut line_offsets);
        true
    }

    fn handle_newline(&mut self) -> Node {
        let nlpos = self.scanner.pos;
        if self.input.as_bytes()[self.scanner.pos] == b'\r' {
            self.scanner.pos += 1;
        }
        if self.input.as_bytes()[self.scanner.pos] == b'\n' {
            self.scanner.pos += 1;
        }
        let inl = if nlpos > 1
            && self.input.as_bytes()[nlpos - 1] == b' '
            && self.input.as_bytes()[nlpos - 2] == b' '
        {
            self.make_inline(NodeValue::LineBreak, nlpos - 2, self.scanner.pos - 1)
        } else {
            self.make_inline(NodeValue::SoftBreak, nlpos, self.scanner.pos - 1)
        };
        self.line += 1;
        self.column_offset = -(self.scanner.pos as isize);
        self.skip_spaces();
        inl
    }

    fn handle_backticks(&mut self, parent_line_offsets: &[usize]) -> Node {
        let startpos = self.scanner.pos;
        let openticks = self.take_while(b'`');
        let endpos = self.scan_to_closing_backtick(openticks);

        match endpos {
            None => {
                self.scanner.pos = startpos + openticks;
                self.make_inline(
                    NodeValue::Text("`".repeat(openticks).into()),
                    startpos,
                    self.scanner.pos - 1,
                )
            }
            Some(endpos) => {
                let buf = &self.input[startpos + openticks..endpos - openticks];
                let buf = strings::normalize_code(buf);
                let code = NodeCode {
                    num_backticks: openticks,
                    literal: buf.into(),
                };
                let node = self.make_inline(NodeValue::Code(code), startpos, endpos - 1);
                self.adjust_node_newlines(
                    node,
                    endpos - startpos - openticks,
                    openticks,
                    parent_line_offsets,
                );
                node
            }
        }
    }

    fn handle_backslash(&mut self) -> Node {
        let startpos = self.scanner.pos;
        self.scanner.pos += 1;

        if self.peek_byte().map_or(false, ispunct) {
            let inl;
            self.scanner.pos += 1;

            let inline_text = self.make_inline(
                NodeValue::Text(
                    self.input[self.scanner.pos - 1..self.scanner.pos]
                        .to_string()
                        .into(),
                ),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            );

            if self.options.render.escaped_char_spans {
                inl = self.make_inline(
                    NodeValue::Escaped,
                    self.scanner.pos - 2,
                    self.scanner.pos - 1,
                );
                inl.append(self.arena, inline_text);
                inl
            } else {
                inline_text
            }
        } else if !self.eof() && self.skip_line_end() {
            let inl = self.make_inline(NodeValue::LineBreak, startpos, self.scanner.pos - 1);
            self.line += 1;
            self.column_offset = -(self.scanner.pos as isize);
            self.skip_spaces();
            inl
        } else {
            self.make_inline(
                NodeValue::Text("\\".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            )
        }
    }

    fn handle_entity(&mut self) -> Node {
        self.scanner.pos += 1;

        match entity::unescape(&self.input[self.scanner.pos..]) {
            None => self.make_inline(
                NodeValue::Text("&".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            ),
            Some((entity, len)) => {
                self.scanner.pos += len;
                self.make_inline(
                    NodeValue::Text(entity),
                    self.scanner.pos - 1 - len,
                    self.scanner.pos - 1,
                )
            }
        }
    }

    fn handle_pointy_brace(&mut self, parent_line_offsets: &[usize]) -> Node {
        self.scanner.pos += 1;

        if let Some(matchlen) = scanners::autolink_uri(&self.input[self.scanner.pos..]) {
            self.scanner.pos += matchlen;
            let inl = self.make_autolink(
                self.scanner.pos - matchlen..self.scanner.pos - 1,
                AutolinkType::Uri,
                self.scanner.pos - 1 - matchlen,
                self.scanner.pos - 1,
            );
            return inl;
        }

        if let Some(matchlen) = scanners::autolink_email(&self.input[self.scanner.pos..]) {
            self.scanner.pos += matchlen;
            let inl = self.make_autolink(
                self.scanner.pos - matchlen..self.scanner.pos - 1,
                AutolinkType::Email,
                self.scanner.pos - 1 - matchlen,
                self.scanner.pos - 1,
            );
            return inl;
        }

        // Most comments below are verbatim from cmark upstream.
        let mut matchlen: Option<usize> = None;

        if self.scanner.pos + 2 <= self.input.len() {
            let b = self.input.as_bytes()[self.scanner.pos];
            if b == b'!' && !self.flags.comment {
                let b = self.input.as_bytes()[self.scanner.pos + 1];
                if b == b'-' && self.peek_byte_n(2) == Some(b'-') {
                    if self.peek_byte_n(3) == Some(b'>') {
                        matchlen = Some(4);
                    } else if self.peek_byte_n(3) == Some(b'-') && self.peek_byte_n(4) == Some(b'>')
                    {
                        matchlen = Some(5);
                    } else if let Some(m) =
                        scanners::html_comment(&self.input[self.scanner.pos + 1..])
                    {
                        matchlen = Some(m + 1);
                    } else {
                        self.flags.comment = true;
                    }
                } else if b == b'[' {
                    if !self.flags.cdata && self.scanner.pos + 3 <= self.input.len() {
                        if let Some(m) = scanners::html_cdata(&self.input[self.scanner.pos + 2..]) {
                            // The regex doesn't require the final "]]>". But if we're not at
                            // the end of input, it must come after the match. Otherwise,
                            // disable subsequent scans to avoid quadratic behavior.

                            // Adding 5 to matchlen for prefix "![", suffix "]]>"
                            if self.scanner.pos + m + 5 > self.input.len() {
                                self.flags.cdata = true;
                            } else {
                                matchlen = Some(m + 5);
                            }
                        }
                    }
                } else if !self.flags.declaration {
                    if let Some(m) = scanners::html_declaration(&self.input[self.scanner.pos + 1..])
                    {
                        // Adding 2 to matchlen for prefix "!", suffix ">"
                        if self.scanner.pos + m + 2 > self.input.len() {
                            self.flags.declaration = true;
                        } else {
                            matchlen = Some(m + 2);
                        }
                    }
                }
            } else if b == b'?' {
                if !self.flags.pi {
                    // Note that we allow an empty match.
                    let m =
                        scanners::html_processing_instruction(&self.input[self.scanner.pos + 1..])
                            .unwrap_or(0);
                    // Adding 3 to matchlen fro prefix "?", suffix "?>"
                    if self.scanner.pos + m + 3 > self.input.len() {
                        self.flags.pi = true;
                    } else {
                        matchlen = Some(m + 3);
                    }
                }
            } else {
                matchlen = scanners::html_tag(&self.input[self.scanner.pos..]);
            }
        }

        if let Some(matchlen) = matchlen {
            let contents = &self.input[self.scanner.pos - 1..self.scanner.pos + matchlen];
            self.scanner.pos += matchlen;
            let inl = self.make_inline(
                NodeValue::HtmlInline(contents.to_string()),
                self.scanner.pos - matchlen - 1,
                self.scanner.pos - 1,
            );
            self.adjust_node_newlines(inl, matchlen, 1, parent_line_offsets);
            return inl;
        }

        self.make_inline(
            NodeValue::Text("<".into()),
            self.scanner.pos - 1,
            self.scanner.pos - 1,
        )
    }

    fn handle_autolink_with(
        &mut self,
        node: Node,
        f: fn(&mut Subject<'a, '_, '_, '_, '_, '_>) -> Option<(Node, usize, usize)>,
    ) -> Option<Node> {
        if !self.options.parse.relaxed_autolinks && self.within_brackets {
            return None;
        }
        let startpos = self.scanner.pos;
        let (post, need_reverse, skip) = f(self)?;

        self.scanner.pos += skip - need_reverse;

        // We need to "rewind" by `need_reverse` chars, which should be in one
        // or more Text nodes beforehand. Typically the chars will *all* be in
        // a single Text node, containing whatever text came before the ":" that
        // triggered this method, eg. "See our website at http" ("://blah.com").
        //
        // relaxed_autolinks allows some slightly pathological cases. First,
        // "://…" is a possible parse, meaning `reverse == 0`. There may also be
        // a scheme including the letter "w", which will split Text inlines due
        // to them being their own trigger (for handle_autolink_w), meaning
        // "wa://…" will need to traverse two Texts to complete the rewind.
        let mut reverse = need_reverse;
        while reverse > 0 {
            let last_child = node.last_child(self.arena).unwrap().data_mut(self.arena);
            let NodeValue::Text(ref mut prev) = last_child.value else {
                panic!("expected text node before autolink colon");
            };
            let prev_len = prev.len();
            if reverse < prev.len() {
                prev.to_mut().truncate(prev_len - reverse);
                last_child.sourcepos.end.column -= reverse;
                reverse = 0;
            } else {
                reverse -= prev_len;
                node.last_child(self.arena).unwrap().detach(self.arena);
            }
        }

        {
            let sp = &mut post.data_mut(self.arena).sourcepos;
            // See [`make_inline`].
            sp.start = (
                self.line,
                (startpos as isize - need_reverse as isize
                    + 1
                    + self.column_offset
                    + self.line_offset as isize) as usize,
            )
                .into();
            sp.end = (
                self.line,
                (self.scanner.pos as isize + self.column_offset + self.line_offset as isize)
                    as usize,
            )
                .into();

            // Inner text node gets the same sp, since there are no surrounding
            // characters for autolinks of these kind.
            post.first_child(self.arena)
                .unwrap()
                .data_mut(self.arena)
                .sourcepos = *sp;
        }

        Some(post)
    }

    #[cfg(feature = "shortcodes")]
    fn handle_shortcodes_colon(&mut self) -> Option<Node> {
        let matchlen = scanners::shortcode(&self.input[self.scanner.pos + 1..])?;

        let shortcode = &self.input[self.scanner.pos + 1..self.scanner.pos + 1 + matchlen - 1];

        let nsc = NodeShortCode::resolve(shortcode)?;
        self.scanner.pos += 1 + matchlen;

        Some(self.make_inline(
            NodeValue::ShortCode(Box::new(nsc)),
            self.scanner.pos - 1 - matchlen,
            self.scanner.pos - 1,
        ))
    }

    fn handle_delim(&mut self, b: u8) -> Node {
        let (numdelims, can_open, can_close) = self.scan_delims(b);

        let contents: Cow<'static, str> = if b == b'\'' && self.options.parse.smart {
            "’".into()
        } else if b == b'"' && self.options.parse.smart {
            if can_close {
                "”".into()
            } else {
                "“".into()
            }
        } else {
            self.input[self.scanner.pos - numdelims..self.scanner.pos]
                .to_string()
                .into()
        };
        let inl = self.make_inline(
            NodeValue::Text(contents),
            self.scanner.pos - numdelims,
            self.scanner.pos - 1,
        );

        let is_valid_strikethrough_delim = if b == b'~' && self.options.extension.strikethrough {
            numdelims <= 2
        } else {
            true
        };

        if (can_open || can_close)
            && (!(b == b'\'' || b == b'"') || self.options.parse.smart)
            && is_valid_strikethrough_delim
        {
            self.push_delimiter(b, can_open, can_close, inl);
        }

        inl
    }

    fn handle_hyphen(&mut self) -> Node {
        let start = self.scanner.pos;
        self.scanner.pos += 1;

        if !self.options.parse.smart || self.peek_byte().map_or(true, |b| b != b'-') {
            return self.make_inline(
                NodeValue::Text("-".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            );
        }

        while self.options.parse.smart && self.peek_byte().map_or(false, |b| b == b'-') {
            self.scanner.pos += 1;
        }

        let numhyphens = (self.scanner.pos - start) as i32;

        let (ens, ems) = if numhyphens % 3 == 0 {
            (0, numhyphens / 3)
        } else if numhyphens % 2 == 0 {
            (numhyphens / 2, 0)
        } else if numhyphens % 3 == 2 {
            (1, (numhyphens - 2) / 3)
        } else {
            (2, (numhyphens - 4) / 3)
        };

        let ens = if ens > 0 { ens as usize } else { 0 };
        let ems = if ems > 0 { ems as usize } else { 0 };

        let mut buf = String::with_capacity(3 * (ems + ens));
        buf.push_str(&"—".repeat(ems));
        buf.push_str(&"–".repeat(ens));
        self.make_inline(NodeValue::Text(buf.into()), start, self.scanner.pos - 1)
    }

    fn handle_period(&mut self) -> Node {
        self.scanner.pos += 1;
        if self.options.parse.smart && self.peek_byte().map_or(false, |b| b == b'.') {
            self.scanner.pos += 1;
            if self.peek_byte().map_or(false, |b| b == b'.') {
                self.scanner.pos += 1;
                self.make_inline(
                    NodeValue::Text("…".into()),
                    self.scanner.pos - 3,
                    self.scanner.pos - 1,
                )
            } else {
                self.make_inline(
                    NodeValue::Text("..".into()),
                    self.scanner.pos - 2,
                    self.scanner.pos - 1,
                )
            }
        } else {
            self.make_inline(
                NodeValue::Text(".".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            )
        }
    }

    // Handles wikilink syntax
    //   [[link text|url]]
    //   [[url|link text]]
    fn handle_wikilink(&mut self) -> Option<Node> {
        let startpos = self.scanner.pos;
        let component = self.wikilink_url_link_label()?;
        let url_clean = strings::clean_url(&component.url);
        let (link_label, link_label_start_column, _link_label_end_column) =
            match component.link_label {
                Some((label, sc, ec)) => (entity::unescape_html(&label).to_string(), sc, ec),
                None => (
                    entity::unescape_html(&component.url).to_string(),
                    startpos + 1,
                    self.scanner.pos - 3,
                ),
            };

        let nl = NodeWikiLink {
            url: url_clean.into(),
        };
        let inl = self.make_inline(NodeValue::WikiLink(nl), startpos - 1, self.scanner.pos - 1);

        self.label_backslash_escapes(inl, &link_label, link_label_start_column);

        Some(inl)
    }

    fn wikilink_url_link_label(&mut self) -> Option<WikilinkComponents> {
        let left_startpos = self.scanner.pos;

        if self.peek_byte() != Some(b'[') {
            return None;
        }

        let found_left = self.wikilink_component();

        if !found_left {
            self.scanner.pos = left_startpos;
            return None;
        }

        let left =
            strings::trim_slice(&self.input[left_startpos + 1..self.scanner.pos]).to_string();

        if self.peek_byte() == Some(b']') && self.peek_byte_n(1) == Some(b']') {
            self.scanner.pos += 2;
            return Some(WikilinkComponents {
                url: left,
                link_label: None,
            });
        } else if self.peek_byte() != Some(b'|') {
            self.scanner.pos = left_startpos;
            return None;
        }

        let right_startpos = self.scanner.pos;
        let found_right = self.wikilink_component();

        if !found_right {
            self.scanner.pos = left_startpos;
            return None;
        }

        let right = strings::trim_slice(&self.input[right_startpos + 1..self.scanner.pos]);

        if self.peek_byte() == Some(b']') && self.peek_byte_n(1) == Some(b']') {
            self.scanner.pos += 2;

            match self.options.extension.wikilinks() {
                Some(WikiLinksMode::UrlFirst) => Some(WikilinkComponents {
                    url: left,
                    link_label: Some((right.into(), right_startpos + 1, self.scanner.pos - 3)),
                }),
                Some(WikiLinksMode::TitleFirst) => Some(WikilinkComponents {
                    url: right.into(),
                    link_label: Some((left, left_startpos + 1, right_startpos - 1)),
                }),
                None => unreachable!(),
            }
        } else {
            self.scanner.pos = left_startpos;
            None
        }
    }

    // Locates the edge of a wikilink component (link label or url), and sets the
    // self.scanner.pos to it's end if it's found.
    fn wikilink_component(&mut self) -> bool {
        let startpos = self.scanner.pos;

        if self.peek_byte() != Some(b'[') && self.peek_byte() != Some(b'|') {
            return false;
        }

        self.scanner.pos += 1;

        let mut length = 0;
        while let Some(b) = self.peek_byte() {
            if b == b'[' || b == b']' || b == b'|' {
                break;
            }

            if b == b'\\' {
                self.scanner.pos += 1;
                length += 1;
                if self.peek_byte().map_or(false, ispunct) {
                    self.scanner.pos += 1;
                    length += 1;
                }
            } else {
                self.scanner.pos += 1;
                length += 1;
            }
            if length > MAX_LINK_LABEL_LENGTH {
                self.scanner.pos = startpos;
                return false;
            }
        }

        true
    }

    // Given a label, handles backslash escaped characters. Appends the resulting
    // nodes to the container
    fn label_backslash_escapes(&mut self, container: Node, label: &str, start_column: usize) {
        let mut startpos = 0;
        let mut offset = 0;
        let bytes = label.as_bytes();
        let len = label.len();

        while offset < len {
            let b = bytes[offset];

            if b == b'\\' && (offset + 1) < len && ispunct(bytes[offset + 1]) {
                let preceding_text = self.make_inline(
                    NodeValue::Text(label[startpos..offset].to_string().into()),
                    start_column + startpos,
                    start_column + offset - 1,
                );

                container.append(self.arena, preceding_text);

                let inline_text = self.make_inline(
                    NodeValue::Text(label[offset + 1..offset + 2].to_string().into()),
                    start_column + offset,
                    start_column + offset + 1,
                );

                if self.options.render.escaped_char_spans {
                    let span = self.make_inline(
                        NodeValue::Escaped,
                        start_column + offset,
                        start_column + offset + 1,
                    );

                    span.append(self.arena, inline_text);
                    container.append(self.arena, span);
                } else {
                    container.append(self.arena, inline_text);
                }

                offset += 2;
                startpos = offset;
            } else {
                offset += 1;
            }
        }

        if startpos != offset {
            let inl = self.make_inline(
                NodeValue::Text(label[startpos..offset].to_string().into()),
                start_column + startpos,
                start_column + offset - 1,
            );
            container.append(self.arena, inl);
        }
    }

    fn handle_inline_footnote(&mut self) -> Option<Node> {
        let startpos = self.scanner.pos;

        // We're at ^, next should be [
        self.scanner.pos += 2; // Skip ^[

        // Find the closing ]
        let mut depth = 1;
        let mut endpos = self.scanner.pos;
        while endpos < self.input.len() && depth > 0 {
            match self.input.as_bytes()[endpos] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                b'\\' if endpos + 1 < self.input.len() => {
                    endpos += 1; // Skip escaped character
                }
                _ => {}
            }
            endpos += 1;
        }

        if depth != 0 {
            // No matching closing bracket, treat as regular text
            self.scanner.pos = startpos + 1;
            return Some(self.make_inline(NodeValue::Text("^".into()), startpos, startpos));
        }

        // endpos is now one past the ], so adjust
        endpos -= 1;

        // Extract the content
        let content = self.input[self.scanner.pos..endpos].to_string();

        // Empty inline footnote should not parse
        if content.is_empty() {
            self.scanner.pos = startpos + 1;
            return Some(self.make_inline(NodeValue::Text("^".into()), startpos, startpos));
        }

        // Generate unique name
        let name = self.footnote_defs.next_name();

        // Create the footnote reference node
        let ref_node = self.make_inline(
            NodeValue::FootnoteReference(NodeFootnoteReference {
                name: name.clone(),
                ref_num: 0,
                ix: 0,
            }),
            startpos,
            endpos,
        );

        // Parse the content as inlines
        let def_node: Node = self
            .arena
            .alloc(
                Ast::new(
                    NodeValue::FootnoteDefinition(NodeFootnoteDefinition {
                        name: name,
                        total_references: 0,
                    }),
                    (self.line, 1).into(),
                )
                .into(),
            )
            .into();

        // Create a paragraph to hold the inline content
        let mut para_ast = Ast::new(
            NodeValue::Paragraph,
            (1, 1).into(), // Use line 1 as base
        );
        // Build line_offsets by scanning for newlines in the content
        para_ast.line_offsets = vec![0];
        for (i, &byte) in content.as_bytes().iter().enumerate() {
            if byte == b'\n' {
                para_ast.line_offsets.push(i + 1);
            }
        }
        let para_node: Node = self.arena.alloc(para_ast.into()).into();
        def_node.append(self.arena, para_node);

        // Parse the content recursively as inlines
        let mut delimiter_arena = id_arena::Arena::new();
        let mut subj = Subject::new(
            self.arena,
            self.options,
            content,
            1, // Use line 1 to match the paragraph's sourcepos
            self.refmap,
            self.footnote_defs,
            &mut delimiter_arena,
        );

        while subj.parse_inline(para_node) {}
        subj.process_emphasis(0);
        subj.clear_brackets();

        // Check if the parsed content is empty or contains only whitespace
        // This handles whitespace-only content, null bytes, etc. generically
        let has_non_whitespace_content = para_node.children(self.arena).any(|child| {
            let child_data = child.data(self.arena);
            match &child_data.value {
                NodeValue::Text(text) => !text.trim().is_empty(),
                NodeValue::SoftBreak | NodeValue::LineBreak => false,
                _ => true, // Any other node type (link, emphasis, etc.) counts as content
            }
        });

        if !has_non_whitespace_content {
            // Content is empty or whitespace-only after parsing, treat as literal text
            self.scanner.pos = startpos + 1;
            return Some(self.make_inline(NodeValue::Text("^".into()), startpos, startpos));
        }

        // Store the footnote definition
        self.footnote_defs.add_definition(def_node);

        // Move position past the closing ]
        self.scanner.pos = endpos + 1;

        Some(ref_node)
    }

    // Heuristics used from https://pandoc.org/MANUAL.html#extension-tex_math_dollars
    fn handle_dollars(&mut self, parent_line_offsets: &[usize]) -> Node {
        if !(self.options.extension.math_dollars || self.options.extension.math_code) {
            self.scanner.pos += 1;
            return self.make_inline(
                NodeValue::Text("$".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            );
        }
        let startpos = self.scanner.pos;
        let opendollars = self.take_while(b'$');
        let mut code_math = false;

        // check for code math
        if opendollars == 1
            && self.options.extension.math_code
            && self.peek_byte().map_or(false, |b| b == b'`')
        {
            code_math = true;
            self.scanner.pos += 1;
        }
        let fence_length = if code_math { 2 } else { opendollars };

        let endpos: Option<usize> = if code_math {
            self.scan_to_closing_code_dollar()
        } else {
            self.scan_to_closing_dollar(opendollars)
        }
        .filter(|endpos| endpos - startpos >= fence_length * 2 + 1);

        if let Some(endpos) = endpos {
            let buf = &self.input[startpos + fence_length..endpos - fence_length];
            let buf = if code_math || opendollars == 1 {
                strings::normalize_code(buf)
            } else {
                buf.into()
            };
            let math = NodeMath {
                dollar_math: !code_math,
                display_math: opendollars == 2,
                literal: buf.into(),
            };
            let node = self.make_inline(NodeValue::Math(math), startpos, endpos - 1);
            self.adjust_node_newlines(
                node,
                endpos - startpos - fence_length,
                fence_length,
                parent_line_offsets,
            );
            node
        } else if code_math {
            self.scanner.pos = startpos + 1;
            self.make_inline(
                NodeValue::Text("$".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            )
        } else {
            self.scanner.pos = startpos + fence_length;
            self.make_inline(
                NodeValue::Text("$".repeat(opendollars).into()),
                self.scanner.pos - fence_length,
                self.scanner.pos - 1,
            )
        }
    }

    /////////////////////////////////////
    // Emphasis and bracket processing //
    /////////////////////////////////////

    // After parsing a block (and sometimes during), this function traverses the
    // stack of `Delimiters`, tokens ("*", "_", etc.) that may delimit regions
    // of text for special rendering: emphasis, strong, superscript, subscript,
    // spoilertext; looking for pairs of opening and closing delimiters,
    // with the goal of placing the intervening nodes into new emphasis,
    // etc AST nodes.
    //
    // The term stack here is a bit of a misnomer, as the `Delimiters` actually
    // form a doubly-linked list. Items are pushed onto the stack during parsing,
    // but during post-processing are removed from arbitrary locations.
    //
    // The `Delimiter` contains references AST `Text` nodes, which are also
    // linked into the AST as siblings in the order they are parsed. This
    // function doesn't know a-priori which ones are markdown syntax and which
    // are just text: candidate delimiters that match have their nodes removed
    // from the AST, as they are markdown, and their intervening siblings
    // lowered into a new AST parent node via the `insert_emph` function;
    // candidate delimiters that don't match are left in the tree.
    //
    // The basic algorithm is to start at the bottom of the stack, walk upwards
    // looking for closing delimiters, and from each closing delimiter walk back
    // down the stack looking for its matching opening delimiter. This traversal
    // favors the smallest matching leftmost pairs, e.g.
    //
    //   _a *b c_ d* e_
    //    ~~~~~~
    //
    // (The emphasis region is wavy-underlined)
    //
    // All of the `_` and `*` tokens are scanned as candidates, but only the
    // region "a *b c" is lowered into an `Emph` node; the other candidate
    // delimiters are all actually text.
    //
    // And in
    //
    //   _a _b c_
    //       ~~~
    //
    // "b c" is the emphasis region, not "a _b c".
    //
    // Note that Delimiters are matched by comparing their `delim_char`, which
    // is simply a value used to compare opening and closing delimiters - the
    // actual text value of the scanned token can theoretically be different.
    //
    // There's some additional trickiness in the logic because "_", "__", and
    // "___" (and etc. etc.) all share the same delim_char, but represent
    // different emphasis. Note also that "_"- and "*"-delimited regions have
    // complex rules for which can be opening and/or closing delimiters,
    // determined in `scan_delims`.
    pub fn process_emphasis(&mut self, stack_bottom: usize) {
        // This array is an important optimization that prevents searching down
        // the stack for openers we've previously searched for and know don't
        // exist, preventing exponential blowup on pathological cases.
        let mut openers_bottom: [usize; 12] = [stack_bottom; 12];

        // This is traversing the stack from the top to the bottom, setting `closer` to
        // the delimiter directly above `stack_bottom`. In the case where we are processing
        // emphasis on an entire block, `stack_bottom` is `None`, so `closer` references
        // the very bottom of the stack.
        let mut candidate = self.last_delimiter;
        let mut closer: Option<DelimiterId> = None;
        while let Some(ci) = candidate {
            let c = &self.delimiters[ci];
            if c.position < stack_bottom {
                break;
            }
            closer = candidate;
            candidate = c.prev.get();
        }

        while let Some(ci) = closer {
            let c = &self.delimiters[ci];
            if c.can_close {
                // Each time through the outer `closer` loop we reset the opener
                // to the element below the closer, and search down the stack
                // for a matching opener.

                let mut opener = c.prev.get();
                let mut opener_found = false;
                let mut mod_three_rule_invoked = false;

                let ix = match c.delim_byte {
                    b'|' => 0,
                    b'~' => 1,
                    b'^' => 2,
                    b'"' => 3,
                    b'\'' => 4,
                    b'_' => 5,
                    b'*' => 6 + (if c.can_open { 3 } else { 0 }) + (c.length % 3),
                    _ => unreachable!(),
                };

                // Here's where we find the opener by searching down the stack,
                // looking for matching delims with the `can_open` flag.
                // On any invocation, on the first time through the outer
                // `closer` loop, this inner `opener` search doesn't succeed:
                // when processing a full block, `opener` starts out `None`;
                // when processing emphasis otherwise, opener will be equal to
                // `stack_bottom`.
                //
                // This search short-circuits for openers we've previously
                // failed to find, avoiding repeatedly rescanning the bottom of
                // the stack, using the openers_bottom array.
                while opener.map_or(false, |o| self.delimiters[o].position >= openers_bottom[ix]) {
                    let oi = opener.unwrap();
                    let o = &self.delimiters[oi];
                    if o.can_open && o.delim_byte == c.delim_byte {
                        // This is a bit convoluted; see points 9 and 10 here:
                        // http://spec.commonmark.org/0.28/#can-open-emphasis.
                        // This is to aid processing of runs like this:
                        // “***hello*there**” or “***hello**there*”. In this
                        // case, the middle delimiter can both open and close
                        // emphasis; when trying to find an opening delimiter
                        // that matches the last ** or *, we need to skip it,
                        // and this algorithm ensures we do. (The sum of the
                        // lengths are a multiple of 3.)
                        let odd_match = (c.can_open || o.can_close)
                            && ((o.length + c.length) % 3 == 0)
                            && !(o.length % 3 == 0 && c.length % 3 == 0);
                        if !odd_match {
                            opener_found = true;
                            break;
                        } else {
                            mod_three_rule_invoked = true;
                        }
                    }
                    opener = o.prev.get();
                }

                // There's a case here for every possible delimiter. If we found
                // a matching opening delimiter for our closing delimiter, they
                // both get passed.
                if self.emph_delim_bytes[c.delim_byte as usize] {
                    if opener_found {
                        // Finally, here's the happy case where the delimiters
                        // match and they are inserted. We get a new closer
                        // delimiter and go around the loop again.
                        //
                        // Note that for "***" and "___" delimiters of length
                        // greater than 2, insert_emph will create a `Strong`
                        // node (i.e. "**"), then _truncate_ the delimiters in
                        // place, turning them into e.g. "*" delimiters, and
                        // hand us back the same mutated closer to be matched
                        // again.
                        //
                        // In general though the closer will be the next
                        // delimiter up the stack.
                        closer = self.insert_emph(opener.unwrap(), ci);
                    } else {
                        // When no matching opener is found we move the closer
                        // up the stack, do some bookkeeping with old_closer
                        // (below), try again.
                        closer = c.next.get();
                    }
                } else if c.delim_byte == b'\'' || c.delim_byte == b'"' {
                    *c.inl.data_mut(self.arena).value.text_mut().unwrap() =
                        if c.delim_byte == b'\'' { "’" } else { "”" }.into();
                    closer = c.next.get();

                    if opener_found {
                        *self.delimiters[opener.unwrap()]
                            .inl
                            .data_mut(self.arena)
                            .value
                            .text_mut()
                            .unwrap() = if c.delim_byte == b'\'' { "‘" } else { "“" }.into();
                        self.remove_delimiter(opener.unwrap());
                        self.remove_delimiter(ci);
                    }
                }

                // If the search for an opener was unsuccessful, then record
                // the position the search started at in the `openers_bottom`
                // so that the `opener` search can avoid looking for this
                // same opener at the bottom of the stack later.
                if !opener_found {
                    let c = &self.delimiters[ci];
                    if !mod_three_rule_invoked {
                        openers_bottom[ix] = c.position;
                    }

                    // Now that we've failed the `opener` search starting from
                    // `old_closer`, future opener searches will be searching it
                    // for openers - if `old_closer` can't be used as an opener
                    // then we know it's just text - remove it from the
                    // delimiter stack, leaving it in the AST as text
                    if !c.can_open {
                        self.remove_delimiter(ci);
                    }
                }
            } else {
                // Closer is !can_close. Move up the stack
                closer = c.next.get();
            }
        }

        // At this point the entire delimiter stack from `stack_bottom` up has
        // been scanned for matches, everything left is just text. Pop it all
        // off.
        self.remove_delimiters(stack_bottom);
    }

    fn remove_delimiter(&mut self, id: DelimiterId) {
        let delimiter = &self.delimiters[id];
        if let Some(next_id) = delimiter.next.get() {
            self.delimiters[next_id].prev.set(delimiter.prev.get());
        } else {
            assert!(Some(id) == self.last_delimiter);
            self.last_delimiter = delimiter.prev.get();
        }

        if let Some(prev_id) = delimiter.prev.get() {
            self.delimiters[prev_id].next.set(delimiter.next.get());
        }
    }

    fn remove_delimiters(&mut self, stack_bottom: usize) {
        while let Some(id) = self.last_delimiter {
            if self.delimiters[id].position < stack_bottom {
                break;
            }
            self.remove_delimiter(id);
        }
    }

    fn push_delimiter(&mut self, b: u8, can_open: bool, can_close: bool, inl: Node) {
        let d = self.delimiters.alloc(Delimiter {
            prev: Cell::new(self.last_delimiter),
            next: Cell::new(None),
            inl,
            position: self.scanner.pos,
            length: inl.data(self.arena).value.text().unwrap().len(),
            delim_byte: b,
            can_open,
            can_close,
        });
        if let Some(last) = self.last_delimiter {
            self.delimiters[last].next.set(Some(d));
        }
        self.last_delimiter = Some(d);
    }

    // Create a new emphasis node, move all the nodes between `opener`
    // and `closer` into it, and insert it into the AST.
    //
    // As a side-effect, handle long "***" and "___" nodes by truncating them in
    // place to be re-matched by `process_emphasis`.
    fn insert_emph(
        &mut self,
        opener_id: DelimiterId,
        closer_id: DelimiterId,
    ) -> Option<DelimiterId> {
        let opener_inl = self.delimiters[opener_id].inl;
        let closer_inl = self.delimiters[closer_id].inl;

        let opener_text = opener_inl.data(self.arena).value.text().unwrap();
        let opener_byte = opener_text.as_bytes()[0];
        let mut opener_num_bytes = opener_text.len();
        let mut closer_num_bytes = closer_inl.data(self.arena).value.text().unwrap().len();

        let use_delims = if closer_num_bytes >= 2 && opener_num_bytes >= 2 {
            2
        } else {
            1
        };

        opener_num_bytes -= use_delims;
        closer_num_bytes -= use_delims;

        if (self.options.extension.strikethrough || self.options.extension.subscript)
            && opener_byte == b'~'
            && (opener_num_bytes != closer_num_bytes || opener_num_bytes > 0)
        {
            return None;
        }

        opener_inl
            .data_mut(self.arena)
            .value
            .text_mut()
            .unwrap()
            .to_mut()
            .truncate(opener_num_bytes);
        closer_inl
            .data_mut(self.arena)
            .value
            .text_mut()
            .unwrap()
            .to_mut()
            .truncate(closer_num_bytes);

        // Remove all the candidate delimiters from between the opener and the
        // closer. None of them are matched pairs. They've been scanned already.
        let mut prev_id = self.delimiters[closer_id].prev.get();
        while prev_id.is_some() && prev_id != Some(opener_id) {
            self.remove_delimiter(prev_id.unwrap());
            prev_id = self.delimiters[prev_id.unwrap()].prev.get();
        }

        let emph = make_inline(
            self.arena,
            if self.options.extension.subscript && opener_byte == b'~' && use_delims == 1 {
                NodeValue::Subscript
            } else if opener_byte == b'~' {
                // Not emphasis
                // Unlike for |, these cases have to be handled because they will match
                // in the event subscript but not strikethrough is enabled
                if self.options.extension.strikethrough {
                    NodeValue::Strikethrough
                } else if use_delims == 1 {
                    NodeValue::EscapedTag("~".to_owned())
                } else {
                    NodeValue::EscapedTag("~~".to_owned())
                }
            } else if self.options.extension.superscript && opener_byte == b'^' {
                NodeValue::Superscript
            } else if self.options.extension.spoiler && opener_byte == b'|' {
                if use_delims == 2 {
                    NodeValue::SpoileredText
                } else {
                    NodeValue::EscapedTag("|".to_owned())
                }
            } else if self.options.extension.underline && opener_byte == b'_' && use_delims == 2 {
                NodeValue::Underline
            } else if use_delims == 1 {
                NodeValue::Emph
            } else {
                NodeValue::Strong
            },
            // These are overriden immediately below.
            (
                opener_inl
                    .data(self.arena)
                    .sourcepos
                    .start
                    .column_add(opener_num_bytes as isize),
                closer_inl
                    .data(self.arena)
                    .sourcepos
                    .end
                    .column_add(-(closer_num_bytes as isize)),
            )
                .into(),
        );

        // Drop all the interior AST nodes into the emphasis node
        // and then insert the emphasis node
        let mut it = opener_inl.next_sibling(self.arena).unwrap();
        while it != closer_inl {
            let next = it.next_sibling(self.arena);
            emph.append(self.arena, it);
            if let Some(n) = next {
                it = n;
            } else {
                break;
            }
        }
        opener_inl.insert_after(self.arena, emph);

        // Drop completely "used up" delimiters, adjust sourcepos of those not,
        // and return the next closest one for processing.
        if opener_num_bytes == 0 {
            opener_inl.detach(self.arena);
            self.remove_delimiter(opener_id);
        } else {
            opener_inl.data_mut(self.arena).sourcepos.end.column -= use_delims;
        }

        if closer_num_bytes == 0 {
            closer_inl.detach(self.arena);
            self.remove_delimiter(closer_id);
            self.delimiters[closer_id].next.get()
        } else {
            closer_inl.data_mut(self.arena).sourcepos.start.column += use_delims;
            Some(closer_id)
        }
    }

    fn push_bracket(&mut self, image: bool, inl_text: Node) {
        if let Some(last) = self.brackets.last_mut() {
            last.bracket_after = true;
        }
        self.brackets.push(Bracket {
            inl_text,
            position: self.scanner.pos,
            image,
            bracket_after: false,
        });
        if !image {
            self.no_link_openers = false;
        }
    }

    fn handle_close_bracket(&mut self) -> Option<Node> {
        self.scanner.pos += 1;
        let initial_pos = self.scanner.pos;

        let Some(last) = self.brackets.last() else {
            return Some(self.make_inline(
                NodeValue::Text("]".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            ));
        };

        let is_image = last.image;

        if !is_image && self.no_link_openers {
            self.brackets.pop();
            return Some(self.make_inline(
                NodeValue::Text("]".into()),
                self.scanner.pos - 1,
                self.scanner.pos - 1,
            ));
        }

        // Ensure there was text if this was a link and not an image link
        if self.options.render.ignore_empty_links && !is_image {
            let mut non_blank_found = false;
            let mut itm = last.inl_text.next_sibling(self.arena);
            while let Some(it) = itm {
                match it.data(self.arena).value {
                    NodeValue::Text(ref s) if is_blank(s) => (),
                    _ => {
                        non_blank_found = true;
                        break;
                    }
                }

                itm = it.next_sibling(self.arena);
            }

            if !non_blank_found {
                self.brackets.pop();
                return Some(self.make_inline(
                    NodeValue::Text("]".into()),
                    self.scanner.pos - 1,
                    self.scanner.pos - 1,
                ));
            }
        }

        let after_link_text_pos = self.scanner.pos;

        // Try to find a link destination within parenthesis

        if self.peek_byte() == Some(b'(') {
            let sps = scanners::spacechars(&self.input[self.scanner.pos + 1..]).unwrap_or(0);
            let offset = self.scanner.pos + 1 + sps;
            if offset < self.input.len() {
                if let Some((url, n)) = manual_scan_link_url(&self.input[offset..]) {
                    let starturl = self.scanner.pos + 1 + sps;
                    let endurl = starturl + n;
                    let starttitle =
                        endurl + scanners::spacechars(&self.input[endurl..]).unwrap_or(0);
                    let endtitle = if starttitle == endurl {
                        starttitle
                    } else {
                        starttitle + scanners::link_title(&self.input[starttitle..]).unwrap_or(0)
                    };
                    let endall =
                        endtitle + scanners::spacechars(&self.input[endtitle..]).unwrap_or(0);

                    if endall < self.input.len() && self.input.as_bytes()[endall] == b')' {
                        self.scanner.pos = endall + 1;
                        let url = strings::clean_url(url);
                        let title = strings::clean_title(&self.input[starttitle..endtitle]);
                        self.close_bracket_match(is_image, url.into(), title.into());
                        return None;
                    } else {
                        self.scanner.pos = after_link_text_pos;
                    }
                }
            }
        }

        // Try to see if this is a reference link

        let (mut lab, mut found_label): (Cow<str>, bool) =
            match self.scanner.link_label(&self.input) {
                Some(lab) => (lab.to_string().into(), true),
                None => ("".into(), false),
            };

        if !found_label {
            self.scanner.pos = initial_pos;
        }

        if (!found_label || lab.is_empty()) && !last.bracket_after {
            lab = self.input[last.position..initial_pos - 1].into();
            found_label = true;
        }

        // Need to normalize both to lookup in refmap and to call callback
        let unfolded_lab = lab.clone();
        let lab = strings::normalize_label(&lab, Case::Fold);
        let mut reff: Option<Cow<ResolvedReference>> = if found_label {
            self.refmap.lookup(&lab).map(Cow::Borrowed)
        } else {
            None
        };

        // Attempt to use the provided broken link callback if a reference cannot be resolved
        if reff.is_none() {
            if let Some(callback) = &self.options.parse.broken_link_callback {
                reff = callback
                    .resolve(BrokenLinkReference {
                        normalized: &lab,
                        original: &unfolded_lab,
                    })
                    .map(Cow::Owned);
            }
        }

        if let Some(reff) = reff {
            self.close_bracket_match(is_image, reff.url.clone(), reff.title.clone());
            return None;
        }

        let bracket_inl_text = last.inl_text;

        if self.options.extension.footnotes
            && match bracket_inl_text.next_sibling(self.arena) {
                Some(n) => {
                    if n.data(self.arena).value.text().is_some() {
                        n.data(self.arena)
                            .value
                            .text()
                            .unwrap()
                            .as_bytes()
                            .starts_with(b"^")
                    } else {
                        false
                    }
                }
                _ => false,
            }
        {
            let mut text = String::new();
            let mut sibling_iterator = bracket_inl_text.following_siblings(self.arena);

            self.scanner.pos = initial_pos;

            // Skip the initial node, which holds the `[`
            sibling_iterator.next().unwrap();

            // The footnote name could have been parsed into multiple text/htmlinline nodes.
            // For example `[^_foo]` gives `^`, `_`, and `foo`. So pull them together.
            // Since we're handling the closing bracket, the only siblings at this point are
            // related to the footnote name.
            for sibling in sibling_iterator {
                match sibling.data(self.arena).value {
                    NodeValue::Text(ref literal) => {
                        text.push_str(literal);
                    }
                    NodeValue::HtmlInline(ref literal) => {
                        text.push_str(literal);
                    }
                    _ => {}
                };
            }

            if text.len() > 1 {
                let inl = make_inline(
                    self.arena,
                    NodeValue::FootnoteReference(NodeFootnoteReference {
                        name: text[1..].to_string(),
                        ref_num: 0,
                        ix: 0,
                    }),
                    (
                        self.line,
                        bracket_inl_text.data(self.arena).sourcepos.start.column,
                        self.line,
                        usize::try_from(
                            self.scanner.pos as isize
                                + self.column_offset
                                + self.line_offset as isize,
                        )
                        .unwrap(),
                    )
                        .into(),
                );
                bracket_inl_text.insert_before(self.arena, inl);

                // detach all the nodes, including bracket_inl_text
                sibling_iterator = bracket_inl_text.following_siblings(self.arena);
                for sibling in sibling_iterator {
                    match sibling.data(self.arena).value {
                        NodeValue::Text(_) | NodeValue::HtmlInline(_) => {
                            sibling.detach(self.arena);
                        }
                        _ => {}
                    };
                }

                // We don't need to process emphasis for footnote names, so cleanup
                // any outstanding delimiters
                self.remove_delimiters(last.position);

                self.brackets.pop();
                return None;
            }
        }

        self.brackets.pop();
        self.scanner.pos = initial_pos;
        Some(self.make_inline(
            NodeValue::Text("]".into()),
            self.scanner.pos - 1,
            self.scanner.pos - 1,
        ))
    }

    fn close_bracket_match(&mut self, is_image: bool, url: String, title: String) {
        let last = self.brackets.pop().unwrap();

        let nl = NodeLink { url, title };
        let inl = make_inline(
            self.arena,
            if is_image {
                NodeValue::Image(Box::new(nl))
            } else {
                NodeValue::Link(Box::new(nl))
            },
            (
                last.inl_text.data(self.arena).sourcepos.start,
                (
                    self.line,
                    usize::try_from(
                        self.scanner.pos as isize + self.column_offset + self.line_offset as isize,
                    )
                    .unwrap(),
                )
                    .into(),
            )
                .into(),
        );

        last.inl_text.insert_before(self.arena, inl);
        let mut itm = last.inl_text.next_sibling(self.arena);
        while let Some(it) = itm {
            itm = it.next_sibling(self.arena);
            inl.append(self.arena, it);
        }
        last.inl_text.detach(self.arena);
        self.process_emphasis(last.position);

        if !is_image {
            self.no_link_openers = true;
        }
    }

    pub fn clear_brackets(&mut self) {
        self.brackets.clear();
    }

    ////////////////////
    // Input scanning //
    ////////////////////

    #[inline]
    fn eof(&self) -> bool {
        self.scanner.eof(&self.input)
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.scanner.peek_byte(&self.input)
    }

    #[inline]
    fn peek_byte_n(&self, n: usize) -> Option<u8> {
        self.scanner.peek_byte_n(&self.input, n)
    }

    #[inline]
    fn skip_spaces(&mut self) -> bool {
        self.scanner.skip_spaces(&self.input)
    }

    #[inline]
    pub fn skip_line_end(&mut self) -> bool {
        self.scanner.skip_line_end(&self.input)
    }

    #[inline]
    fn take_while(&mut self, b: u8) -> usize {
        self.scanner.take_while(&self.input, b)
    }

    #[inline]
    fn take_while_with_limit(&mut self, b: u8, limit: usize) -> usize {
        self.scanner.take_while_with_limit(&self.input, b, limit)
    }

    fn find_special_char(&self) -> usize {
        for n in self.scanner.pos..self.input.len() {
            if self.special_char_bytes[self.input.as_bytes()[n] as usize] {
                if self.input.as_bytes()[n] == b'^' && self.within_brackets {
                    // NO OP
                } else {
                    return n;
                }
            }
            if self.options.parse.smart && self.smart_char_bytes[self.input.as_bytes()[n] as usize]
            {
                return n;
            }
        }

        self.input.len()
    }

    fn scan_to_closing_backtick(&mut self, openticklength: usize) -> Option<usize> {
        if openticklength > MAXBACKTICKS {
            return None;
        }

        if self.scanned_for_backticks && self.backticks[openticklength] <= self.scanner.pos {
            return None;
        }

        loop {
            while self.peek_byte().map_or(false, |b| b != b'`') {
                self.scanner.pos += 1;
            }
            if self.scanner.pos >= self.input.len() {
                self.scanned_for_backticks = true;
                return None;
            }
            let numticks = self.take_while(b'`');
            if numticks <= MAXBACKTICKS {
                self.backticks[numticks] = self.scanner.pos - numticks;
            }
            if numticks == openticklength {
                return Some(self.scanner.pos);
            }
        }
    }

    fn scan_to_closing_dollar(&mut self, opendollarlength: usize) -> Option<usize> {
        if !self.options.extension.math_dollars || opendollarlength > MAX_MATH_DOLLARS {
            return None;
        }

        // space not allowed after initial $
        if opendollarlength == 1 && self.peek_byte().map_or(false, isspace) {
            return None;
        }

        loop {
            while self.peek_byte().map_or(false, |b| b != b'$') {
                self.scanner.pos += 1;
            }

            if self.scanner.pos >= self.input.len() {
                return None;
            }

            let b = self.input.as_bytes()[self.scanner.pos - 1];

            // space not allowed before ending $
            if opendollarlength == 1 && isspace(b) {
                return None;
            }

            // dollar signs must also be backslash-escaped if they occur within math
            if opendollarlength == 1 && b == b'\\' {
                self.scanner.pos += 1;
                continue;
            }

            let numdollars = self.take_while_with_limit(b'$', opendollarlength);

            // ending $ can't be followed by a digit
            if opendollarlength == 1 && self.peek_byte().map_or(false, isdigit) {
                return None;
            }

            if numdollars == opendollarlength {
                return Some(self.scanner.pos);
            }
        }
    }

    fn scan_to_closing_code_dollar(&mut self) -> Option<usize> {
        assert!(self.options.extension.math_code);

        loop {
            while self.peek_byte().map_or(false, |b| b != b'$') {
                self.scanner.pos += 1;
            }

            if self.scanner.pos >= self.input.len() {
                return None;
            }

            let b = self.input.as_bytes()[self.scanner.pos - 1];
            self.scanner.pos += 1;
            if b == b'`' {
                return Some(self.scanner.pos);
            }
        }
    }

    fn get_before_char(&self, pos: usize) -> (char, Option<usize>) {
        if pos == 0 {
            return ('\n', None);
        }
        let mut before_char_pos = pos - 1;
        while before_char_pos > 0
            && (self.input.as_bytes()[before_char_pos] >> 6 == 2
                || self.skip_char_bytes[self.input.as_bytes()[before_char_pos] as usize])
        {
            before_char_pos -= 1;
        }
        match self.input[before_char_pos..pos].chars().next() {
            Some(x) => {
                if (x as usize) < 256 && self.skip_char_bytes[x as usize] {
                    ('\n', None)
                } else {
                    (x, Some(before_char_pos))
                }
            }
            None => ('\n', None),
        }
    }

    fn scan_delims(&mut self, b: u8) -> (usize, bool, bool) {
        let (before_char, before_char_pos) = self.get_before_char(self.scanner.pos);

        let mut numdelims = 0;
        if b == b'\'' || b == b'"' {
            numdelims += 1;
            self.scanner.pos += 1;
        } else {
            while self.peek_byte() == Some(b) {
                numdelims += 1;
                self.scanner.pos += 1;
            }
        }

        let after_char = if self.eof() {
            '\n'
        } else {
            let mut after_char_pos = self.scanner.pos;
            while after_char_pos < self.input.len() - 1
                && self.skip_char_bytes[self.input.as_bytes()[after_char_pos] as usize]
            {
                after_char_pos += 1;
            }
            match self.input[after_char_pos..].chars().next() {
                Some(x) => {
                    if (x as usize) < 256 && self.skip_char_bytes[x as usize] {
                        '\n'
                    } else {
                        x
                    }
                }
                None => '\n',
            }
        };

        let cjk_friendly = self.options.extension.cjk_friendly_emphasis;
        let mut two_before_char: Option<char> = None;

        let left_flanking = numdelims > 0
            && !after_char.is_whitespace()
            && (!after_char.is_cmark_punctuation()
                || (self.options.extension.superscript && b == b'^')
                || (self.options.extension.subscript && b == b'~')
                || before_char.is_whitespace()
                || if !cjk_friendly {
                    before_char.is_cmark_punctuation()
                } else {
                    after_char.is_cjk()
                        || if before_char.is_non_emoji_general_purpose_vs() {
                            if let Some(before_char_pos) = before_char_pos {
                                let (two_before_char_, _) = self.get_before_char(before_char_pos);
                                two_before_char = Some(two_before_char_);
                                two_before_char_.is_cjk()
                                    || two_before_char_.is_cmark_punctuation()
                                    || two_before_char_.is_cjk_ambiguous_punctuation_candidate()
                                        && before_char == '\u{fe01}'
                            } else {
                                false
                            }
                        } else {
                            before_char.is_cjk_or_ideographic_vs()
                                || before_char.is_cmark_punctuation()
                        }
                });
        let right_flanking = numdelims > 0
            && !before_char.is_whitespace()
            && (!if !cjk_friendly {
                before_char.is_cmark_punctuation()
            } else {
                !after_char.is_cjk()
                    && if before_char.is_non_emoji_general_purpose_vs() {
                        let two_before_char = if let Some(two_before_char_) = two_before_char {
                            two_before_char_
                        } else if let Some(before_char_pos) = before_char_pos {
                            let (two_before_char_, _) = self.get_before_char(before_char_pos);
                            two_before_char = Some(two_before_char_);
                            two_before_char_
                        } else {
                            '\n'
                        };
                        !two_before_char.is_cjk()
                            && two_before_char.is_cmark_punctuation()
                            && !(two_before_char.is_cjk_ambiguous_punctuation_candidate()
                                && before_char == '\u{fe01}')
                    } else {
                        !before_char.is_cjk() && before_char.is_cmark_punctuation()
                    }
            } || after_char.is_whitespace()
                || after_char.is_cmark_punctuation());

        if b == b'_' {
            (
                numdelims,
                left_flanking
                    && (!right_flanking
                        || if !(cjk_friendly && before_char.is_non_emoji_general_purpose_vs()) {
                            before_char.is_cmark_punctuation()
                        } else {
                            let two_before_char = if let Some(two_before_char_) = two_before_char {
                                two_before_char_
                            } else if let Some(before_char_pos) = before_char_pos {
                                self.get_before_char(before_char_pos).0
                            } else {
                                '\n'
                            };
                            two_before_char.is_cmark_punctuation()
                        }),
                right_flanking && (!left_flanking || after_char.is_cmark_punctuation()),
            )
        } else if b == b'\'' || b == b'"' {
            (
                numdelims,
                left_flanking
                    && (!right_flanking || before_char == '(' || before_char == '[')
                    && before_char != ']'
                    && before_char != ')',
                right_flanking,
            )
        } else {
            (numdelims, left_flanking, right_flanking)
        }
    }

    /////////////
    // Utility //
    /////////////

    fn adjust_node_newlines(
        &mut self,
        node: Node,
        matchlen: usize,
        extra: usize,
        parent_line_offsets: &[usize],
    ) {
        let (newlines, since_newline) = count_newlines(
            &self.input[self.scanner.pos - matchlen - extra..self.scanner.pos - extra],
        );

        if newlines > 0 {
            self.line += newlines;
            let node_ast = &mut node.data_mut(self.arena);
            node_ast.sourcepos.end.line += newlines;
            let adjusted_line = self.line - node_ast.sourcepos.start.line;
            node_ast.sourcepos.end.column =
                parent_line_offsets[adjusted_line] + since_newline + extra;
            self.column_offset =
                -(self.scanner.pos as isize) + since_newline as isize + extra as isize;
        }
    }
}

pub struct RefMap {
    pub map: HashMap<String, ResolvedReference>,
    pub(crate) max_ref_size: usize,
    ref_size: Cell<usize>,
}

impl RefMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            max_ref_size: usize::MAX,
            ref_size: Cell::new(0),
        }
    }

    fn lookup(&self, lab: &str) -> Option<&ResolvedReference> {
        match self.map.get(lab) {
            Some(entry) => {
                let size = entry.url.len() + entry.title.len();
                let ref_size = self.ref_size.get();
                if size > self.max_ref_size - ref_size {
                    None
                } else {
                    self.ref_size.set(ref_size + size);
                    Some(entry)
                }
            }
            None => None,
        }
    }
}

pub struct FootnoteDefs {
    defs: RefCell<Vec<Node>>,
    counter: RefCell<usize>,
}

impl FootnoteDefs {
    pub fn new() -> Self {
        Self {
            defs: RefCell::new(Vec::new()),
            counter: RefCell::new(0),
        }
    }

    pub fn next_name(&self) -> String {
        let mut counter = self.counter.borrow_mut();
        *counter += 1;
        format!("__inline_{}", *counter)
    }

    pub fn add_definition(&self, def: Node) {
        self.defs.borrow_mut().push(def);
    }

    pub fn definitions(&self) -> std::cell::Ref<'_, Vec<Node>> {
        self.defs.borrow()
    }
}

pub struct Delimiter {
    inl: Node,
    position: usize,
    length: usize,
    delim_byte: u8,
    can_open: bool,
    can_close: bool,
    prev: Cell<Option<DelimiterId>>,
    next: Cell<Option<DelimiterId>>,
}

type DelimiterId = id_arena::Id<Delimiter>;
type DelimiterArena = id_arena::Arena<Delimiter>;

impl std::fmt::Debug for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[pos {}, len {}, delim_char {:?}, open? {} close? {} -- {}]",
            self.position,
            self.length,
            self.delim_byte,
            self.can_open,
            self.can_close,
            "XXX --- need arena" // self.inl
                                 //     .try_data()
                                 //     .map_or("<couldn't borrow>".to_string(), |d| format!(
                                 //         "{}",
                                 //         d.sourcepos
                                 //     ))
        )
    }
}

struct Bracket {
    inl_text: Node,
    position: usize,
    image: bool,
    bracket_after: bool,
}

#[derive(Clone)]
struct WikilinkComponents {
    url: String,
    link_label: Option<(String, usize, usize)>,
}

pub(crate) fn manual_scan_link_url(input: &str) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let len = input.len();
    let mut i = 0;

    if i < len && bytes[i] == b'<' {
        i += 1;
        while i < len {
            let b = bytes[i];
            if b == b'>' {
                i += 1;
                break;
            } else if b == b'\\' {
                i += 2;
            } else if b == b'\n' || b == b'<' {
                return None;
            } else {
                i += 1;
            }
        }
    } else {
        return manual_scan_link_url_2(input);
    }

    if i >= len {
        None
    } else {
        Some((&input[1..i - 1], i))
    }
}

pub(crate) fn manual_scan_link_url_2(input: &str) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let len = input.len();
    let mut i = 0;
    let mut nb_p = 0;

    while i < len {
        if bytes[i] == b'\\' && i + 1 < len && ispunct(bytes[i + 1]) {
            i += 2;
        } else if bytes[i] == b'(' {
            nb_p += 1;
            i += 1;
            if nb_p > 32 {
                return None;
            }
        } else if bytes[i] == b')' {
            if nb_p == 0 {
                break;
            }
            nb_p -= 1;
            i += 1;
        } else if isspace(bytes[i]) || bytes[i].is_ascii_control() {
            if i == 0 {
                return None;
            }
            break;
        } else {
            i += 1;
        }
    }

    if i >= len || nb_p != 0 {
        None
    } else {
        Some((&input[..i], i))
    }
}

pub(crate) fn make_inline(arena: &mut Arena, value: NodeValue, sourcepos: Sourcepos) -> Node {
    let ast = Ast {
        value,
        content: String::new(),
        sourcepos,
        open: false,
        last_line_blank: false,
        table_visited: false,
        line_offsets: Vec::new(),
    };
    arena.alloc(ast.into()).into()
}

pub(crate) fn count_newlines(input: &str) -> (usize, usize) {
    let mut nls = 0;
    let mut since_nl = 0;

    for &b in input.as_bytes() {
        if b == b'\n' {
            nls += 1;
            since_nl = 0;
        } else {
            since_nl += 1;
        }
    }

    (nls, since_nl)
}

#[derive(Default)]
pub struct Scanner {
    pub pos: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    fn eof(&self, input: &str) -> bool {
        self.pos >= input.len()
    }

    #[inline]
    pub fn peek_byte(&self, input: &str) -> Option<u8> {
        self.peek_byte_n(input, 0)
    }

    #[inline]
    fn peek_byte_n(&self, input: &str, n: usize) -> Option<u8> {
        if self.pos + n >= input.len() {
            None
        } else {
            let b = input.as_bytes()[self.pos + n];
            assert!(b > 0);
            Some(b)
        }
    }

    pub fn spnl(&mut self, input: &str) {
        self.skip_spaces(input);
        if self.skip_line_end(input) {
            self.skip_spaces(input);
        }
    }

    pub fn skip_spaces(&mut self, input: &str) -> bool {
        let mut skipped = false;
        while self
            .peek_byte(input)
            .map_or(false, |b| b == b' ' || b == b'\t')
        {
            self.pos += 1;
            skipped = true;
        }
        skipped
    }

    pub fn skip_line_end(&mut self, input: &str) -> bool {
        let old_pos = self.pos;
        if self.peek_byte(input) == Some(b'\r') {
            self.pos += 1;
        }
        if self.peek_byte(input) == Some(b'\n') {
            self.pos += 1;
        }
        self.pos > old_pos || self.eof(input)
    }

    fn take_while(&mut self, input: &str, b: u8) -> usize {
        let start_pos = self.pos;
        while self.peek_byte(input) == Some(b) {
            self.pos += 1;
        }
        self.pos - start_pos
    }

    fn take_while_with_limit(&mut self, input: &str, b: u8, limit: usize) -> usize {
        let start_pos = self.pos;
        let mut count = 0;
        while count < limit && self.peek_byte(input) == Some(b) {
            self.pos += 1;
            count += 1;
        }
        self.pos - start_pos
    }

    pub fn link_label<'i>(&mut self, input: &'i str) -> Option<&'i str> {
        let startpos = self.pos;

        if self.peek_byte(input) != Some(b'[') {
            return None;
        }

        self.pos += 1;

        let mut length = 0;
        while let Some(b) = self.peek_byte(input) {
            if b == b']' {
                let raw_label = strings::trim_slice(&input[startpos + 1..self.pos]);
                self.pos += 1;
                return Some(raw_label);
            }
            if b == b'[' {
                break;
            }
            if b == b'\\' {
                self.pos += 1;
                length += 1;
                if self.peek_byte(input).map_or(false, ispunct) {
                    self.pos += 1;
                    length += 1;
                }
            } else {
                self.pos += 1;
                length += 1;
            }
            if length > MAX_LINK_LABEL_LENGTH {
                self.pos = startpos;
                return None;
            }
        }

        self.pos = startpos;
        None
    }
}
