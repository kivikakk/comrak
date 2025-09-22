use crate::character_set::character_set;
use crate::nodes::{AstNode, ListType, NodeCode, NodeMath, NodeTable, NodeValue};
use crate::parser::{Options, Plugins};
use std::cmp;
use std::io::{self, Write};

use crate::nodes::NodeHtmlBlock;

const MAX_INDENT: u32 = 40;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
) -> io::Result<()> {
    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
    plugins: &Plugins,
) -> io::Result<()> {
    output.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_all(b"<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;

    XmlFormatter::new(options, output, plugins).format(root, false)
}

struct XmlFormatter<'o, 'c> {
    output: &'o mut dyn Write,
    options: &'o Options<'c>,
    _plugins: &'o Plugins<'o>,
    indent: u32,
}

impl<'o, 'c> XmlFormatter<'o, 'c> {
    fn new(options: &'o Options<'c>, output: &'o mut dyn Write, plugins: &'o Plugins) -> Self {
        XmlFormatter {
            options,
            output,
            _plugins: plugins,
            indent: 0,
        }
    }

    fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
        const XML_UNSAFE: [bool; 256] = character_set!(b"&<>\"");

        let mut offset = 0;
        for (i, &byte) in buffer.iter().enumerate() {
            if XML_UNSAFE[byte as usize] {
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
                    let new_plain = if plain {
                        match node.data.borrow().value {
                            NodeValue::Text(ref literal)
                            | NodeValue::Code(NodeCode { ref literal, .. })
                            | NodeValue::HtmlInline(ref literal)
                            | NodeValue::Raw(ref literal) => {
                                self.escape(literal.as_bytes())?;
                            }
                            NodeValue::LineBreak | NodeValue::SoftBreak => {
                                self.output.write_all(b" ")?;
                            }
                            NodeValue::Math(NodeMath { ref literal, .. }) => {
                                self.escape(literal.as_bytes())?;
                            }
                            _ => (),
                        }
                        plain
                    } else {
                        stack.push((node, false, Phase::Post));
                        self.format_node(node, true)?
                    };

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

    fn indent(&mut self) -> io::Result<()> {
        for _ in 0..(cmp::min(self.indent, MAX_INDENT)) {
            self.output.write_all(b" ")?;
        }
        Ok(())
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
        if entering {
            self.indent()?;

            let ast = node.data.borrow();

            write!(self.output, "<{}", ast.value.xml_node_name())?;

            if self.options.render.sourcepos && ast.sourcepos.start.line != 0 {
                write!(self.output, " sourcepos=\"{}\"", ast.sourcepos)?;
            }

            let mut was_literal = false;

            match ast.value {
                NodeValue::Document => self
                    .output
                    .write_all(b" xmlns=\"http://commonmark.org/xml/1.0\"")?,
                NodeValue::Text(ref literal)
                | NodeValue::Code(NodeCode { ref literal, .. })
                | NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. })
                | NodeValue::HtmlInline(ref literal)
                | NodeValue::Raw(ref literal) => {
                    self.output.write_all(b" xml:space=\"preserve\">")?;
                    self.escape(literal.as_bytes())?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::List(ref nl) => {
                    match nl.list_type {
                        ListType::Bullet => {
                            self.output.write_all(b" type=\"bullet\"")?;
                        }
                        ListType::Ordered => {
                            write!(
                                self.output,
                                " type=\"ordered\" start=\"{}\" delim=\"{}\"",
                                nl.start,
                                nl.delimiter.xml_name()
                            )?;
                        }
                    }
                    if nl.is_task_list {
                        self.output.write_all(b" tasklist=\"true\"")?;
                    }
                    write!(self.output, " tight=\"{}\"", nl.tight)?;
                }
                NodeValue::FrontMatter(_) => (),
                NodeValue::BlockQuote => {}
                NodeValue::MultilineBlockQuote(..) => {}
                NodeValue::Item(..) => {}
                NodeValue::DescriptionList => {}
                NodeValue::DescriptionItem(..) => (),
                NodeValue::DescriptionTerm => {}
                NodeValue::DescriptionDetails => {}
                NodeValue::Heading(ref nh) => {
                    write!(self.output, " level=\"{}\"", nh.level)?;
                }
                NodeValue::CodeBlock(ref ncb) => {
                    if !ncb.info.is_empty() {
                        self.output.write_all(b" info=\"")?;
                        self.output.write_all(ncb.info.as_bytes())?;
                        self.output.write_all(b"\"")?;

                        if ncb.info.eq("math") {
                            self.output.write_all(b" math_style=\"display\"")?;
                        }
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

                    if let (
                        NodeValue::TableRow(true),
                        NodeValue::Table(NodeTable { alignments, .. }),
                    ) = (header_row, table)
                    {
                        let ix = node.preceding_siblings().count() - 1;
                        if let Some(xml_align) = alignments[ix].xml_name() {
                            write!(self.output, " align=\"{}\"", xml_align)?;
                        }
                    }
                }
                NodeValue::FootnoteDefinition(ref fd) => {
                    self.output.write_all(b" label=\"")?;
                    self.escape(fd.name.as_bytes())?;
                    self.output.write_all(b"\"")?;
                }
                NodeValue::FootnoteReference(ref nfr) => {
                    self.output.write_all(b" label=\"")?;
                    self.escape(nfr.name.as_bytes())?;
                    self.output.write_all(b"\"")?;
                }
                NodeValue::TaskItem(Some(_)) => {
                    self.output.write_all(b" completed=\"true\"")?;
                }
                NodeValue::TaskItem(None) => {
                    self.output.write_all(b" completed=\"false\"")?;
                }
                #[cfg(feature = "shortcodes")]
                NodeValue::ShortCode(ref nsc) => {
                    self.output.write_all(b" id=\"")?;
                    self.escape(nsc.code.as_bytes())?;
                    self.output.write_all(b"\"")?;
                }
                NodeValue::Escaped => {
                    // noop
                }
                NodeValue::Math(ref math, ..) => {
                    if math.display_math {
                        self.output.write_all(b" math_style=\"display\"")?;
                    } else {
                        self.output.write_all(b" math_style=\"inline\"")?;
                    }
                    self.output.write_all(b" xml:space=\"preserve\">")?;
                    self.escape(math.literal.as_bytes())?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::WikiLink(ref nl) => {
                    self.output.write_all(b" destination=\"")?;
                    self.escape(nl.url.as_bytes())?;
                    self.output.write_all(b"\"")?;
                }
                NodeValue::Underline => {}
                NodeValue::Subscript => {}
                NodeValue::SpoileredText => {}
                NodeValue::EscapedTag(ref data) => {
                    self.output.write_all(data.as_bytes())?;
                }
                NodeValue::Alert(ref alert) => {
                    self.output.write_all(b" type=\"")?;
                    self.output
                        .write_all(alert.alert_type.default_title().to_lowercase().as_bytes())?;
                    self.output.write_all(b"\"")?;
                    if alert.title.is_some() {
                        let title = alert.title.as_ref().unwrap();

                        self.output.write_all(b" title=\"")?;
                        self.escape(title.as_bytes())?;
                        self.output.write_all(b"\"")?;
                    }

                    if alert.multiline {
                        self.output.write_all(b" multiline=\"true\"")?;
                    }
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
            writeln!(
                self.output,
                "</{}>",
                node.data.borrow().value.xml_node_name()
            )?;
        }
        Ok(false)
    }
}
