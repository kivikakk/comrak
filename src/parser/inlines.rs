use arena_tree::Node;
use ctype::{isspace, ispunct};
use entity;
use nodes::{NodeValue, Ast, NodeLink, AstNode};
use parser::{unwrap_into, unwrap_into_copy, ComrakOptions, Reference, AutolinkType};
use scanners;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr;
use strings;
use typed_arena::Arena;
use unicode_categories::UnicodeCategories;

const MAXBACKTICKS: usize = 80;
const MAX_LINK_LABEL_LENGTH: usize = 1000;

pub struct Subject<'a: 'd, 'r, 'o, 'd, 'i> {
    pub arena: &'a Arena<AstNode<'a>>,
    options: &'o ComrakOptions,
    pub input: &'i str,
    pub pos: usize,
    pub refmap: &'r mut HashMap<String, Reference>,
    delimiter_arena: &'d Arena<Delimiter<'a, 'd>>,
    last_delimiter: Option<&'d Delimiter<'a, 'd>>,
    brackets: Vec<Bracket<'a, 'd>>,
    pub backticks: [usize; MAXBACKTICKS + 1],
    pub scanned_for_backticks: bool,
    special_chars: Vec<bool>,
}

pub struct Delimiter<'a: 'd, 'd> {
    inl: &'a AstNode<'a>,
    delim_char: u8,
    can_open: bool,
    can_close: bool,
    prev: Cell<Option<&'d Delimiter<'a, 'd>>>,
    next: Cell<Option<&'d Delimiter<'a, 'd>>>,
}

struct Bracket<'a: 'd, 'd> {
    previous_delimiter: Option<&'d Delimiter<'a, 'd>>,
    inl_text: &'a AstNode<'a>,
    position: usize,
    image: bool,
    active: bool,
    bracket_after: bool,
}

impl<'a, 'r, 'o, 'd, 'i> Subject<'a, 'r, 'o, 'd, 'i> {
    pub fn new(
        arena: &'a Arena<AstNode<'a>>,
        options: &'o ComrakOptions,
        input: &'i str,
        refmap: &'r mut HashMap<String, Reference>,
        delimiter_arena: &'d Arena<Delimiter<'a, 'd>>,
    ) -> Self {
        let mut s = Subject {
            arena: arena,
            options: options,
            input: input,
            pos: 0,
            refmap: refmap,
            delimiter_arena: delimiter_arena,
            last_delimiter: None,
            brackets: vec![],
            backticks: [0; MAXBACKTICKS + 1],
            scanned_for_backticks: false,
            special_chars: vec![],
        };
        s.special_chars.extend_from_slice(&[false; 256]);
        for &c in &[
            b'\n',
            b'\r',
            b'_',
            b'*',
            b'"',
            b'`',
            b'\\',
            b'&',
            b'<',
            b'[',
            b']',
            b'!',
        ]
        {
            s.special_chars[c as usize] = true;
        }
        if options.ext_strikethrough {
            s.special_chars[b'~' as usize] = true;
        }
        if options.ext_superscript {
            s.special_chars[b'^' as usize] = true;
        }
        s
    }

    pub fn pop_bracket(&mut self) -> bool {
        self.brackets.pop().is_some()
    }

    pub fn parse_inline(&mut self, node: &'a AstNode<'a>) -> bool {
        let new_inl: Option<&'a AstNode<'a>>;
        let c = match self.peek_char() {
            None => return false,
            Some(ch) => *ch as char,
        };

        match c {
            '\0' => return false,
            '\r' | '\n' => new_inl = Some(self.handle_newline()),
            '`' => new_inl = Some(self.handle_backticks()),
            '\\' => new_inl = Some(self.handle_backslash()),
            '&' => new_inl = Some(self.handle_entity()),
            '<' => new_inl = Some(self.handle_pointy_brace()),
            '*' | '_' | '\'' | '"' => new_inl = Some(self.handle_delim(c as u8)),
            // TODO: smart characters. Eh.
            //'-' => new_inl => Some(self.handle_hyphen()),
            //'.' => new_inl => Some(self.handle_period()),
            '[' => {
                self.pos += 1;
                let inl = make_inline(self.arena, NodeValue::Text("[".to_string()));
                new_inl = Some(inl);
                self.push_bracket(false, inl);
            }
            ']' => new_inl = self.handle_close_bracket(),
            '!' => {
                self.pos += 1;
                if self.peek_char() == Some(&(b'[')) {
                    self.pos += 1;
                    let inl = make_inline(self.arena, NodeValue::Text("![".to_string()));
                    new_inl = Some(inl);
                    self.push_bracket(true, inl);
                } else {
                    new_inl = Some(make_inline(self.arena, NodeValue::Text("!".to_string())));
                }
            }
            _ => {
                if self.options.ext_strikethrough && c == '~' {
                    new_inl = Some(self.handle_delim(b'~'));
                } else if self.options.ext_superscript && c == '^' {
                    new_inl = Some(self.handle_delim(b'^'));
                } else {
                    let endpos = self.find_special_char();
                    let mut contents = self.input[self.pos..endpos].to_string();
                    self.pos = endpos;

                    if self.peek_char().map_or(
                        false,
                        |&c| strings::is_line_end_char(c),
                    )
                    {
                        strings::rtrim(&mut contents);
                    }

                    new_inl = Some(make_inline(self.arena, NodeValue::Text(contents)));
                }
            }
        }

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

    pub fn process_emphasis(&mut self, stack_bottom: Option<&'d Delimiter<'a, 'd>>) {
        let mut closer = self.last_delimiter;
        let mut openers_bottom: [[Option<&'d Delimiter<'a, 'd>>; 128]; 3] = [[None; 128]; 3];
        for i in &mut openers_bottom {
            i['*' as usize] = stack_bottom;
            i['_' as usize] = stack_bottom;
            i['\'' as usize] = stack_bottom;
            i['"' as usize] = stack_bottom;
        }

        while closer.is_some() && !Self::del_ref_eq(closer.unwrap().prev.get(), stack_bottom) {
            closer = closer.unwrap().prev.get();
        }

        while closer.is_some() {
            if closer.unwrap().can_close {
                let mut opener = closer.unwrap().prev.get();
                let mut opener_found = false;

                while opener.is_some() && !Self::del_ref_eq(opener, stack_bottom) &&
                    !Self::del_ref_eq(
                        opener,
                        openers_bottom[closer
                                           .unwrap()
                                           .inl
                                           .data
                                           .borrow()
                                           .value
                                           .text()
                                           .unwrap()
                                           .len() % 3]
                            [closer.unwrap().delim_char as usize],
                    )
                {
                    if opener.unwrap().can_open &&
                        opener.unwrap().delim_char == closer.unwrap().delim_char
                    {
                        let odd_match = (closer.unwrap().can_open || opener.unwrap().can_close) &&
                            ((opener
                                  .unwrap()
                                  .inl
                                  .data
                                  .borrow()
                                  .value
                                  .text()
                                  .unwrap()
                                  .len() +
                                  closer
                                      .unwrap()
                                      .inl
                                      .data
                                      .borrow()
                                      .value
                                      .text()
                                      .unwrap()
                                      .len()) % 3 == 0);
                        if !odd_match {
                            opener_found = true;
                            break;
                        }
                    }
                    opener = opener.unwrap().prev.get();
                }

                let old_closer = closer;

                if closer.unwrap().delim_char == b'*' || closer.unwrap().delim_char == b'_' ||
                    (self.options.ext_strikethrough && closer.unwrap().delim_char == b'~') ||
                    (self.options.ext_superscript && closer.unwrap().delim_char == b'^')
                {
                    if opener_found {
                        closer = self.insert_emph(opener.unwrap(), closer.unwrap());
                    } else {
                        closer = closer.unwrap().next.get();
                    }
                } else if closer.unwrap().delim_char == b'\'' {
                    *closer
                        .unwrap()
                        .inl
                        .data
                        .borrow_mut()
                        .value
                        .text_mut()
                        .unwrap() = "’".to_string();
                    if opener_found {
                        *opener
                            .unwrap()
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text_mut()
                            .unwrap() = "‘".to_string();
                    }
                    closer = closer.unwrap().next.get();
                } else if closer.unwrap().delim_char == b'"' {
                    *closer
                        .unwrap()
                        .inl
                        .data
                        .borrow_mut()
                        .value
                        .text_mut()
                        .unwrap() = "”".to_string();
                    if opener_found {
                        *opener
                            .unwrap()
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text_mut()
                            .unwrap() = "“".to_string();
                    }
                    closer = closer.unwrap().next.get();
                }
                if !opener_found {
                    let ix = old_closer
                        .unwrap()
                        .inl
                        .data
                        .borrow()
                        .value
                        .text()
                        .unwrap()
                        .len() % 3;
                    openers_bottom[ix][old_closer.unwrap().delim_char as usize] =
                        old_closer.unwrap().prev.get();
                    if !old_closer.unwrap().can_open {
                        self.remove_delimiter(old_closer.unwrap());
                    }
                }
            } else {
                closer = closer.unwrap().next.get();
            }
        }

        while self.last_delimiter.is_some() &&
            !Self::del_ref_eq(self.last_delimiter, stack_bottom)
        {
            let last_del = self.last_delimiter.unwrap();
            self.remove_delimiter(last_del);
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

    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn peek_char(&self) -> Option<&u8> {
        if self.eof() {
            None
        } else {
            let c = &self.input.as_bytes()[self.pos];
            assert!(*c > 0);
            Some(c)
        }
    }

    pub fn find_special_char(&self) -> usize {
        for n in self.pos..self.input.len() {
            if self.special_chars[self.input.as_bytes()[n] as usize] {
                return n;
            }
        }

        self.input.len()
    }

    pub fn handle_newline(&mut self) -> &'a AstNode<'a> {
        let nlpos = self.pos;
        if self.input.as_bytes()[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.input.as_bytes()[self.pos] == b'\n' {
            self.pos += 1;
        }
        self.skip_spaces();
        if nlpos > 1 && self.input.as_bytes()[nlpos - 1] == b' ' &&
            self.input.as_bytes()[nlpos - 2] == b' '
        {
            make_inline(self.arena, NodeValue::LineBreak)
        } else {
            make_inline(self.arena, NodeValue::SoftBreak)
        }
    }

    pub fn take_while(&mut self, c: u8) -> String {
        let mut v = String::with_capacity(10);
        while self.peek_char() == Some(&c) {
            v.push(self.input.as_bytes()[self.pos] as char);
            self.pos += 1;
        }
        v
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
            let numticks = self.take_while(b'`').len();
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
        let endpos = self.scan_to_closing_backtick(openticks.len());

        match endpos {
            None => {
                self.pos = startpos;
                make_inline(self.arena, NodeValue::Text(openticks))
            }
            Some(endpos) => {
                let mut buf: &str = &self.input[startpos..endpos - openticks.len()];
                buf = strings::trim_slice(buf);
                let buf = strings::normalize_whitespace(buf);
                make_inline(self.arena, NodeValue::Code(buf))
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

        let contents = self.input[self.pos - numdelims..self.pos].to_string();
        let inl = make_inline(self.arena, NodeValue::Text(contents));

        if (can_open || can_close) && c != b'\'' && c != b'"' {
            self.push_delimiter(c, can_open, can_close, inl);
        }

        inl
    }

    pub fn scan_delims(&mut self, c: u8) -> (usize, bool, bool) {
        let before_char = if self.pos == 0 {
            '\n'
        } else {
            let mut before_char_pos = self.pos - 1;
            while before_char_pos > 0 && self.input.as_bytes()[before_char_pos] >> 6 == 2 {
                before_char_pos -= 1;
            }
            self.input[before_char_pos..].chars().next().unwrap()
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
            self.input[self.pos..].chars().next().unwrap()
        };

        let left_flanking = numdelims > 0 && !after_char.is_whitespace() &&
            !(after_char.is_punctuation() && !before_char.is_whitespace() &&
                  !before_char.is_punctuation());
        let right_flanking = numdelims > 0 && !before_char.is_whitespace() &&
            !(before_char.is_punctuation() && !after_char.is_whitespace() &&
                  !after_char.is_punctuation());

        if c == b'_' {
            (
                numdelims,
                left_flanking && (!right_flanking || before_char.is_punctuation()),
                right_flanking && (!left_flanking || after_char.is_punctuation()),
            )
        } else if c == b'\'' || c == b'"' {
            (numdelims, left_flanking && !right_flanking, right_flanking)
        } else {
            (numdelims, left_flanking, right_flanking)
        }
    }

    pub fn push_delimiter(&mut self, c: u8, can_open: bool, can_close: bool, inl: &'a AstNode<'a>) {
        let d = self.delimiter_arena.alloc(Delimiter {
            prev: Cell::new(self.last_delimiter),
            next: Cell::new(None),
            inl: inl,
            delim_char: c,
            can_open: can_open,
            can_close: can_close,
        });
        if d.prev.get().is_some() {
            d.prev.get().unwrap().next.set(Some(d));
        }
        self.last_delimiter = Some(d);
    }

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

        if self.options.ext_strikethrough && opener_char == b'~' {
            opener_num_chars = 0;
            closer_num_chars = 0;
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

        let mut delim = closer.prev.get();
        while delim.is_some() && !Self::del_ref_eq(delim, Some(opener)) {
            self.remove_delimiter(delim.unwrap());
            delim = delim.unwrap().prev.get();
        }

        let emph = make_inline(
            self.arena,
            if self.options.ext_strikethrough && opener_char == b'~' {
                NodeValue::Strikethrough
            } else if self.options.ext_superscript && opener_char == b'^' {
                NodeValue::Superscript
            } else if use_delims == 1 {
                NodeValue::Emph
            } else {
                NodeValue::Strong
            },
        );

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
        self.pos += 1;
        if self.peek_char().map_or(false, |&c| ispunct(c)) {
            self.pos += 1;
            make_inline(
                self.arena,
                NodeValue::Text((self.input.as_bytes()[self.pos - 1] as char).to_string()),
            )
        } else if !self.eof() && self.skip_line_end() {
            make_inline(self.arena, NodeValue::LineBreak)
        } else {
            make_inline(self.arena, NodeValue::Text("\\".to_string()))
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
            None => make_inline(self.arena, NodeValue::Text("&".to_string())),
            Some((entity, len)) => {
                self.pos += len;
                make_inline(self.arena, NodeValue::Text(entity))
            }
        }
    }

    pub fn handle_pointy_brace(&mut self) -> &'a AstNode<'a> {
        self.pos += 1;

        if let Some(matchlen) = scanners::autolink_uri(&self.input[self.pos..]) {
            let inl = make_autolink(
                self.arena,
                &self.input[self.pos..self.pos + matchlen - 1],
                AutolinkType::URI,
            );
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::autolink_email(&self.input[self.pos..]) {
            let inl = make_autolink(
                self.arena,
                &self.input[self.pos..self.pos + matchlen - 1],
                AutolinkType::Email,
            );
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::html_tag(&self.input[self.pos..]) {
            let contents = &self.input[self.pos - 1..self.pos + matchlen];
            let inl = make_inline(self.arena, NodeValue::HtmlInline(contents.to_string()));
            self.pos += matchlen;
            return inl;
        }

        make_inline(self.arena, NodeValue::Text("<".to_string()))
    }

    pub fn push_bracket(&mut self, image: bool, inl_text: &'a AstNode<'a>) {
        let len = self.brackets.len();
        if len > 0 {
            self.brackets[len - 1].bracket_after = true;
        }
        self.brackets.push(Bracket {
            previous_delimiter: self.last_delimiter,
            inl_text: inl_text,
            position: self.pos,
            image: image,
            active: true,
            bracket_after: false,
        });
    }

    pub fn handle_close_bracket(&mut self) -> Option<&'a AstNode<'a>> {
        self.pos += 1;
        let initial_pos = self.pos;

        let brackets_len = self.brackets.len();
        if brackets_len == 0 {
            return Some(make_inline(self.arena, NodeValue::Text("]".to_string())));
        }

        if !self.brackets[brackets_len - 1].active {
            self.brackets.pop();
            return Some(make_inline(self.arena, NodeValue::Text("]".to_string())));
        }

        let is_image = self.brackets[brackets_len - 1].image;
        let after_link_text_pos = self.pos;

        let mut sps = 0;
        let mut n = 0;
        if self.peek_char() == Some(&(b'(')) &&
            {
                sps = scanners::spacechars(&self.input[self.pos + 1..]).unwrap_or(0);
                unwrap_into(
                    manual_scan_link_url(&self.input[self.pos + 1 + sps..]),
                    &mut n,
                )
            }
        {
            let starturl = self.pos + 1 + sps;
            let endurl = starturl + n;
            let starttitle = endurl + scanners::spacechars(&self.input[endurl..]).unwrap_or(0);
            let endtitle = if starttitle == endurl {
                starttitle
            } else {
                starttitle + scanners::link_title(&self.input[starttitle..]).unwrap_or(0)
            };
            let endall = endtitle + scanners::spacechars(&self.input[endtitle..]).unwrap_or(0);

            if self.input.as_bytes()[endall] == b')' {
                self.pos = endall + 1;
                let url = strings::clean_url(&self.input[starturl..endurl]);
                let title = strings::clean_title(&self.input[starttitle..endtitle]);
                self.close_bracket_match(is_image, url, title);
                return None;
            } else {
                self.pos = after_link_text_pos;
            }
        }

        let (mut lab, mut found_label) = match self.link_label() {
            Some(lab) => (lab.to_string(), true),
            None => (String::new(), false),
        };

        if !found_label {
            self.pos = initial_pos;
        }

        if (!found_label || lab.is_empty()) && !self.brackets[brackets_len - 1].bracket_after {
            lab = self.input[self.brackets[brackets_len - 1].position..initial_pos - 1].to_string();
            found_label = true;
        }

        let reff: Option<Reference> = if found_label {
            lab = strings::normalize_reference_label(&lab);
            self.refmap.get(&lab).cloned()
        } else {
            None
        };

        if let Some(reff) = reff {
            self.close_bracket_match(is_image, reff.url.clone(), reff.title.clone());
            return None;
        }

        self.brackets.pop();
        self.pos = initial_pos;
        Some(make_inline(self.arena, NodeValue::Text("]".to_string())))
    }

    pub fn close_bracket_match(&mut self, is_image: bool, url: String, title: String) {
        let nl = NodeLink {
            url: url,
            title: title,
        };
        let inl = make_inline(
            self.arena,
            if is_image {
                NodeValue::Image(nl)
            } else {
                NodeValue::Link(nl)
            },
        );

        let mut brackets_len = self.brackets.len();
        self.brackets[brackets_len - 1].inl_text.insert_before(inl);
        let mut tmpch = self.brackets[brackets_len - 1].inl_text.next_sibling();
        while let Some(tmp) = tmpch {
            tmpch = tmp.next_sibling();
            inl.append(tmp);
        }
        self.brackets[brackets_len - 1].inl_text.detach();
        let previous_delimiter = self.brackets[brackets_len - 1].previous_delimiter;
        self.process_emphasis(previous_delimiter);
        self.brackets.pop();
        brackets_len -= 1;

        if !is_image {
            let mut i = brackets_len as i32 - 1;
            while i >= 0 {
                if !self.brackets[i as usize].image {
                    if !self.brackets[i as usize].active {
                        break;
                    } else {
                        self.brackets[i as usize].active = false;
                    }
                }
                i -= 1;
            }
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
            Some(raw_label)
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
}

pub fn manual_scan_link_url(input: &str) -> Option<usize> {
    let len = input.len();
    let mut i = 0;
    let mut nb_p = 0;

    if i < len && input.as_bytes()[i] == b'<' {
        i += 1;
        while i < len {
            let b = input.as_bytes()[i];
            if b == b'>' {
                i += 1;
                break;
            } else if b == b'\\' {
                i += 2;
            } else if isspace(b) || b == b'<' {
                return None
            } else {
                i += 1;
            }
        }
    } else {
        while i < len {
            if input.as_bytes()[i] == b'\\' {
                i += 2;
            } else if input.as_bytes()[i] == b'(' {
                nb_p += 1;
                i += 1;
                if nb_p > 32 {
                    return None
                }
            } else if input.as_bytes()[i] == b')' {
                if nb_p == 0 {
                    break;
                }
                nb_p -= 1;
                i += 1;
            } else if isspace(input.as_bytes()[i]) {
                break;
            } else {
                i += 1;
            }
        }
    }

    if i >= len { None } else { Some(i) }
}

pub fn make_inline<'a>(arena: &'a Arena<AstNode<'a>>, value: NodeValue) -> &'a AstNode<'a> {
    let ast = Ast {
        value: value,
        content: String::new(),
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        open: false,
        last_line_blank: false,
    };
    arena.alloc(Node::new(RefCell::new(ast)))
}

fn make_autolink<'a>(
    arena: &'a Arena<AstNode<'a>>,
    url: &str,
    kind: AutolinkType,
) -> &'a AstNode<'a> {
    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url: strings::clean_autolink(url, kind),
            title: String::new(),
        }),
    );
    inl.append(make_inline(
        arena,
        NodeValue::Text(entity::unescape_html(url)),
    ));
    inl
}
