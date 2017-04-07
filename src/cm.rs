use {std, Node, AstCell, ComrakOptions, NodeValue, ListType, ListDelimType, NodeLink, scanners};
use node::TableAlignment;
use ctype::{isspace, isdigit, isalpha};
use std::cmp::max;
use std::io::Write;

pub fn format_document<'a>(root: &'a Node<'a, AstCell>, options: &ComrakOptions) -> String {
    let mut f = CommonMarkFormatter::new(root, options);
    f.format(root);
    if f.v.len() > 0 && f.v[f.v.len() - 1] != '\n' as u8 {
        f.v.push('\n' as u8);
    }
    String::from_utf8(f.v).unwrap()
}

struct CommonMarkFormatter<'a, 'o> {
    node: &'a Node<'a, AstCell>,
    options: &'o ComrakOptions,
    v: Vec<u8>,
    prefix: Vec<u8>,
    column: usize,
    need_cr: u8,
    last_breakable: usize,
    begin_line: bool,
    begin_content: bool,
    no_linebreaks: bool,
    in_tight_list_item: bool,
    custom_escape: Option<fn(&'a Node<'a, AstCell>, u8) -> bool>,
}

#[derive(PartialEq, Clone, Copy)]
enum Escaping {
    Literal,
    Normal,
    URL,
    Title,
}

impl<'a, 'o> Write for CommonMarkFormatter<'a, 'o> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output(buf, false, Escaping::Literal);
        std::result::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::result::Result::Ok(())
    }
}

impl<'a, 'o> CommonMarkFormatter<'a, 'o> {
    fn new(node: &'a Node<'a, AstCell>, options: &'o ComrakOptions) -> Self {
        CommonMarkFormatter {
            node: node,
            options: options,
            v: vec![],
            prefix: vec![],
            column: 0,
            need_cr: 0,
            last_breakable: 0,
            begin_line: true,
            begin_content: true,
            no_linebreaks: false,
            in_tight_list_item: false,
            custom_escape: None,
        }
    }

    fn output(&mut self, buf: &[u8], wrap: bool, escaping: Escaping) {
        let wrap = wrap && !self.no_linebreaks;

        if self.in_tight_list_item && self.need_cr > 1 {
            self.need_cr = 1;
        }

        let mut k = self.v.len() as i32 - 1;
        while self.need_cr > 0 {
            if k < 0 || self.v[k as usize] == '\n' as u8 {
                k -= 1;
            } else {
                self.v.push('\n' as u8);
                if self.need_cr > 1 {
                    self.v.extend(&self.prefix);
                }
            }
            self.column = 0;
            self.begin_line = true;
            self.begin_content = true;
            self.need_cr -= 1;
        }

        let mut i = 0;
        while i < buf.len() {
            if self.begin_line {
                self.v.extend(&self.prefix);
                self.column = self.prefix.len();
            }

            if self.custom_escape.is_some() && self.custom_escape.unwrap()(self.node, buf[i]) {
                self.v.push('\\' as u8);
            }

            let nextc = buf.get(i + 1);
            if buf[i] == ' ' as u8 && wrap {
                if !self.begin_line {
                    let last_nonspace = self.v.len();
                    self.v.push(' ' as u8);
                    self.column += 1;
                    self.begin_line = false;
                    self.begin_content = false;
                    while buf.get(i + 1) == Some(&(' ' as u8)) {
                        i += 1;
                    }
                    if !buf.get(i + 1).map_or(false, |&c| isdigit(&(c as char))) {
                        self.last_breakable = last_nonspace;
                    }
                }
            } else if buf[i] == '\n' as u8 {
                self.v.push('\n' as u8);
                self.column = 0;
                self.begin_line = true;
                self.begin_content = true;
                self.last_breakable = 0;
            } else if escaping == Escaping::Literal {
                self.v.push(buf[i]);
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(&(buf[i] as char));
            } else {
                self.outc(buf[i], escaping, nextc);
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(&(buf[i] as char));
            }

            if self.options.width > 0 && self.column > self.options.width && !self.begin_line &&
               self.last_breakable > 0 {
                let remainder = self.v[self.last_breakable + 1..].to_vec();
                self.v.truncate(self.last_breakable);
                self.v.push('\n' as u8);
                self.v.extend(&self.prefix);
                self.v.extend(&remainder);
                self.column = self.prefix.len() + remainder.len();
                self.last_breakable = 0;
                self.begin_line = false;
                self.begin_content = false;
            }

            i += 1;
        }
    }

    fn outc(&mut self, c: u8, escaping: Escaping, nextc: Option<&u8>) {
        let follows_digit = self.v.len() > 0 && isdigit(&(self.v[self.v.len() - 1] as char));

        let nextc = nextc.map_or(0, |&c| c);

        let needs_escaping =
            c < 0x80 && escaping != Escaping::Literal &&
            ((escaping == Escaping::Normal &&
              (c == '*' as u8 || c == '_' as u8 || c == '[' as u8 || c == ']' as u8 ||
               c == '#' as u8 || c == '<' as u8 ||
               c == '>' as u8 || c == '\\' as u8 || c == '`' as u8 ||
               c == '!' as u8 || (c == '&' as u8 && isalpha(&(nextc as char))) ||
               (c == '!' as u8 && nextc == 0x5b) ||
               (self.begin_content && (c == '-' as u8 || c == '+' as u8 || c == '=' as u8) &&
                !follows_digit) ||
               (self.begin_content && (c == '.' as u8 || c == ')' as u8) && follows_digit &&
                (nextc == 0 || isspace(&(nextc as char)))))) ||
             (escaping == Escaping::URL &&
              (c == '`' as u8 || c == '<' as u8 || c == '>' as u8 || isspace(&(c as char)) ||
               c == '\\' as u8 || c == ')' as u8 ||
               c == '(' as u8)) ||
             (escaping == Escaping::Title &&
              (c == '`' as u8 || c == '<' as u8 || c == '>' as u8 || c == '"' as u8 ||
               c == '\\' as u8)));

        if needs_escaping {
            if isspace(&(c as char)) {
                write!(self.v, "%{:2x}", c).unwrap();
                self.column += 3;
            } else {
                write!(self.v, "\\{}", c as char).unwrap();
            }
        } else {
            self.v.push(c);
        }
    }

    fn cr(&mut self) {
        self.need_cr = max(self.need_cr, 1);
    }

    fn blankline(&mut self) {
        self.need_cr = max(self.need_cr, 2);
    }

    fn format_children(&mut self, node: &'a Node<'a, AstCell>) {
        for n in node.children() {
            self.format(n);
        }
    }

    fn format(&mut self, node: &'a Node<'a, AstCell>) {
        if self.format_node(node, true) {
            self.format_children(node);
            self.format_node(node, false);
        }
    }

    fn get_in_tight_list_item(&self, node: &'a Node<'a, AstCell>) -> bool {
        let tmp = match node.containing_block() {
            Some(tmp) => tmp,
            None => return false,
        };

        if let &NodeValue::Item(..) = &tmp.data.borrow().value {
            if let &NodeValue::List(ref nl) = &tmp.parent().unwrap().data.borrow().value {
                return nl.tight;
            }
            return false;
        }

        let parent = match tmp.parent() {
            Some(parent) => parent,
            None => return false,
        };

        if let &NodeValue::Item(..) = &parent.data.borrow().value {
            if let &NodeValue::List(ref nl) = &parent.parent().unwrap().data.borrow().value {
                return nl.tight;
            }
        }

        return false;
    }

    fn format_node(&mut self, node: &'a Node<'a, AstCell>, entering: bool) -> bool {
        self.node = node;
        let allow_wrap = self.options.width > 0 && !self.options.hardbreaks;

        if !(match &node.data.borrow().value {
            &NodeValue::Item(..) => true,
            _ => false,
        } && node.previous_sibling().is_none() && entering) {
            self.in_tight_list_item = self.get_in_tight_list_item(node);
        }

        match &node.data.borrow().value {
            &NodeValue::Document => (),
            &NodeValue::BlockQuote => {
                if entering {
                    write!(self, "> ").unwrap();
                    self.begin_content = true;
                    write!(self.prefix, "> ").unwrap();
                } else {
                    let new_len = self.prefix.len() - 2;
                    self.prefix.truncate(new_len);
                    self.blankline();
                }
            }
            &NodeValue::List(..) => {
                if !entering &&
                   match node.next_sibling() {
                    Some(next_sibling) => {
                        match &next_sibling.data.borrow().value {
                            &NodeValue::CodeBlock(..) |
                            &NodeValue::List(..) => true,
                            _ => false,
                        }
                    }
                    _ => false,
                } {
                    self.cr();
                    write!(self, "<!-- end list -->").unwrap();
                    self.blankline();
                }
            }
            &NodeValue::Item(..) => {
                let parent = match &node.parent().unwrap().data.borrow().value {
                    &NodeValue::List(ref nl) => nl.clone(),
                    _ => unreachable!(),
                };

                let mut listmarker = vec![];

                let marker_width = if parent.list_type == ListType::Bullet {
                    4
                } else {
                    let mut list_number = parent.start;
                    let list_delim = parent.delimiter;
                    let mut tmpch = node;
                    while let Some(tmp) = tmpch.previous_sibling() {
                        tmpch = tmp;
                        list_number += 1;
                    }
                    write!(listmarker,
                           "{}{}{}",
                           list_number,
                           if list_delim == ListDelimType::Paren {
                               ")"
                           } else {
                               "."
                           },
                           if list_number < 10 { "  " } else { " " })
                        .unwrap();
                    listmarker.len()
                };

                if entering {
                    if parent.list_type == ListType::Bullet {
                        write!(self, "  - ").unwrap();
                    } else {
                        self.write_all(&listmarker).unwrap();
                    }
                    self.begin_content = true;
                    for _ in 0..marker_width {
                        write!(self.prefix, " ").unwrap();
                    }
                } else {
                    let new_len = self.prefix.len() - marker_width;
                    self.prefix.truncate(new_len);
                    self.cr();
                }
            }
            &NodeValue::Heading(ref nch) => {
                if entering {
                    for _ in 0..nch.level {
                        write!(self, "#").unwrap();
                    }
                    write!(self, " ").unwrap();
                    self.begin_content = true;
                    self.no_linebreaks = true;
                } else {
                    self.no_linebreaks = false;
                    self.blankline();
                }
            }
            &NodeValue::CodeBlock(ref ncb) => {
                if entering {
                    let first_in_list_item = node.previous_sibling().is_none() &&
                                             match node.parent() {
                        Some(parent) => {
                            match &parent.data.borrow().value {
                                &NodeValue::Item(..) => true,
                                _ => false,
                            }
                        }
                        _ => false,
                    };

                    if !first_in_list_item {
                        self.blankline();
                    }

                    if ncb.info.len() == 0 &&
                       (ncb.literal.len() > 2 && !isspace(&ncb.literal[0]) &&
                        !(isspace(&ncb.literal[ncb.literal.len() - 1]) &&
                          isspace(&ncb.literal[ncb.literal.len() - 2]))) &&
                       !first_in_list_item {
                        write!(self, "    ").unwrap();
                        write!(self.prefix, "    ").unwrap();
                        write!(self, "{}", ncb.literal.iter().collect::<String>()).unwrap();
                        let new_len = self.prefix.len() - 4;
                        self.prefix.truncate(new_len);
                    } else {
                        let numticks = max(3, longest_backtick_sequence(&ncb.literal) + 1);
                        for _ in 0..numticks {
                            write!(self, "`").unwrap();
                        }
                        if ncb.info.len() > 0 {
                            write!(self, " {}", ncb.info.iter().collect::<String>()).unwrap();
                        }
                        self.cr();
                        write!(self, "{}", ncb.literal.iter().collect::<String>()).unwrap();
                        self.cr();
                        for _ in 0..numticks {
                            write!(self, "`").unwrap();
                        }
                    }
                    self.blankline();
                }
            }
            &NodeValue::HtmlBlock(ref nhb) => {
                if entering {
                    self.blankline();
                    self.write_all(nhb.literal.iter().collect::<String>().as_bytes()).unwrap();
                    self.blankline();
                }
            }
            &NodeValue::CustomBlock => {
                assert!(false)
                // TODO
            }
            &NodeValue::ThematicBreak => {
                if entering {
                    self.blankline();
                    write!(self, "-----").unwrap();
                    self.blankline();
                }
            }
            &NodeValue::Paragraph => {
                if !entering {
                    self.blankline();
                }
            }
            &NodeValue::Text(ref literal) => {
                if entering {
                    self.output(literal.iter().collect::<String>().as_bytes(),
                                allow_wrap,
                                Escaping::Normal);
                }
            }
            &NodeValue::LineBreak => {
                if entering {
                    if !self.options.hardbreaks {
                        write!(self, "  ").unwrap();
                    }
                    self.cr();
                }
            }
            &NodeValue::SoftBreak => {
                if entering {
                    if !self.no_linebreaks && self.options.width == 0 && !self.options.hardbreaks {
                        self.cr();
                    } else {
                        self.output(&[' ' as u8], allow_wrap, Escaping::Literal);
                    }
                }
            }
            &NodeValue::Code(ref literal) => {
                if entering {
                    let numticks = shortest_unused_sequence(literal, '`');
                    for _ in 0..numticks {
                        write!(self, "`").unwrap();
                    }
                    if literal.len() == 0 || literal[0] == '`' {
                        write!(self, " ").unwrap();
                    }
                    self.output(literal.iter().collect::<String>().as_bytes(),
                                allow_wrap,
                                Escaping::Literal);
                    if literal.len() == 0 || literal[literal.len() - 1] == '`' {
                        write!(self, " ").unwrap();
                    }
                    for _ in 0..numticks {
                        write!(self, "`").unwrap();
                    }
                }
            }
            &NodeValue::HtmlInline(ref literal) => {
                if entering {
                    self.write_all(literal.into_iter().collect::<String>().as_bytes()).unwrap();
                }
            }
            &NodeValue::CustomInline => {
                assert!(false)
                // TODO
            }
            &NodeValue::Strong => {
                if entering {
                    write!(self, "**").unwrap();
                } else {
                    write!(self, "**").unwrap();
                }
            }
            &NodeValue::Emph => {
                let emph_delim = if match node.parent() {
                    Some(parent) => {
                        match &parent.data.borrow().value {
                            &NodeValue::Emph => true,
                            _ => false,
                        }
                    }
                    _ => false,
                } && node.next_sibling().is_none() &&
                                    node.previous_sibling().is_none() {
                    '_' as u8
                } else {
                    '*' as u8
                };

                if entering {
                    self.write_all(&[emph_delim]).unwrap();
                } else {
                    self.write_all(&[emph_delim]).unwrap();
                }
            }
            &NodeValue::Strikethrough => {
                if entering {
                    write!(self, "~").unwrap();
                } else {
                    write!(self, "~").unwrap();
                }
            }
            &NodeValue::Link(ref nl) => {
                if is_autolink(node, nl) {
                    if entering {
                        write!(self, "<").unwrap();
                        if &nl.url[..7] == &['m', 'a', 'i', 'l', 't', 'o', ':'] {
                            self.write_all(nl.url[7..].into_iter().collect::<String>().as_bytes())
                                .unwrap();
                        } else {
                            self.write_all(nl.url.iter().collect::<String>().as_bytes()).unwrap();
                        }
                        write!(self, ">").unwrap();
                        return false;
                    }
                } else {
                    if entering {
                        write!(self, "[").unwrap();
                    } else {
                        write!(self, "](").unwrap();
                        self.output(nl.url.iter().collect::<String>().as_bytes(),
                                    false,
                                    Escaping::URL);
                        if nl.title.len() > 0 {
                            write!(self, " \"").unwrap();
                            self.output(nl.title.iter().collect::<String>().as_bytes(),
                                        false,
                                        Escaping::Title);
                            write!(self, "\"").unwrap();
                        }
                        write!(self, ")").unwrap();
                    }
                }
            }
            &NodeValue::Image(ref nl) => {
                if entering {
                    write!(self, "![").unwrap();
                } else {
                    write!(self, "](").unwrap();
                    self.output(nl.url.iter().collect::<String>().as_bytes(),
                                false,
                                Escaping::URL);
                    if nl.title.len() > 0 {
                        self.output(&[' ' as u8, '"' as u8], allow_wrap, Escaping::Literal);
                        self.output(nl.title.iter().collect::<String>().as_bytes(),
                                    false,
                                    Escaping::Title);
                        write!(self, "\"").unwrap();
                    }
                    write!(self, ")").unwrap();
                }
            }
            &NodeValue::Table(..) => {
                if entering {
                    self.custom_escape = Some(table_escape);
                } else {
                    self.custom_escape = None;
                }
                self.blankline();
            }
            &NodeValue::TableRow(..) => {
                if entering {
                    self.cr();
                    write!(self, "|").unwrap();
                }
            }
            &NodeValue::TableCell => {
                if entering {
                    write!(self, " ").unwrap();
                } else {
                    write!(self, " |").unwrap();

                    let ref row = node.parent().unwrap().data.borrow().value;
                    let in_header = match row {
                        &NodeValue::TableRow(header) => header,
                        _ => panic!(),
                    };

                    if in_header && node.next_sibling().is_none() {
                        let ref table =
                            node.parent().unwrap().parent().unwrap().data.borrow().value;
                        let alignments = match table {
                            &NodeValue::Table(ref alignments) => alignments,
                            _ => panic!(),
                        };

                        self.cr();
                        write!(self, "|").unwrap();
                        for a in alignments {
                            write!(self,
                                   " {} |",
                                   match a {
                                       &TableAlignment::Left => ":--",
                                       &TableAlignment::Center => ":-:",
                                       &TableAlignment::Right => "--:",
                                       &TableAlignment::None => "---",
                                   })
                                .unwrap();
                        }
                        self.cr();
                    }
                }
            }
        };
        true
    }
}

fn longest_backtick_sequence(literal: &String) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for c in literal {
        if *c == '`' {
            current += 1;
        } else {
            if current > longest {
                longest = current;
            }
            current = 0;
        }
    }
    if current > longest {
        longest = current;
    }
    longest
}

fn shortest_unused_sequence(literal: &String, f: char) -> usize {
    let mut used = 1;
    let mut current = 0;
    for c in literal {
        if *c == f {
            current += 1;
        } else {
            if current > 0 {
                used |= 1 << current;
            }
            current = 0;
        }
    }

    if current > 0 {
        used |= 1 << current;
    }

    let mut i = 0;
    while used & 1 != 0 {
        used = used >> 1;
        i += 1;
    }
    i
}

fn is_autolink<'a>(node: &'a Node<'a, AstCell>, nl: &NodeLink) -> bool {
    if nl.url.len() == 0 || scanners::scheme(&nl.url).is_none() {
        return false;
    }

    if nl.title.len() > 0 {
        return false;
    }

    let link_text = match node.first_child() {
        None => return false,
        Some(child) => {
            match &child.data.borrow().value {
                &NodeValue::Text(ref t) => t.clone(),
                _ => return false,
            }
        }
    };

    let mut real_url = nl.url.as_slice();
    if &real_url[..7] == &['m', 'a', 'i', 'l', 't', 'o', ':'] {
        real_url = &real_url[7..];
    }

    real_url == link_text.as_slice()
}

fn table_escape<'a>(node: &'a Node<'a, AstCell>, c: u8) -> bool {
    match &node.data.borrow().value {
        &NodeValue::Table(..) |
        &NodeValue::TableRow(..) |
        &NodeValue::TableCell => false,
        _ => c == '|' as u8,
    }
}
