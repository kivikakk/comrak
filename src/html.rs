//! The HTML renderer for the CommonMark AST, as well as helper functions.
mod anchorizer;

use crate::adapters::HeadingMeta;
use crate::character_set::character_set;
use crate::ctype::isspace;
use crate::nodes::{
    AstNode, ListType, NodeCode, NodeFootnoteDefinition, NodeMath, NodeTable, NodeValue,
    TableAlignment,
};
use crate::parser::{Options, Plugins};
use crate::scanners;
use std::cell::Cell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::str;

pub use anchorizer::Anchorizer;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
) -> io::Result<()> {
    HtmlFormatter::format_document(root, options, output)
}

/// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn Write,
    plugins: &Plugins,
) -> io::Result<()> {
    HtmlFormatter::format_document_with_plugins(root, options, output, plugins)
}

/// TODO
pub struct WriteWithLast<'w> {
    /// TODO
    pub output: &'w mut dyn Write,
    /// TODO
    pub last_was_lf: Cell<bool>,
}

impl<'w> std::fmt::Debug for WriteWithLast<'w> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str("<WriteWithLast>")
    }
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

/// TODO
fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.extend_from_slice(literal.as_bytes())
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        NodeValue::Math(NodeMath { ref literal, .. }) => {
            output.extend_from_slice(literal.as_bytes())
        }
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}

/// TODO
#[derive(Debug)]
pub struct Context<'o, 'c> {
    /// TODO
    pub output: &'o mut WriteWithLast<'o>,
    /// TODO
    pub options: &'o Options<'c>,
    /// TODO
    pub anchorizer: Anchorizer,
    /// TODO
    pub footnote_ix: u32,
    /// TODO
    pub written_footnote_ix: u32,
    /// TODO
    pub plugins: &'o Plugins<'o>,
}

impl<'o, 'c> Context<'o, 'c> {
    /// TODO
    pub fn cr(&mut self) -> io::Result<()> {
        if !self.output.last_was_lf.get() {
            self.output.write_all(b"\n")?;
        }
        Ok(())
    }

    /// TODO
    pub fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
        escape(&mut self.output, buffer)
    }

    /// TODO
    pub fn escape_href(&mut self, buffer: &[u8]) -> io::Result<()> {
        escape_href(&mut self.output, buffer)
    }
}

/// TODO
#[macro_export]
macro_rules! create_formatter {
    ($name:ident) => {
        create_formatter!($name, |_x, _y| {,});
    };
    ($name:ident, |$output:ident, $entering:ident| { $( $pat:pat => $case:tt ),*, }) => {
        /// TODO
        #[derive(Debug)]
        #[allow(missing_copy_implementations)]
        pub struct $name;

        impl $name {
            /// Formats an AST as HTML, modified by the given options.
            pub fn format_document<'a>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &$crate::Options,
                output: &mut dyn Write,
            ) -> ::std::io::Result<()> {
                Self::format_document_with_plugins(root, options, output, &$crate::Plugins::default())
            }

            /// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
            pub fn format_document_with_plugins<'a, 'o, 'c: 'o>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &'o $crate::Options<'c>,
                output: &'o mut dyn ::std::io::Write,
                plugins: &'o $crate::Plugins<'o>,
            ) -> ::std::io::Result<()> {
                // Traverse the AST iteratively using a work stack, with pre- and
                // post-child-traversal phases. During pre-order traversal render the
                // opening tags, then push the node back onto the stack for the
                // post-order traversal phase, then push the children in reverse order
                // onto the stack and begin rendering first child.
                let mut writer = $crate::html::WriteWithLast {
                    output,
                    last_was_lf: ::std::cell::Cell::new(true),
                };

                let mut context = $crate::html::Context {
                    options,
                    output: &mut writer,
                    anchorizer: $crate::html::Anchorizer::new(),
                    footnote_ix: 0,
                    written_footnote_ix: 0,
                    plugins,
                };

                enum Phase { Pre, Post }
                let mut stack = vec![(root, false, Phase::Pre)];

                while let Some((node, plain, phase)) = stack.pop() {
                    match phase {
                        Phase::Pre => {
                            let new_plain = if plain {
                                match node.data.borrow().value {
                                    $crate::nodes::NodeValue::Text(ref literal)
                                    | $crate::nodes::NodeValue::Code($crate::nodes::NodeCode { ref literal, .. })
                                    | $crate::nodes::NodeValue::HtmlInline(ref literal) => {
                                        context.escape(literal.as_bytes())?;
                                    }
                                    $crate::nodes::NodeValue::LineBreak | $crate::nodes::NodeValue::SoftBreak => {
                                        ::std::io::Write::write_all(context.output, b" ")?;
                                    }
                                    $crate::nodes::NodeValue::Math($crate::nodes::NodeMath { ref literal, .. }) => {
                                        context.escape(literal.as_bytes())?;
                                    }
                                    _ => (),
                                }
                                plain
                            } else {
                                stack.push((node, false, Phase::Post));
                                !Self::format_node(&mut context, node, true)?
                            };

                            for ch in node.reverse_children() {
                                stack.push((ch, new_plain, Phase::Pre));
                            }
                        }
                        Phase::Post => {
                            debug_assert!(!plain);
                            Self::format_node(&mut context, node, false)?;
                        }
                    }
                }

                if context.footnote_ix > 0 {
                    context.output.write_all(b"</ol>\n</section>\n")?;
                }

                Ok(())
            }

            fn format_node<'a>(context: &mut $crate::html::Context, node: &'a $crate::nodes::AstNode<'a>, $entering: bool) -> ::std::io::Result<bool> {
                match node.data.borrow().value {
                    $(
                        $pat => {
                            let $output = &mut context.output;
                            $case
                            Ok(true)
                        }
                    ),*
                    _ => $crate::html::format_node_default(context, node, $entering),
                }
            }
        }
    };
}

create_formatter!(HtmlFormatter);

/// TODO
pub fn format_node_default<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let processing_complete: bool = match node.data.borrow().value {
        // Commonmark
        NodeValue::BlockQuote => render_block_quote(context, node, entering)?,
        NodeValue::Code(_) => render_code(context, node, entering)?,
        NodeValue::CodeBlock(_) => render_code_block(context, node, entering)?,
        NodeValue::Document => render_document(context, node, entering)?,
        NodeValue::Emph => render_emph(context, node, entering)?,
        NodeValue::Heading(_) => render_heading(context, node, entering)?,
        NodeValue::HtmlBlock(_) => render_html_block(context, node, entering)?,
        NodeValue::HtmlInline(_) => render_html_inline(context, node, entering)?,
        NodeValue::Image(_) => render_image(context, node, entering)?,
        NodeValue::Item(_) => render_item(context, node, entering)?,
        NodeValue::LineBreak => render_line_break(context, node, entering)?,
        NodeValue::Link(_) => render_link(context, node, entering)?,
        NodeValue::List(_) => render_list(context, node, entering)?,
        NodeValue::Paragraph => render_paragraph(context, node, entering)?,
        NodeValue::SoftBreak => render_soft_break(context, node, entering)?,
        NodeValue::Strong => render_strong(context, node, entering)?,
        NodeValue::Text(_) => render_text(context, node, entering)?,
        NodeValue::ThematicBreak => render_thematic_break(context, node, entering)?,

        // GFM
        NodeValue::FootnoteDefinition(_) => render_footnote_definition(context, node, entering)?,
        NodeValue::FootnoteReference(_) => render_footnote_reference(context, node, entering)?,
        NodeValue::Strikethrough => render_strikethrough(context, node, entering)?,
        NodeValue::Table(_) => render_table(context, node, entering)?,
        NodeValue::TableCell => render_table_cell(context, node, entering)?,
        NodeValue::TableRow(_) => render_table_row(context, node, entering)?,
        NodeValue::TaskItem(_) => render_task_item(context, node, entering)?,

        // Extensions
        NodeValue::Alert(_) => render_alert(context, node, entering)?,
        NodeValue::DescriptionDetails => render_description_details(context, node, entering)?,
        NodeValue::DescriptionItem(_) => render_description_item(context, node, entering)?,
        NodeValue::DescriptionList => render_description_list(context, node, entering)?,
        NodeValue::DescriptionTerm => render_description_term(context, node, entering)?,
        NodeValue::Escaped => render_escaped(context, node, entering)?,
        NodeValue::EscapedTag(_) => render_escaped_tag(context, node, entering)?,
        NodeValue::FrontMatter(_) => render_frontmatter(context, node, entering)?,
        NodeValue::Math(_) => render_math(context, node, entering)?,
        NodeValue::MultilineBlockQuote(_) => render_multiline_block_quote(context, node, entering)?,
        NodeValue::Raw(_) => render_raw(context, node, entering)?,
        #[cfg(feature = "shortcodes")]
        NodeValue::ShortCode(_) => render_short_code(context, node, entering)?,
        NodeValue::SpoileredText => render_spoiler_text(context, node, entering)?,
        NodeValue::Subscript => render_subscript(context, node, entering)?,
        NodeValue::Superscript => render_superscript(context, node, entering)?,
        NodeValue::Underline => render_underline(context, node, entering)?,
        NodeValue::WikiLink(_) => render_wiki_link(context, node, entering)?,
    };

    Ok(processing_complete)
}

// Commonmark

/// TODO
pub fn render_sourcepos<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
) -> io::Result<()> {
    if context.options.render.sourcepos {
        let ast = node.data.borrow();
        if ast.sourcepos.start.line > 0 {
            write!(context.output, " data-sourcepos=\"{}\"", ast.sourcepos)?;
        }
    }
    Ok(())
}

/// TODO
pub fn render_block_quote<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<blockquote")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">\n")?;
    } else {
        context.cr()?;
        context.output.write_all(b"</blockquote>\n")?;
    }
    Ok(true)
}

/// TODO
pub fn render_code<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Code(NodeCode { ref literal, .. }) = node.data.borrow().value else {
        panic!()
    };

    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<code")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
        context.escape(literal.as_bytes())?;
        context.output.write_all(b"</code>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_code_block<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::CodeBlock(ref ncb) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        if ncb.info.eq("math") {
            render_math_code_block(context, node, &ncb.literal)?;
        } else {
            context.cr()?;

            let mut first_tag = 0;
            let mut pre_attributes: HashMap<String, String> = HashMap::new();
            let mut code_attributes: HashMap<String, String> = HashMap::new();
            let code_attr: String;

            let literal = &ncb.literal.as_bytes();
            let info = &ncb.info.as_bytes();

            if !info.is_empty() {
                while first_tag < info.len() && !isspace(info[first_tag]) {
                    first_tag += 1;
                }

                let lang_str = std::str::from_utf8(&info[..first_tag]).unwrap();
                let info_str = std::str::from_utf8(&info[first_tag..]).unwrap().trim();

                if context.options.render.github_pre_lang {
                    pre_attributes.insert(String::from("lang"), lang_str.to_string());

                    if context.options.render.full_info_string && !info_str.is_empty() {
                        pre_attributes
                            .insert(String::from("data-meta"), info_str.trim().to_string());
                    }
                } else {
                    code_attr = format!("language-{}", lang_str);
                    code_attributes.insert(String::from("class"), code_attr);

                    if context.options.render.full_info_string && !info_str.is_empty() {
                        code_attributes.insert(String::from("data-meta"), info_str.to_string());
                    }
                }
            }

            if context.options.render.sourcepos {
                let ast = node.data.borrow();
                pre_attributes.insert("data-sourcepos".to_string(), ast.sourcepos.to_string());
            }

            match context.plugins.render.codefence_syntax_highlighter {
                None => {
                    write_opening_tag(context.output, "pre", pre_attributes)?;
                    write_opening_tag(context.output, "code", code_attributes)?;

                    context.escape(literal)?;

                    context.output.write_all(b"</code></pre>\n")?
                }
                Some(highlighter) => {
                    highlighter.write_pre_tag(context.output, pre_attributes)?;
                    highlighter.write_code_tag(context.output, code_attributes)?;

                    highlighter.write_highlighted(
                        context.output,
                        match std::str::from_utf8(&info[..first_tag]) {
                            Ok(lang) => Some(lang),
                            Err(_) => None,
                        },
                        &ncb.literal,
                    )?;

                    context.output.write_all(b"</code></pre>\n")?
                }
            }
        }
    }

    Ok(true)
}

/// TODO
pub fn render_document<'o, 'c, 'a>(
    _context: &mut Context<'o, 'c>,
    _node: &'a AstNode<'a>,
    _entering: bool,
) -> io::Result<bool> {
    Ok(true)
}

/// TODO
pub fn render_emph<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<em")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</em>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_heading<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Heading(ref nch) = node.data.borrow().value else {
        panic!()
    };

    match context.plugins.render.heading_adapter {
        None => {
            if entering {
                context.cr()?;
                write!(context.output, "<h{}", nch.level)?;
                render_sourcepos(context, node)?;
                context.output.write_all(b">")?;

                if let Some(ref prefix) = context.options.extension.header_ids {
                    let mut text_content = Vec::with_capacity(20);
                    collect_text(node, &mut text_content);

                    let mut id = String::from_utf8(text_content).unwrap();
                    id = context.anchorizer.anchorize(id);
                    write!(
                        context.output,
                        "<a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}{}\"></a>",
                        id, prefix, id
                    )?;
                }
            } else {
                writeln!(context.output, "</h{}>", nch.level)?;
            }
        }
        Some(adapter) => {
            let mut text_content = Vec::with_capacity(20);
            collect_text(node, &mut text_content);
            let content = String::from_utf8(text_content).unwrap();
            let heading = HeadingMeta {
                level: nch.level,
                content,
            };

            if entering {
                context.cr()?;
                adapter.enter(
                    context.output,
                    &heading,
                    if context.options.render.sourcepos {
                        Some(node.data.borrow().sourcepos)
                    } else {
                        None
                    },
                )?;
            } else {
                adapter.exit(context.output, &heading)?;
            }
        }
    }

    Ok(true)
}

/// TODO
pub fn render_html_block<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::HtmlBlock(ref nhb) = node.data.borrow().value else {
        panic!()
    };

    // No sourcepos.
    if entering {
        context.cr()?;
        let literal = nhb.literal.as_bytes();
        if context.options.render.escape {
            context.escape(literal)?;
        } else if !context.options.render.unsafe_ {
            context.output.write_all(b"<!-- raw HTML omitted -->")?;
        } else if context.options.extension.tagfilter {
            tagfilter_block(literal, &mut context.output)?;
        } else {
            context.output.write_all(literal)?;
        }
        context.cr()?;
    }

    Ok(true)
}

/// TODO
pub fn render_html_inline<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::HtmlInline(ref literal) = node.data.borrow().value else {
        panic!()
    };

    // No sourcepos.
    if entering {
        let literal = literal.as_bytes();
        if context.options.render.escape {
            context.escape(literal)?;
        } else if !context.options.render.unsafe_ {
            context.output.write_all(b"<!-- raw HTML omitted -->")?;
        } else if context.options.extension.tagfilter && tagfilter(literal) {
            context.output.write_all(b"&lt;")?;
            context.output.write_all(&literal[1..])?;
        } else {
            context.output.write_all(literal)?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_image<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Image(ref nl) = node.data.borrow().value else {
        panic!()
    };

    // Unreliable sourcepos.
    if entering {
        if context.options.render.figure_with_caption {
            context.output.write_all(b"<figure>")?;
        }
        context.output.write_all(b"<img")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b" src=\"")?;
        let url = nl.url.as_bytes();
        if context.options.render.unsafe_ || !dangerous_url(url) {
            if let Some(rewriter) = &context.options.extension.image_url_rewriter {
                context.escape_href(rewriter.to_html(&nl.url).as_bytes())?;
            } else {
                context.escape_href(url)?;
            }
        }
        context.output.write_all(b"\" alt=\"")?;
        return Ok(false);
    } else {
        if !nl.title.is_empty() {
            context.output.write_all(b"\" title=\"")?;
            context.escape(nl.title.as_bytes())?;
        }
        context.output.write_all(b"\" />")?;
        if context.options.render.figure_with_caption {
            if !nl.title.is_empty() {
                context.output.write_all(b"<figcaption>")?;
                context.escape(nl.title.as_bytes())?;
                context.output.write_all(b"</figcaption>")?;
            }
            context.output.write_all(b"</figure>")?;
        };
    }

    Ok(true)
}

/// TODO
pub fn render_item<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<li")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</li>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_line_break<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<br")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b" />\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_link<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Link(ref nl) = node.data.borrow().value else {
        panic!()
    };

    // Unreliable sourcepos.
    let parent_node = node.parent();

    if !context.options.parse.relaxed_autolinks
        || (parent_node.is_none()
            || !matches!(
                parent_node.unwrap().data.borrow().value,
                NodeValue::Link(..)
            ))
    {
        if entering {
            context.output.write_all(b"<a")?;
            if context.options.render.experimental_inline_sourcepos {
                render_sourcepos(context, node)?;
            }
            context.output.write_all(b" href=\"")?;
            let url = nl.url.as_bytes();
            if context.options.render.unsafe_ || !dangerous_url(url) {
                if let Some(rewriter) = &context.options.extension.link_url_rewriter {
                    context.escape_href(rewriter.to_html(&nl.url).as_bytes())?;
                } else {
                    context.escape_href(url)?;
                }
            }
            if !nl.title.is_empty() {
                context.output.write_all(b"\" title=\"")?;
                context.escape(nl.title.as_bytes())?;
            }
            context.output.write_all(b"\">")?;
        } else {
            context.output.write_all(b"</a>")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_list<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::List(ref nl) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        context.cr()?;
        match nl.list_type {
            ListType::Bullet => {
                context.output.write_all(b"<ul")?;
                if nl.is_task_list && context.options.render.tasklist_classes {
                    context.output.write_all(b" class=\"contains-task-list\"")?;
                }
                render_sourcepos(context, node)?;
                context.output.write_all(b">\n")?;
            }
            ListType::Ordered => {
                context.output.write_all(b"<ol")?;
                if nl.is_task_list && context.options.render.tasklist_classes {
                    context.output.write_all(b" class=\"contains-task-list\"")?;
                }
                render_sourcepos(context, node)?;
                if nl.start == 1 {
                    context.output.write_all(b">\n")?;
                } else {
                    writeln!(context.output, " start=\"{}\">", nl.start)?;
                }
            }
        }
    } else if nl.list_type == ListType::Bullet {
        context.output.write_all(b"</ul>\n")?;
    } else {
        context.output.write_all(b"</ol>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_paragraph<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let tight = match node
        .parent()
        .and_then(|n| n.parent())
        .map(|n| n.data.borrow().value.clone())
    {
        Some(NodeValue::List(nl)) => nl.tight,
        Some(NodeValue::DescriptionItem(nd)) => nd.tight,
        _ => false,
    };

    let tight = tight
        || matches!(
            node.parent().map(|n| n.data.borrow().value.clone()),
            Some(NodeValue::DescriptionTerm)
        );

    if !tight {
        if entering {
            context.cr()?;
            context.output.write_all(b"<p")?;
            render_sourcepos(context, node)?;
            context.output.write_all(b">")?;
        } else {
            if let NodeValue::FootnoteDefinition(nfd) = &node.parent().unwrap().data.borrow().value
            {
                if node.next_sibling().is_none() {
                    context.output.write_all(b" ")?;
                    put_footnote_backref(context, nfd)?;
                }
            }
            context.output.write_all(b"</p>\n")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_soft_break<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        if context.options.render.hardbreaks {
            context.output.write_all(b"<br")?;
            if context.options.render.experimental_inline_sourcepos {
                render_sourcepos(context, node)?;
            }
            context.output.write_all(b" />\n")?;
        } else {
            context.output.write_all(b"\n")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_strong<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    let parent_node = node.parent();
    if !context.options.render.gfm_quirks
        || (parent_node.is_none()
            || !matches!(parent_node.unwrap().data.borrow().value, NodeValue::Strong))
    {
        if entering {
            context.output.write_all(b"<strong")?;
            if context.options.render.experimental_inline_sourcepos {
                render_sourcepos(context, node)?;
            }
            context.output.write_all(b">")?;
        } else {
            context.output.write_all(b"</strong>")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_text<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Text(ref literal) = node.data.borrow().value else {
        panic!()
    };

    // Nowhere to put sourcepos.
    if entering {
        context.escape(literal.as_bytes())?;
    }

    Ok(true)
}

/// TODO
pub fn render_thematic_break<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<hr")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b" />\n")?;
    }

    Ok(true)
}

// GFM

/// TODO
pub fn render_footnote_definition<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::FootnoteDefinition(ref nfd) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        if context.footnote_ix == 0 {
            context.output.write_all(b"<section")?;
            render_sourcepos(context, node)?;
            context
                .output
                .write_all(b" class=\"footnotes\" data-footnotes>\n<ol>\n")?;
        }
        context.footnote_ix += 1;
        context.output.write_all(b"<li")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b" id=\"fn-")?;
        context.escape_href(nfd.name.as_bytes())?;
        context.output.write_all(b"\">")?;
    } else {
        if put_footnote_backref(context, nfd)? {
            context.output.write_all(b"\n")?;
        }
        context.output.write_all(b"</li>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_footnote_reference<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::FootnoteReference(ref nfr) = node.data.borrow().value else {
        panic!()
    };

    // Unreliable sourcepos.
    if entering {
        let mut ref_id = format!("fnref-{}", nfr.name);
        if nfr.ref_num > 1 {
            ref_id = format!("{}-{}", ref_id, nfr.ref_num);
        }

        context.output.write_all(b"<sup")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context
            .output
            .write_all(b" class=\"footnote-ref\"><a href=\"#fn-")?;
        context.escape_href(nfr.name.as_bytes())?;
        context.output.write_all(b"\" id=\"")?;
        context.escape_href(ref_id.as_bytes())?;
        write!(context.output, "\" data-footnote-ref>{}</a></sup>", nfr.ix)?;
    }

    Ok(true)
}

/// TODO
pub fn render_strikethrough<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<del")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</del>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_table<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<table")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">\n")?;
    } else {
        if !node
            .last_child()
            .unwrap()
            .same_node(node.first_child().unwrap())
        {
            context.cr()?;
            context.output.write_all(b"</tbody>\n")?;
        }
        context.cr()?;
        context.output.write_all(b"</table>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_table_cell<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let row = &node.parent().unwrap().data.borrow().value;
    let in_header = match *row {
        NodeValue::TableRow(header) => header,
        _ => panic!(),
    };

    let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
    let alignments = match *table {
        NodeValue::Table(NodeTable { ref alignments, .. }) => alignments,
        _ => panic!(),
    };

    if entering {
        context.cr()?;
        if in_header {
            context.output.write_all(b"<th")?;
            render_sourcepos(context, node)?;
        } else {
            context.output.write_all(b"<td")?;
            render_sourcepos(context, node)?;
        }

        let mut start = node.parent().unwrap().first_child().unwrap();
        let mut i = 0;
        while !start.same_node(node) {
            i += 1;
            start = start.next_sibling().unwrap();
        }

        match alignments[i] {
            TableAlignment::Left => {
                context.output.write_all(b" align=\"left\"")?;
            }
            TableAlignment::Right => {
                context.output.write_all(b" align=\"right\"")?;
            }
            TableAlignment::Center => {
                context.output.write_all(b" align=\"center\"")?;
            }
            TableAlignment::None => (),
        }

        context.output.write_all(b">")?;
    } else if in_header {
        context.output.write_all(b"</th>")?;
    } else {
        context.output.write_all(b"</td>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_table_row<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::TableRow(header) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        context.cr()?;
        if header {
            context.output.write_all(b"<thead>\n")?;
        } else if let Some(n) = node.previous_sibling() {
            if let NodeValue::TableRow(true) = n.data.borrow().value {
                context.output.write_all(b"<tbody>\n")?;
            }
        }
        context.output.write_all(b"<tr")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">")?;
    } else {
        context.cr()?;
        context.output.write_all(b"</tr>")?;
        if header {
            context.cr()?;
            context.output.write_all(b"</thead>")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_task_item<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::TaskItem(symbol) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        context.cr()?;
        context.output.write_all(b"<li")?;
        if context.options.render.tasklist_classes {
            context.output.write_all(b" class=\"task-list-item\"")?;
        }
        render_sourcepos(context, node)?;
        context.output.write_all(b">")?;
        context.output.write_all(b"<input type=\"checkbox\"")?;
        if context.options.render.tasklist_classes {
            context
                .output
                .write_all(b" class=\"task-list-item-checkbox\"")?;
        }
        if symbol.is_some() {
            context.output.write_all(b" checked=\"\"")?;
        }
        context.output.write_all(b" disabled=\"\" /> ")?;
    } else {
        context.output.write_all(b"</li>\n")?;
    }

    Ok(true)
}

// Extensions

/// TODO
pub fn render_alert<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Alert(ref alert) = node.data.borrow().value else {
        panic!()
    };

    if entering {
        context.cr()?;
        context.output.write_all(b"<div class=\"markdown-alert ")?;
        context
            .output
            .write_all(alert.alert_type.css_class().as_bytes())?;
        context.output.write_all(b"\"")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">\n")?;
        context
            .output
            .write_all(b"<p class=\"markdown-alert-title\">")?;
        match alert.title {
            Some(ref title) => context.escape(title.as_bytes())?,
            None => {
                context
                    .output
                    .write_all(alert.alert_type.default_title().as_bytes())?;
            }
        }
        context.output.write_all(b"</p>\n")?;
    } else {
        context.cr()?;
        context.output.write_all(b"</div>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_description_details<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.output.write_all(b"<dd")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</dd>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_description_item<'o, 'c, 'a>(
    _context: &mut Context<'o, 'c>,
    _node: &'a AstNode<'a>,
    _entering: bool,
) -> io::Result<bool> {
    Ok(true)
}

/// TODO
pub fn render_description_list<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<dl")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">\n")?;
    } else {
        context.output.write_all(b"</dl>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_description_term<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.output.write_all(b"<dt")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</dt>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_escaped<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if context.options.render.escaped_char_spans {
        if entering {
            context.output.write_all(b"<span data-escaped-char")?;
            if context.options.render.experimental_inline_sourcepos {
                render_sourcepos(context, node)?;
            }
            context.output.write_all(b">")?;
        } else {
            context.output.write_all(b"</span>")?;
        }
    }

    Ok(true)
}

/// TODO
pub fn render_escaped_tag<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    _entering: bool,
) -> io::Result<bool> {
    let NodeValue::EscapedTag(ref net) = node.data.borrow().value else {
        panic!()
    };

    // Nowhere to put sourcepos.
    context.output.write_all(net.as_bytes())?;

    Ok(true)
}

/// TODO
pub fn render_frontmatter<'o, 'c, 'a>(
    _context: &mut Context<'o, 'c>,
    _node: &'a AstNode<'a>,
    _entering: bool,
) -> io::Result<bool> {
    Ok(true)
}

/// Renders a math dollar inline, `$...$` and `$$...$$` using `<span>` to be
/// similar to other renderers.
pub fn render_math<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Math(NodeMath {
        ref literal,
        display_math,
        dollar_math,
        ..
    }) = node.data.borrow().value
    else {
        panic!()
    };

    if entering {
        let mut tag_attributes: Vec<(String, String)> = Vec::new();
        let style_attr = if display_math { "display" } else { "inline" };
        let tag: &str = if dollar_math { "span" } else { "code" };

        tag_attributes.push((String::from("data-math-style"), String::from(style_attr)));

        // Unreliable sourcepos.
        if context.options.render.experimental_inline_sourcepos && context.options.render.sourcepos
        {
            let ast = node.data.borrow();
            tag_attributes.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
        }

        write_opening_tag(context.output, tag, tag_attributes)?;
        context.escape(literal.as_bytes())?;
        write!(context.output, "</{}>", tag)?;
    }

    Ok(true)
}

/// Renders a math code block, ```` ```math ```` using `<pre><code>`.
pub fn render_math_code_block<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    literal: &String,
) -> io::Result<bool> {
    context.cr()?;

    // use vectors to ensure attributes always written in the same order,
    // for testing stability
    let mut pre_attributes: Vec<(String, String)> = Vec::new();
    let mut code_attributes: Vec<(String, String)> = Vec::new();
    let lang_str = "math";

    if context.options.render.github_pre_lang {
        pre_attributes.push((String::from("lang"), lang_str.to_string()));
        pre_attributes.push((String::from("data-math-style"), String::from("display")));
    } else {
        let code_attr = format!("language-{}", lang_str);
        code_attributes.push((String::from("class"), code_attr));
        code_attributes.push((String::from("data-math-style"), String::from("display")));
    }

    if context.options.render.sourcepos {
        let ast = node.data.borrow();
        pre_attributes.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
    }

    write_opening_tag(context.output, "pre", pre_attributes)?;
    write_opening_tag(context.output, "code", code_attributes)?;

    context.escape(literal.as_bytes())?;
    context.output.write_all(b"</code></pre>\n")?;

    Ok(true)
}

/// TODO
pub fn render_multiline_block_quote<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    if entering {
        context.cr()?;
        context.output.write_all(b"<blockquote")?;
        render_sourcepos(context, node)?;
        context.output.write_all(b">\n")?;
    } else {
        context.cr()?;
        context.output.write_all(b"</blockquote>\n")?;
    }

    Ok(true)
}

/// TODO
pub fn render_raw<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::Raw(ref literal) = node.data.borrow().value else {
        panic!()
    };

    // No sourcepos.
    if entering {
        context.output.write_all(literal.as_bytes())?;
    }

    Ok(true)
}

/// TODO
#[cfg(feature = "shortcodes")]
pub fn render_short_code<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::ShortCode(ref nsc) = node.data.borrow().value else {
        panic!()
    };

    // Nowhere to put sourcepos.
    if entering {
        context.output.write_all(nsc.emoji.as_bytes())?;
    }

    Ok(true)
}

/// TODO
pub fn render_spoiler_text<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<span")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b" class=\"spoiler\">")?;
    } else {
        context.output.write_all(b"</span>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_subscript<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<sub")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</sub>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_superscript<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<sup")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</sup>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_underline<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<u")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b">")?;
    } else {
        context.output.write_all(b"</u>")?;
    }

    Ok(true)
}

/// TODO
pub fn render_wiki_link<'o, 'c, 'a>(
    context: &mut Context<'o, 'c>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> io::Result<bool> {
    let NodeValue::WikiLink(ref nl) = node.data.borrow().value else {
        panic!()
    };

    // Unreliable sourcepos.
    if entering {
        context.output.write_all(b"<a")?;
        if context.options.render.experimental_inline_sourcepos {
            render_sourcepos(context, node)?;
        }
        context.output.write_all(b" href=\"")?;
        let url = nl.url.as_bytes();
        if context.options.render.unsafe_ || !dangerous_url(url) {
            context.escape_href(url)?;
        }
        context.output.write_all(b"\" data-wikilink=\"true")?;
        context.output.write_all(b"\">")?;
    } else {
        context.output.write_all(b"</a>")?;
    }

    Ok(true)
}

/// TODO
pub fn put_footnote_backref<'o, 'c>(
    context: &mut Context<'o, 'c>,
    nfd: &NodeFootnoteDefinition,
) -> io::Result<bool> {
    if context.written_footnote_ix >= context.footnote_ix {
        return Ok(false);
    }

    context.written_footnote_ix = context.footnote_ix;

    let mut ref_suffix = String::new();
    let mut superscript = String::new();

    for ref_num in 1..=nfd.total_references {
        if ref_num > 1 {
            ref_suffix = format!("-{}", ref_num);
            superscript = format!("<sup class=\"footnote-ref\">{}</sup>", ref_num);
            write!(context.output, " ")?;
        }

        context.output.write_all(b"<a href=\"#fnref-")?;
        context.escape_href(nfd.name.as_bytes())?;
        write!(
            context.output,
            "{}\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"{}{}\" aria-label=\"Back to reference {}{}\">↩{}</a>",
            ref_suffix, context.footnote_ix, ref_suffix, context.footnote_ix, ref_suffix, superscript
        )?;
    }
    Ok(true)
}

/// TODO
pub fn tagfilter(literal: &[u8]) -> bool {
    static TAGFILTER_BLACKLIST: [&str; 9] = [
        "title",
        "textarea",
        "style",
        "xmp",
        "iframe",
        "noembed",
        "noframes",
        "script",
        "plaintext",
    ];

    if literal.len() < 3 || literal[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if literal[i] == b'/' {
        i += 1;
    }

    let lc = unsafe { String::from_utf8_unchecked(literal[i..].to_vec()) }.to_lowercase();
    for t in TAGFILTER_BLACKLIST.iter() {
        if lc.starts_with(t) {
            let j = i + t.len();
            return isspace(literal[j])
                || literal[j] == b'>'
                || (literal[j] == b'/' && literal.len() >= j + 2 && literal[j + 1] == b'>');
        }
    }

    false
}

/// TODO
pub fn tagfilter_block(input: &[u8], o: &mut dyn Write) -> io::Result<()> {
    let size = input.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && input[i] != b'<' {
            i += 1;
        }

        if i > org {
            o.write_all(&input[org..i])?;
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            o.write_all(b"&lt;")?;
        } else {
            o.write_all(b"<")?;
        }

        i += 1;
    }

    Ok(())
}

/// TODO
pub fn dangerous_url(input: &[u8]) -> bool {
    scanners::dangerous_url(input).is_some()
}

/// Writes buffer to output, escaping anything that could be interpreted as an
/// HTML tag.
///
/// Namely:
///
/// * U+0022 QUOTATION MARK " is rendered as &quot;
/// * U+0026 AMPERSAND & is rendered as &amp;
/// * U+003C LESS-THAN SIGN < is rendered as &lt;
/// * U+003E GREATER-THAN SIGN > is rendered as &gt;
/// * Everything else is passed through unchanged.
///
/// Note that this is appropriate and sufficient for free text, but not for
/// URLs in attributes.  See escape_href.
pub fn escape(output: &mut dyn Write, buffer: &[u8]) -> io::Result<()> {
    const HTML_UNSAFE: [bool; 256] = character_set!(b"&<>\"");

    let mut offset = 0;
    for (i, &byte) in buffer.iter().enumerate() {
        if HTML_UNSAFE[byte as usize] {
            let esc: &[u8] = match byte {
                b'"' => b"&quot;",
                b'&' => b"&amp;",
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                _ => unreachable!(),
            };
            output.write_all(&buffer[offset..i])?;
            output.write_all(esc)?;
            offset = i + 1;
        }
    }
    output.write_all(&buffer[offset..])?;
    Ok(())
}

/// Writes buffer to output, escaping in a manner appropriate for URLs in HTML
/// attributes.
///
/// Namely:
///
/// * U+0026 AMPERSAND & is rendered as &amp;
/// * U+0027 APOSTROPHE ' is rendered as &#x27;
/// * Alphanumeric and a range of non-URL safe characters.
///
/// The inclusion of characters like "%" in those which are not escaped is
/// explained somewhat here:
///
/// <https://github.com/github/cmark-gfm/blob/c32ef78bae851cb83b7ad52d0fbff880acdcd44a/src/houdini_href_e.c#L7-L31>
///
/// In other words, if a CommonMark user enters:
///
/// ```markdown
/// [hi](https://ddg.gg/?q=a%20b)
/// ```
///
/// We assume they actually want the query string "?q=a%20b", a search for
/// the string "a b", rather than "?q=a%2520b", a search for the literal
/// string "a%20b".
pub fn escape_href(output: &mut dyn Write, buffer: &[u8]) -> io::Result<()> {
    const HREF_SAFE: [bool; 256] = character_set!(
        b"-_.+!*(),%#@?=;:/,+$~",
        b"abcdefghijklmnopqrstuvwxyz",
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    );

    let size = buffer.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && HREF_SAFE[buffer[i] as usize] {
            i += 1;
        }

        if i > org {
            output.write_all(&buffer[org..i])?;
        }

        if i >= size {
            break;
        }

        match buffer[i] as char {
            '&' => {
                output.write_all(b"&amp;")?;
            }
            '\'' => {
                output.write_all(b"&#x27;")?;
            }
            _ => write!(output, "%{:02X}", buffer[i])?,
        }

        i += 1;
    }

    Ok(())
}

/// Writes an opening HTML tag, using an iterator to enumerate the attributes.
/// Note that attribute values are automatically escaped.
pub fn write_opening_tag<Str>(
    output: &mut dyn Write,
    tag: &str,
    attributes: impl IntoIterator<Item = (Str, Str)>,
) -> io::Result<()>
where
    Str: AsRef<str>,
{
    write!(output, "<{}", tag)?;
    for (attr, val) in attributes {
        write!(output, " {}=\"", attr.as_ref())?;
        escape(output, val.as_ref().as_bytes())?;
        output.write_all(b"\"")?;
    }
    output.write_all(b">")?;
    Ok(())
}
