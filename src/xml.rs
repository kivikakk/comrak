use crate::nodes::{AstNode, ListType, NodeCode, NodeValue};
use crate::parser::{ComrakOptions, ComrakPlugins};
use once_cell::sync::Lazy;
use std::io::{self, Write};

use crate::nodes::NodeHtmlBlock;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut dyn Write,
) -> io::Result<()> {
    format_document_with_plugins(root, &options, output, &ComrakPlugins::default())
}

/// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &ComrakOptions,
    output: &mut dyn Write,
    plugins: &ComrakPlugins,
) -> io::Result<()> {
    output.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_all(b"<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;

    XmlFormatter::new(options, output, plugins).format(root, false)
}

struct XmlFormatter<'o> {
    output: &'o mut dyn Write,
    options: &'o ComrakOptions,
    _plugins: &'o ComrakPlugins<'o>,
    indent: u32,
}

impl<'o> XmlFormatter<'o> {
    fn new(
        options: &'o ComrakOptions,
        output: &'o mut dyn Write,
        plugins: &'o ComrakPlugins,
    ) -> Self {
        XmlFormatter {
            options,
            output,
            _plugins: plugins,
            indent: 0,
        }
    }

    fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
        static XML_SAFE: Lazy<[bool; 256]> = Lazy::new(|| {
            let mut a = [true; 256];
            for &c in b"&<>\"".iter() {
                a[c as usize] = false;
            }
            a
        });

        let mut offset = 0;
        for (i, &byte) in buffer.iter().enumerate() {
            if !XML_SAFE[byte as usize] {
                let esc: &[u8] = match byte {
                    b'"' => b"&quot;",
                    b'&' => b"&amp;",
                    b'<' => b"&lt;",
                    b'>' => b"&gt;",
                    _ => unreachable!(),
                };
                self.output.write_all(&buffer[offset..i])?;
                self.output.write_all(esc)?;
                offset = i + 1;
            }
        }
        self.output.write_all(&buffer[offset..])?;
        Ok(())
    }

    fn format<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) -> io::Result<()> {
        // Traverse the AST iteratively using a work stack, with pre- and
        // post-child-traversal phases. During pre-order traversal render the
        // opening tags, then push the node back onto the stack for the
        // post-order traversal phase, then push the children in reverse order
        // onto the stack and begin rendering first child.

        enum Phase {
            Pre,
            Post,
        }
        let mut stack = vec![(node, plain, Phase::Pre)];

        while let Some((node, plain, phase)) = stack.pop() {
            match phase {
                Phase::Pre => {
                    let new_plain;
                    if plain {
                        match node.data.borrow().value {
                            NodeValue::Text(ref literal)
                            | NodeValue::Code(NodeCode { ref literal, .. })
                            | NodeValue::HtmlInline(ref literal) => {
                                self.escape(literal.as_bytes())?;
                            }
                            NodeValue::LineBreak | NodeValue::SoftBreak => {
                                self.output.write_all(b" ")?;
                            }
                            _ => (),
                        }
                        new_plain = plain;
                    } else {
                        stack.push((node, false, Phase::Post));
                        new_plain = self.format_node(node, true)?;
                    }

                    for ch in node.reverse_children() {
                        stack.push((ch, new_plain, Phase::Pre));
                    }
                }
                Phase::Post => {
                    debug_assert!(!plain);
                    self.format_node(node, false)?;
                }
            }
        }

        Ok(())
    }

    fn _collect_text<'a>(&self, node: &'a AstNode<'a>, output: &mut Vec<u8>) {
        match node.data.borrow().value {
            NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
                output.extend_from_slice(literal.as_bytes())
            }
            NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
            _ => {
                for n in node.children() {
                    self._collect_text(n, output);
                }
            }
        }
    }

    fn indent(&mut self) -> io::Result<()> {
        for _ in 0..self.indent {
            self.output.write_all(b" ")?;
        }
        Ok(())
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
        if entering {
            self.indent()?;

            let ast = node.data.borrow();

            write!(self.output, "<{}", ast.value.xml_node_name())?;

            if self.options.render.sourcepos && ast.start_line != 0 {
                write!(
                    self.output,
                    " sourcepos=\"{}:{}-{}:{}\"",
                    ast.start_line, ast.start_column, ast.end_line, ast.end_column,
                )?;
            }

            let mut was_literal = false;

            match ast.value {
                NodeValue::Document => self
                    .output
                    .write_all(b" xmlns=\"http://commonmark.org/xml/1.0\"")?,
                NodeValue::Text(ref literal)
                | NodeValue::Code(NodeCode { ref literal, .. })
                | NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. })
                | NodeValue::HtmlInline(ref literal) => {
                    self.output.write_all(b" xml:space=\"preserve\">")?;
                    self.escape(literal.as_bytes())?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::List(ref nl) => {
                    if nl.list_type == ListType::Bullet {
                        self.output.write_all(b" type=\"bullet\"")?;
                    } else {
                        write!(
                            self.output,
                            " type=\"ordered\" start=\"{}\" delim=\"{}\"",
                            nl.start,
                            nl.delimiter.xml_name()
                        )?;
                    }
                    write!(self.output, " tight=\"{}\"", nl.tight)?;
                }
                NodeValue::FrontMatter(_) => (),
                NodeValue::BlockQuote => {}
                NodeValue::Item(..) => {}
                NodeValue::DescriptionList => {}
                NodeValue::DescriptionItem(..) => (),
                NodeValue::DescriptionTerm => {}
                NodeValue::DescriptionDetails => {}
                NodeValue::Heading(ref nch) => {
                    write!(self.output, " level=\"{}\"", nch.level)?;
                }
                NodeValue::CodeBlock(ref ncb) => {
                    if !ncb.info.is_empty() {
                        self.output.write_all(b" info=\"")?;
                        self.output.write_all(ncb.info.as_bytes())?;
                        self.output.write_all(b"\"")?;
                    }
                    self.output.write_all(b" xml:space=\"preserve\">")?;
                    self.escape(ncb.literal.as_bytes())?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::ThematicBreak => {}
                NodeValue::Paragraph => {}
                NodeValue::LineBreak => {}
                NodeValue::SoftBreak => {}
                NodeValue::Strong => {}
                NodeValue::Emph => {}
                NodeValue::Strikethrough => {}
                NodeValue::Superscript => {}
                NodeValue::Link(ref nl) | NodeValue::Image(ref nl) => {
                    self.output.write_all(b" destination=\"")?;
                    self.escape(nl.url.as_bytes())?;
                    self.output.write_all(b"\" title=\"")?;
                    self.escape(nl.title.as_bytes())?;
                    self.output.write_all(b"\"")?;
                }
                NodeValue::Table(..) => {
                    // noop
                }
                NodeValue::TableRow(..) => {
                    // noop
                }
                NodeValue::TableCell => {
                    let mut ancestors = node.ancestors().skip(1);

                    let header_row = &ancestors.next().unwrap().data.borrow().value;
                    let table = &ancestors.next().unwrap().data.borrow().value;

                    if let (NodeValue::TableRow(true), NodeValue::Table(aligns)) =
                        (header_row, table)
                    {
                        let ix = node.preceding_siblings().count() - 1;
                        if let Some(xml_align) = aligns[ix].xml_name() {
                            write!(self.output, " align=\"{}\"", xml_align)?;
                        }
                    }

                    // if let Some(header_row) = table.first_child() {
                    //     if header_row == node {
                    //         if let Some(alignment) = node.data.borrow().alignment {
                    //             write!(self.output, " align=\"{}\"", alignment.xml_name())?;
                    //         }
                    //     }
                    // }
                }
                NodeValue::FootnoteDefinition(_) => {
                    // TODO
                    // if entering {
                    //     if self.footnote_ix == 0 {
                    //         self.output
                    //             .write_all(b"<section class=\"footnotes\">\n<ol>\n")?;
                    //     }
                    //     self.footnote_ix += 1;
                    //     writeln!(self.output, "<li id=\"fn{}\">", self.footnote_ix)?;
                    // } else {
                    //     if self.put_footnote_backref()? {
                    //         self.output.write_all(b"\n")?;
                    //     }
                    //     self.output.write_all(b"</li>\n")?;
                    // }
                }
                NodeValue::FootnoteReference(ref _r) => {
                    // TODO
                    // if entering {
                    //     let r = str::from_utf8(r).unwrap();
                    //     write!(
                    //         self.output,
                    //         "<sup class=\"footnote-ref\"><a href=\"#fn{}\" id=\"fnref{}\">{}</a></sup>",
                    //         r, r, r
                    //     )?;
                    // }
                }
                NodeValue::TaskItem(_) => {
                    // TODO
                    // if entering {
                    //     if checked {
                    //         self.output.write_all(
                    //             b"<input type=\"checkbox\" disabled=\"\" checked=\"\" /> ",
                    //         )?;
                    //     } else {
                    //         self.output
                    //             .write_all(b"<input type=\"checkbox\" disabled=\"\" /> ")?;
                    //     }
                    // }
                }
                #[cfg(feature = "shortcodes")]
                NodeValue::ShortCode(_) => {
                    // TODO
                }
            }

            if node.first_child().is_some() {
                self.indent += 2;
            } else if !was_literal {
                self.output.write_all(b" /")?;
            }
            self.output.write_all(b">\n")?;
        } else if node.first_child().is_some() {
            self.indent -= 2;
            self.indent()?;
            write!(
                self.output,
                "</{}>\n",
                node.data.borrow().value.xml_node_name()
            )?;
        }
        Ok(false)
    }
}
