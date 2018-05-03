use ctype::isspace;
use nodes::{AstNode, ListType, NodeValue, TableAlignment};
use parser::ComrakOptions;
use regex::Regex;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashSet;
use std::io::{self, Write};
use std::str;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut Write,
) -> io::Result<()> {
    let mut writer = WriteWithLast {
        output: output,
        last_was_lf: Cell::new(true),
    };
    let mut f = HtmlFormatter::new(options, &mut writer);
    try!(f.format(root, false));
    if f.footnote_ix > 0 {
        try!(f.output.write_all(b"</ol>\n</section>\n"));
    }
    Ok(())
}

pub struct WriteWithLast<'w> {
    output: &'w mut Write,
    pub last_was_lf: Cell<bool>,
}

impl<'w> Write for WriteWithLast<'w> {
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let l = buf.len();
        if l > 0 {
            self.last_was_lf.set(buf[l - 1] == 10);
        }
        self.output.write(buf)
    }
}

struct HtmlFormatter<'o> {
    output: &'o mut WriteWithLast<'o>,
    options: &'o ComrakOptions,
    seen_anchors: HashSet<String>,
    footnote_ix: u32,
    written_footnote_ix: u32,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const NEEDS_ESCAPED : [bool; 256] = [
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, true,  false, false, false, true,  false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, true, false, true, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false,
];

fn tagfilter(literal: &[u8]) -> bool {
    lazy_static! {
        static ref TAGFILTER_BLACKLIST: [&'static str; 9] =
            ["title", "textarea", "style", "xmp", "iframe",
             "noembed", "noframes", "script", "plaintext"];
    }

    if literal.len() < 3 || literal[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if literal[i] == b'/' {
        i += 1;
    }

    for t in TAGFILTER_BLACKLIST.iter() {
        if unsafe { String::from_utf8_unchecked(literal[i..].to_vec()) }
            .to_lowercase()
            .starts_with(t)
        {
            let j = i + t.len();
            return isspace(literal[j]) || literal[j] == b'>'
                || (literal[j] == b'/' && literal.len() >= j + 2 && literal[j + 1] == b'>');
        }
    }

    false
}

fn tagfilter_block(input: &[u8], o: &mut Write) -> io::Result<()> {
    let size = input.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && input[i] != b'<' {
            i += 1;
        }

        if i > org {
            try!(o.write_all(&input[org..i]));
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            try!(o.write_all(b"&lt;"));
        } else {
            try!(o.write_all(b"<"));
        }

        i += 1;
    }

    Ok(())
}

impl<'o> HtmlFormatter<'o> {
    fn new(options: &'o ComrakOptions, output: &'o mut WriteWithLast<'o>) -> Self {
        HtmlFormatter {
            options: options,
            output: output,
            seen_anchors: HashSet::new(),
            footnote_ix: 0,
            written_footnote_ix: 0,
        }
    }

    fn cr(&mut self) -> io::Result<()> {
        if !self.output.last_was_lf.get() {
            try!(self.output.write_all(b"\n"));
        }
        Ok(())
    }

    fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
        let size = buffer.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && !NEEDS_ESCAPED[buffer[i] as usize] {
                i += 1;
            }

            if i > org {
                try!(self.output.write_all(&buffer[org..i]));
            }

            if i >= size {
                break;
            }

            match buffer[i] as char {
                '"' => {
                    try!(self.output.write_all(b"&quot;"));
                }
                '&' => {
                    try!(self.output.write_all(b"&amp;"));
                }
                '<' => {
                    try!(self.output.write_all(b"&lt;"));
                }
                '>' => {
                    try!(self.output.write_all(b"&gt;"));
                }
                _ => unreachable!(),
            }

            i += 1;
        }

        Ok(())
    }

    fn escape_href(&mut self, buffer: &[u8]) -> io::Result<()> {
        lazy_static! {
            static ref HREF_SAFE: [bool; 256] = {
                let mut a = [false; 256];
                for &c in b"-_.+!*'(),%#@?=;:/,+&$abcdefghijklmnopqrstuvwxyz".iter() {
                    a[c as usize] = true;
                }
                for &c in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".iter() {
                    a[c as usize] = true;
                }
                a
            };
        }

        let size = buffer.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && HREF_SAFE[buffer[i] as usize] {
                i += 1;
            }

            if i > org {
                try!(self.output.write_all(&buffer[org..i]));
            }

            if i >= size {
                break;
            }

            match buffer[i] as char {
                '&' => {
                    try!(self.output.write_all(b"&amp;"));
                }
                '\'' => {
                    try!(self.output.write_all(b"&#x27;"));
                }
                _ => try!(write!(self.output, "%{:02X}", buffer[i])),
            }

            i += 1;
        }

        Ok(())
    }

    fn format<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) -> io::Result<()> {
        // Traverse the AST iteratively using a work stack, with pre- and
        // post-child-traversal phases. During pre-order traversal render the
        // opening tags, then push the node back onto the stack for the
        // post-order traversal phase, then push the children in reverse order
        // onto the stack and begin rendering first child.

        enum Phase { Pre, Post }
        let mut stack = vec![(node, plain, Phase::Pre)];

        while let Some((node, plain, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    let new_plain;
                    if plain {
                        match node.data.borrow().value {
                            NodeValue::Text(ref literal)
                            | NodeValue::Code(ref literal)
                            | NodeValue::HtmlInline(ref literal) => {
                                try!(self.escape(literal));
                            }
                            NodeValue::LineBreak | NodeValue::SoftBreak => {
                                try!(self.output.write_all(b" "));
                            }
                            _ => (),
                        }
                        new_plain = plain;
                    }
                    else {
                        stack.push((node, false, Phase::Post));
                        new_plain = try!(self.format_node(node, true));
                    }

                    for ch in node.reverse_children() {
                        stack.push((ch, new_plain, Phase::Pre));
                    }
                }
                Phase::Post => {
                    debug_assert!(!plain);
                    try!(self.format_node(node, false));
                }
            }
        }

        Ok(())
    }

    fn collect_text<'a>(&self, node: &'a AstNode<'a>, output: &mut Vec<u8>) {
        match node.data.borrow().value {
            NodeValue::Text(ref literal) | NodeValue::Code(ref literal) => {
                output.extend_from_slice(literal)
            }
            NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
            _ => for n in node.children() {
                self.collect_text(n, output);
            },
        }
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::BlockQuote => if entering {
                try!(self.cr());
                try!(self.output.write_all(b"<blockquote>\n"));
            } else {
                try!(self.cr());
                try!(self.output.write_all(b"</blockquote>\n"));
            },
            NodeValue::List(ref nl) => if entering {
                try!(self.cr());
                if nl.list_type == ListType::Bullet {
                    try!(self.output.write_all(b"<ul>\n"));
                } else if nl.start == 1 {
                    try!(self.output.write_all(b"<ol>\n"));
                } else {
                    try!(write!(self.output, "<ol start=\"{}\">\n", nl.start));
                }
            } else if nl.list_type == ListType::Bullet {
                try!(self.output.write_all(b"</ul>\n"));
            } else {
                try!(self.output.write_all(b"</ol>\n"));
            },
            NodeValue::Item(..) => if entering {
                try!(self.cr());
                try!(self.output.write_all(b"<li>"));
            } else {
                try!(self.output.write_all(b"</li>\n"));
            },
            NodeValue::Heading(ref nch) => {
                lazy_static! {
                    static ref REJECTED_CHARS: Regex = Regex::new(r"[^\p{L}\p{M}\p{N}\p{Pc} -]").unwrap();
                }

                if entering {
                    try!(self.cr());
                    try!(write!(self.output, "<h{}>", nch.level));

                    if let Some(ref prefix) = self.options.ext_header_ids {
                        let mut text_content = Vec::with_capacity(20);
                        self.collect_text(node, &mut text_content);

                        let mut id = String::from_utf8(text_content).unwrap();
                        id = id.to_lowercase();
                        id = REJECTED_CHARS.replace(&id, "").to_string();
                        id = id.replace(' ', "-");

                        let mut uniq = 0;
                        id = loop {
                            let anchor = if uniq == 0 {
                                Cow::from(&*id)
                            } else {
                                Cow::from(format!("{}-{}", &id, uniq))
                            };

                            if !self.seen_anchors.contains(&*anchor) {
                                break anchor.to_string();
                            }

                            uniq += 1;
                        };

                        self.seen_anchors.insert(id.clone());

                        try!(write!(
                            self.output,
                            "<a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}{}\"></a>",
                            id,
                            prefix,
                            id
                        ));
                    }
                } else {
                    try!(write!(self.output, "</h{}>\n", nch.level));
                }
            }
            NodeValue::CodeBlock(ref ncb) => if entering {
                try!(self.cr());

                if ncb.info.is_empty() {
                    try!(self.output.write_all(b"<pre><code>"));
                } else {
                    let mut first_tag = 0;
                    while first_tag < ncb.info.len() && !isspace(ncb.info[first_tag]) {
                        first_tag += 1;
                    }

                    if self.options.github_pre_lang {
                        try!(self.output.write_all(b"<pre lang=\""));
                        try!(self.escape(&ncb.info[..first_tag]));
                        try!(self.output.write_all(b"\"><code>"));
                    } else {
                        try!(self.output.write_all(b"<pre><code class=\"language-"));
                        try!(self.escape(&ncb.info[..first_tag]));
                        try!(self.output.write_all(b"\">"));
                    }
                }
                try!(self.escape(&ncb.literal));
                try!(self.output.write_all(b"</code></pre>\n"));
            },
            NodeValue::HtmlBlock(ref nhb) => if entering {
                try!(self.cr());
                if self.options.ext_tagfilter {
                    try!(tagfilter_block(&nhb.literal, &mut self.output));
                } else {
                    try!(self.output.write_all(&nhb.literal));
                }
                try!(self.cr());
            },
            NodeValue::ThematicBreak => if entering {
                try!(self.cr());
                try!(self.output.write_all(b"<hr />\n"));
            },
            NodeValue::Paragraph => {
                let tight = match node.parent()
                    .and_then(|n| n.parent())
                    .map(|n| n.data.borrow().value.clone())
                {
                    Some(NodeValue::List(nl)) => nl.tight,
                    _ => false,
                };

                if !tight {
                    if entering {
                        try!(self.cr());
                        try!(self.output.write_all(b"<p>"));
                    } else {
                        if match node.parent().unwrap().data.borrow().value {
                            NodeValue::FootnoteDefinition(..) => true,
                            _ => false,
                        } && node.next_sibling().is_none()
                        {
                            try!(self.output.write_all(b" "));
                            try!(self.put_footnote_backref());
                        }
                        try!(self.output.write_all(b"</p>\n"));
                    }
                }
            }
            NodeValue::Text(ref literal) => if entering {
                try!(self.escape(literal));
            },
            NodeValue::LineBreak => if entering {
                try!(self.output.write_all(b"<br />\n"));
            },
            NodeValue::SoftBreak => if entering {
                if self.options.hardbreaks {
                    try!(self.output.write_all(b"<br />\n"));
                } else {
                    try!(self.output.write_all(b"\n"));
                }
            },
            NodeValue::Code(ref literal) => if entering {
                try!(self.output.write_all(b"<code>"));
                try!(self.escape(literal));
                try!(self.output.write_all(b"</code>"));
            },
            NodeValue::HtmlInline(ref literal) => if entering {
                if self.options.ext_tagfilter && tagfilter(literal) {
                    try!(self.output.write_all(b"&lt;"));
                    try!(self.output.write_all(&literal[1..]));
                } else {
                    try!(self.output.write_all(literal));
                }
            },
            NodeValue::Strong => if entering {
                try!(self.output.write_all(b"<strong>"));
            } else {
                try!(self.output.write_all(b"</strong>"));
            },
            NodeValue::Emph => if entering {
                try!(self.output.write_all(b"<em>"));
            } else {
                try!(self.output.write_all(b"</em>"));
            },
            NodeValue::Strikethrough => if entering {
                try!(self.output.write_all(b"<del>"));
            } else {
                try!(self.output.write_all(b"</del>"));
            },
            NodeValue::Superscript => if entering {
                try!(self.output.write_all(b"<sup>"));
            } else {
                try!(self.output.write_all(b"</sup>"));
            },
            NodeValue::Link(ref nl) => if entering {
                try!(self.output.write_all(b"<a href=\""));
                try!(self.escape_href(&nl.url));
                if !nl.title.is_empty() {
                    try!(self.output.write_all(b"\" title=\""));
                    try!(self.escape(&nl.title));
                }
                try!(self.output.write_all(b"\">"));
            } else {
                try!(self.output.write_all(b"</a>"));
            },
            NodeValue::Image(ref nl) => if entering {
                try!(self.output.write_all(b"<img src=\""));
                try!(self.escape_href(&nl.url));
                try!(self.output.write_all(b"\" alt=\""));
                return Ok(true);
            } else {
                if !nl.title.is_empty() {
                    try!(self.output.write_all(b"\" title=\""));
                    try!(self.escape(&nl.title));
                }
                try!(self.output.write_all(b"\" />"));
            },
            NodeValue::Table(..) => if entering {
                try!(self.cr());
                try!(self.output.write_all(b"<table>\n"));
            } else {
                if !node.last_child()
                    .unwrap()
                    .same_node(node.first_child().unwrap())
                {
                    try!(self.output.write_all(b"</tbody>"));
                }
                try!(self.output.write_all(b"</table>\n"));
            },
            NodeValue::TableRow(header) => if entering {
                try!(self.cr());
                if header {
                    try!(self.output.write_all(b"<thead>"));
                    try!(self.cr());
                } else if let Some(n) = node.previous_sibling() {
                    if let NodeValue::TableRow(true) = n.data.borrow().value {
                        try!(self.output.write_all(b"<tbody>"));
                        try!(self.cr());
                    }
                }
                try!(self.output.write_all(b"<tr>"));
            } else {
                try!(self.cr());
                try!(self.output.write_all(b"</tr>"));
                if header {
                    try!(self.cr());
                    try!(self.output.write_all(b"</thead>"));
                }
            },
            NodeValue::TableCell => {
                let row = &node.parent().unwrap().data.borrow().value;
                let in_header = match *row {
                    NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
                let alignments = match *table {
                    NodeValue::Table(ref alignments) => alignments,
                    _ => panic!(),
                };

                if entering {
                    try!(self.cr());
                    if in_header {
                        try!(self.output.write_all(b"<th"));
                    } else {
                        try!(self.output.write_all(b"<td"));
                    }

                    let mut start = node.parent().unwrap().first_child().unwrap();
                    let mut i = 0;
                    while !start.same_node(node) {
                        i += 1;
                        start = start.next_sibling().unwrap();
                    }

                    match alignments[i] {
                        TableAlignment::Left => {
                            try!(self.output.write_all(b" align=\"left\""));
                        }
                        TableAlignment::Right => {
                            try!(self.output.write_all(b" align=\"right\""));
                        }
                        TableAlignment::Center => {
                            try!(self.output.write_all(b" align=\"center\""));
                        }
                        TableAlignment::None => (),
                    }

                    try!(self.output.write_all(b">"));
                } else if in_header {
                    try!(self.output.write_all(b"</th>"));
                } else {
                    try!(self.output.write_all(b"</td>"));
                }
            }
            NodeValue::FootnoteDefinition(_) => if entering {
                if self.footnote_ix == 0 {
                    try!(
                        self.output
                            .write_all(b"<section class=\"footnotes\">\n<ol>\n")
                    );
                }
                self.footnote_ix += 1;
                try!(write!(self.output, "<li id=\"fn{}\">\n", self.footnote_ix));
            } else {
                if try!(self.put_footnote_backref()) {
                    try!(self.output.write_all(b"\n"));
                }
                try!(self.output.write_all(b"</li>\n"));
            },
            NodeValue::FootnoteReference(ref r) => if entering {
                let r = str::from_utf8(r).unwrap();
                try!(write!(
                    self.output,
                    "<sup class=\"footnote-ref\"><a href=\"#fn{}\" id=\"fnref{}\">[{}]</a></sup>",
                    r, r, r
                ));
            },
        }
        Ok(false)
    }

    fn put_footnote_backref(&mut self) -> io::Result<bool> {
        if self.written_footnote_ix >= self.footnote_ix {
            return Ok(false);
        }

        self.written_footnote_ix = self.footnote_ix;
        try!(write!(
            self.output,
            "<a href=\"#fnref{}\" class=\"footnote-backref\">↩</a>",
            self.footnote_ix
        ));
        Ok(true)
    }
}
