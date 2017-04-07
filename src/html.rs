use std;
use std::io::Write;
use std::iter::FromIterator;
use std::collections::BTreeMap;

use arena_tree::Node;
use node::{TableAlignment, NodeValue, AstCell, ListType};
use parser::ComrakOptions;
use ctype::isspace;

pub fn format_document<'a>(root: &'a Node<'a, AstCell>, options: &ComrakOptions) -> String {
    let mut f = HtmlFormatter::new(options);
    f.format(root, false);
    String::from_utf8(f.v).unwrap()
}

struct HtmlFormatter<'o> {
    v: Vec<u8>,
    options: &'o ComrakOptions,
}

impl<'o> Write for HtmlFormatter<'o> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.v.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.v.flush()
    }
}

lazy_static! {
    static ref HREF_SAFE: [bool; 256] = {
        let mut a = [false; 256];
        for &c in concat!("-_.+!*'(),%#@?=;:/,+&$abcdefghijkl",
        "mnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789").as_bytes() {
            a[c as usize] = true;
        }
        a
    };

    static ref TAGFILTER_BLACKLIST: [&'static str; 9] =
        ["title", "textarea", "style", "xmp", "iframe",
         "noembed", "noframes", "script", "plaintext"];
}

fn tagfilter(literal: &str) -> bool {
    if literal.len() < 3 || literal.as_bytes()[0] != '<' as u8 {
        return false;
    }

    let mut i = 1;
    if literal.as_bytes()[i] == '/' as u8 {
        i += 1;
    }

    for t in TAGFILTER_BLACKLIST.iter() {
        let j = i + t.len();
        if j < literal.len() && &&literal[i..j] == t {
            return isspace(&literal.as_bytes()[j]) || literal.as_bytes()[j] == '>' as u8 ||
                   (literal.as_bytes()[j] == '/' as u8 && literal.len() >= j + 2 &&
                    literal.as_bytes()[j + 1] == '>' as u8);
        }
    }

    false
}

fn tagfilter_block<W: Write>(input: &str, mut w: &mut W) {
    let mut i = 0;
    let len = input.len();

    while i < len {
        if tagfilter(&input[i..]) {
            write!(w, "&lt;").unwrap();
        } else {
            w.write_all(&[input.as_bytes()[i]]).unwrap();
        }

        i += 1;
    }
}

impl<'o> HtmlFormatter<'o> {
    fn new(options: &'o ComrakOptions) -> Self {
        HtmlFormatter {
            v: vec![],
            options: options,
        }
    }

    fn cr(&mut self) {
        let l = self.v.len();
        if l > 0 && self.v[l - 1] != '\n' as u8 {
            self.v.push('\n' as u8);
        }
    }

    fn escape(&mut self, buffer: &str) {
        lazy_static! {
            static ref ESCAPE_TABLE: BTreeMap<u8, &'static str> = BTreeMap::from_iter(
                vec![('"' as u8, "&quot;"),
                     ('&' as u8, "&amp;"),
                     // Secure mode only:
                     // ('\'', "&#39;"),
                     // ('/', "&#47;"),
                     ('<' as u8, "&lt;"),
                     ('>' as u8, "&gt;"),
            ]);
        }

        for c in buffer.as_bytes() {
            match ESCAPE_TABLE.get(c) {
                Some(s) => {
                    self.write_all(s.as_bytes()).unwrap();
                }
                None => {
                    self.write_all(&[*c]).unwrap();
                }
            };
        }
    }

    fn escape_href(&mut self, buffer: &str) {
        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && HREF_SAFE[src[i] as usize] {
                i += 1;
            }

            if i > org {
                self.v.extend_from_slice(&src[org..i]);
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '&' => write!(self, "&amp;").unwrap(),
                '\'' => write!(self, "&#x27;").unwrap(),
                _ => write!(self, "%{:02X}", src[i]).unwrap(),
            }

            i += 1;
        }
    }

    fn format_children<'a>(&mut self, node: &'a Node<'a, AstCell>, plain: bool) {
        for n in node.children() {
            self.format(n, plain);
        }
    }

    fn format<'a>(&mut self, node: &'a Node<'a, AstCell>, plain: bool) {
        if plain {
            match &node.data.borrow().value {
                &NodeValue::Text(ref literal) |
                &NodeValue::Code(ref literal) |
                &NodeValue::HtmlInline(ref literal) => self.escape(literal),
                &NodeValue::LineBreak |
                &NodeValue::SoftBreak => self.v.push(' ' as u8),
                _ => (),
            }
            self.format_children(node, true);
        } else {
            let new_plain = self.format_node(node, true);
            self.format_children(node, new_plain);
            self.format_node(node, false);
        }
    }

    fn format_node<'a>(&mut self, node: &'a Node<'a, AstCell>, entering: bool) -> bool {
        match &node.data.borrow().value {
            &NodeValue::Document => (),
            &NodeValue::BlockQuote => {
                if entering {
                    self.cr();
                    write!(self, "<blockquote>\n").unwrap();
                } else {
                    self.cr();
                    write!(self, "</blockquote>\n").unwrap();
                }
            }
            &NodeValue::List(ref nl) => {
                if entering {
                    self.cr();
                    if nl.list_type == ListType::Bullet {
                        write!(self, "<ul>\n").unwrap();
                    } else if nl.start == 1 {
                        write!(self, "<ol>\n").unwrap();
                    } else {
                        write!(self, "<ol start=\"{}\">\n", nl.start).unwrap();
                    }
                } else {
                    if nl.list_type == ListType::Bullet {
                        write!(self, "</ul>\n").unwrap();
                    } else {
                        write!(self, "</ol>\n").unwrap();
                    }
                }
            }
            &NodeValue::Item(..) => {
                if entering {
                    self.cr();
                    write!(self, "<li>").unwrap();
                } else {
                    write!(self, "</li>\n").unwrap();
                }
            }
            &NodeValue::Heading(ref nch) => {
                if entering {
                    self.cr();
                    write!(self, "<h{}>", nch.level).unwrap();
                } else {
                    write!(self, "</h{}>\n", nch.level).unwrap();
                }
            }
            &NodeValue::CodeBlock(ref ncb) => {
                if entering {
                    self.cr();

                    if ncb.info.len() == 0 {
                        write!(self, "<pre><code>").unwrap();
                    } else {
                        let mut first_tag = 0;
                        while first_tag < ncb.info.len() &&
                              !isspace(&ncb.info.as_bytes()[first_tag]) {
                            first_tag += 1;
                        }

                        if self.options.github_pre_lang {
                            write!(self, "<pre lang=\"").unwrap();
                            self.escape(&ncb.info[..first_tag]);
                            write!(self, "\"><code>").unwrap();
                        } else {
                            write!(self, "<pre><code class=\"language-").unwrap();
                            self.escape(&ncb.info[..first_tag]);
                            write!(self, "\">").unwrap();
                        }
                    }
                    self.escape(&ncb.literal);
                    write!(self, "</code></pre>\n").unwrap();
                }
            }
            &NodeValue::HtmlBlock(ref nhb) => {
                if entering {
                    self.cr();
                    if self.options.ext_tagfilter {
                        tagfilter_block(&nhb.literal, self);
                    } else {
                        self.write_all(nhb.literal.as_bytes()).unwrap();
                    }
                    self.cr();
                }
            }
            &NodeValue::CustomBlock => {
                assert!(false)
                // TODO
            }
            &NodeValue::ThematicBreak => {
                if entering {
                    self.cr();
                    write!(self, "<hr />\n").unwrap();
                }
            }
            &NodeValue::Paragraph => {
                let tight = match node.parent()
                    .and_then(|ref n| n.parent())
                    .map(|ref n| n.data.borrow().value.clone()) {
                    Some(NodeValue::List(ref nl)) => nl.tight,
                    _ => false,
                };

                if entering {
                    if !tight {
                        self.cr();
                        write!(self, "<p>").unwrap();
                    }
                } else {
                    if !tight {
                        write!(self, "</p>\n").unwrap();
                    }
                }
            }
            &NodeValue::Text(ref literal) => {
                if entering {
                    self.escape(literal);
                }
            }
            &NodeValue::LineBreak => {
                if entering {
                    write!(self, "<br />\n").unwrap();
                }
            }
            &NodeValue::SoftBreak => {
                if entering {
                    if self.options.hardbreaks {
                        write!(self, "<br />\n").unwrap();
                    } else {
                        write!(self, "\n").unwrap();
                    }
                }
            }
            &NodeValue::Code(ref literal) => {
                if entering {
                    write!(self, "<code>").unwrap();
                    self.escape(literal);
                    write!(self, "</code>").unwrap();
                }
            }
            &NodeValue::HtmlInline(ref literal) => {
                if entering {
                    if self.options.ext_tagfilter && tagfilter(literal) {
                        write!(self, "&lt;{}", &literal[1..]).unwrap();
                    } else {
                        write!(self, "{}", literal).unwrap();
                    }
                }
            }
            &NodeValue::CustomInline => {
                assert!(false)
                // TODO
            }
            &NodeValue::Strong => {
                if entering {
                    write!(self, "<strong>").unwrap();
                } else {
                    write!(self, "</strong>").unwrap();
                }
            }
            &NodeValue::Emph => {
                if entering {
                    write!(self, "<em>").unwrap();
                } else {
                    write!(self, "</em>").unwrap();
                }
            }
            &NodeValue::Strikethrough => {
                if entering {
                    write!(self, "<del>").unwrap();
                } else {
                    write!(self, "</del>").unwrap();
                }
            }
            &NodeValue::Link(ref nl) => {
                if entering {
                    write!(self, "<a href=\"").unwrap();
                    self.escape_href(&nl.url);
                    if nl.title.len() > 0 {
                        write!(self, "\" title=\"").unwrap();
                        self.escape(&nl.title);
                    }
                    write!(self, "\">").unwrap();
                } else {
                    write!(self, "</a>").unwrap();
                }
            }
            &NodeValue::Image(ref nl) => {
                if entering {
                    write!(self, "<img src=\"").unwrap();
                    self.escape_href(&nl.url);
                    write!(self, "\" alt=\"").unwrap();
                    return true;
                } else {
                    if nl.title.len() > 0 {
                        write!(self, "\" title=\"").unwrap();
                        self.escape(&nl.title);
                    }
                    write!(self, "\" />").unwrap();
                }
            }
            &NodeValue::Table(..) => {
                if entering {
                    self.cr();
                    write!(self, "<table>\n").unwrap();
                } else {
                    if !node.last_child().unwrap().same_node(node.first_child().unwrap()) {
                        write!(self, "</tbody>").unwrap();
                    }
                    write!(self, "</table>\n").unwrap();
                }
            }
            &NodeValue::TableRow(header) => {
                if entering {
                    self.cr();
                    if header {
                        write!(self, "<thead>").unwrap();
                        self.cr();
                    }
                    write!(self, "<tr>").unwrap();
                } else {
                    self.cr();
                    write!(self, "</tr>").unwrap();
                    if header {
                        self.cr();
                        write!(self, "</thead>").unwrap();
                        self.cr();
                        write!(self, "<tbody>").unwrap();
                    }
                }
            }
            &NodeValue::TableCell => {
                let ref row = node.parent().unwrap().data.borrow().value;
                let in_header = match row {
                    &NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                let ref table = node.parent().unwrap().parent().unwrap().data.borrow().value;
                let alignments = match table {
                    &NodeValue::Table(ref alignments) => alignments,
                    _ => panic!(),
                };

                if entering {
                    self.cr();
                    if in_header {
                        write!(self, "<th").unwrap();
                    } else {
                        write!(self, "<td").unwrap();
                    }

                    let mut start = node.parent().unwrap().first_child().unwrap();
                    let mut i = 0;
                    while !start.same_node(node) {
                        i += 1;
                        start = start.next_sibling().unwrap();
                    }

                    match alignments[i] {
                        TableAlignment::Left => write!(self, " align=\"left\"").unwrap(),
                        TableAlignment::Right => write!(self, " align=\"right\"").unwrap(),
                        TableAlignment::Center => write!(self, " align=\"center\"").unwrap(),
                        TableAlignment::None => (),
                    }

                    write!(self, ">").unwrap();
                } else {
                    if in_header {
                        write!(self, "</th>").unwrap();
                    } else {
                        write!(self, "</td>").unwrap();
                    }
                }
            }
        }
        false
    }
}
