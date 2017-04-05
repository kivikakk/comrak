use unicode_categories::UnicodeCategories;

use std::cell::RefCell;
use {Arena, Node, AstCell, unwrap_into, unwrap_into_copy, entity, NodeValue, Ast, NodeLink,
     isspace, MAX_LINK_LABEL_LENGTH, ispunct, Reference, scanners, MAXBACKTICKS};
use strings::*;
use std::collections::{BTreeSet, HashMap};

pub struct Subject<'a, 'r> {
    pub arena: &'a Arena<Node<'a, AstCell>>,
    pub input: Vec<char>,
    pub pos: usize,
    pub refmap: &'r mut HashMap<Vec<char>, Reference>,
    delimiters: Vec<Delimiter<'a>>,
    brackets: Vec<Bracket<'a>>,
    pub backticks: [usize; MAXBACKTICKS + 1],
    pub scanned_for_backticks: bool,
}

struct Delimiter<'a> {
    inl: &'a Node<'a, AstCell>,
    delim_char: char,
    can_open: bool,
    can_close: bool,
}

struct Bracket<'a> {
    previous_delimiter: i32,
    inl_text: &'a Node<'a, AstCell>,
    position: usize,
    image: bool,
    active: bool,
    bracket_after: bool,
}

impl<'a, 'r> Subject<'a, 'r> {
    pub fn new(arena: &'a Arena<Node<'a, AstCell>>,
               input: &[char],
               refmap: &'r mut HashMap<Vec<char>, Reference>)
               -> Self {
        Subject {
            arena: arena,
            input: input.to_vec(),
            pos: 0,
            refmap: refmap,
            delimiters: vec![],
            brackets: vec![],
            backticks: [0; MAXBACKTICKS + 1],
            scanned_for_backticks: false,
        }
    }

    pub fn pop_bracket(&mut self) -> bool {
        self.brackets.pop().is_some()
    }

    pub fn parse_inline(&mut self, node: &'a Node<'a, AstCell>) -> bool {
        let new_inl: Option<&'a Node<'a, AstCell>>;
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
            '*' | '_' | '\'' | '"' => new_inl = Some(self.handle_delim(c)),
            // TODO: smart characters. Eh.
            //'-' => new_inl => Some(self.handle_hyphen()),
            //'.' => new_inl => Some(self.handle_period()),
            '[' => {
                self.pos += 1;
                let inl = make_inline(self.arena, NodeValue::Text(vec!['[']));
                new_inl = Some(inl);
                self.push_bracket(false, inl);
            },
            ']' => new_inl = self.handle_close_bracket(),
            '!' => {
                self.pos += 1;
                if self.peek_char() == Some(&'[') {
                    self.pos += 1;
                    let inl = make_inline(self.arena, NodeValue::Text(vec!['!', '[']));
                    new_inl = Some(inl);
                    self.push_bracket(true, inl);
                } else {
                    new_inl = Some(make_inline(self.arena, NodeValue::Text(vec!['!'])));
                }
            },
            _ => {
                let endpos = self.find_special_char();
                let mut contents = self.input[self.pos..endpos].to_vec();
                self.pos = endpos;

                if self.peek_char().map_or(false, is_line_end_char) {
                    rtrim(&mut contents);
                }

                new_inl = Some(make_inline(self.arena, NodeValue::Text(contents)));
            }
        }

        if let Some(inl) = new_inl {
            node.append(inl);
        }

        true
    }

    pub fn process_emphasis(&mut self, stack_bottom: i32) {
        let mut closer = self.delimiters.len() as i32 - 1;
        let mut openers_bottom: Vec<[i32; 128]> = vec![];
        for _ in 0..3 {
            let mut a = [-1; 128];
            a['*' as usize] = stack_bottom;
            a['_' as usize] = stack_bottom;
            a['\'' as usize] = stack_bottom;
            a['"' as usize] = stack_bottom;
            openers_bottom.push(a)
        }

        while closer != -1 && closer - 1 > stack_bottom {
            closer -= 1;
        }

        while closer != -1 && (closer as usize) < self.delimiters.len() {
            if self.delimiters[closer as usize].can_close {
                let mut opener = closer - 1;
                let mut opener_found = false;

                while opener != -1 && opener != stack_bottom &&
                      opener !=
                      openers_bottom[self.delimiters[closer as usize]
                    .inl
                    .data
                    .borrow_mut()
                    .value
                    .text()
                    .unwrap()
                    .len() % 3][self.delimiters[closer as usize]
                    .delim_char as usize] {
                    if self.delimiters[opener as usize].can_open &&
                       self.delimiters[opener as usize].delim_char ==
                       self.delimiters[closer as usize].delim_char {
                        let odd_match = (self.delimiters[closer as usize].can_open ||
                                         self.delimiters[opener as usize].can_close) &&
                                        ((self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap()
                            .len() +
                                          self.delimiters[closer as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap()
                            .len()) % 3 == 0);
                        if !odd_match {
                            opener_found = true;
                            break;
                        }
                    }
                    opener -= 1;
                }
                let old_closer = closer;

                if self.delimiters[closer as usize].delim_char == '*' ||
                   self.delimiters[closer as usize].delim_char == '_' {
                    if opener_found {
                        closer = self.insert_emph(opener, closer);
                    } else {
                        closer += 1;
                    }
                } else if self.delimiters[closer as usize].delim_char == '\'' {
                    *self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "’".chars().collect();
                    if opener_found {
                        *self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "‘".chars().collect();
                    }
                    closer += 1;
                } else if self.delimiters[closer as usize].delim_char == '"' {
                    *self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap() =
                        "”".chars().collect();
                    if opener_found {
                        *self.delimiters[opener as usize]
                            .inl
                            .data
                            .borrow_mut()
                            .value
                            .text()
                            .unwrap() = "“".chars().collect();
                    }
                    closer += 1;
                }
                if !opener_found {
                    let ix = self.delimiters[old_closer as usize]
                        .inl
                        .data
                        .borrow_mut()
                        .value
                        .text()
                        .unwrap()
                        .len() % 3;
                    openers_bottom[ix][self.delimiters[old_closer as usize].delim_char as usize] =
                        old_closer - 1;
                    if !self.delimiters[old_closer as usize].can_open {
                        self.delimiters.remove(old_closer as usize);
                    }
                }
            } else {
                closer += 1;
            }
        }

        // TODO truncate instead!
        while self.delimiters.len() > (stack_bottom + 1) as usize {
            self.delimiters.pop();
        }
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn peek_char<'x>(&'x self) -> Option<&'x char> {
        self.input.get(self.pos).map(|c| {
            assert!(*c > '\0');
            c
        })
    }

    pub fn find_special_char(&self) -> usize {
        lazy_static! {
            static ref SPECIAL_CHARS: BTreeSet<char> =
                ['\n',
                '\r',
                '_',
                '*',
                '"',
                '`',
                '\\',
                '&',
                '<',
                '[',
                ']',
                '!',
                ].iter().cloned().collect();
        }

        for n in self.pos..self.input.len() {
            if SPECIAL_CHARS.contains(&self.input[n]) {
                return n;
            }
        }

        self.input.len()
    }

    pub fn handle_newline(&mut self) -> &'a Node<'a, AstCell> {
        let nlpos = self.pos;
        if self.input[self.pos] == '\r' {
            self.pos += 1;
        }
        if self.input[self.pos] == '\n' {
            self.pos += 1;
        }
        self.skip_spaces();
        if nlpos > 1 && self.input[nlpos - 1] == ' ' && self.input[nlpos - 2] == ' ' {
            make_inline(self.arena, NodeValue::LineBreak)
        } else {
            make_inline(self.arena, NodeValue::SoftBreak)
        }
    }

    pub fn take_while(&mut self, c: char) -> Vec<char> {
        let mut v = vec![];
        while self.peek_char() == Some(&c) {
            v.push(self.input[self.pos]);
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
            while self.peek_char().map_or(false, |&c| c != '`') {
                self.pos += 1;
            }
            if self.pos >= self.input.len() {
                self.scanned_for_backticks = true;
                return None;
            }
            let numticks = self.take_while('`').len();
            if numticks <= MAXBACKTICKS {
                self.backticks[numticks] = self.pos - numticks;
            }
            if numticks == openticklength {
                return Some(self.pos);
            }
        }
    }

    pub fn handle_backticks(&mut self) -> &'a Node<'a, AstCell> {
        let openticks = self.take_while('`');
        let startpos = self.pos;
        let endpos = self.scan_to_closing_backtick(openticks.len());

        match endpos {
            None => {
                self.pos = startpos;
                return make_inline(self.arena, NodeValue::Text(openticks));
            }
            Some(endpos) => {
                let mut buf = self.input[startpos..endpos - openticks.len()].to_vec();
                trim(&mut buf);
                normalize_whitespace(&mut buf);
                make_inline(self.arena, NodeValue::Code(buf))
            }
        }
    }

    pub fn skip_spaces(&mut self) -> bool {
        let mut skipped = false;
        while self.peek_char().map_or(false, |&c| c == ' ' || c == '\t') {
            self.pos += 1;
            skipped = true;
        }
        skipped
    }

    pub fn handle_delim(&mut self, c: char) -> &'a Node<'a, AstCell> {
        let (numdelims, can_open, can_close) = self.scan_delims(c);

        let contents = self.input[self.pos - numdelims..self.pos].to_vec();
        let inl = make_inline(self.arena, NodeValue::Text(contents));

        if (can_open || can_close) && c != '\'' && c != '"' {
            self.push_delimiter(c, can_open, can_close, inl);
        }

        inl
    }

    pub fn scan_delims(&mut self, c: char) -> (usize, bool, bool) {
        let before_char = if self.pos == 0 {
            '\n'
        } else {
            self.input[self.pos - 1]
        };

        let mut numdelims = 0;
        if c == '\'' || c == '"' {
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
            self.input[self.pos]
        };

        let left_flanking = numdelims > 0 && !after_char.is_whitespace() &&
                            !(after_char.is_punctuation() && !before_char.is_whitespace() &&
                              !before_char.is_punctuation());
        let right_flanking = numdelims > 0 && !before_char.is_whitespace() &&
                             !(before_char.is_punctuation() && !after_char.is_whitespace() &&
                               !after_char.is_punctuation());

        if c == '_' {
            (numdelims,
             left_flanking && (!right_flanking || before_char.is_punctuation()),
             right_flanking && (!left_flanking || after_char.is_punctuation()))
        } else if c == '\'' || c == '"' {
            (numdelims, left_flanking && !right_flanking, right_flanking)
        } else {
            (numdelims, left_flanking, right_flanking)
        }
    }

    pub fn push_delimiter(&mut self,
                          c: char,
                          can_open: bool,
                          can_close: bool,
                          inl: &'a Node<'a, AstCell>) {
        self.delimiters.push(Delimiter {
            inl: inl,
            delim_char: c,
            can_open: can_open,
            can_close: can_close,
        });
    }

    pub fn insert_emph(&mut self, opener: i32, mut closer: i32) -> i32 {
        let mut opener_num_chars =
            self.delimiters[opener as usize].inl.data.borrow_mut().value.text().unwrap().len();
        let mut closer_num_chars =
            self.delimiters[closer as usize].inl.data.borrow_mut().value.text().unwrap().len();
        let use_delims = if closer_num_chars >= 2 && opener_num_chars >= 2 {
            2
        } else {
            1
        };

        opener_num_chars -= use_delims;
        closer_num_chars -= use_delims;
        self.delimiters[opener as usize]
            .inl
            .data
            .borrow_mut()
            .value
            .text()
            .unwrap()
            .truncate(opener_num_chars);
        self.delimiters[closer as usize]
            .inl
            .data
            .borrow_mut()
            .value
            .text()
            .unwrap()
            .truncate(closer_num_chars);

        // TODO: just remove the range directly
        let mut delim = closer - 1;
        while delim != -1 && delim != opener {
            self.delimiters.remove(delim as usize);
            delim -= 1;
            closer -= 1;
        }

        let emph = make_inline(self.arena,
                               if use_delims == 1 {
                                   NodeValue::Emph
                               } else {
                                   NodeValue::Strong
                               });

        let mut tmp = self.delimiters[opener as usize].inl.next_sibling().unwrap();
        while !tmp.same_node(self.delimiters[closer as usize].inl) {
            let next = tmp.next_sibling();
            emph.append(tmp);
            if let Some(n) = next {
                tmp = n;
            } else {
                break;
            }
        }
        self.delimiters[opener as usize].inl.insert_after(emph);

        if opener_num_chars == 0 {
            self.delimiters[opener as usize].inl.detach();
            self.delimiters.remove(opener as usize);
            closer -= 1;
        }

        if closer_num_chars == 0 {
            self.delimiters[closer as usize].inl.detach();
            self.delimiters.remove(closer as usize);
        }

        if closer == -1 || (closer as usize) < self.delimiters.len() {
            closer
        } else {
            -1
        }
    }

    pub fn handle_backslash(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;
        if self.peek_char().map_or(false, ispunct) {
            self.pos += 1;
            return make_inline(self.arena, NodeValue::Text(vec![self.input[self.pos - 1]]));
        } else if !self.eof() && self.skip_line_end() {
            return make_inline(self.arena, NodeValue::LineBreak);
        } else {
            return make_inline(self.arena, NodeValue::Text(vec!['\\']));
        }
    }

    pub fn skip_line_end(&mut self) -> bool {
        let mut seen_line_end_char = false;
        if self.peek_char() == Some(&'\r') {
            self.pos += 1;
            seen_line_end_char = true;
        }
        if self.peek_char() == Some(&'\n') {
            self.pos += 1;
            seen_line_end_char = true;
        }
        seen_line_end_char || self.eof()
    }

    pub fn handle_entity(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;

        match entity::unescape(&self.input[self.pos..]) {
            None => make_inline(self.arena, NodeValue::Text(vec!['&'])),
            Some((entity, len)) => {
                self.pos += len;
                make_inline(self.arena, NodeValue::Text(entity))
            }
        }
    }

    pub fn handle_pointy_brace(&mut self) -> &'a Node<'a, AstCell> {
        self.pos += 1;

        if let Some(matchlen) = scanners::autolink_uri(&self.input[self.pos..]) {
            let inl = make_autolink(self.arena,
                                    &self.input[self.pos..self.pos + matchlen - 1],
                                    AutolinkType::URI);
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::autolink_email(&self.input[self.pos..]) {
            let inl = make_autolink(self.arena,
                                    &self.input[self.pos..self.pos + matchlen - 1],
                                    AutolinkType::Email);
            self.pos += matchlen;
            return inl;
        }

        if let Some(matchlen) = scanners::html_tag(&self.input[self.pos..]) {
            let contents = &self.input[self.pos - 1..self.pos + matchlen];
            let inl = make_inline(self.arena, NodeValue::HtmlInline(contents.to_vec()));
            self.pos += matchlen;
            return inl;
        }

        make_inline(self.arena, NodeValue::Text(vec!['<']))
    }

    pub fn push_bracket(&mut self, image: bool, inl_text: &'a Node<'a, AstCell>) {
        let len = self.brackets.len();
        if len > 0 {
            self.brackets[len - 1].bracket_after = true;
        }
        self.brackets.push(Bracket {
            previous_delimiter: self.delimiters.len() as i32 - 1,
            inl_text: inl_text,
            position: self.pos,
            image: image,
            active: true,
            bracket_after: false,
        });
    }

    pub fn handle_close_bracket(&mut self) -> Option<&'a Node<'a, AstCell>> {
        self.pos += 1;
        let initial_pos = self.pos;

        let brackets_len = self.brackets.len();
        if brackets_len == 0 {
            return Some(make_inline(self.arena, NodeValue::Text(vec![']'])));
        }

        if !self.brackets[brackets_len - 1].active {
            self.brackets.pop();
            return Some(make_inline(self.arena, NodeValue::Text(vec![']'])));
        }

        let is_image = self.brackets[brackets_len - 1].image;
        let after_link_text_pos = self.pos;

        let mut sps = 0;
        let mut n = 0;
        if self.peek_char() == Some(&'(') &&
           {
            sps = scanners::spacechars(&self.input[self.pos + 1..]).unwrap_or(0);
            unwrap_into(manual_scan_link_url(&self.input[self.pos + 1 + sps..]),
                        &mut n)
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

            if self.input.get(endall) == Some(&')') {
                self.pos = endall + 1;
                let url = clean_url(&self.input[starturl..endurl]);
                let title = clean_title(&self.input[starttitle..endtitle]);
                self.close_bracket_match(is_image, url, title);
                return None;
            } else {
                self.pos = after_link_text_pos;
            }
        }

        let (mut lab, mut found_label) = match self.link_label() {
            Some(lab) => (lab.to_vec(), true),
            None => (vec![], false),
        };

        if !found_label {
            self.pos = initial_pos;
        }

        if (!found_label || lab.len() == 0) && !self.brackets[brackets_len - 1].bracket_after {
            lab = self.input[self.brackets[brackets_len - 1].position..initial_pos - 1].to_vec();
            found_label = true;
        }

        let reff: Option<Reference> = if found_label {
            lab = normalize_reference_label(&lab);
            self.refmap.get(&lab).map(|c| c.clone())
        } else {
            None
        };

        if let Some(reff) = reff {
            self.close_bracket_match(is_image, reff.url.clone(), reff.title.clone());
            return None;
        }

        self.brackets.pop();
        self.pos = initial_pos;
        Some(make_inline(self.arena, NodeValue::Text(vec![']'])))
    }

    pub fn close_bracket_match(&mut self, is_image: bool, url: Vec<char>, title: Vec<char>) {
        let nl = NodeLink {
            url: url,
            title: title,
        };
        let inl = make_inline(self.arena,
                              if is_image {
                                  NodeValue::Image(nl)
                              } else {
                                  NodeValue::Link(nl)
                              });

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

    pub fn link_label(&mut self) -> Option<&[char]> {
        let startpos = self.pos;

        if self.peek_char() != Some(&'[') {
            return None;
        }

        self.pos += 1;

        let mut length = 0;
        let mut c = '\0';
        while unwrap_into_copy(self.peek_char(), &mut c) && c != '[' && c != ']' {
            if c == '\\' {
                self.pos += 1;
                length += 1;
                if self.peek_char().map_or(false, ispunct) {
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

        if c == ']' {
            let raw_label = trim_slice(&self.input[startpos + 1..self.pos]);
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

pub fn manual_scan_link_url(input: &[char]) -> Option<usize> {
    let len = input.len();
    let mut i = 0;
    let mut nb_p = 0;

    if i < len && input[i] == '<' {
        i += 1;
        while i < len {
            if input[i] == '>' {
                i += 1;
                break;
            } else if input[i] == '\\' {
                i += 2;
            } else if isspace(&input[i]) {
                return None;
            } else {
                i += 1;
            }
        }
    } else {
        while i < len {
            if input[i] == '\\' {
                i += 2;
            } else if input[i] == '(' {
                nb_p += 1;
                i += 1;
            } else if input[i] == ')' {
                if nb_p == 0 {
                    break;
                }
                nb_p -= 1;
                i += 1;
            } else if isspace(&input[i]) {
                break;
            } else {
                i += 1;
            }
        }
    }

    if i >= len { None } else { Some(i) }
}

fn make_inline<'a>(arena: &'a Arena<Node<'a, AstCell>>, value: NodeValue) -> &'a Node<'a, AstCell> {
    let ast = Ast {
        value: value,
        content: vec![],
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        open: false,
        last_line_blank: false,
    };
    arena.alloc(Node::new(RefCell::new(ast)))
}

#[derive(PartialEq)]
pub enum AutolinkType {
    URI,
    Email,
}

fn make_autolink<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                     url: &[char],
                     kind: AutolinkType)
                     -> &'a Node<'a, AstCell> {
    let inl = make_inline(arena,
                          NodeValue::Link(NodeLink {
                              url: clean_autolink(url, kind),
                              title: vec![],
                          }));
    inl.append(make_inline(arena, NodeValue::Text(entity::unescape_html(url))));
    inl
}
