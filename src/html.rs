use std::io::Write;

use ::{NodeVal, Node, N};

pub fn format_document<'a>(root: &'a Node<'a, N>) -> String {
    let mut res = vec![];
    format_node(&mut res, root);
    String::from_utf8(res).unwrap()
}

fn format_node<'a>(w: &mut Write, node: &'a Node<'a, N>) {
    match &node.data.borrow().typ {
        &NodeVal::Document => {
            for n in node.children() {
                format_node(w, n);
            }
        },
        &NodeVal::BlockQuote => {
            write!(w, "<blockquote>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</blockquote>\n").unwrap()
        },
        &NodeVal::List => {
            // TODO
        },
        &NodeVal::Item => {
            // TODO
        },
        &NodeVal::Heading(ref nch) => {
            write!(w, "<h{}>", nch.level).unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</h{}>\n", nch.level).unwrap();
        },
        &NodeVal::CodeBlock(..) => {
            // TODO
        },
        &NodeVal::HtmlBlock(..) => {
            // TODO
        },
        &NodeVal::CustomBlock => {
            // TODO
        },
        &NodeVal::ThematicBreak => {
            // TODO
        },
        &NodeVal::Paragraph => {
            // TODO: tight list setting
            write!(w, "<p>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</p>\n").unwrap();
        },
        &NodeVal::Text(ref literal) => {
            // TODO: escape HTML
            write!(w, "{}", literal).unwrap();
        },
        &NodeVal::LineBreak => {
            write!(w, "<br />\n").unwrap();
        },
        &NodeVal::SoftBreak => {
            // TODO
            write!(w, "<br />\n").unwrap();
        },
        &NodeVal::Code => {
            // TODO
        },
        &NodeVal::HtmlInline => {
            // TODO
        },
        &NodeVal::CustomInline => {
            // TODO
        },
        &NodeVal::Strong => {
            write!(w, "<strong>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</strong>").unwrap();
        },
        &NodeVal::Emph => {
            write!(w, "<em>").unwrap();
            for n in node.children() {
                format_node(w, n);
            }
            write!(w, "</em>").unwrap();
        },
        &NodeVal::Link => {
            // TODO
        },
        &NodeVal::Image => {
            // TODO
        },
    }
}

