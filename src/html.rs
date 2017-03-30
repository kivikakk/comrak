use std::io::Write;

use ::{NodeValue, Node, AstCell, ListType};

pub fn format_document<'a>(root: &'a Node<'a, AstCell>) -> String {
    let mut res = vec![];
    format_node(&mut res, root);
    String::from_utf8(res).unwrap()
}

struct HtmlFormatter<'a> {
    v: Vec<u8>,
    w: &'a mut Write,
}

fn format_node<'a>(w: &mut Write, node: &'a Node<'a, AstCell>) {
    match &node.data.borrow().value {
        &NodeValue::Document => {
            for n in node.children() {
                format_node(w, n);
            }
        }
        &NodeValue::BlockQuote => {
            cr();
            write!(w, "<blockquote>\n").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            cr();
            write!(w, "</blockquote>\n").unwrap()
        }
        &NodeValue::List(ref nl) => {
            cr();
            if nl.list_type == ListType::Bullet {
                write!(w, "<ul>\n").unwrap();
            } else if nl.start == 1 {
                write!(w, "<ol>\n").unwrap();
            } else {
                write!(w, "<ol start=\"{}\">\n", nl.start).unwrap();
            }

            for n in node.children() {
                format_node(w, n);
            }

            if nl.list_type == ListType::Bullet {
                write!(w, "</ul>\n").unwrap();
            } else {
                write!(w, "</ol>\n").unwrap();
            }
        }
        &NodeValue::Item(..) => {
            cr();
            write!(w, "<li>");
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</li>\n");
        }
        &NodeValue::Heading(ref nch) => {
            cr();
            write!(w, "<h{}>", nch.level).unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</h{}>\n", nch.level).unwrap();
        }
        &NodeValue::CodeBlock(ref ncb) => {
            cr();
            write!(w, "<pre><code").unwrap();
            if ncb.info != "" {
                write!(w, " class=\"language-{}\"", ncb.info).unwrap();
            }
            write!(w,
                   ">{}</code></pre>\n",
                   String::from_utf8(ncb.literal.clone()).unwrap())
                .unwrap();
        }
        &NodeValue::HtmlBlock(..) => {
            assert!(false)
            // TODO
        }
        &NodeValue::CustomBlock => {
            assert!(false)
            // TODO
        }
        &NodeValue::ThematicBreak => {
            assert!(false)
            // TODO
        }
        &NodeValue::Paragraph => {
            let tight = match node.parent().and_then(|ref n| n.parent()).map(|ref n| n.data.borrow().value.clone()) {
                Some(NodeValue::List(ref nl)) => nl.tight,
                _ => false,
            };

            if !tight {
                cr();
                write!(w, "<p>").unwrap();
            }
            for n in node.children() {
                format_node(w, n);
            }
            if !tight {
                write!(w, "</p>\n").unwrap();
            }
        }
        &NodeValue::Text(ref literal) => {
            // TODO: escape HTML
            write!(w, "{}", String::from_utf8(literal.clone()).unwrap()).unwrap();
        }
        &NodeValue::LineBreak => {
            write!(w, "<br />\n").unwrap();
        }
        &NodeValue::SoftBreak => {
            // TODO: if HARDBREAKS option set.
            write!(w, "\n").unwrap();
        }
        &NodeValue::Code => {
            assert!(false)
            // TODO
        }
        &NodeValue::HtmlInline => {
            assert!(false)
            // TODO
        }
        &NodeValue::CustomInline => {
            assert!(false)
            // TODO
        }
        &NodeValue::Strong => {
            write!(w, "<strong>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</strong>").unwrap();
        }
        &NodeValue::Emph => {
            write!(w, "<em>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</em>").unwrap();
        }
        &NodeValue::Link => {
            assert!(false)
            // TODO
        }
        &NodeValue::Image => {
            assert!(false)
            // TODO
        }
    }
}

fn cr() {}
