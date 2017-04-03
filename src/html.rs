use std::io::Write;
use std::iter::FromIterator;
use std::collections::BTreeMap;

use ::{NodeValue, Node, AstCell, ListType, std};

pub fn format_document<'a>(root: &'a Node<'a, AstCell>) -> String {
    let mut f = HtmlFormatter::new();
    f.format(root);
    String::from_utf8(f.v).unwrap()
}

struct HtmlFormatter {
    v: Vec<u8>,
}

impl Write for HtmlFormatter {
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
}

impl HtmlFormatter {
    fn new() -> Self {
        HtmlFormatter { v: vec![] }
    }

    fn cr(&mut self) {
        let l = self.v.len();
        if l > 0 && self.v[l - 1] != '\n' as u8 {
            self.v.push('\n' as u8);
        }
    }

    fn escape(&mut self, buffer: &[char]) {
        lazy_static! {
            static ref ESCAPE_TABLE: BTreeMap<char, &'static str> = BTreeMap::from_iter(
                vec![('"', "&quot;"),
                     ('&', "&amp;"),
                     // Secure mode only:
                     // ('\'', "&#39;"),
                     // ('/', "&#47;"),
                     ('<', "&lt;"),
                     ('>', "&gt;"),
            ]);
        }

        for c in buffer {
            match ESCAPE_TABLE.get(c) {
                Some(s) => {
                    self.write(s.as_bytes()).unwrap();
                }
                None => {
                    write!(self, "{}", c).unwrap();
                }
            };
        }
    }

    fn escape_href(&mut self, buffer: &[char]) {
        let s = buffer.into_iter().collect::<String>();
        let src = s.as_bytes();
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
                _ => write!(self, "&#x{:x};", src[i]).unwrap(),
            }

            i += 1;
        }
    }

    fn format_children<'a>(&mut self, node: &'a Node<'a, AstCell>) {
        for n in node.children() {
            self.format(n);
        }
    }

    fn format<'a>(&mut self, node: &'a Node<'a, AstCell>) {
        match &node.data.borrow().value {
            &NodeValue::Document => {
                self.format_children(node);
            }
            &NodeValue::BlockQuote => {
                self.cr();
                write!(self, "<blockquote>\n").unwrap();
                self.format_children(node);
                self.cr();
                write!(self, "</blockquote>\n").unwrap()
            }
            &NodeValue::List(ref nl) => {
                self.cr();
                if nl.list_type == ListType::Bullet {
                    write!(self, "<ul>\n").unwrap();
                } else if nl.start == 1 {
                    write!(self, "<ol>\n").unwrap();
                } else {
                    write!(self, "<ol start=\"{}\">\n", nl.start).unwrap();
                }

                self.format_children(node);

                if nl.list_type == ListType::Bullet {
                    write!(self, "</ul>\n").unwrap();
                } else {
                    write!(self, "</ol>\n").unwrap();
                }
            }
            &NodeValue::Item(..) => {
                self.cr();
                write!(self, "<li>").unwrap();
                self.format_children(node);
                write!(self, "</li>\n").unwrap();
            }
            &NodeValue::Heading(ref nch) => {
                self.cr();
                write!(self, "<h{}>", nch.level).unwrap();
                self.format_children(node);
                write!(self, "</h{}>\n", nch.level).unwrap();
            }
            &NodeValue::CodeBlock(ref ncb) => {
                self.cr();
                write!(self, "<pre><code").unwrap();
                if ncb.info.len() > 0 {
                    write!(self, " class=\"language-").unwrap();
                    self.escape(&ncb.info);
                    write!(self, "\"").unwrap();
                }
                write!(self, ">").unwrap();
                self.escape(&ncb.literal);
                write!(self, "</code></pre>\n").unwrap();
            }
            &NodeValue::HtmlBlock(ref nhb) => {
                self.cr();
                self.write(nhb.literal.iter().collect::<String>().as_bytes()).unwrap();
                self.cr();
            }
            &NodeValue::CustomBlock => {
                assert!(false)
                // TODO
            }
            &NodeValue::ThematicBreak => {
                self.cr();
                write!(self, "<hr />\n").unwrap();
            }
            &NodeValue::Paragraph => {
                let tight = match node.parent()
                    .and_then(|ref n| n.parent())
                    .map(|ref n| n.data.borrow().value.clone()) {
                    Some(NodeValue::List(ref nl)) => nl.tight,
                    _ => false,
                };

                if !tight {
                    self.cr();
                    write!(self, "<p>").unwrap();
                }
                self.format_children(node);
                if !tight {
                    write!(self, "</p>\n").unwrap();
                }
            }
            &NodeValue::Text(ref literal) => {
                self.escape(literal);
            }
            &NodeValue::LineBreak => {
                write!(self, "<br />\n").unwrap();
            }
            &NodeValue::SoftBreak => {
                // TODO: if HARDBREAKS option set.
                write!(self, "\n").unwrap();
            }
            &NodeValue::Code(ref literal) => {
                write!(self, "<code>").unwrap();
                self.escape(literal);
                write!(self, "</code>").unwrap();
            }
            &NodeValue::HtmlInline(ref literal) => {
                write!(self, "{}", literal.into_iter().collect::<String>()).unwrap();
            }
            &NodeValue::CustomInline => {
                assert!(false)
                // TODO
            }
            &NodeValue::Strong => {
                write!(self, "<strong>").unwrap();
                self.format_children(node);
                write!(self, "</strong>").unwrap();
            }
            &NodeValue::Emph => {
                write!(self, "<em>").unwrap();
                self.format_children(node);
                write!(self, "</em>").unwrap();
            }
            &NodeValue::Link(ref nl) => {
                write!(self, "<a href=\"").unwrap();
                self.escape_href(&nl.url);
                if nl.title.len() > 0 {
                    write!(self, "\" title=\"").unwrap();
                    self.escape(&nl.title);
                }
                write!(self, "\">").unwrap();
                self.format_children(node);
                write!(self, "</a>").unwrap();
            }
            &NodeValue::Image(..) => {
                assert!(false)
                // TODO
            }
        }
    }
}
