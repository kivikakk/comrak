use ctype::{isalpha, isdigit, isspace};
use nodes;
use nodes::{AstNode, ListDelimType, ListType, NodeLink, NodeValue};
use nodes::TableAlignment;
use parser::ComrakOptions;
use scanners;
use std;
use std::cmp::max;
use std::io::{self, Write};

/// Formats an AST as CommonMark, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut Write,
) -> io::Result<()> {
    let mut f = CommonMarkFormatter::new(root, options);
    f.format(root);
    if !f.v.is_empty() && f.v[f.v.len() - 1] != b'\n' {
        f.v.push(b'\n');
    }
    try!(output.write_all(&f.v));
    Ok(())
}

struct CommonMarkFormatter<'a, 'o> {
    node: &'a AstNode<'a>,
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
    custom_escape: Option<fn(&'a AstNode<'a>, u8) -> bool>,
    footnote_ix: u32,
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
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a, 'o> CommonMarkFormatter<'a, 'o> {
    fn new(node: &'a AstNode<'a>, options: &'o ComrakOptions) -> Self {
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
            footnote_ix: 0,
        }
    }

    fn output(&mut self, buf: &[u8], wrap: bool, escaping: Escaping) {
        let wrap = wrap && !self.no_linebreaks;

        if self.in_tight_list_item && self.need_cr > 1 {
            self.need_cr = 1;
        }

        let mut k = self.v.len() as i32 - 1;
        while self.need_cr > 0 {
            if k < 0 || self.v[k as usize] == b'\n' {
                k -= 1;
            } else {
                self.v.push(b'\n');
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
                self.v.push(b'\\');
            }

            let nextc = buf.get(i + 1);
            if buf[i] == b' ' && wrap {
                if !self.begin_line {
                    let last_nonspace = self.v.len();
                    self.v.push(b' ');
                    self.column += 1;
                    self.begin_line = false;
                    self.begin_content = false;
                    while buf.get(i + 1) == Some(&(b' ')) {
                        i += 1;
                    }
                    if !buf.get(i + 1).map_or(false, |&c| isdigit(c)) {
                        self.last_breakable = last_nonspace;
                    }
                }
            } else if buf[i] == b'\n' {
                self.v.push(b'\n');
                self.column = 0;
                self.begin_line = true;
                self.begin_content = true;
                self.last_breakable = 0;
            } else if escaping == Escaping::Literal {
                self.v.push(buf[i]);
                self.column += 1;
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(buf[i]);
            } else {
                self.outc(buf[i], escaping, nextc);
                self.begin_line = false;
                self.begin_content = self.begin_content && isdigit(buf[i]);
            }

            if self.options.width > 0 && self.column > self.options.width && !self.begin_line
                && self.last_breakable > 0
            {
                let remainder = self.v[self.last_breakable + 1..].to_vec();
                self.v.truncate(self.last_breakable);
                self.v.push(b'\n');
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
        let follows_digit = !self.v.is_empty() && isdigit(self.v[self.v.len() - 1]);

        let nextc = nextc.map_or(0, |&c| c);

        let needs_escaping = c < 0x80 && escaping != Escaping::Literal
            && ((escaping == Escaping::Normal
                && (c == b'*' || c == b'_' || c == b'[' || c == b']' || c == b'#' || c == b'<'
                    || c == b'>' || c == b'\\' || c == b'`' || c == b'!'
                    || (c == b'&' && isalpha(nextc))
                    || (c == b'!' && nextc == 0x5b)
                    || (self.begin_content && (c == b'-' || c == b'+' || c == b'=')
                        && !follows_digit)
                    || (self.begin_content && (c == b'.' || c == b')') && follows_digit
                        && (nextc == 0 || isspace(nextc)))))
                || (escaping == Escaping::URL
                    && (c == b'`' || c == b'<' || c == b'>' || isspace(c) || c == b'\\' || c == b')'
                        || c == b'('))
                || (escaping == Escaping::Title
                    && (c == b'`' || c == b'<' || c == b'>' || c == b'"' || c == b'\\')));

        if needs_escaping {
            if isspace(c) {
                write!(self.v, "%{:2x}", c).unwrap();
                self.column += 3;
            } else {
                write!(self.v, "\\{}", c as char).unwrap();
                self.column += 2;
            }
        } else {
            self.v.push(c);
            self.column += 1;
        }
    }

    fn cr(&mut self) {
        self.need_cr = max(self.need_cr, 1);
    }

    fn blankline(&mut self) {
        self.need_cr = max(self.need_cr, 2);
    }

    fn format(&mut self, node: &'a AstNode<'a>) {

        enum Phase { Pre, Post }
        let mut stack = vec![(node, Phase::Pre)];

        while let Some((node, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    if self.format_node(node, true) {
                        stack.push((node, Phase::Post));
                        for ch in node.reverse_children() {
                            stack.push((ch, Phase::Pre));
                        }
                    }
                }
                Phase::Post => {
                    self.format_node(node, false);
                }
            }
        }
    }

    fn get_in_tight_list_item(&self, node: &'a AstNode<'a>) -> bool {
        let tmp = match nodes::containing_block(node) {
            Some(tmp) => tmp,
            None => return false,
        };

        if let NodeValue::Item(..) = tmp.data.borrow().value {
            if let NodeValue::List(ref nl) = tmp.parent().unwrap().data.borrow().value {
                return nl.tight;
            }
            return false;
        }

        let parent = match tmp.parent() {
            Some(parent) => parent,
            None => return false,
        };

        if let NodeValue::Item(..) = parent.data.borrow().value {
            if let NodeValue::List(ref nl) = parent.parent().unwrap().data.borrow().value {
                return nl.tight;
            }
        }

        false
    }

    fn format_node(&mut self, node: &'a AstNode<'a>, entering: bool) -> bool {
        self.node = node;
        let allow_wrap = self.options.width > 0 && !self.options.hardbreaks;

        if !(match node.data.borrow().value {
            NodeValue::Item(..) => true,
            _ => false,
        } && node.previous_sibling().is_none() && entering)
        {
            self.in_tight_list_item = self.get_in_tight_list_item(node);
        }

        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::BlockQuote => if entering {
                write!(self, "> ").unwrap();
                self.begin_content = true;
                write!(self.prefix, "> ").unwrap();
            } else {
                let new_len = self.prefix.len() - 2;
                self.prefix.truncate(new_len);
                self.blankline();
            },
            NodeValue::List(..) => if !entering && match node.next_sibling() {
                Some(next_sibling) => match next_sibling.data.borrow().value {
                    NodeValue::CodeBlock(..) | NodeValue::List(..) => true,
                    _ => false,
                },
                _ => false,
            } {
                self.cr();
                write!(self, "<!-- end list -->").unwrap();
                self.blankline();
            },
            NodeValue::Item(..) => {
                let parent = match node.parent().unwrap().data.borrow().value {
                    NodeValue::List(ref nl) => *nl,
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
                    write!(
                        listmarker,
                        "{}{}{}",
                        list_number,
                        if list_delim == ListDelimType::Paren {
                            ")"
                        } else {
                            "."
                        },
                        if list_number < 10 { "  " } else { " " }
                    ).unwrap();
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
            NodeValue::Heading(ref nch) => if entering {
                for _ in 0..nch.level {
                    write!(self, "#").unwrap();
                }
                write!(self, " ").unwrap();
                self.begin_content = true;
                self.no_linebreaks = true;
            } else {
                self.no_linebreaks = false;
                self.blankline();
            },
            NodeValue::CodeBlock(ref ncb) => if entering {
                let first_in_list_item = node.previous_sibling().is_none() && match node.parent() {
                    Some(parent) => match parent.data.borrow().value {
                        NodeValue::Item(..) => true,
                        _ => false,
                    },
                    _ => false,
                };

                if !first_in_list_item {
                    self.blankline();
                }

                if ncb.info.is_empty()
                    && (ncb.literal.len() > 2 && !isspace(ncb.literal[0])
                        && !(isspace(ncb.literal[ncb.literal.len() - 1])
                            && isspace(ncb.literal[ncb.literal.len() - 2])))
                    && !first_in_list_item
                {
                    write!(self, "    ").unwrap();
                    write!(self.prefix, "    ").unwrap();
                    self.write_all(&ncb.literal).unwrap();
                    let new_len = self.prefix.len() - 4;
                    self.prefix.truncate(new_len);
                } else {
                    let numticks = max(3, longest_backtick_sequence(&ncb.literal) + 1);
                    for _ in 0..numticks {
                        write!(self, "`").unwrap();
                    }
                    if !ncb.info.is_empty() {
                        write!(self, " ").unwrap();
                        self.write_all(&ncb.info).unwrap();
                    }
                    self.cr();
                    self.write_all(&ncb.literal).unwrap();
                    self.cr();
                    for _ in 0..numticks {
                        write!(self, "`").unwrap();
                    }
                }
                self.blankline();
            },
            NodeValue::HtmlBlock(ref nhb) => if entering {
                self.blankline();
                self.write_all(&nhb.literal).unwrap();
                self.blankline();
            },
            NodeValue::ThematicBreak => if entering {
                self.blankline();
                write!(self, "-----").unwrap();
                self.blankline();
            },
            NodeValue::Paragraph => if !entering {
                self.blankline();
            },
            NodeValue::Text(ref literal) => if entering {
                self.output(literal, allow_wrap, Escaping::Normal);
            },
            NodeValue::LineBreak => if entering {
                if !self.options.hardbreaks {
                    write!(self, "  ").unwrap();
                }
                self.cr();
            },
            NodeValue::SoftBreak => if entering {
                if !self.no_linebreaks && self.options.width == 0 && !self.options.hardbreaks {
                    self.cr();
                } else {
                    self.output(&[b' '], allow_wrap, Escaping::Literal);
                }
            },
            NodeValue::Code(ref literal) => if entering {
                let numticks = shortest_unused_sequence(literal, b'`');
                for _ in 0..numticks {
                    write!(self, "`").unwrap();
                }
                if literal.is_empty() || literal[0] == b'`' {
                    write!(self, " ").unwrap();
                }
                self.output(literal, allow_wrap, Escaping::Literal);
                if literal.is_empty() || literal[literal.len() - 1] == b'`' {
                    write!(self, " ").unwrap();
                }
                for _ in 0..numticks {
                    write!(self, "`").unwrap();
                }
            },
            NodeValue::HtmlInline(ref literal) => if entering {
                self.write_all(literal).unwrap();
            },
            NodeValue::Strong => if entering {
                write!(self, "**").unwrap();
            } else {
                write!(self, "**").unwrap();
            },
            NodeValue::Emph => {
                let emph_delim = if match node.parent() {
                    Some(parent) => match parent.data.borrow().value {
                        NodeValue::Emph => true,
                        _ => false,
                    },
                    _ => false,
                } && node.next_sibling().is_none()
                    && node.previous_sibling().is_none()
                {
                    b'_'
                } else {
                    b'*'
                };

                self.write_all(&[emph_delim]).unwrap();
            }
            NodeValue::Strikethrough => if entering {
                write!(self, "~").unwrap();
            } else {
                write!(self, "~").unwrap();
            },
            NodeValue::Superscript => if entering {
                write!(self, "^").unwrap();
            } else {
                write!(self, "^").unwrap();
            },
            NodeValue::Link(ref nl) => if is_autolink(node, nl) {
                if entering {
                    write!(self, "<").unwrap();
                    if nl.url.len() >= 7 && &nl.url[..7] == b"mailto:" {
                        self.write_all(&nl.url[7..]).unwrap();
                    } else {
                        self.write_all(&nl.url).unwrap();
                    }
                    write!(self, ">").unwrap();
                    return false;
                }
            } else if entering {
                write!(self, "[").unwrap();
            } else {
                write!(self, "](").unwrap();
                self.output(&nl.url, false, Escaping::URL);
                if !nl.title.is_empty() {
                    write!(self, " \"").unwrap();
                    self.output(&nl.title, false, Escaping::Title);
                    write!(self, "\"").unwrap();
                }
                write!(self, ")").unwrap();
            },
            NodeValue::Image(ref nl) => if entering {
                write!(self, "![").unwrap();
            } else {
                write!(self, "](").unwrap();
                self.output(&nl.url, false, Escaping::URL);
                if !nl.title.is_empty() {
                    self.output(&[b' ', b'"'], allow_wrap, Escaping::Literal);
                    self.output(&nl.title, false, Escaping::Title);
                    write!(self, "\"").unwrap();
                }
                write!(self, ")").unwrap();
            },
            NodeValue::Table(..) => {
                if entering {
                    self.custom_escape = Some(table_escape);
                } else {
                    self.custom_escape = None;
                }
                self.blankline();
            }
            NodeValue::TableRow(..) => if entering {
                self.cr();
                write!(self, "|").unwrap();
            },
            NodeValue::TableCell => if entering {
                write!(self, " ").unwrap();
            } else {
                write!(self, " |").unwrap();

                let row = &node.parent().unwrap().data.borrow().value;
                let in_header = match *row {
                    NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                if in_header && node.next_sibling().is_none() {
                    let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
                    let alignments = match *table {
                        NodeValue::Table(ref alignments) => alignments,
                        _ => panic!(),
                    };

                    self.cr();
                    write!(self, "|").unwrap();
                    for a in alignments {
                        write!(
                            self,
                            " {} |",
                            match *a {
                                TableAlignment::Left => ":--",
                                TableAlignment::Center => ":-:",
                                TableAlignment::Right => "--:",
                                TableAlignment::None => "---",
                            }
                        ).unwrap();
                    }
                    self.cr();
                }
            },
            NodeValue::FootnoteDefinition(_) => if entering {
                self.footnote_ix += 1;
                let footnote_ix = self.footnote_ix;
                write!(self, "[^{}]:\n", footnote_ix).unwrap();
                write!(self.prefix, "    ").unwrap();
            } else {
                let new_len = self.prefix.len() - 4;
                self.prefix.truncate(new_len);
            },
            NodeValue::FootnoteReference(ref r) => if entering {
                self.write_all(b"[^").unwrap();
                self.write_all(r).unwrap();
                self.write_all(b"]").unwrap();
            },
        };
        true
    }
}

fn longest_backtick_sequence(literal: &[u8]) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for c in literal {
        if *c == b'`' {
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

fn shortest_unused_sequence(literal: &[u8], f: u8) -> usize {
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
        used >>= 1;
        i += 1;
    }
    i
}

fn is_autolink<'a>(node: &'a AstNode<'a>, nl: &NodeLink) -> bool {
    if nl.url.is_empty() || scanners::scheme(&nl.url).is_none() {
        return false;
    }

    if !nl.title.is_empty() {
        return false;
    }

    let link_text = match node.first_child() {
        None => return false,
        Some(child) => match child.data.borrow().value {
            NodeValue::Text(ref t) => t.clone(),
            _ => return false,
        },
    };

    let mut real_url: &[u8] = &nl.url;
    if real_url.len() >=7 && &real_url[..7] == b"mailto:" {
        real_url = &real_url[7..];
    }

    real_url == &*link_text
}

fn table_escape<'a>(node: &'a AstNode<'a>, c: u8) -> bool {
    match node.data.borrow().value {
        NodeValue::Table(..) | NodeValue::TableRow(..) | NodeValue::TableCell => false,
        _ => c == b'|',
    }
}
