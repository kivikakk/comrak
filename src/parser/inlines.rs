use crate::arena_tree::Node;
use crate::ctype::{ispunct, isspace};
use crate::entity;
use crate::nodes::{Ast, AstNode, NodeCode, NodeLink, NodeValue, Sourcepos};
#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;
use crate::parser::{
    unwrap_into_2, unwrap_into_copy, AutolinkType, Callback, ComrakOptions, Reference,
};
use crate::scanners;
use crate::strings;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ptr;
use std::str;
use typed_arena::Arena;
use unicode_categories::UnicodeCategories;

const MAXBACKTICKS: usize = 80;
const MAX_LINK_LABEL_LENGTH: usize = 1000;

pub struct Subject<'a: 'd, 'r, 'o, 'd, 'i, 'c: 'subj, 'subj> {
    pub arena: &'a Arena<AstNode<'a>>,
    options: &'o ComrakOptions,
    pub input: &'i [u8],
    line: usize,
    pub pos: usize,
    block_offset: usize,
    column_offset: isize,
    flags: Flags,
    pub refmap: &'r mut RefMap,
    delimiter_arena: &'d Arena<Delimiter<'a, 'd>>,
    last_delimiter: Option<&'d Delimiter<'a, 'd>>,
    brackets: Vec<Bracket<'a>>,
    within_brackets: bool,
    pub backticks: [usize; MAXBACKTICKS + 1],
    pub scanned_for_backticks: bool,
    no_link_openers: bool,
    special_chars: [bool; 256],
    skip_chars: [bool; 256],
    smart_chars: [bool; 256],
    // Need to borrow the callback from the parser only for the lifetime of the Subject, 'subj, and
    // then give it back when the Subject goes out of scope. Needs to be a mutable reference so we
    // can call the FnMut and let it mutate its captured variables.
    callback: Option<&'subj mut Callback<'c>>,
}

#[derive(Default)]
struct Flags {
    skip_html_cdata: bool,
    skip_html_declaration: bool,
    skip_html_pi: bool,
    skip_html_comment: bool,
}

pub struct RefMap {
    pub map: HashMap<String, Reference>,
    pub(crate) max_ref_size: usize,
    ref_size: usize,
}

impl RefMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            max_ref_size: usize::MAX,
            ref_size: 0,
        }
    }

    fn lookup(&mut self, lab: &str) -> Option<Reference> {
        match self.map.get(lab) {
            Some(entry) => {
                let size = entry.url.len() + entry.title.len();
                if size > self.max_ref_size - self.ref_size {
                    None
                } else {
                    self.ref_size += size;
                    Some(entry.clone())
                }
            }
            None => None,
        }
    }
}

pub struct Delimiter<'a: 'd, 'd> {
    inl: &'a AstNode<'a>,
    position: usize,
    length: usize,
    delim_char: u8,
    can_open: bool,
    can_close: bool,
    prev: Cell<Option<&'d Delimiter<'a, 'd>>>,
    next: Cell<Option<&'d Delimiter<'a, 'd>>>,
}

struct Bracket<'a> {
    inl_text: &'a AstNode<'a>,
    position: usize,
    image: bool,
    bracket_after: bool,
}

impl<'a, 'r, 'o, 'd, 'i, 'c, 'subj> Subject<'a, 'r, 'o, 'd, 'i, 'c, 'subj> {
    pub fn new(
        arena: &'a Arena<AstNode<'a>>,
        options: &'o ComrakOptions,
        input: &'i [u8],
        line: usize,
        block_offset: usize,
        refmap: &'r mut RefMap,
        delimiter_arena: &'d Arena<Delimiter<'a, 'd>>,
        callback: Option<&'subj mut Callback<'c>>,
    ) -> Self {
        let mut s = Subject {
            arena,
            options,
            input,
            line,
            pos: 0,
            block_offset,
            column_offset: 0,
            flags: Flags::default(),
            refmap,
            delimiter_arena,
            last_delimiter: None,
            brackets: vec![],
            within_brackets: false,
            backticks: [0; MAXBACKTICKS + 1],
            scanned_for_backticks: false,
            no_link_openers: true,
            special_chars: [false; 256],
            skip_chars: [false; 256],
            smart_chars: [false; 256],
            callback,
        };
        for &c in &[
            b'\n', b'\r', b'_', b'*', b'"', b'`', b'\\', b'&', b'<', b'[', b']', b'!',
        ] {
            s.special_chars[c as usize] = true;
        }
        if options.extension.strikethrough {
            s.special_chars[b'~' as usize] = true;
            s.skip_chars[b'~' as usize] = true;
        }
        if options.extension.superscript {
            s.special_chars[b'^' as usize] = true;
        }
        #[cfg(feature = "shortcodes")]
        if options.extension.shortcodes {
            s.special_chars[b':' as usize] = true;
        }
        for &c in &[b'"', b'\'', b'.', b'-'] {
            s.smart_chars[c as usize] = true;
        }
        s
    }

    pub fn pop_bracket(&mut self) -> bool {
        self.brackets.pop().is_some()
    }

    pub fn parse_inline(&mut self, node: &'a AstNode<'a>) -> bool {
        let c = match self.peek_char() {
            None => return false,
            Some(ch) => *ch as char,
        };
        let new_inl: Option<&'a AstNode<'a>> = match c {
            '\0' => return false,
            '\r' | '\n' => Some(self.handle_newline()),
            '`' => Some(self.handle_backticks()),
            '\\' => Some(self.handle_backslash()),
            '&' => Some(self.handle_entity()),
            '<' => Some(self.handle_pointy_brace()),
            #[cfg(feature = "shortcodes")]
            ':' if self.options.extension.shortcodes => Some(self.handle_colons()),
            '*' | '_' | '\'' | '"' => Some(self.handle_delim(c as u8)),
            '-' => Some(self.handle_hyphen()),
            '.' => Some(self.handle_period()),
            '[' => {
                self.pos += 1;
                let inl =
                    self.make_inline(NodeValue::Text("[".to_string()), self.pos - 1, self.pos - 1);
                self.push_bracket(false, inl);
                self.within_brackets = true;
                Some(inl)
            }
            ']' => {
                self.within_brackets = false;
                self.handle_close_bracket()
            }
            '!' => {
                self.pos += 1;
                if self.peek_char() == Some(&(b'[')) && self.peek_char_n(1) != Some(&(b'^')) {
                    self.pos += 1;
                    let inl = self.make_inline(
                        NodeValue::Text("![".to_string()),
                        self.pos - 2,
                        self.pos - 1,
                    );
                    self.push_bracket(true, inl);
                    Some(inl)
                } else {
                    Some(self.make_inline(
                        NodeValue::Text("!".to_string()),
                        self.pos - 1,
                        self.pos - 1,
                    ))
                }
            }
            '~' if self.options.extension.strikethrough => Some(self.handle_delim(b'~')),
            '^' if self.options.extension.superscript && !self.within_brackets => {
                Some(self.handle_delim(b'^'))
            }
            _ => {
                let endpos = self.find_special_char();
                let mut contents = self.input[self.pos..endpos].to_vec();
                let startpos = self.pos;
                self.pos = endpos;

                if self
                    .peek_char()
                    .map_or(false, |&c| strings::is_line_end_char(c))
                {
                    strings::rtrim(&mut contents);
                }

                // if we've just produced a LineBreak, then we should consume any leading
                // space on this line
                if node.last_child().map_or(false, |n| {
                    matches!(n.data.borrow().value, NodeValue::LineBreak)
                }) {
                    strings::ltrim(&mut contents);
                }

                Some(self.make_inline(
                    NodeValue::Text(String::from_utf8(contents).unwrap()),
                    startpos,
                    endpos - 1,
                ))
            }
        };

        if let Some(inl) = new_inl {
            node.append(inl);
        }

        true
    }

    fn del_ref_eq(lhs: Option<&'d Delimiter<'a, 'd>>, rhs: Option<&'d Delimiter<'a, 'd>>) -> bool {
        match (lhs, rhs) {
            (None, None) => true,
            (Some(l), Some(r)) => ptr::eq(l, r),
            _ => false,
        }
    }

    // After parsing a block (and sometimes during), this function traverses the
    // stack of `Delimiters`, tokens ("*", "_", etc.) that may delimit regions
    // of text for special rendering: emphasis, strong, superscript,
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
        let mut openers_bottom: [usize; 11] = [stack_bottom; 11];

        // This is traversing the stack from the top to the bottom, setting `closer` to
        // the delimiter directly above `stack_bottom`. In the case where we are processing
        // emphasis on an entire block, `stack_bottom` is `None`, so `closer` references
        // the very bottom of the stack.
        let mut candidate = self.last_delimiter;
        let mut closer: Option<&Delimiter> = None;
        while candidate.map_or(false, |c| c.position >= stack_bottom) {
            closer = candidate;
            candidate = candidate.unwrap().prev.get();
        }

        while let Some(c) = closer {
            if c.can_close {
                // Each time through the outer `closer` loop we reset the opener
                // to the element below the closer, and search down the stack
                // for a matching opener.

                let mut opener = c.prev.get();
                let mut opener_found = false;
                let mut mod_three_rule_invoked = false;

                let ix = match c.delim_char {
                    b'~' => 0,
                    b'^' => 1,
                    b'"' => 2,
                    b'\'' => 3,
                    b'_' => 4,
                    b'*' => 5 + (if c.can_open { 3 } else { 0 }) + (c.length % 3),
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
                while opener.map_or(false, |o| o.position >= openers_bottom[ix]) {
                    let o = opener.unwrap();
                    if o.can_open && o.delim_char == c.delim_char {
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

                let old_c = c;

                // There's a case here for every possible delimiter. If we found
                // a matching opening delimiter for our closing delimiter, they
                // both get passed.
                if c.delim_char == b'*'
                    || c.delim_char == b'_'
                    || (self.options.extension.strikethrough && c.delim_char == b'~')
                    || (self.options.extension.superscript && c.delim_char == b'^')
                {
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
                        closer = self.insert_emph(opener.unwrap(), c);
                    } else {
                        // When no matching opener is found we move the closer
                        // up the stack, do some bookkeeping with old_closer
                        // (below), try again.
                        closer = c.next.get();
                    }
                } else if c.delim_char == b'\'' || c.delim_char == b'"' {
                    *c.inl.data.borrow_mut().value.text_mut().unwrap() =
                        if c.delim_char == b'\'' { "’" } else { "”" }.to_string();
                    closer = c.next.get();

                    if opener_found {
                        *opener
                            .unwrap()
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text_mut()
                            .unwrap() = if old_c.delim_char == b'\'' {
                            "‘"
                        } else {
                            "“"
                        }
                        .to_string();
                        self.remove_delimiter(opener.unwrap());
                        self.remove_delimiter(old_c);
                    }
                }

                // If the search for an opener was unsuccessful, then record
                // the position the search started at in the `openers_bottom`
                // so that the `opener` search can avoid looking for this
                // same opener at the bottom of the stack later.
                if !opener_found {
                    if !mod_three_rule_invoked {
                        openers_bottom[ix] = old_c.position;
                    }

                    // Now that we've failed the `opener` search starting from
                    // `old_closer`, future opener searches will be searching it
                    // for openers - if `old_closer` can't be used as an opener
                    // then we know it's just text - remove it from the
                    // delimiter stack, leaving it in the AST as text
                    if !old_c.can_open {
                        self.remove_delimiter(old_c);
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
        while self
            .last_delimiter
            .map_or(false, |d| d.position >= stack_bottom)
        {
            self.remove_delimiter(self.last_delimiter.unwrap());
        }
    }

    fn remove_delimiter(&mut self, delimiter: &'d Delimiter<'a, 'd>) {
        if delimiter.next.get().is_none() {
            assert!(ptr::eq(delimiter, self.last_delimiter.unwrap()));
            self.last_delimiter = delimiter.prev.get();
        } else {
            delimiter.next.get().unwrap().prev.set(delimiter.prev.get());
        }
        if delimiter.prev.get().is_some() {
            delimiter.prev.get().unwrap().next.set(delimiter.next.get());
        }
    }

    #[inline]
    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    #[inline]
    pub fn peek_char(&self) -> Option<&u8> {
        self.peek_char_n(0)
    }

    #[inline]
    fn peek_char_n(&self, n: usize) -> Option<&u8> {
        if self.pos + n >= self.input.len() {
            None
        } else {
            let c = &self.input[self.pos + n];
            assert!(*c > 0);
            Some(c)
        }
    }

    pub fn find_special_char(&self) -> usize {
        for n in self.pos..self.input.len() {
            if self.special_chars[self.input[n] as usize] {
                if self.input[n] == b'^' && self.within_brackets {
                    // NO OP
                } else {
                    return n;
                }
            }
            if self.options.parse.smart && self.smart_chars[self.input[n] as usize] {
                return n;
            }
        }

        self.input.len()
    }

    fn adjust_node_newlines(&mut self, node: &'a AstNode<'a>, matchlen: usize, extra: usize) {
        if !self.options.render.sourcepos {
            return;
        }

        let (newlines, since_newline) =
            count_newlines(&self.input[self.pos - matchlen - extra..self.pos - extra]);

        if newlines > 0 {
            self.line += newlines;
            let node_ast = &mut node.data.borrow_mut();
            node_ast.sourcepos.end.line += newlines;
            node_ast.sourcepos.end.column = since_newline;
            self.column_offset = -(self.pos as isize) + since_newline as isize + extra as isize;
        }
    }

    pub fn handle_newline(&mut self) -> &'a AstNode<'a> {
        let nlpos = self.pos;
        if self.input[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.input[self.pos] == b'\n' {
            self.pos += 1;
        }
        let inl = if nlpos > 1 && self.input[nlpos - 1] == b' ' && self.input[nlpos - 2] == b' ' {
            self.make_inline(NodeValue::LineBreak, nlpos, self.pos - 1)
        } else {
            self.make_inline(NodeValue::SoftBreak, nlpos, self.pos - 1)
        };
        self.line += 1;
        self.column_offset = -(self.pos as isize);
        self.skip_spaces();
        inl
    }

    pub fn take_while(&mut self, c: u8) -> usize {
        let start_pos = self.pos;
        while self.peek_char() == Some(&c) {
            self.pos += 1;
        }
        self.pos - start_pos
    }

    pub fn scan_to_closing_backtick(&mut self, openticklength: usize) -> Option<usize> {
        if openticklength > MAXBACKTICKS {
            return None;
        }

        if self.scanned_for_backticks && self.backticks[openticklength] <= self.pos {
            return None;
        }

        loop {
            while self.peek_char().map_or(false, |&c| c != b'`') {
                self.pos += 1;
            }
            if self.pos >= self.input.len() {
                self.scanned_for_backticks = true;
                return None;
            }
            let numticks = self.take_while(b'`');
            if numticks <= MAXBACKTICKS {
                self.backticks[numticks] = self.pos - numticks;
            }
            if numticks == openticklength {
                return Some(self.pos);
            }
        }
    }

    pub fn handle_backticks(&mut self) -> &'a AstNode<'a> {
        let openticks = self.take_while(b'`');
        let startpos = self.pos;
        let endpos = self.scan_to_closing_backtick(openticks);

        match endpos {
            None => {
                self.pos = startpos;
                self.make_inline(NodeValue::Text("`".repeat(openticks)), self.pos, self.pos)
            }
            Some(endpos) => {
                let buf = &self.input[startpos..endpos - openticks];
                let buf = strings::normalize_code(buf);
                let code = NodeCode {
                    num_backticks: openticks,
                    literal: String::from_utf8(buf).unwrap(),
                };
                let node =
                    self.make_inline(NodeValue::Code(code), startpos, endpos - openticks - 1);
                self.adjust_node_newlines(node, endpos - startpos, openticks);
                node
            }
        }
    }

    pub fn skip_spaces(&mut self) -> bool {
        let mut skipped = false;
        while self.peek_char().map_or(false, |&c| c == b' ' || c == b'\t') {
            self.pos += 1;
            skipped = true;
        }
        skipped
    }

    pub fn handle_delim(&mut self, c: u8) -> &'a AstNode<'a> {
        let (numdelims, can_open, can_close) = self.scan_delims(c);

        let contents = if c == b'\'' && self.options.parse.smart {
            "’".to_string()
        } else if c == b'"' && self.options.parse.smart {
            if can_close {
                "”".to_string()
            } else {
                "“".to_string()
            }
        } else {
            str::from_utf8(&self.input[self.pos - numdelims..self.pos])
                .unwrap()
                .to_string()
        };
        let inl = self.make_inline(
            NodeValue::Text(contents),
            self.pos - numdelims,
            self.pos - 1,
        );

        if (can_open || can_close) && (!(c == b'\'' || c == b'"') || self.options.parse.smart) {
            self.push_delimiter(c, can_open, can_close, inl);
        }

        inl
    }

    pub fn handle_hyphen(&mut self) -> &'a AstNode<'a> {
        let start = self.pos;
        self.pos += 1;

        if !self.options.parse.smart || self.peek_char().map_or(true, |&c| c != b'-') {
            return self.make_inline(NodeValue::Text("-".to_string()), self.pos - 1, self.pos - 1);
        }

        while self.options.parse.smart && self.peek_char().map_or(false, |&c| c == b'-') {
            self.pos += 1;
        }

        let numhyphens = (self.pos - start) as i32;

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
        self.make_inline(NodeValue::Text(buf), start, self.pos - 1)
    }

    pub fn handle_period(&mut self) -> &'a AstNode<'a> {
        self.pos += 1;
        if self.options.parse.smart && self.peek_char().map_or(false, |&c| c == b'.') {
            self.pos += 1;
            if self.peek_char().map_or(false, |&c| c == b'.') {
                self.pos += 1;
                self.make_inline(NodeValue::Text("…".to_string()), self.pos - 3, self.pos - 1)
            } else {
                self.make_inline(
                    NodeValue::Text("..".to_string()),
                    self.pos - 2,
                    self.pos - 1,
                )
            }
        } else {
            self.make_inline(NodeValue::Text(".".to_string()), self.pos - 1, self.pos - 1)
        }
    }

    pub fn scan_delims(&mut self, c: u8) -> (usize, bool, bool) {
        let before_char = if self.pos == 0 {
            '\n'
        } else {
            let mut before_char_pos = self.pos - 1;
            while before_char_pos > 0
                && (self.input[before_char_pos] >> 6 == 2
                    || self.skip_chars[self.input[before_char_pos] as usize])
            {
                before_char_pos -= 1;
            }
            match unsafe { str::from_utf8_unchecked(&self.input[before_char_pos..self.pos]) }
                .chars()
                .next()
            {
                Some(x) => {
                    if (x as usize) < 256 && self.skip_chars[x as usize] {
                        '\n'
                    } else {
                        x
                    }
                }
                None => '\n',
            }
        };

        let mut numdelims = 0;
        if c == b'\'' || c == b'"' {
            numdelims += 1;
            self.pos += 1;
        } else {
            while self.peek_char() == Some(&c) {
                numdelims += 1;
                self.pos += 1;
            }
        }

        let after_char = if self.eof() {
            '\n'
        } else {
            let mut after_char_pos = self.pos;
            while after_char_pos < self.input.len() - 1
                && self.skip_chars[self.input[after_char_pos] as usize]
            {
                after_char_pos += 1;
            }
            match unsafe { str::from_utf8_unchecked(&self.input[after_char_pos..]) }
                .chars()
                .next()
            {
                Some(x) => {
                    if (x as usize) < 256 && self.skip_chars[x as usize] {
                        '\n'
                    } else {
                        x
                    }
                }
                None => '\n',
            }
        };

        let left_flanking = numdelims > 0
            && !after_char.is_whitespace()
            && !(after_char.is_punctuation()
                && !before_char.is_whitespace()
                && !before_char.is_punctuation());
        let right_flanking = numdelims > 0
            && !before_char.is_whitespace()
            && !(before_char.is_punctuation()
                && !after_char.is_whitespace()
                && !after_char.is_punctuation());

        if c == b'_' {
            (
                numdelims,
                left_flanking && (!right_flanking || before_char.is_punctuation()),
                right_flanking && (!left_flanking || after_char.is_punctuation()),
            )
        } else if c == b'\'' || c == b'"' {
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

    pub fn push_delimiter(&mut self, c: u8, can_open: bool, can_close: bool, inl: &'a AstNode<'a>) {
        let d = self.delimiter_arena.alloc(Delimiter {
            prev: Cell::new(self.last_delimiter),
            next: Cell::new(None),
            inl,
            position: self.pos,
            length: inl.data.borrow().value.text().unwrap().len(),
            delim_char: c,
            can_open,
            can_close,
        });
        if d.prev.get().is_some() {
            d.prev.get().unwrap().next.set(Some(d));
        }
        self.last_delimiter = Some(d);
    }

    // Create a new emphasis node, move all the nodes between `opener`
    // and `closer` into it, and insert it into the AST.
    //
    // As a side-effect, handle long "***" and "___" nodes by truncating them in
    // place to be re-matched by `process_emphasis`.
    pub fn insert_emph(
        &mut self,
        opener: &'d Delimiter<'a, 'd>,
        closer: &'d Delimiter<'a, 'd>,
    ) -> Option<&'d Delimiter<'a, 'd>> {
        let opener_char = opener.inl.data.borrow().value.text().unwrap().as_bytes()[0];
        let mut opener_num_chars = opener.inl.data.borrow().value.text().unwrap().len();
        let mut closer_num_chars = closer.inl.data.borrow().value.text().unwrap().len();
        let use_delims = if closer_num_chars >= 2 && opener_num_chars >= 2 {
            2
        } else {
            1
        };

        opener_num_chars -= use_delims;
        closer_num_chars -= use_delims;

        if self.options.extension.strikethrough
            && opener_char == b'~'
            && (opener_num_chars != closer_num_chars || opener_num_chars > 0)
        {
            return None;
        }

        opener
            .inl
            .data
            .borrow_mut()
            .value
            .text_mut()
            .unwrap()
            .truncate(opener_num_chars);
        closer
            .inl
            .data
            .borrow_mut()
            .value
            .text_mut()
            .unwrap()
            .truncate(closer_num_chars);

        // Remove all the candidate delimiters from between the opener and the
        // closer. None of them are matched pairs. They've been scanned already.
        let mut delim = closer.prev.get();
        while delim.is_some() && !Self::del_ref_eq(delim, Some(opener)) {
            self.remove_delimiter(delim.unwrap());
            delim = delim.unwrap().prev.get();
        }

        let emph = self.make_inline(
            if self.options.extension.strikethrough && opener_char == b'~' {
                NodeValue::Strikethrough
            } else if self.options.extension.superscript && opener_char == b'^' {
                NodeValue::Superscript
            } else if use_delims == 1 {
                NodeValue::Emph
            } else {
                NodeValue::Strong
            },
            // These are overriden immediately below.
            self.pos,
            self.pos,
        );
        {
            emph.data.borrow_mut().sourcepos = (
                opener.inl.data.borrow().sourcepos.start.line,
                opener.inl.data.borrow().sourcepos.start.column,
                closer.inl.data.borrow().sourcepos.end.line,
                closer.inl.data.borrow().sourcepos.end.column,
            )
                .into();
        }

        // Drop all the interior AST nodes into the emphasis node
        // and then insert the emphasis node
        let mut tmp = opener.inl.next_sibling().unwrap();
        while !tmp.same_node(closer.inl) {
            let next = tmp.next_sibling();
            emph.append(tmp);
            if let Some(n) = next {
                tmp = n;
            } else {
                break;
            }
        }
        opener.inl.insert_after(emph);

        // Drop the delimiters and return the next closer to process

        if opener_num_chars == 0 {
            opener.inl.detach();
            self.remove_delimiter(opener);
        }

        if closer_num_chars == 0 {
            closer.inl.detach();
            self.remove_delimiter(closer);
            closer.next.get()
        } else {
            Some(closer)
        }
    }

    pub fn handle_backslash(&mut self) -> &'a AstNode<'a> {
        let startpos = self.pos;
        self.pos += 1;
        if self.peek_char().map_or(false, |&c| ispunct(c)) {
            self.pos += 1;
            self.make_inline(
                NodeValue::Text(String::from_utf8(vec![self.input[self.pos - 1]]).unwrap()),
                self.pos - 2,
                self.pos - 1,
            )
        } else if !self.eof() && self.skip_line_end() {
            self.make_inline(NodeValue::LineBreak, startpos, self.pos - 1)
        } else {
            self.make_inline(
                NodeValue::Text("\\".to_string()),
                self.pos - 1,
                self.pos - 1,
            )
        }
    }

    pub fn skip_line_end(&mut self) -> bool {
        let old_pos = self.pos;
        if self.peek_char() == Some(&(b'\r')) {
            self.pos += 1;
        }
        if self.peek_char() == Some(&(b'\n')) {
            self.pos += 1;
        }
        self.pos > old_pos || self.eof()
    }

    pub fn handle_entity(&mut self) -> &'a AstNode<'a> {
        self.pos += 1;

        match entity::unescape(&self.input[self.pos..]) {
            None => self.make_inline(NodeValue::Text("&".to_string()), self.pos - 1, self.pos - 1),
            Some((entity, len)) => {
                self.pos += len;
                self.make_inline(
                    NodeValue::Text(String::from_utf8(entity).unwrap()),
                    self.pos - 1 - len,
                    self.pos - 1,
                )
            }
        }
    }

    #[cfg(feature = "shortcodes")]
    pub fn handle_colons(&mut self) -> &'a AstNode<'a> {
        self.pos += 1;

        if let Some(matchlen) = scanners::shortcode(&self.input[self.pos..]) {
            let shortcode =
                unsafe { str::from_utf8_unchecked(&self.input[self.pos..self.pos + matchlen - 1]) };

            if let Ok(nsc) = NodeShortCode::try_from(shortcode) {
                self.pos += matchlen;
                let inl = self.make_inline(
                    NodeValue::ShortCode(nsc),
                    self.pos - 1 - matchlen,
                    self.pos - 1,
                );
                return inl;
            }
        }

        self.make_inline(NodeValue::Text(":".to_string()), self.pos - 1, self.pos - 1)
    }

    pub fn handle_pointy_brace(&mut self) -> &'a AstNode<'a> {
        self.pos += 1;

        if let Some(matchlen) = scanners::autolink_uri(&self.input[self.pos..]) {
            self.pos += matchlen;
            let inl = self.make_autolink(
                &self.input[self.pos - matchlen..self.pos - 1],
                AutolinkType::Uri,
                self.pos - 1 - matchlen,
                self.pos - 1,
            );
            return inl;
        }

        if let Some(matchlen) = scanners::autolink_email(&self.input[self.pos..]) {
            self.pos += matchlen;
            let inl = self.make_autolink(
                &self.input[self.pos - matchlen..self.pos - 1],
                AutolinkType::Email,
                self.pos - 1 - matchlen,
                self.pos - 1,
            );
            return inl;
        }

        // Most comments below are verbatim from cmark upstream.
        let mut matchlen: Option<usize> = None;

        if self.pos + 2 <= self.input.len() {
            let c = self.input[self.pos];
            if c == b'!' && !self.flags.skip_html_comment {
                let c = self.input[self.pos + 1];
                if c == b'-' && self.peek_char_n(2) == Some(&b'-') {
                    if self.peek_char_n(3) == Some(&b'>') {
                        matchlen = Some(4);
                    } else if self.peek_char_n(3) == Some(&b'-')
                        && self.peek_char_n(4) == Some(&b'>')
                    {
                        matchlen = Some(5);
                    } else if let Some(m) = scanners::html_comment(&self.input[self.pos + 1..]) {
                        matchlen = Some(m + 1);
                    } else {
                        self.flags.skip_html_comment = true;
                    }
                } else if c == b'[' {
                    if !self.flags.skip_html_cdata && self.pos + 3 <= self.input.len() {
                        if let Some(m) = scanners::html_cdata(&self.input[self.pos + 2..]) {
                            // The regex doesn't require the final "]]>". But if we're not at
                            // the end of input, it must come after the match. Otherwise,
                            // disable subsequent scans to avoid quadratic behavior.

                            // Adding 5 to matchlen for prefix "![", suffix "]]>"
                            if self.pos + m + 5 > self.input.len() {
                                self.flags.skip_html_cdata = true;
                            } else {
                                matchlen = Some(m + 5);
                            }
                        }
                    }
                } else if !self.flags.skip_html_declaration {
                    if let Some(m) = scanners::html_declaration(&self.input[self.pos + 1..]) {
                        // Adding 2 to matchlen for prefix "!", suffix ">"
                        if self.pos + m + 2 > self.input.len() {
                            self.flags.skip_html_declaration = true;
                        } else {
                            matchlen = Some(m + 2);
                        }
                    }
                }
            } else if c == b'?' {
                if !self.flags.skip_html_pi {
                    // Note that we allow an empty match.
                    let m = scanners::html_processing_instruction(&self.input[self.pos + 1..])
                        .unwrap_or(0);
                    // Adding 3 to matchlen fro prefix "?", suffix "?>"
                    if self.pos + m + 3 > self.input.len() {
                        self.flags.skip_html_pi = true;
                    } else {
                        matchlen = Some(m + 3);
                    }
                }
            } else {
                matchlen = scanners::html_tag(&self.input[self.pos..]);
            }
        }

        if let Some(matchlen) = matchlen {
            let contents = &self.input[self.pos - 1..self.pos + matchlen];
            self.pos += matchlen;
            let inl = self.make_inline(
                NodeValue::HtmlInline(str::from_utf8(contents).unwrap().to_string()),
                self.pos - matchlen - 1,
                self.pos - 1,
            );
            self.adjust_node_newlines(inl, matchlen, 1);
            return inl;
        }

        self.make_inline(NodeValue::Text("<".to_string()), self.pos - 1, self.pos - 1)
    }

    pub fn push_bracket(&mut self, image: bool, inl_text: &'a AstNode<'a>) {
        let len = self.brackets.len();
        if len > 0 {
            self.brackets[len - 1].bracket_after = true;
        }
        self.brackets.push(Bracket {
            inl_text,
            position: self.pos,
            image,
            bracket_after: false,
        });
        if !image {
            self.no_link_openers = false;
        }
    }

    pub fn handle_close_bracket(&mut self) -> Option<&'a AstNode<'a>> {
        self.pos += 1;
        let initial_pos = self.pos;

        let brackets_len = self.brackets.len();
        if brackets_len == 0 {
            return Some(self.make_inline(
                NodeValue::Text("]".to_string()),
                self.pos - 1,
                self.pos - 1,
            ));
        }

        let is_image = self.brackets[brackets_len - 1].image;

        if !is_image && self.no_link_openers {
            self.brackets.pop();
            return Some(self.make_inline(
                NodeValue::Text("]".to_string()),
                self.pos - 1,
                self.pos - 1,
            ));
        }

        let after_link_text_pos = self.pos;

        // Try to find a link destination within parenthesis

        let mut sps = 0;
        let mut url: &[u8] = &[];
        let mut n: usize = 0;
        if self.peek_char() == Some(&(b'(')) && {
            sps = scanners::spacechars(&self.input[self.pos + 1..]).unwrap_or(0);
            let offset = self.pos + 1 + sps;
            offset < self.input.len()
                && unwrap_into_2(
                    manual_scan_link_url(&self.input[offset..]),
                    &mut url,
                    &mut n,
                )
        } {
            let starturl = self.pos + 1 + sps;
            let endurl = starturl + n;
            let starttitle = endurl + scanners::spacechars(&self.input[endurl..]).unwrap_or(0);
            let endtitle = if starttitle == endurl {
                starttitle
            } else {
                starttitle + scanners::link_title(&self.input[starttitle..]).unwrap_or(0)
            };
            let endall = endtitle + scanners::spacechars(&self.input[endtitle..]).unwrap_or(0);

            if endall < self.input.len() && self.input[endall] == b')' {
                self.pos = endall + 1;
                let url = strings::clean_url(url);
                let title = strings::clean_title(&self.input[starttitle..endtitle]);
                self.close_bracket_match(
                    is_image,
                    String::from_utf8(url).unwrap(),
                    String::from_utf8(title).unwrap(),
                );
                return None;
            } else {
                self.pos = after_link_text_pos;
            }
        }

        // Try to see if this is a reference link

        let (mut lab, mut found_label) = match self.link_label() {
            Some(lab) => (lab.to_string(), true),
            None => ("".to_string(), false),
        };

        if !found_label {
            self.pos = initial_pos;
        }

        if (!found_label || lab.is_empty()) && !self.brackets[brackets_len - 1].bracket_after {
            lab = str::from_utf8(
                &self.input[self.brackets[brackets_len - 1].position..initial_pos - 1],
            )
            .unwrap()
            .to_string();
            found_label = true;
        }

        // Need to normalize both to lookup in refmap and to call callback
        let lab = strings::normalize_label(&lab);
        let mut reff = if found_label {
            self.refmap.lookup(&lab)
        } else {
            None
        };

        // Attempt to use the provided broken link callback if a reference cannot be resolved
        if reff.is_none() {
            if let Some(ref mut callback) = self.callback {
                reff = callback(&lab).map(|(url, title)| Reference { url, title });
            }
        }

        if let Some(reff) = reff {
            self.close_bracket_match(is_image, reff.url.clone(), reff.title);
            return None;
        }

        let mut text: Option<String> = None;
        if self.options.extension.footnotes
            && match self.brackets[brackets_len - 1].inl_text.next_sibling() {
                Some(n) => {
                    text = n.data.borrow().value.text().cloned();
                    text.is_some() && n.next_sibling().is_none()
                }
                _ => false,
            }
        {
            let text = text.unwrap();
            if text.len() > 1 && text.as_bytes()[0] == b'^' {
                let inl = self.make_inline(
                    NodeValue::FootnoteReference(text[1..].to_string()),
                    // Overridden immediately below.
                    self.pos,
                    self.pos,
                );
                inl.data.borrow_mut().sourcepos.start.column = self.brackets[brackets_len - 1]
                    .inl_text
                    .data
                    .borrow()
                    .sourcepos
                    .start
                    .column;
                inl.data.borrow_mut().sourcepos.end.column = usize::try_from(
                    self.pos as isize + self.column_offset + self.block_offset as isize,
                )
                .unwrap();
                self.brackets[brackets_len - 1].inl_text.insert_before(inl);
                self.brackets[brackets_len - 1]
                    .inl_text
                    .next_sibling()
                    .unwrap()
                    .detach();
                self.brackets[brackets_len - 1].inl_text.detach();
                self.process_emphasis(self.brackets[brackets_len - 1].position);
                self.brackets.pop();
                return None;
            }
        }

        self.brackets.pop();
        self.pos = initial_pos;
        Some(self.make_inline(NodeValue::Text("]".to_string()), self.pos - 1, self.pos - 1))
    }

    pub fn close_bracket_match(&mut self, is_image: bool, url: String, title: String) {
        let brackets_len = self.brackets.len();

        let nl = NodeLink { url, title };
        let inl = self.make_inline(
            if is_image {
                NodeValue::Image(nl)
            } else {
                NodeValue::Link(nl)
            },
            // Manually set below.
            self.pos,
            self.pos,
        );
        inl.data.borrow_mut().sourcepos.start.column = self.brackets[brackets_len - 1]
            .inl_text
            .data
            .borrow()
            .sourcepos
            .start
            .column;
        inl.data.borrow_mut().sourcepos.end.column =
            usize::try_from(self.pos as isize + self.column_offset + self.block_offset as isize)
                .unwrap();

        self.brackets[brackets_len - 1].inl_text.insert_before(inl);
        let mut tmpch = self.brackets[brackets_len - 1].inl_text.next_sibling();
        while let Some(tmp) = tmpch {
            tmpch = tmp.next_sibling();
            inl.append(tmp);
        }
        self.brackets[brackets_len - 1].inl_text.detach();
        self.process_emphasis(self.brackets[brackets_len - 1].position);
        self.brackets.pop();

        if !is_image {
            self.no_link_openers = true;
        }
    }

    pub fn link_label(&mut self) -> Option<&str> {
        let startpos = self.pos;

        if self.peek_char() != Some(&(b'[')) {
            return None;
        }

        self.pos += 1;

        let mut length = 0;
        let mut c = 0;
        while unwrap_into_copy(self.peek_char(), &mut c) && c != b'[' && c != b']' {
            if c == b'\\' {
                self.pos += 1;
                length += 1;
                if self.peek_char().map_or(false, |&c| ispunct(c)) {
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

        if c == b']' {
            let raw_label = strings::trim_slice(&self.input[startpos + 1..self.pos]);
            self.pos += 1;
            Some(str::from_utf8(raw_label).unwrap())
        } else {
            self.pos = startpos;
            None
        }
    }

    pub fn spnl(&mut self) {
        self.skip_spaces();
        if self.skip_line_end() {
            self.skip_spaces();
        }
    }

    fn make_inline(
        &self,
        value: NodeValue,
        start_column: usize,
        end_column: usize,
    ) -> &'a AstNode<'a> {
        let start_column =
            start_column as isize + 1 + self.column_offset + self.block_offset as isize;
        let end_column = end_column as isize + 1 + self.column_offset + self.block_offset as isize;

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
            internal_offset: 0,
            open: false,
            last_line_blank: false,
            table_visited: false,
        };
        self.arena.alloc(Node::new(RefCell::new(ast)))
    }

    fn make_autolink(
        &self,
        url: &[u8],
        kind: AutolinkType,
        start_column: usize,
        end_column: usize,
    ) -> &'a AstNode<'a> {
        let inl = self.make_inline(
            NodeValue::Link(NodeLink {
                url: String::from_utf8(strings::clean_autolink(url, kind)).unwrap(),
                title: String::new(),
            }),
            start_column + 1,
            end_column + 1,
        );
        inl.append(self.make_inline(
            NodeValue::Text(String::from_utf8(entity::unescape_html(url)).unwrap()),
            start_column + 1,
            end_column - 1,
        ));
        inl
    }
}

pub fn manual_scan_link_url(input: &[u8]) -> Option<(&[u8], usize)> {
    let len = input.len();
    let mut i = 0;

    if i < len && input[i] == b'<' {
        i += 1;
        while i < len {
            let b = input[i];
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

pub fn manual_scan_link_url_2(input: &[u8]) -> Option<(&[u8], usize)> {
    let len = input.len();
    let mut i = 0;
    let mut nb_p = 0;

    while i < len {
        if input[i] == b'\\' && i + 1 < len && ispunct(input[i + 1]) {
            i += 2;
        } else if input[i] == b'(' {
            nb_p += 1;
            i += 1;
            if nb_p > 32 {
                return None;
            }
        } else if input[i] == b')' {
            if nb_p == 0 {
                break;
            }
            nb_p -= 1;
            i += 1;
        } else if isspace(input[i]) || input[i].is_ascii_control() {
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

pub fn make_inline<'a>(
    arena: &'a Arena<AstNode<'a>>,
    value: NodeValue,
    sourcepos: Sourcepos,
) -> &'a AstNode<'a> {
    let ast = Ast {
        value,
        content: String::new(),
        sourcepos,
        internal_offset: 0,
        open: false,
        last_line_blank: false,
        table_visited: false,
    };
    arena.alloc(Node::new(RefCell::new(ast)))
}

pub fn count_newlines(input: &[u8]) -> (usize, usize) {
    let mut nls = 0;
    let mut since_nl = 0;

    for &c in input {
        if c == b'\n' {
            nls += 1;
            since_nl = 0;
        } else {
            since_nl += 1;
        }
    }

    (nls, since_nl)
}
