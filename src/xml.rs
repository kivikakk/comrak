use indextree::Arena;
use std::cmp;
use std::fmt::{self, Write};

use crate::character_set::character_set;
use crate::nodes::{Ast, NodeHtmlBlock};
use crate::nodes::{AstNode, ListType, NodeCode, NodeMath, NodeTable, NodeValue};
use crate::parser::{Options, Plugins};

const MAX_INDENT: u32 = 40;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    arena: &'a Arena<Ast>,
    root: AstNode,
    options: &Options,
    output: &mut dyn Write,
) -> fmt::Result {
    format_document_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    arena: &'a Arena<Ast>,
    root: AstNode,
    options: &Options,
    output: &mut dyn Write,
    plugins: &Plugins,
) -> fmt::Result {
    output.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;

    XmlFormatter::new(arena, options, output, plugins).format(root, false)
}

struct XmlFormatter<'a, 'o, 'c> {
    arena: &'a Arena<Ast>,
    output: &'o mut dyn Write,
    options: &'o Options<'c>,
    _plugins: &'o Plugins<'o>,
    indent: u32,
}

impl<'a, 'o, 'c> XmlFormatter<'a, 'o, 'c> {
    fn new(
        arena: &'a Arena<Ast>,
        options: &'o Options<'c>,
        output: &'o mut dyn Write,
        plugins: &'o Plugins,
    ) -> Self {
        XmlFormatter {
            arena,
            options,
            output,
            _plugins: plugins,
            indent: 0,
        }
    }

    fn escape(&mut self, buffer: &str) -> fmt::Result {
        let bytes = buffer.as_bytes();
        const XML_UNSAFE: [bool; 256] = character_set!(b"&<>\"");

        let mut offset = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            if XML_UNSAFE[byte as usize] {
                let esc: &str = match byte {
                    b'"' => "&quot;",
                    b'&' => "&amp;",
                    b'<' => "&lt;",
                    b'>' => "&gt;",
                    _ => unreachable!(),
                };
                self.output.write_str(&buffer[offset..i])?;
                self.output.write_str(esc)?;
                offset = i + 1;
            }
        }
        self.output.write_str(&buffer[offset..])?;
        Ok(())
    }

    fn format(&mut self, node: AstNode, plain: bool) -> fmt::Result {
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
                        match node.get(self.arena).value {
                            NodeValue::Text(ref literal)
                            | NodeValue::Code(NodeCode { ref literal, .. })
                            | NodeValue::HtmlInline(ref literal)
                            | NodeValue::Raw(ref literal) => {
                                self.escape(literal)?;
                            }
                            NodeValue::LineBreak | NodeValue::SoftBreak => {
                                self.output.write_str(" ")?;
                            }
                            NodeValue::Math(NodeMath { ref literal, .. }) => {
                                self.escape(literal)?;
                            }
                            _ => (),
                        }
                        plain
                    } else {
                        stack.push((node, false, Phase::Post));
                        self.format_node(node, true)?
                    };

                    for ch in node.children(self.arena).rev() {
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

    fn indent(&mut self) -> fmt::Result {
        for _ in 0..(cmp::min(self.indent, MAX_INDENT)) {
            self.output.write_str(" ")?;
        }
        Ok(())
    }

    fn format_node(&mut self, node: AstNode, entering: bool) -> Result<bool, std::fmt::Error> {
        if entering {
            self.indent()?;

            let ast = node.get(self.arena);

            write!(self.output, "<{}", ast.value.xml_node_name())?;

            if self.options.render.sourcepos && ast.sourcepos.start.line != 0 {
                write!(self.output, " sourcepos=\"{}\"", ast.sourcepos)?;
            }

            let mut was_literal = false;

            match ast.value {
                NodeValue::Document => self
                    .output
                    .write_str(" xmlns=\"http://commonmark.org/xml/1.0\"")?,
                NodeValue::Text(ref literal)
                | NodeValue::Code(NodeCode { ref literal, .. })
                | NodeValue::HtmlBlock(NodeHtmlBlock { ref literal, .. })
                | NodeValue::HtmlInline(ref literal)
                | NodeValue::Raw(ref literal) => {
                    self.output.write_str(" xml:space=\"preserve\">")?;
                    self.escape(literal)?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::List(ref nl) => {
                    match nl.list_type {
                        ListType::Bullet => {
                            self.output.write_str(" type=\"bullet\"")?;
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
                        self.output.write_str(" tasklist=\"true\"")?;
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
                        self.output.write_str(" info=\"")?;
                        self.output.write_str(&ncb.info)?;
                        self.output.write_str("\"")?;

                        if ncb.info.eq("math") {
                            self.output.write_str(" math_style=\"display\"")?;
                        }
                    }
                    self.output.write_str(" xml:space=\"preserve\">")?;
                    self.escape(&ncb.literal)?;
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
                    self.output.write_str(" destination=\"")?;
                    self.escape(&nl.url)?;
                    self.output.write_str("\" title=\"")?;
                    self.escape(&nl.title)?;
                    self.output.write_str("\"")?;
                }
                NodeValue::Table(..) => {
                    // noop
                }
                NodeValue::TableRow(..) => {
                    // noop
                }
                NodeValue::TableCell => {
                    let mut ancestors = node.ancestors(self.arena).skip(1);

                    let header_row = &ancestors.next().unwrap().get(self.arena).value;
                    let table = &ancestors.next().unwrap().get(self.arena).value;

                    if let (
                        NodeValue::TableRow(true),
                        NodeValue::Table(NodeTable { alignments, .. }),
                    ) = (header_row, table)
                    {
                        let ix = node.preceding_siblings(self.arena).count() - 1;
                        if let Some(xml_align) = alignments[ix].xml_name() {
                            write!(self.output, " align=\"{}\"", xml_align)?;
                        }
                    }
                }
                NodeValue::FootnoteDefinition(ref fd) => {
                    self.output.write_str(" label=\"")?;
                    self.escape(&fd.name)?;
                    self.output.write_str("\"")?;
                }
                NodeValue::FootnoteReference(ref nfr) => {
                    self.output.write_str(" label=\"")?;
                    self.escape(&nfr.name)?;
                    self.output.write_str("\"")?;
                }
                NodeValue::TaskItem(Some(_)) => {
                    self.output.write_str(" completed=\"true\"")?;
                }
                NodeValue::TaskItem(None) => {
                    self.output.write_str(" completed=\"false\"")?;
                }
                #[cfg(feature = "shortcodes")]
                NodeValue::ShortCode(ref nsc) => {
                    self.output.write_str(" id=\"")?;
                    self.escape(&nsc.code)?;
                    self.output.write_str("\"")?;
                }
                NodeValue::Escaped => {
                    // noop
                }
                NodeValue::Math(ref math, ..) => {
                    if math.display_math {
                        self.output.write_str(" math_style=\"display\"")?;
                    } else {
                        self.output.write_str(" math_style=\"inline\"")?;
                    }
                    self.output.write_str(" xml:space=\"preserve\">")?;
                    self.escape(&math.literal)?;
                    write!(self.output, "</{}", ast.value.xml_node_name())?;
                    was_literal = true;
                }
                NodeValue::WikiLink(ref nl) => {
                    self.output.write_str(" destination=\"")?;
                    self.escape(&nl.url)?;
                    self.output.write_str("\"")?;
                }
                NodeValue::Underline => {}
                NodeValue::Subscript => {}
                NodeValue::SpoileredText => {}
                NodeValue::EscapedTag(ref data) => {
                    self.output.write_str(data)?;
                }
                NodeValue::Alert(ref alert) => {
                    self.output.write_str(" type=\"")?;
                    self.output
                        .write_str(&alert.alert_type.default_title().to_lowercase())?;
                    self.output.write_str("\"")?;
                    if alert.title.is_some() {
                        let title = alert.title.as_ref().unwrap();

                        self.output.write_str(" title=\"")?;
                        self.escape(&title)?;
                        self.output.write_str("\"")?;
                    }

                    if alert.multiline {
                        self.output.write_str(" multiline=\"true\"")?;
                    }
                }
            }

            if node.first_child(self.arena).is_some() {
                self.indent += 2;
            } else if !was_literal {
                self.output.write_str(" /")?;
            }
            self.output.write_str(">\n")?;
        } else if node.first_child(self.arena).is_some() {
            self.indent -= 2;
            self.indent()?;
            writeln!(
                self.output,
                "</{}>",
                node.get(self.arena).value.xml_node_name()
            )?;
        }
        Ok(false)
    }
}
