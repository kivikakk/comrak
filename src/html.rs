//! HTML renderering infrastructure for the CommonMark AST, as well as helper
//! functions. [`format_document`] and [`format_document_with_plugins`]
//! use the standard formatter. The
//! [`create_formatter!`][super::create_formatter] macro allows specialisation
//! of formatting for specific node types.

mod anchorizer;
mod context;

use crate::adapters::HeadingMeta;
use crate::character_set::character_set;
use crate::ctype::isspace;
use crate::nodes::{
    AstNode, ListType, NodeCode, NodeFootnoteDefinition, NodeMath, NodeTable, NodeValue,
    TableAlignment,
};
use crate::parser::{Options, Plugins};
use crate::{node_matches, scanners};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::str;

#[doc(hidden)]
pub use anchorizer::Anchorizer;
pub use context::Context;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(
    root: AstNode,
    options: &Options,
    output: &mut dyn Write,
) -> fmt::Result {
    format_document_with_formatter(
        root,
        options,
        output,
        &Plugins::default(),
        format_node_default,
        (),
    )
}

/// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
pub fn format_document_with_plugins<'a>(
    root: AstNode,
    options: &Options,
    output: &mut dyn fmt::Write,
    plugins: &Plugins,
) -> fmt::Result {
    format_document_with_formatter(root, options, output, plugins, format_node_default, ())
}

/// Returned by the [`format_document_with_formatter`] callback to indicate
/// whether children of a node should be rendered with full HTML as usual, in
/// "plain" mode, such as for the `title` attribute of an image (which is a full
/// subtree in CommonMark — there are probably few other use cases for "plain"
/// mode), or whether they should be skipped.
#[derive(Debug, Clone, Copy)]
pub enum ChildRendering {
    /// Indicates children should be rendered in full HTML as usual.
    HTML,
    /// Indicates children should be rendered in "plain" mode; see the source of
    /// [`format_document_with_formatter`] for details.
    Plain,
    /// Indicates children should be skipped.
    Skip,
}

/// Create a formatter with specialised rules for certain node types.
///
/// Give the name of the newly created struct, and then a list of [`NodeValue`]
/// match cases within curly braces. The left-hand side are regular patterns and
/// can include captures. The right-hand side starts with a mandatory list of
/// contextual captures, similar to lambdas. The following contextual captures
/// are available:
///
/// * `context`: the <code>[&mut] [Context]</code>, giving access to rendering
///   options, plugins, and output appending via its <code>[Write]</code>
///   implementation.
/// * `node`: the <code>[&][&][AstNode]</code> being formatted, when the
///   [`NodeValue`]'s contents aren't enough.
/// * `entering`: [`true`] when the node is being first descended into,
///   [`false`] when being exited.
///
/// By default, an overridden formatter will return [`ChildRendering::HTML`],
/// causing children of the node to be rendered as HTML as usual.  You can
/// return one of these enum values (wrapped in [`Ok`]) from within your
/// override to change this behaviour, in some or all cases.  These values are
/// only noted when `entering` a node.
///
/// If you supply a type parameter after the name of your formatter, it will be
/// taken as an additional argument on the generated `format_document` method,
/// is available on the [`Context`] as the `user` property, and becomes the
/// return value of `format_document`.
///
/// ```
/// # use comrak::{create_formatter, parse_document, Arena, Options, nodes::NodeValue, html::ChildRendering};
/// # use std::fmt::Write;
/// create_formatter!(CustomFormatter<usize>, {
///     NodeValue::Emph => |context, entering| {
///         context.user += 1;
///         if entering {
///             context.write_str("<i>")?;
///         } else {
///             context.write_str("</i>")?;
///         }
///     },
///     NodeValue::Strong => |context, entering| {
///         context.user += 1;
///         context.write_str(if entering { "<b>" } else { "</b>" })?;
///     },
///     NodeValue::Image(ref nl) => |context, node, entering| {
///         assert!(node.data.borrow().sourcepos == (3, 1, 3, 18).into());
///         if entering {
///             context.write_str(&nl.url.to_uppercase())?;
///             return Ok(ChildRendering::Skip);
///         }
///     },
/// });
///
/// let options = Options::default();
/// let arena = Arena::new();
/// let doc = parse_document(
///     &arena,
///     "_Hello_, **world**.\n\n![title](/img.png)",
///     &options,
/// );
///
/// let mut result: String = String::new();
/// let converted_count = CustomFormatter::format_document(doc, &options, &mut result, 0).unwrap();
///
/// assert_eq!(
///     result,
///     "<p><i>Hello</i>, <b>world</b>.</p>\n<p>/IMG.PNG</p>\n"
/// );
///
/// assert_eq!(converted_count, 4);
/// ```
#[macro_export]
macro_rules! create_formatter {
    // Permit lack of trailing comma by adding one.
    ($name:ident, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),* }) => {
        $crate::create_formatter!($name, { $( $pat => | $( $capture ),* | $case ),*, });
    };

    ($name:ident<$type:ty>, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),* }) => {
        $crate::create_formatter!($name<$type>, { $( $pat => | $( $capture ),* | $case ),*, });
    };

    ($name:ident, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),*, }) => {
        $crate::create_formatter!($name<()>, { $( $pat => | $( $capture ),* | $case ),*, });
    };

    // TODO: is there a nice way to deduplicate the below two clauses? When a
    // type isn't given, we default to `()`; in turn, we specialise the macro
    // when `()` is the type and supply the `()` value on the user's behalf.
    // This preserves the API from before the user type was added, and is just
    // neater/cleaner besides.
    //
    // If you are reading this comment, you might know of a nice way to do this!
    // I'd rather not resort to proc macros! TIA!
    ($name:ident<()>, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),*, }) => {
        #[allow(missing_copy_implementations)]
        #[allow(missing_debug_implementations)]
        /// Created by [`comrak::create_formatter!`][crate::create_formatter].
        pub struct $name;

        impl $name {
            /// Formats an AST as HTML, modified by the given options.
            #[inline]
            pub fn format_document<'a>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &$crate::Options,
                output: &mut dyn ::std::fmt::Write,
            ) -> ::std::fmt::Result {
                $crate::html::format_document_with_formatter(
                    root,
                    options,
                    output,
                    &$crate::Plugins::default(),
                    Self::formatter,
                    ()
                )
            }

            /// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
            #[inline]
            pub fn format_document_with_plugins<'a, 'o, 'c: 'o>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &'o $crate::Options<'c>,
                output: &'o mut dyn ::std::fmt::Write,
                plugins: &'o $crate::Plugins<'o>,
            ) -> ::std::fmt::Result {
                $crate::html::format_document_with_formatter(
                    root,
                    options,
                    output,
                    plugins,
                    Self::formatter,
                    ()
                )
            }

            fn formatter<'a>(
                context: &mut $crate::html::Context<()>,
                node: &'a $crate::nodes::AstNode<'a>,
                entering: bool,
            ) -> ::std::result::Result<$crate::html::ChildRendering, ::std::fmt::Error> {
                match node.data.borrow().value {
                    $(
                        $pat => {
                            $crate::formatter_captures!((context, node, entering), ($( $capture ),*));
                            $case
                            // Don't warn on unconditional return in user code.
                            #[allow(unreachable_code)]
                            ::std::result::Result::Ok($crate::html::ChildRendering::HTML)
                        }
                    ),*
                    _ => $crate::html::format_node_default(context, node, entering),
                }
            }
        }
    };

    ($name:ident<$type:ty>, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),*, }) => {
        #[allow(missing_copy_implementations)]
        #[allow(missing_debug_implementations)]
        /// Created by [`comrak::create_formatter!`][crate::create_formatter].
        pub struct $name;

        impl $name {
            /// Formats an AST as HTML, modified by the given options.
            #[inline]
            pub fn format_document<'a>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &$crate::Options,
                output: &mut dyn ::std::fmt::Write,
                user: $type,
            ) -> ::std::result::Result<$type, ::std::fmt::Error> {
                $crate::html::format_document_with_formatter(
                    root,
                    options,
                    output,
                    &$crate::Plugins::default(),
                    Self::formatter,
                    user
                )
            }

            /// Formats an AST as HTML, modified by the given options. Accepts custom plugins.
            #[inline]
            pub fn format_document_with_plugins<'a, 'o, 'c: 'o>(
                root: &'a $crate::nodes::AstNode<'a>,
                options: &'o $crate::Options<'c>,
                output: &'o mut dyn ::std::fmt::Write,
                plugins: &'o $crate::Plugins<'o>,
                user: $type,
            ) -> ::std::result::Result<$type, ::std::fmt::Error> {
                $crate::html::format_document_with_formatter(
                    root,
                    options,
                    output,
                    plugins,
                    Self::formatter,
                    user
                )
            }

            fn formatter<'a>(
                context: &mut $crate::html::Context<$type>,
                node: &'a $crate::nodes::AstNode<'a>,
                entering: bool,
            ) -> ::std::result::Result<$crate::html::ChildRendering, ::std::fmt::Error> {
                match node.data.borrow().value {
                    $(
                        $pat => {
                            $crate::formatter_captures!((context, node, entering), ($( $capture ),*));
                            $case
                            // Don't warn on unconditional return in user code.
                            #[allow(unreachable_code)]
                            ::std::result::Result::Ok($crate::html::ChildRendering::HTML)
                        }
                    ),*
                    _ => $crate::html::format_node_default(context, node, entering),
                }
            }
        }
    };
}

/// This must be exported so its uses in [`create_formatter!`] can be expanded,
/// but it's not intended for direct use.
#[doc(hidden)]
#[macro_export]
macro_rules! formatter_captures {
    (($context:ident, $node:ident, $entering:ident), context, $bind:ident) => {
        let $bind = $context;
    };
    (($context:ident, $node:ident, $entering:ident), node, $bind:ident) => {
        let $bind = $node;
    };
    (($context:ident, $node:ident, $entering:ident), entering, $bind:ident) => {
        let $bind = $entering;
    };
    (($context:ident, $node:ident, $entering:ident), $unknown:ident, $bind:ident) => {
        compile_error!(concat!("unknown capture '", stringify!($unknown), "'; available are 'context', 'node', 'entering'"));
    };
    (($context:ident, $node:ident, $entering:ident), ($capture:ident)) => {
        $crate::formatter_captures!(($context, $node, $entering), $capture, $capture);
    };
    (($context:ident, $node:ident, $entering:ident), ($capture:ident, $( $rest:ident ),*)) => {
        $crate::formatter_captures!(($context, $node, $entering), $capture, $capture);
        $crate::formatter_captures!(($context, $node, $entering), ($( $rest ),*));
    };
}

/// Formats the given AST with all options and formatter function specified.
///
/// The default formatter as used by [`format_document`] is
/// [`format_node_default`]. It is given the [`Context`], [`AstNode`], and a
/// boolean indicating whether the node is being entered into or exited.  The
/// returned [`ChildRendering`] is used to inform whether and how the node's
/// children are recursed into automatically.
pub fn format_document_with_formatter<'a, 'o, 'c: 'o, T>(
    root: AstNode,
    options: &'o Options<'c>,
    output: &'o mut dyn Write,
    plugins: &'o Plugins<'o>,
    formatter: fn(
        context: &mut Context<T>,
        node: AstNode,
        entering: bool,
    ) -> Result<ChildRendering, fmt::Error>,
    user: T,
) -> Result<T, fmt::Error> {
    // Traverse the AST iteratively using a work stack, with pre- and
    // post-child-traversal phases. During pre-order traversal render the
    // opening tags, then push the node back onto the stack for the
    // post-order traversal phase, then push the children in reverse order
    // onto the stack and begin rendering first child.

    let mut context = Context::new(output, options, plugins, user);

    enum Phase {
        Pre,
        Post,
    }
    let mut stack = vec![(root, ChildRendering::HTML, Phase::Pre)];

    while let Some((node, child_rendering, phase)) = stack.pop() {
        match phase {
            Phase::Pre => {
                let new_cr = match child_rendering {
                    ChildRendering::Plain => {
                        match node.data.borrow().value {
                            NodeValue::Text(ref literal)
                            | NodeValue::Code(NodeCode { ref literal, .. })
                            | NodeValue::HtmlInline(ref literal) => {
                                context.escape(literal)?;
                            }
                            NodeValue::LineBreak | NodeValue::SoftBreak => {
                                fmt::Write::write_str(&mut context, " ")?;
                            }
                            NodeValue::Math(NodeMath { ref literal, .. }) => {
                                context.escape(literal)?;
                            }
                            _ => (),
                        }
                        ChildRendering::Plain
                    }
                    ChildRendering::HTML => {
                        stack.push((node, ChildRendering::HTML, Phase::Post));
                        formatter(&mut context, node, true)?
                    }
                    ChildRendering::Skip => {
                        // We never push a node with ChildRendering::Skip.
                        unreachable!()
                    }
                };

                if !matches!(new_cr, ChildRendering::Skip) {
                    for ch in node.reverse_children() {
                        stack.push((ch, new_cr, Phase::Pre));
                    }
                }
            }
            Phase::Post => {
                debug_assert!(matches!(child_rendering, ChildRendering::HTML));
                formatter(&mut context, node, false)?;
            }
        }
    }

    context.finish()
}

/// Default node formatting function, used by [`format_document`],
/// [`format_document_with_plugins`] and as the fallback for any node types not
/// handled in custom formatters created by [`create_formatter!`].
#[inline]
pub fn format_node_default<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    match node.data.borrow().value {
        // Commonmark
        NodeValue::BlockQuote => render_block_quote(context, node, entering),
        NodeValue::Code(_) => render_code(context, node, entering),
        NodeValue::CodeBlock(_) => render_code_block(context, node, entering),
        NodeValue::Document => render_document(context, node, entering),
        NodeValue::Emph => render_emph(context, node, entering),
        NodeValue::Heading(_) => render_heading(context, node, entering),
        NodeValue::HtmlBlock(_) => render_html_block(context, node, entering),
        NodeValue::HtmlInline(_) => render_html_inline(context, node, entering),
        NodeValue::Image(_) => render_image(context, node, entering),
        NodeValue::Item(_) => render_item(context, node, entering),
        NodeValue::LineBreak => render_line_break(context, node, entering),
        NodeValue::Link(_) => render_link(context, node, entering),
        NodeValue::List(_) => render_list(context, node, entering),
        NodeValue::Paragraph => render_paragraph(context, node, entering),
        NodeValue::SoftBreak => render_soft_break(context, node, entering),
        NodeValue::Strong => render_strong(context, node, entering),
        NodeValue::Text(_) => render_text(context, node, entering),
        NodeValue::ThematicBreak => render_thematic_break(context, node, entering),

        // GFM
        NodeValue::FootnoteDefinition(_) => render_footnote_definition(context, node, entering),
        NodeValue::FootnoteReference(_) => render_footnote_reference(context, node, entering),
        NodeValue::Strikethrough => render_strikethrough(context, node, entering),
        NodeValue::Table(_) => render_table(context, node, entering),
        NodeValue::TableCell => render_table_cell(context, node, entering),
        NodeValue::TableRow(_) => render_table_row(context, node, entering),
        NodeValue::TaskItem(_) => render_task_item(context, node, entering),

        // Extensions
        NodeValue::Alert(_) => render_alert(context, node, entering),
        NodeValue::DescriptionDetails => render_description_details(context, node, entering),
        NodeValue::DescriptionItem(_) => render_description_item(context, node, entering),
        NodeValue::DescriptionList => render_description_list(context, node, entering),
        NodeValue::DescriptionTerm => render_description_term(context, node, entering),
        NodeValue::Escaped => render_escaped(context, node, entering),
        NodeValue::EscapedTag(_) => render_escaped_tag(context, node, entering),
        NodeValue::FrontMatter(_) => render_frontmatter(context, node, entering),
        NodeValue::Math(_) => render_math(context, node, entering),
        NodeValue::MultilineBlockQuote(_) => render_multiline_block_quote(context, node, entering),
        NodeValue::Raw(_) => render_raw(context, node, entering),
        #[cfg(feature = "shortcodes")]
        NodeValue::ShortCode(_) => render_short_code(context, node, entering),
        NodeValue::SpoileredText => render_spoiler_text(context, node, entering),
        NodeValue::Subscript => render_subscript(context, node, entering),
        NodeValue::Superscript => render_superscript(context, node, entering),
        NodeValue::Underline => render_underline(context, node, entering),
        NodeValue::WikiLink(_) => render_wiki_link(context, node, entering),
    }
}

// Commonmark

/// Renders sourcepos data for the given node to the supplied [`Context`].
///
/// This function renders anything iff `context.options.render.sourcepos` is
/// true, and includes a leading space if so, so you can use it  unconditionally
/// immediately before writing a closing `>` in your opening HTML tag.
pub fn render_sourcepos<'a, T>(context: &mut Context<T>, node: AstNode) -> fmt::Result {
    if context.options.render.sourcepos {
        let ast = node.data.borrow();
        if ast.sourcepos.start.line > 0 {
            write!(context, " data-sourcepos=\"{}\"", ast.sourcepos)?;
        }
    }
    Ok(())
}

fn render_block_quote<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<blockquote")?;
        render_sourcepos(context, node)?;
        context.write_str(">\n")?;
    } else {
        context.cr()?;
        context.write_str("</blockquote>\n")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_code<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Code(NodeCode { ref literal, .. }) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        context.write_str("<code")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
        context.escape(literal)?;
        context.write_str("</code>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_code_block<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::CodeBlock(ref ncb) = node.data.borrow().value else {
        unreachable!()
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

            let literal = &ncb.literal;
            let info = &ncb.info;
            let info_bytes = info.as_bytes();

            if !info.is_empty() {
                while first_tag < info.len() && !isspace(info_bytes[first_tag]) {
                    first_tag += 1;
                }

                let lang_str = &info[..first_tag];
                let info_str = &info[first_tag..].trim();

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
                    write_opening_tag(context, "pre", pre_attributes)?;
                    write_opening_tag(context, "code", code_attributes)?;

                    context.escape(literal)?;

                    context.write_str("</code></pre>\n")?
                }
                Some(highlighter) => {
                    highlighter.write_pre_tag(context, pre_attributes)?;
                    highlighter.write_code_tag(context, code_attributes)?;

                    highlighter.write_highlighted(
                        context,
                        Some(&info[..first_tag]),
                        &ncb.literal,
                    )?;

                    context.write_str("</code></pre>\n")?
                }
            }
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_document<'a, T>(
    _context: &mut Context<T>,
    _node: AstNode,
    _entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    Ok(ChildRendering::HTML)
}

fn render_emph<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<em")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</em>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_heading<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Heading(ref nh) = node.data.borrow().value else {
        unreachable!()
    };

    match context.plugins.render.heading_adapter {
        None => {
            if entering {
                context.cr()?;
                write!(context, "<h{}", nh.level)?;
                render_sourcepos(context, node)?;
                context.write_str(">")?;

                if let Some(ref prefix) = context.options.extension.header_ids {
                    let text_content = collect_text(node);
                    let id = context.anchorizer.anchorize(&text_content);
                    write!(
                        context,
                        "<a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}{}\"></a>",
                        id, prefix, id
                    )?;
                }
            } else {
                writeln!(context, "</h{}>", nh.level)?;
            }
        }
        Some(adapter) => {
            let text_content = collect_text(node);
            let heading = HeadingMeta {
                level: nh.level,
                content: text_content,
            };

            if entering {
                context.cr()?;
                let sp = if context.options.render.sourcepos {
                    Some(node.data.borrow().sourcepos)
                } else {
                    None
                };
                adapter.enter(context, &heading, sp)?;
            } else {
                adapter.exit(context, &heading)?;
            }
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_html_block<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::HtmlBlock(ref nhb) = node.data.borrow().value else {
        unreachable!()
    };

    // No sourcepos.
    if entering {
        context.cr()?;
        let literal = &nhb.literal;
        if context.options.render.escape {
            context.escape(literal)?;
        } else if !context.options.render.unsafe_ {
            context.write_str("<!-- raw HTML omitted -->")?;
        } else if context.options.extension.tagfilter {
            tagfilter_block(literal, context)?;
        } else {
            context.write_str(literal)?;
        }
        context.cr()?;
    }

    Ok(ChildRendering::HTML)
}

fn render_html_inline<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::HtmlInline(ref literal) = node.data.borrow().value else {
        unreachable!()
    };

    // No sourcepos.
    if entering {
        if context.options.render.escape {
            context.escape(literal)?;
        } else if !context.options.render.unsafe_ {
            context.write_str("<!-- raw HTML omitted -->")?;
        } else if context.options.extension.tagfilter && tagfilter(literal) {
            context.write_str("&lt;")?;
            context.write_str(&literal[1..])?;
        } else {
            context.write_str(literal)?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_image<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Image(ref nl) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        if context.options.render.figure_with_caption {
            context.write_str("<figure>")?;
        }
        context.write_str("<img")?;
        render_sourcepos(context, node)?;
        context.write_str(" src=\"")?;
        let url = &nl.url;
        if context.options.render.unsafe_ || !dangerous_url(url) {
            if let Some(rewriter) = &context.options.extension.image_url_rewriter {
                context.escape_href(&rewriter.to_html(&nl.url))?;
            } else {
                context.escape_href(url)?;
            }
        }
        context.write_str("\" alt=\"")?;
        return Ok(ChildRendering::Plain);
    } else {
        if !nl.title.is_empty() {
            context.write_str("\" title=\"")?;
            context.escape(&nl.title)?;
        }
        context.write_str("\" />")?;
        if context.options.render.figure_with_caption {
            if !nl.title.is_empty() {
                context.write_str("<figcaption>")?;
                context.escape(&nl.title)?;
                context.write_str("</figcaption>")?;
            }
            context.write_str("</figure>")?;
        };
    }

    Ok(ChildRendering::HTML)
}

fn render_item<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<li")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</li>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_line_break<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<br")?;
        render_sourcepos(context, node)?;
        context.write_str(" />\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_link<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Link(ref nl) = node.data.borrow().value else {
        unreachable!()
    };

    let parent_node = node.parent();

    if !context.options.parse.relaxed_autolinks
        || (parent_node.is_none()
            || !matches!(
                parent_node.unwrap().data.borrow().value,
                NodeValue::Link(..)
            ))
    {
        if entering {
            context.write_str("<a")?;
            render_sourcepos(context, node)?;
            context.write_str(" href=\"")?;
            let url = &nl.url;
            if context.options.render.unsafe_ || !dangerous_url(url) {
                if let Some(rewriter) = &context.options.extension.link_url_rewriter {
                    context.escape_href(&rewriter.to_html(&nl.url))?;
                } else {
                    context.escape_href(url)?;
                }
            }
            if !nl.title.is_empty() {
                context.write_str("\" title=\"")?;
                context.escape(&nl.title)?;
            }
            context.write_str("\">")?;
        } else {
            context.write_str("</a>")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_list<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::List(ref nl) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        context.cr()?;
        match nl.list_type {
            ListType::Bullet => {
                context.write_str("<ul")?;
                if nl.is_task_list && context.options.render.tasklist_classes {
                    context.write_str(" class=\"contains-task-list\"")?;
                }
                render_sourcepos(context, node)?;
                context.write_str(">\n")?;
            }
            ListType::Ordered => {
                context.write_str("<ol")?;
                if nl.is_task_list && context.options.render.tasklist_classes {
                    context.write_str(" class=\"contains-task-list\"")?;
                }
                render_sourcepos(context, node)?;
                if nl.start == 1 {
                    context.write_str(">\n")?;
                } else {
                    writeln!(context, " start=\"{}\">", nl.start)?;
                }
            }
        }
    } else if nl.list_type == ListType::Bullet {
        context.write_str("</ul>\n")?;
    } else {
        context.write_str("</ol>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_paragraph<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
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
            context.write_str("<p")?;
            render_sourcepos(context, node)?;
            context.write_str(">")?;
        } else {
            if let Some(NodeValue::FootnoteDefinition(nfd)) =
                &node.parent().map(|n| n.data.borrow().value.clone())
            {
                if node.next_sibling().is_none() {
                    context.write_str(" ")?;
                    put_footnote_backref(context, nfd)?;
                }
            }
            context.write_str("</p>\n")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_soft_break<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        if context.options.render.hardbreaks {
            context.write_str("<br")?;
            render_sourcepos(context, node)?;
            context.write_str(" />\n")?;
        } else {
            context.write_str("\n")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_strong<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let parent_node = node.parent();
    if !context.options.render.gfm_quirks
        || (parent_node.is_none()
            || !matches!(parent_node.unwrap().data.borrow().value, NodeValue::Strong))
    {
        if entering {
            context.write_str("<strong")?;
            render_sourcepos(context, node)?;
            context.write_str(">")?;
        } else {
            context.write_str("</strong>")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_text<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Text(ref literal) = node.data.borrow().value else {
        unreachable!()
    };

    // Nowhere to put sourcepos.
    if entering {
        context.escape(literal)?;
    }

    Ok(ChildRendering::HTML)
}

fn render_thematic_break<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<hr")?;
        render_sourcepos(context, node)?;
        context.write_str(" />\n")?;
    }

    Ok(ChildRendering::HTML)
}

// GFM

fn render_footnote_definition<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::FootnoteDefinition(ref nfd) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        if context.footnote_ix == 0 {
            context.write_str("<section")?;
            render_sourcepos(context, node)?;
            context.write_str(" class=\"footnotes\" data-footnotes>\n<ol>\n")?;
        }
        context.footnote_ix += 1;
        context.write_str("<li")?;
        render_sourcepos(context, node)?;
        context.write_str(" id=\"fn-")?;
        context.escape_href(&nfd.name)?;
        context.write_str("\">")?;
    } else {
        if put_footnote_backref(context, nfd)? {
            context.write_str("\n")?;
        }
        context.write_str("</li>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_footnote_reference<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::FootnoteReference(ref nfr) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        let mut ref_id = format!("fnref-{}", nfr.name);
        if nfr.ref_num > 1 {
            ref_id = format!("{}-{}", ref_id, nfr.ref_num);
        }

        context.write_str("<sup")?;
        render_sourcepos(context, node)?;
        context.write_str(" class=\"footnote-ref\"><a href=\"#fn-")?;
        context.escape_href(&nfr.name)?;
        context.write_str("\" id=\"")?;
        context.escape_href(&ref_id)?;
        write!(context, "\" data-footnote-ref>{}</a></sup>", nfr.ix)?;
    }

    Ok(ChildRendering::HTML)
}

fn render_strikethrough<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<del")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</del>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_table<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<table")?;
        render_sourcepos(context, node)?;
        context.write_str(">\n")?;
    } else {
        if let Some(true) = node
            .last_child()
            .map(|n| !n.same_node(node.first_child().unwrap()))
        // node.first_child() guaranteed to exist in block since last_child does!
        {
            context.cr()?;
            context.write_str("</tbody>\n")?;
        }
        context.cr()?;
        context.write_str("</table>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_table_cell<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let Some(row_node) = node.parent() else {
        panic!("rendered a table cell without a containing table row");
    };
    let row = &row_node.data.borrow().value;
    let in_header = match *row {
        NodeValue::TableRow(header) => header,
        _ => panic!("rendered a table cell contained by something other than a table row"),
    };

    let Some(table_node) = row_node.parent() else {
        panic!("rendered a table cell without a containing table");
    };
    let table = &table_node.data.borrow().value;
    let alignments = match *table {
        NodeValue::Table(NodeTable { ref alignments, .. }) => alignments,
        _ => {
            panic!("rendered a table cell in a table row contained by something other than a table")
        }
    };

    if entering {
        context.cr()?;
        if in_header {
            context.write_str("<th")?;
            render_sourcepos(context, node)?;
        } else {
            context.write_str("<td")?;
            render_sourcepos(context, node)?;
        }

        let mut start = row_node.first_child().unwrap(); // guaranteed to exist because `node' itself does!
        let mut i = 0;
        while !start.same_node(node) {
            i += 1;
            start = start.next_sibling().unwrap();
        }

        match alignments[i] {
            TableAlignment::Left => {
                context.write_str(" align=\"left\"")?;
            }
            TableAlignment::Right => {
                context.write_str(" align=\"right\"")?;
            }
            TableAlignment::Center => {
                context.write_str(" align=\"center\"")?;
            }
            TableAlignment::None => (),
        }

        context.write_str(">")?;
    } else if in_header {
        context.write_str("</th>")?;
    } else {
        context.write_str("</td>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_table_row<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::TableRow(header) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        context.cr()?;
        if header {
            context.write_str("<thead>\n")?;
        } else if let Some(n) = node.previous_sibling() {
            if let NodeValue::TableRow(true) = n.data.borrow().value {
                context.write_str("<tbody>\n")?;
            }
        }
        context.write_str("<tr")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.cr()?;
        context.write_str("</tr>")?;
        if header {
            context.cr()?;
            context.write_str("</thead>")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_task_item<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::TaskItem(symbol) = node.data.borrow().value else {
        unreachable!()
    };

    let write_li = node
        .parent()
        .map(|p| node_matches!(p, NodeValue::List(_)))
        .unwrap_or_default();

    if entering {
        context.cr()?;
        if write_li {
            context.write_str("<li")?;
            if context.options.render.tasklist_classes {
                context.write_str(" class=\"task-list-item\"")?;
            }
            render_sourcepos(context, node)?;
            context.write_str(">")?;
        }
        context.write_str("<input type=\"checkbox\"")?;
        if !write_li {
            render_sourcepos(context, node)?;
        }
        if context.options.render.tasklist_classes {
            context.write_str(" class=\"task-list-item-checkbox\"")?;
        }
        if symbol.is_some() {
            context.write_str(" checked=\"\"")?;
        }
        context.write_str(" disabled=\"\" /> ")?;
    } else if write_li {
        context.write_str("</li>\n")?;
    }

    Ok(ChildRendering::HTML)
}

// Extensions

fn render_alert<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Alert(ref alert) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        context.cr()?;
        context.write_str("<div class=\"markdown-alert ")?;
        context.write_str(&alert.alert_type.css_class())?;
        context.write_str("\"")?;
        render_sourcepos(context, node)?;
        context.write_str(">\n")?;
        context.write_str("<p class=\"markdown-alert-title\">")?;
        match alert.title {
            Some(ref title) => context.escape(title)?,
            None => {
                context.write_str(&alert.alert_type.default_title())?;
            }
        }
        context.write_str("</p>\n")?;
    } else {
        context.cr()?;
        context.write_str("</div>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_description_details<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<dd")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</dd>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_description_item<'a, T>(
    _context: &mut Context<T>,
    _node: AstNode,
    _entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    Ok(ChildRendering::HTML)
}

fn render_description_list<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<dl")?;
        render_sourcepos(context, node)?;
        context.write_str(">\n")?;
    } else {
        context.write_str("</dl>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_description_term<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<dt")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</dt>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_escaped<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if context.options.render.escaped_char_spans {
        if entering {
            context.write_str("<span data-escaped-char")?;
            render_sourcepos(context, node)?;
            context.write_str(">")?;
        } else {
            context.write_str("</span>")?;
        }
    }

    Ok(ChildRendering::HTML)
}

fn render_escaped_tag<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    _entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::EscapedTag(ref net) = node.data.borrow().value else {
        unreachable!()
    };

    // Nowhere to put sourcepos.
    context.write_str(net)?;

    Ok(ChildRendering::HTML)
}

fn render_frontmatter<'a, T>(
    _context: &mut Context<T>,
    _node: AstNode,
    _entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    Ok(ChildRendering::HTML)
}

/// Renders a math dollar inline, `$...$` and `$$...$$` using `<span>` to be
/// similar to other renderers.
pub fn render_math<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Math(NodeMath {
        ref literal,
        display_math,
        dollar_math,
        ..
    }) = node.data.borrow().value
    else {
        unreachable!()
    };

    if entering {
        let mut tag_attributes: Vec<(String, String)> = Vec::new();
        let style_attr = if display_math { "display" } else { "inline" };
        let tag: &str = if dollar_math { "span" } else { "code" };

        tag_attributes.push((String::from("data-math-style"), String::from(style_attr)));

        if context.options.render.sourcepos {
            let ast = node.data.borrow();
            tag_attributes.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
        }

        write_opening_tag(context, tag, tag_attributes)?;
        context.escape(literal)?;
        write!(context, "</{tag}>")?;
    }

    Ok(ChildRendering::HTML)
}

/// Renders a math code block, ```` ```math ```` using `<pre><code>`.
pub fn render_math_code_block<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    literal: &str,
) -> Result<ChildRendering, fmt::Error> {
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

    write_opening_tag(context, "pre", pre_attributes)?;
    write_opening_tag(context, "code", code_attributes)?;

    context.escape(literal)?;
    context.write_str("</code></pre>\n")?;

    Ok(ChildRendering::HTML)
}

fn render_multiline_block_quote<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<blockquote")?;
        render_sourcepos(context, node)?;
        context.write_str(">\n")?;
    } else {
        context.cr()?;
        context.write_str("</blockquote>\n")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_raw<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::Raw(ref literal) = node.data.borrow().value else {
        unreachable!()
    };

    // No sourcepos.
    if entering {
        context.write_str(literal)?;
    }

    Ok(ChildRendering::HTML)
}

#[cfg(feature = "shortcodes")]
fn render_short_code<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::ShortCode(ref nsc) = node.data.borrow().value else {
        unreachable!()
    };

    // Nowhere to put sourcepos.
    if entering {
        context.write_str(&nsc.emoji)?;
    }

    Ok(ChildRendering::HTML)
}

fn render_spoiler_text<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<span")?;
        render_sourcepos(context, node)?;
        context.write_str(" class=\"spoiler\">")?;
    } else {
        context.write_str("</span>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_subscript<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<sub")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</sub>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_superscript<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<sup")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</sup>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_underline<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<u")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</u>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_wiki_link<'a, T>(
    context: &mut Context<T>,
    node: AstNode,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let NodeValue::WikiLink(ref nl) = node.data.borrow().value else {
        unreachable!()
    };

    if entering {
        context.write_str("<a")?;
        render_sourcepos(context, node)?;
        context.write_str(" href=\"")?;
        let url = &nl.url;
        if context.options.render.unsafe_ || !dangerous_url(url) {
            context.escape_href(url)?;
        }
        context.write_str("\" data-wikilink=\"true")?;
        context.write_str("\">")?;
    } else {
        context.write_str("</a>")?;
    }

    Ok(ChildRendering::HTML)
}

// Helpers

/// Recurses through a node and all of its children in depth-first (document)
/// order, returning the concatenated literal contents of text, code and math
/// blocks. Line breaks and soft breaks are represented as a single whitespace
/// character.
pub fn collect_text<'a>(node: AstNode) -> String {
    let mut text = String::with_capacity(20);
    collect_text_append(node, &mut text);
    text
}

/// Recurses through a node and all of its children in depth-first (document)
/// order, appending the literal contents of text, code and math blocks to
/// an output buffer. Line breaks and soft breaks are represented as a single
/// whitespace character.
pub fn collect_text_append<'a>(node: AstNode, output: &mut String) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.push_str(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
        NodeValue::Math(NodeMath { ref literal, .. }) => output.push_str(literal),
        _ => {
            for n in node.children() {
                collect_text_append(n, output);
            }
        }
    }
}

fn put_footnote_backref<T>(
    context: &mut Context<T>,
    nfd: &NodeFootnoteDefinition,
) -> Result<bool, fmt::Error> {
    if context.written_footnote_ix >= context.footnote_ix {
        return Ok(false);
    }

    context.written_footnote_ix = context.footnote_ix;

    let mut ref_suffix = String::new();
    let mut superscript = String::new();

    for ref_num in 1..=nfd.total_references {
        if ref_num > 1 {
            ref_suffix = format!("-{ref_num}");
            superscript = format!("<sup class=\"footnote-ref\">{ref_num}</sup>");
            write!(context, " ")?;
        }

        context.write_str("<a href=\"#fnref-")?;
        context.escape_href(&nfd.name)?;
        let fnix = context.footnote_ix;
        write!(
            context,
            "{ref_suffix}\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"{fnix}{ref_suffix}\" aria-label=\"Back to reference {fnix}{ref_suffix}\">↩{superscript}</a>",
        )?;
    }
    Ok(true)
}

fn tagfilter(literal: &str) -> bool {
    let bytes = literal.as_bytes();

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

    if bytes.len() < 3 || bytes[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if bytes[i] == b'/' {
        i += 1;
    }

    let lc = literal[i..].to_lowercase();
    for t in TAGFILTER_BLACKLIST.iter() {
        if lc.starts_with(t) {
            let j = i + t.len();
            return isspace(bytes[j])
                || bytes[j] == b'>'
                || (bytes[j] == b'/' && bytes.len() >= j + 2 && bytes[j + 1] == b'>');
        }
    }

    false
}

fn tagfilter_block(input: &str, o: &mut dyn Write) -> fmt::Result {
    let bytes = input.as_bytes();
    let size = input.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && bytes[i] != b'<' {
            i += 1;
        }

        if i > org {
            o.write_str(&input[org..i])?;
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            o.write_str("&lt;")?;
        } else {
            o.write_str("<")?;
        }

        i += 1;
    }

    Ok(())
}

/// Check if the input would be considered a dangerous url
pub fn dangerous_url(input: &str) -> bool {
    scanners::dangerous_url(input.as_bytes()).is_some()
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
pub fn escape(output: &mut dyn Write, buffer: &str) -> fmt::Result {
    let bytes = buffer.as_bytes();
    const HTML_UNSAFE: [bool; 256] = character_set!(b"&<>\"");

    let mut offset = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if HTML_UNSAFE[byte as usize] {
            let esc: &str = match byte {
                b'"' => "&quot;",
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                _ => unreachable!(),
            };
            output.write_str(&buffer[offset..i])?;
            output.write_str(esc)?;
            offset = i + 1;
        }
    }
    output.write_str(&buffer[offset..])?;
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
///
/// We take some care with the square bracket characters `[` and `]`. We permit
/// them to be unescaped in the output iff they surround what reasonably appears
/// to be an IPv6 address in the host part of a URI.  If `relaxed_ipv6` is
/// `true`, any scheme is permitted for such an address.  Otherwise, only `http`
/// or `https` are permitted.
pub fn escape_href(output: &mut dyn Write, buffer: &str, relaxed_ipv6: bool) -> fmt::Result {
    const HREF_SAFE: [bool; 256] = character_set!(
        b"-_.+!*(),%#@?=;:/,+$~",
        b"abcdefghijklmnopqrstuvwxyz",
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    );

    let bytes = buffer.as_bytes();
    let size = buffer.len();
    let mut i = 0;

    let possible_ipv6_url_end = if relaxed_ipv6 {
        scanners::ipv6_relaxed_url_start(bytes)
    } else {
        scanners::ipv6_url_start(bytes)
    };
    if let Some(ipv6_url_end) = possible_ipv6_url_end {
        output.write_str(&buffer[0..ipv6_url_end])?;
        i = ipv6_url_end;
    }

    while i < size {
        let org = i;
        while i < size && HREF_SAFE[bytes[i] as usize] {
            i += 1;
        }

        if i > org {
            output.write_str(&buffer[org..i])?;
        }

        if i >= size {
            break;
        }

        match bytes[i] {
            b'&' => {
                output.write_str("&amp;")?;
            }
            b'\'' => {
                output.write_str("&#x27;")?;
            }
            _ => write!(output, "%{:02X}", bytes[i])?,
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
) -> fmt::Result
where
    Str: AsRef<str>,
{
    write!(output, "<{tag}")?;
    for (attr, val) in attributes {
        write!(output, " {}=\"", attr.as_ref())?;
        escape(output, val.as_ref())?;
        output.write_str("\"")?;
    }
    output.write_str(">")?;
    Ok(())
}
