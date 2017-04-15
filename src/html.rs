use ctype::isspace;
use nodes::{TableAlignment, NodeValue, ListType, AstNode};
use parser::ComrakOptions;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(root: &'a AstNode<'a>, options: &ComrakOptions) -> String {
    let mut f = HtmlFormatter::new(options);
    f.format(root, false);
    f.s
}

struct HtmlFormatter<'o> {
    s: String,
    options: &'o ComrakOptions,
}

fn tagfilter(literal: &str) -> bool {
    lazy_static! {
        static ref TAGFILTER_BLACKLIST: [&'static str; 9] =
            ["title", "textarea", "style", "xmp", "iframe",
             "noembed", "noframes", "script", "plaintext"];
    }

    if literal.len() < 3 || literal.as_bytes()[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if literal.as_bytes()[i] == b'/' {
        i += 1;
    }

    for t in TAGFILTER_BLACKLIST.iter() {
        if literal[i..].starts_with(t) {
            let j = i + t.len();
            return isspace(literal.as_bytes()[j]) || literal.as_bytes()[j] == b'>' ||
                   (literal.as_bytes()[j] == b'/' && literal.len() >= j + 2 &&
                    literal.as_bytes()[j + 1] == b'>');
        }
    }

    false
}

fn tagfilter_block(input: &str, mut o: &mut String) {
    let src = input.as_bytes();
    let size = src.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && src[i] != b'<' {
            i += 1;
        }

        if i > org {
            *o += &input[org..i];
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            *o += "&lt;";
        } else {
            o.push('<');
        }

        i += 1;
    }
}

impl<'o> HtmlFormatter<'o> {
    fn new(options: &'o ComrakOptions) -> Self {
        HtmlFormatter {
            s: String::with_capacity(1024),
            options: options,
        }
    }

    fn cr(&mut self) {
        let l = self.s.len();
        if l > 0 && self.s.as_bytes()[l - 1] != b'\n' {
            self.s += "\n";
        }
    }

    fn escape(&mut self, buffer: &str) {
        lazy_static! {
            static ref NEEDS_ESCAPED: [bool; 256] = {
                let mut sc = [false; 256];
                for &c in &['"', '&', '<', '>'] {
                    sc[c as usize] = true;
                }
                sc
            };
        }

        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && !NEEDS_ESCAPED[src[i] as usize] {
                i += 1;
            }

            if i > org {
                self.s += &buffer[org..i];
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '"' => self.s += "&quot;",
                '&' => self.s += "&amp;",
                '<' => self.s += "&lt;",
                '>' => self.s += "&gt;",
                _ => unreachable!(),
            }

            i += 1;
        }
    }

    fn escape_href(&mut self, buffer: &str) {
        lazy_static! {
            static ref HREF_SAFE: [bool; 256] = {
                let mut a = [false; 256];
                for &c in concat!("-_.+!*'(),%#@?=;:/,+&$abcdefghijkl",
                "mnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789").as_bytes() {
                    a[c as usize] = true;
                }
                a
            };
        }

        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && HREF_SAFE[src[i] as usize] {
                i += 1;
            }

            if i > org {
                self.s += &buffer[org..i];
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '&' => self.s += "&amp;",
                '\'' => self.s += "&#x27;",
                _ => self.s += &format!("%{:02X}", src[i]),
            }

            i += 1;
        }
    }

    fn format_children<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) {
        for n in node.children() {
            self.format(n, plain);
        }
    }

    fn format<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) {
        if plain {
            match node.data.borrow().value {
                NodeValue::Text(ref literal) |
                NodeValue::Code(ref literal) |
                NodeValue::HtmlInline(ref literal) => self.escape(literal),
                NodeValue::LineBreak | NodeValue::SoftBreak => self.s.push(' '),
                _ => (),
            }
            self.format_children(node, true);
        } else {
            let new_plain = self.format_node(node, true);
            self.format_children(node, new_plain);
            self.format_node(node, false);
        }
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> bool {
        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::BlockQuote => {
                if entering {
                    self.cr();
                    self.s += "<blockquote>\n";
                } else {
                    self.cr();
                    self.s += "</blockquote>\n";
                }
            }
            NodeValue::List(ref nl) => {
                if entering {
                    self.cr();
                    if nl.list_type == ListType::Bullet {
                        self.s += "<ul>\n";
                    } else if nl.start == 1 {
                        self.s += "<ol>\n";
                    } else {
                        self.s += &format!("<ol start=\"{}\">\n", nl.start);
                    }
                } else if nl.list_type == ListType::Bullet {
                    self.s += "</ul>\n";
                } else {
                    self.s += "</ol>\n";
                }
            }
            NodeValue::Item(..) => {
                if entering {
                    self.cr();
                    self.s += "<li>";
                } else {
                    self.s += "</li>\n";
                }
            }
            NodeValue::Heading(ref nch) => {
                if entering {
                    self.cr();
                    self.s += &format!("<h{}>", nch.level);
                } else {
                    self.s += &format!("</h{}>\n", nch.level);
                }
            }
            NodeValue::CodeBlock(ref ncb) => {
                if entering {
                    self.cr();

                    if ncb.info.is_empty() {
                        self.s += "<pre><code>";
                    } else {
                        let mut first_tag = 0;
                        while first_tag < ncb.info.len() &&
                              !isspace(ncb.info.as_bytes()[first_tag]) {
                            first_tag += 1;
                        }

                        if self.options.github_pre_lang {
                            self.s += "<pre lang=\"";
                            self.escape(&ncb.info[..first_tag]);
                            self.s += "\"><code>";
                        } else {
                            self.s += "<pre><code class=\"language-";
                            self.escape(&ncb.info[..first_tag]);
                            self.s += "\">";
                        }
                    }
                    self.escape(&ncb.literal);
                    self.s += "</code></pre>\n";
                }
            }
            NodeValue::HtmlBlock(ref nhb) => {
                if entering {
                    self.cr();
                    if self.options.ext_tagfilter {
                        tagfilter_block(&nhb.literal, &mut self.s);
                    } else {
                        self.s += &nhb.literal;
                    }
                    self.cr();
                }
            }
            NodeValue::ThematicBreak => {
                if entering {
                    self.cr();
                    self.s += "<hr />\n";
                }
            }
            NodeValue::Paragraph => {
                let tight = match node.parent()
                          .and_then(|n| n.parent())
                          .map(|n| n.data.borrow().value.clone()) {
                    Some(NodeValue::List(nl)) => nl.tight,
                    _ => false,
                };

                if entering {
                    if !tight {
                        self.cr();
                        self.s += "<p>";
                    }
                } else if !tight {
                    self.s += "</p>\n";
                }
            }
            NodeValue::Text(ref literal) => {
                if entering {
                    self.escape(literal);
                }
            }
            NodeValue::LineBreak => {
                if entering {
                    self.s += "<br />\n";
                }
            }
            NodeValue::SoftBreak => {
                if entering {
                    if self.options.hardbreaks {
                        self.s += "<br />\n";
                    } else {
                        self.s += "\n";
                    }
                }
            }
            NodeValue::Code(ref literal) => {
                if entering {
                    self.s += "<code>";
                    self.escape(literal);
                    self.s += "</code>";
                }
            }
            NodeValue::HtmlInline(ref literal) => {
                if entering {
                    if self.options.ext_tagfilter && tagfilter(literal) {
                        self.s += "&lt;";
                        self.s += &literal[1..];
                    } else {
                        self.s += literal;
                    }
                }
            }
            NodeValue::Strong => {
                if entering {
                    self.s += "<strong>";
                } else {
                    self.s += "</strong>";
                }
            }
            NodeValue::Emph => {
                if entering {
                    self.s += "<em>";
                } else {
                    self.s += "</em>";
                }
            }
            NodeValue::Strikethrough => {
                if entering {
                    self.s += "<del>";
                } else {
                    self.s += "</del>";
                }
            }
            NodeValue::Superscript => {
                if entering {
                    self.s += "<sup>";
                } else {
                    self.s += "</sup>";
                }
            }
            NodeValue::Link(ref nl) => {
                if entering {
                    self.s += "<a href=\"";
                    self.escape_href(&nl.url);
                    if !nl.title.is_empty() {
                        self.s += "\" title=\"";
                        self.escape(&nl.title);
                    }
                    self.s += "\">";
                } else {
                    self.s += "</a>";
                }
            }
            NodeValue::Image(ref nl) => {
                if entering {
                    self.s += "<img src=\"";
                    self.escape_href(&nl.url);
                    self.s += "\" alt=\"";
                    return true;
                } else {
                    if !nl.title.is_empty() {
                        self.s += "\" title=\"";
                        self.escape(&nl.title);
                    }
                    self.s += "\" />";
                }
            }
            NodeValue::Table(..) => {
                if entering {
                    self.cr();
                    self.s += "<table>\n";
                } else {
                    if !node.last_child()
                            .unwrap()
                            .same_node(node.first_child().unwrap()) {
                        self.s += "</tbody>";
                    }
                    self.s += "</table>\n";
                }
            }
            NodeValue::TableRow(header) => {
                if entering {
                    self.cr();
                    if header {
                        self.s += "<thead>";
                        self.cr();
                    }
                    self.s += "<tr>";
                } else {
                    self.cr();
                    self.s += "</tr>";
                    if header {
                        self.cr();
                        self.s += "</thead>";
                        self.cr();
                        self.s += "<tbody>";
                    }
                }
            }
            NodeValue::TableCell => {
                let row = &node.parent().unwrap().data.borrow().value;
                let in_header = match *row {
                    NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                let table = &node.parent()
                                 .unwrap()
                                 .parent()
                                 .unwrap()
                                 .data
                                 .borrow()
                                 .value;
                let alignments = match *table {
                    NodeValue::Table(ref alignments) => alignments,
                    _ => panic!(),
                };

                if entering {
                    self.cr();
                    if in_header {
                        self.s += "<th";
                    } else {
                        self.s += "<td";
                    }

                    let mut start = node.parent().unwrap().first_child().unwrap();
                    let mut i = 0;
                    while !start.same_node(node) {
                        i += 1;
                        start = start.next_sibling().unwrap();
                    }

                    match alignments[i] {
                        TableAlignment::Left => self.s += " align=\"left\"",
                        TableAlignment::Right => self.s += " align=\"right\"",
                        TableAlignment::Center => self.s += " align=\"center\"",
                        TableAlignment::None => (),
                    }

                    self.s += ">";
                } else if in_header {
                    self.s += "</th>";
                } else {
                    self.s += "</td>";
                }
            }
        }
        false
    }
}
