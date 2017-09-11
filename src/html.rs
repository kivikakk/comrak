use ctype::isspace;
use nodes::{TableAlignment, NodeValue, ListType, AstNode};
use parser::ComrakOptions;
use std::cell::Cell;
use std::io::{self, Write};

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
}

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
        if literal[i..].to_string().to_lowercase().starts_with(t) {
            let j = i + t.len();
            return isspace(literal.as_bytes()[j]) || literal.as_bytes()[j] == b'>' ||
                (literal.as_bytes()[j] == b'/' && literal.len() >= j + 2 &&
                     literal.as_bytes()[j + 1] == b'>');
        }
    }

    false
}

fn tagfilter_block(input: &str, o: &mut Write) -> io::Result<()> {
    let src = input.as_bytes();
    let size = src.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && src[i] != b'<' {
            i += 1;
        }

        if i > org {
            try!(o.write_all(&src[org..i]));
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
        }
    }

    fn cr(&mut self) -> io::Result<()> {
        if !self.output.last_was_lf.get() {
            try!(self.output.write_all(b"\n"));
        }
        Ok(())
    }

    fn escape(&mut self, buffer: &str) -> io::Result<()> {
        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && !NEEDS_ESCAPED[src[i] as usize] {
                i += 1;
            }

            if i > org {
                try!(self.output.write_all(&src[org..i]));
            }

            if i >= size {
                break;
            }

            match src[i] as char {
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

    fn escape_href(&mut self, buffer: &str) -> io::Result<()> {
        lazy_static! {
            static ref HREF_SAFE: [bool; 256] = {
                let mut a = [false; 256];
                for &c in b"-_.+!*'(),%#@?=;:/,+&$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".iter() {
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
                try!(self.output.write_all(&src[org..i]));
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '&' => {
                    try!(self.output.write_all(b"&amp;"));
                }
                '\'' => {
                    try!(self.output.write_all(b"&#x27;"));
                }
                _ => try!(write!(self.output, "%{:02X}", src[i])),
            }

            i += 1;
        }

        Ok(())
    }

    fn format_children<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) -> io::Result<()> {
        for n in node.children() {
            try!(self.format(n, plain));
        }
        Ok(())
    }

    fn format<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) -> io::Result<()> {
        if plain {
            match node.data.borrow().value {
                NodeValue::Text(ref literal) |
                NodeValue::Code(ref literal) |
                NodeValue::HtmlInline(ref literal) => {
                    try!(self.escape(literal));
                }
                NodeValue::LineBreak | NodeValue::SoftBreak => {
                    try!(self.output.write_all(b" "));
                }
                _ => (),
            }
            try!(self.format_children(node, true));
        } else {
            let new_plain = try!(self.format_node(node, true));
            try!(self.format_children(node, new_plain));
            try!(self.format_node(node, false));
        }

        Ok(())
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
        match node.data.borrow().value {
            NodeValue::Document => (),
            NodeValue::BlockQuote => {
                if entering {
                    try!(self.cr());
                    try!(self.output.write_all(b"<blockquote>\n"));
                } else {
                    try!(self.cr());
                    try!(self.output.write_all(b"</blockquote>\n"));
                }
            }
            NodeValue::List(ref nl) => {
                if entering {
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
                }
            }
            NodeValue::Item(..) => {
                if entering {
                    try!(self.cr());
                    try!(self.output.write_all(b"<li>"));
                } else {
                    try!(self.output.write_all(b"</li>\n"));
                }
            }
            NodeValue::Heading(ref nch) => {
                if entering {
                    try!(self.cr());
                    try!(write!(self.output, "<h{}>", nch.level));
                } else {
                    try!(write!(self.output, "</h{}>\n", nch.level));
                }
            }
            NodeValue::CodeBlock(ref ncb) => {
                if entering {
                    try!(self.cr());

                    if ncb.info.is_empty() {
                        try!(self.output.write_all(b"<pre><code>"));
                    } else {
                        let mut first_tag = 0;
                        while first_tag < ncb.info.len() &&
                            !isspace(ncb.info.as_bytes()[first_tag])
                        {
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
                }
            }
            NodeValue::HtmlBlock(ref nhb) => {
                if entering {
                    try!(self.cr());
                    if self.options.ext_tagfilter {
                        try!(tagfilter_block(&nhb.literal, &mut self.output));
                    } else {
                        try!(self.output.write_all(nhb.literal.as_bytes()));
                    }
                    try!(self.cr());
                }
            }
            NodeValue::ThematicBreak => {
                if entering {
                    try!(self.cr());
                    try!(self.output.write_all(b"<hr />\n"));
                }
            }
            NodeValue::Paragraph => {
                let tight = match node.parent().and_then(|n| n.parent()).map(|n| {
                    n.data.borrow().value.clone()
                }) {
                    Some(NodeValue::List(nl)) => nl.tight,
                    _ => false,
                };

                if entering {
                    if !tight {
                        try!(self.cr());
                        try!(self.output.write_all(b"<p>"));
                    }
                } else if !tight {
                    try!(self.output.write_all(b"</p>\n"));
                }
            }
            NodeValue::Text(ref literal) => {
                if entering {
                    try!(self.escape(literal));
                }
            }
            NodeValue::LineBreak => {
                if entering {
                    try!(self.output.write_all(b"<br />\n"));
                }
            }
            NodeValue::SoftBreak => {
                if entering {
                    if self.options.hardbreaks {
                        try!(self.output.write_all(b"<br />\n"));
                    } else {
                        try!(self.output.write_all(b"\n"));
                    }
                }
            }
            NodeValue::Code(ref literal) => {
                if entering {
                    try!(self.output.write_all(b"<code>"));
                    try!(self.escape(literal));
                    try!(self.output.write_all(b"</code>"));
                }
            }
            NodeValue::HtmlInline(ref literal) => {
                if entering {
                    if self.options.ext_tagfilter && tagfilter(literal) {
                        try!(self.output.write_all(b"&lt;"));
                        try!(self.output.write_all(literal[1..].as_bytes()));
                    } else {
                        try!(self.output.write_all(literal.as_bytes()));
                    }
                }
            }
            NodeValue::Strong => {
                if entering {
                    try!(self.output.write_all(b"<strong>"));
                } else {
                    try!(self.output.write_all(b"</strong>"));
                }
            }
            NodeValue::Emph => {
                if entering {
                    try!(self.output.write_all(b"<em>"));
                } else {
                    try!(self.output.write_all(b"</em>"));
                }
            }
            NodeValue::Strikethrough => {
                if entering {
                    try!(self.output.write_all(b"<del>"));
                } else {
                    try!(self.output.write_all(b"</del>"));
                }
            }
            NodeValue::Superscript => {
                if entering {
                    try!(self.output.write_all(b"<sup>"));
                } else {
                    try!(self.output.write_all(b"</sup>"));
                }
            }
            NodeValue::Link(ref nl) => {
                if entering {
                    try!(self.output.write_all(b"<a href=\""));
                    try!(self.escape_href(&nl.url));
                    if !nl.title.is_empty() {
                        try!(self.output.write_all(b"\" title=\""));
                        try!(self.escape(&nl.title));
                    }
                    try!(self.output.write_all(b"\">"));
                } else {
                    try!(self.output.write_all(b"</a>"));
                }
            }
            NodeValue::Image(ref nl) => {
                if entering {
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
                }
            }
            NodeValue::Table(..) => {
                if entering {
                    try!(self.cr());
                    try!(self.output.write_all(b"<table>\n"));
                } else {
                    if !node.last_child().unwrap().same_node(
                        node.first_child().unwrap(),
                    )
                    {
                        try!(self.output.write_all(b"</tbody>"));
                    }
                    try!(self.output.write_all(b"</table>\n"));
                }
            }
            NodeValue::TableRow(header) => {
                if entering {
                    try!(self.cr());
                    if header {
                        try!(self.output.write_all(b"<thead>"));
                        try!(self.cr());
                    }
                    try!(self.output.write_all(b"<tr>"));
                } else {
                    try!(self.cr());
                    try!(self.output.write_all(b"</tr>"));
                    if header {
                        try!(self.cr());
                        try!(self.output.write_all(b"</thead>"));
                        try!(self.cr());
                        try!(self.output.write_all(b"<tbody>"));
                    }
                }
            }
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
        }
        Ok(false)
    }
}
