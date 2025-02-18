//! The HTML renderer for the CommonMark AST, as well as helper functions.
use crate::character_set::character_set;
use crate::ctype::isspace;
use crate::nodes::AstNode;
use crate::parser::{Options, Plugins};
use crate::scanners;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::str;
use unicode_categories::UnicodeCategories;

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
    let mut writer = WriteWithLast {
        output,
        last_was_lf: Cell::new(true),
    };
    let mut f = HtmlFormatter::new(options, &mut writer, plugins);
    f.format(root, false)?;
    if f.footnote_ix > 0 {
        f.output.write_all(b"</ol>\n</section>\n")?;
    }
    Ok(())
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

/// Converts header strings to canonical, unique, but still human-readable,
/// anchors.
///
/// To guarantee uniqueness, an anchorizer keeps track of the anchors it has
/// returned; use one per output file.
///
/// ## Example
///
/// ```
/// # use comrak::Anchorizer;
/// let mut anchorizer = Anchorizer::new();
/// // First "stuff" is unsuffixed.
/// assert_eq!("stuff".to_string(), anchorizer.anchorize("Stuff".to_string()));
/// // Second "stuff" has "-1" appended to make it unique.
/// assert_eq!("stuff-1".to_string(), anchorizer.anchorize("Stuff".to_string()));
/// ```
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct Anchorizer(HashSet<String>);

impl Anchorizer {
    /// Construct a new anchorizer.
    pub fn new() -> Self {
        Anchorizer(HashSet::new())
    }

    /// Returns a String that has been converted into an anchor using the
    /// GFM algorithm, which involves changing spaces to dashes, removing
    /// problem characters and, if needed, adding a suffix to make the
    /// resultant anchor unique.
    ///
    /// ```
    /// # use comrak::Anchorizer;
    /// let mut anchorizer = Anchorizer::new();
    /// let source = "Ticks aren't in";
    /// assert_eq!("ticks-arent-in".to_string(), anchorizer.anchorize(source.to_string()));
    /// ```
    pub fn anchorize(&mut self, header: String) -> String {
        fn is_permitted_char(&c: &char) -> bool {
            c == ' '
                || c == '-'
                || c.is_letter()
                || c.is_mark()
                || c.is_number()
                || c.is_punctuation_connector()
        }

        let mut id = header.to_lowercase();
        id = id
            .chars()
            .filter(is_permitted_char)
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();

        let mut uniq = 0;
        id = loop {
            let anchor = if uniq == 0 {
                Cow::from(&id)
            } else {
                Cow::from(format!("{}-{}", id, uniq))
            };

            if !self.0.contains(&*anchor) {
                break anchor.into_owned();
            }

            uniq += 1;
        };
        self.0.insert(id.clone());
        id
    }
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

/// TODO
#[macro_export]
macro_rules! create_formatter {
    ($name:ident) => {
        create_formatter!($name, |_x, _y| {,});
    };
    ($name:ident, |$output:ident, $entering:ident| { $( $pat:pat => $case:tt ),*, }) => {
        /// TODO
        #[derive(Debug)]
        pub struct $name<'o, 'c> {
            output: &'o mut crate::html::WriteWithLast<'o>,
            options: &'o Options<'c>,
            anchorizer: Anchorizer,
            footnote_ix: u32,
            written_footnote_ix: u32,
            plugins: &'o Plugins<'o>,
        }


        impl<'o, 'c> $name<'o, 'c>
        where
            'c: 'o,
        {
            fn new(
                options: &'o Options<'c>,
                output: &'o mut crate::html::WriteWithLast<'o>,
                plugins: &'o Plugins,
            ) -> Self {
                $name {
                    options,
                    output,
                    anchorizer: Anchorizer::new(),
                    footnote_ix: 0,
                    written_footnote_ix: 0,
                    plugins,
                }
            }

            fn cr(&mut self) -> io::Result<()> {
                if !self.output.last_was_lf.get() {
                    self.output.write_all(b"\n")?;
                }
                Ok(())
            }

            fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
                crate::html::escape(&mut self.output, buffer)
            }

            fn escape_href(&mut self, buffer: &[u8]) -> io::Result<()> {
                crate::html::escape_href(&mut self.output, buffer)
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
                                    crate::nodes::NodeValue::Text(ref literal)
                                    | crate::nodes::NodeValue::Code(crate::nodes::NodeCode { ref literal, .. })
                                    | crate::nodes::NodeValue::HtmlInline(ref literal) => {
                                        self.escape(literal.as_bytes())?;
                                    }
                                    crate::nodes::NodeValue::LineBreak | crate::nodes::NodeValue::SoftBreak => {
                                        self.output.write_all(b" ")?;
                                    }
                                    crate::nodes::NodeValue::Math(crate::nodes::NodeMath { ref literal, .. }) => {
                                        self.escape(literal.as_bytes())?;
                                    }
                                    _ => (),
                                }
                                plain
                            } else {
                                stack.push((node, false, Phase::Post));
                                !self.format_node(node, true)?
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

            fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
                match node.data.borrow().value {
                    crate::nodes::NodeValue::Text(ref literal) | crate::nodes::NodeValue::Code(crate::nodes::NodeCode { ref literal, .. }) => {
                        output.extend_from_slice(literal.as_bytes())
                    }
                    crate::nodes::NodeValue::LineBreak | crate::nodes::NodeValue::SoftBreak => output.push(b' '),
                    crate::nodes::NodeValue::Math(crate::nodes::NodeMath { ref literal, .. }) => {
                        output.extend_from_slice(literal.as_bytes())
                    }
                    _ => {
                        for n in node.children() {
                            Self::collect_text(n, output);
                        }
                    }
                }
            }

            fn format_node<'a>(&mut self, node: &'a AstNode<'a>, $entering: bool) -> io::Result<bool> {
                match node.data.borrow().value {
                    $(
                        $pat => {
                            let $output = &mut self.output;
                            $case
                            Ok(true)
                        }
                    ),*
                    _ => {
                        Ok(self.format_node_default(node, $entering)?)
                    },
                }
            }

            fn format_node_default<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let processing_complete: bool = match node.data.borrow().value {
                    // Commonmark
                    crate::nodes::NodeValue::BlockQuote => self.render_block_quote(node, entering)?,
                    crate::nodes::NodeValue::Code(_) => self.render_code(node, entering)?,
                    crate::nodes::NodeValue::CodeBlock(_) => self.render_code_block(node, entering)?,
                    crate::nodes::NodeValue::Document => self.render_document(node, entering)?,
                    crate::nodes::NodeValue::Emph => self.render_emph(node, entering)?,
                    crate::nodes::NodeValue::Heading(_) => self.render_heading(node, entering)?,
                    crate::nodes::NodeValue::HtmlBlock(_) => self.render_html_block(node, entering)?,
                    crate::nodes::NodeValue::HtmlInline(_) => self.render_html_inline(node, entering)?,
                    crate::nodes::NodeValue::Image(_) => self.render_image(node, entering)?,
                    crate::nodes::NodeValue::Item(_) => self.render_item(node, entering)?,
                    crate::nodes::NodeValue::LineBreak => self.render_line_break(node, entering)?,
                    crate::nodes::NodeValue::Link(_) => self.render_link(node, entering)?,
                    crate::nodes::NodeValue::List(_) => self.render_list(node, entering)?,
                    crate::nodes::NodeValue::Paragraph => self.render_paragraph(node, entering)?,
                    crate::nodes::NodeValue::SoftBreak => self.render_soft_break(node, entering)?,
                    crate::nodes::NodeValue::Strong => self.render_strong(node, entering)?,
                    crate::nodes::NodeValue::Text(_) => self.render_text(node, entering)?,
                    crate::nodes::NodeValue::ThematicBreak => self.render_thematic_break(node, entering)?,

                    // GFM
                    crate::nodes::NodeValue::FootnoteDefinition(_) => self.render_footnote_definition(node, entering)?,
                    crate::nodes::NodeValue::FootnoteReference(_) => self.render_footnote_reference(node, entering)?,
                    crate::nodes::NodeValue::Strikethrough => self.render_strikethrough(node, entering)?,
                    crate::nodes::NodeValue::Table(_) => self.render_table(node, entering)?,
                    crate::nodes::NodeValue::TableCell => self.render_table_cell(node, entering)?,
                    crate::nodes::NodeValue::TableRow(_) => self.render_table_row(node, entering)?,
                    crate::nodes::NodeValue::TaskItem(_) => self.render_task_item(node, entering)?,

                    // Extensions
                    crate::nodes::NodeValue::Alert(_) => self.render_alert(node, entering)?,
                    crate::nodes::NodeValue::DescriptionDetails => self.render_description_details(node, entering)?,
                    crate::nodes::NodeValue::DescriptionItem(_) => self.render_description_item(node, entering)?,
                    crate::nodes::NodeValue::DescriptionList => self.render_description_list(node, entering)?,
                    crate::nodes::NodeValue::DescriptionTerm => self.render_description_term(node, entering)?,
                    crate::nodes::NodeValue::Escaped => self.render_escaped(node, entering)?,
                    crate::nodes::NodeValue::EscapedTag(_) => self.render_escaped_tag(node, entering)?,
                    crate::nodes::NodeValue::FrontMatter(_) => self.render_frontmatter(node, entering)?,
                    crate::nodes::NodeValue::Math(_) => self.render_math(node, entering)?,
                    crate::nodes::NodeValue::MultilineBlockQuote(_) => {
                        self.render_multiline_block_quote(node, entering)?
                    }
                    crate::nodes::NodeValue::Raw(_) => self.render_raw(node, entering)?,
                    #[cfg(feature = "shortcodes")]
                    crate::nodes::NodeValue::ShortCode(_) => self.render_short_code(node, entering)?,
                    crate::nodes::NodeValue::SpoileredText => self.render_spoiler_text(node, entering)?,
                    crate::nodes::NodeValue::Subscript => self.render_subscript(node, entering)?,
                    crate::nodes::NodeValue::Superscript => self.render_superscript(node, entering)?,
                    crate::nodes::NodeValue::Underline => self.render_underline(node, entering)?,
                    crate::nodes::NodeValue::WikiLink(_) => self.render_wiki_link(node, entering)?,
                };

                Ok(processing_complete)
            }

            // Commonmark

            fn render_block_quote<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<blockquote")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">\n")?;
                } else {
                    self.cr()?;
                    self.output.write_all(b"</blockquote>\n")?;
                }
                Ok(true)
            }

            fn render_code<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Code(crate::nodes::NodeCode { ref literal, .. }) = node.data.borrow().value else {
                    panic!()
                };

                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<code")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                    self.escape(literal.as_bytes())?;
                    self.output.write_all(b"</code>")?;
                }

                Ok(true)
            }

            fn render_code_block<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::CodeBlock(ref ncb) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    if ncb.info.eq("math") {
                        self.render_math_code_block(node, &ncb.literal)?;
                    } else {
                        self.cr()?;

                        let mut first_tag = 0;
                        let mut pre_attributes: HashMap<String, String> = HashMap::new();
                        let mut code_attributes: HashMap<String, String> = HashMap::new();
                        let code_attr: String;

                        let literal = &ncb.literal.as_bytes();
                        let info = &ncb.info.as_bytes();

                        if !info.is_empty() {
                            while first_tag < info.len() && !crate::ctype::isspace(info[first_tag]) {
                                first_tag += 1;
                            }

                            let lang_str = std::str::from_utf8(&info[..first_tag]).unwrap();
                            let info_str = std::str::from_utf8(&info[first_tag..]).unwrap().trim();

                            if self.options.render.github_pre_lang {
                                pre_attributes.insert(String::from("lang"), lang_str.to_string());

                                if self.options.render.full_info_string && !info_str.is_empty() {
                                    pre_attributes
                                        .insert(String::from("data-meta"), info_str.trim().to_string());
                                }
                            } else {
                                code_attr = format!("language-{}", lang_str);
                                code_attributes.insert(String::from("class"), code_attr);

                                if self.options.render.full_info_string && !info_str.is_empty() {
                                    code_attributes.insert(String::from("data-meta"), info_str.to_string());
                                }
                            }
                        }

                        if self.options.render.sourcepos {
                            let ast = node.data.borrow();
                            pre_attributes.insert("data-sourcepos".to_string(), ast.sourcepos.to_string());
                        }

                        match self.plugins.render.codefence_syntax_highlighter {
                            None => {
                                crate::html::write_opening_tag(self.output, "pre", pre_attributes)?;
                                crate::html::write_opening_tag(self.output, "code", code_attributes)?;

                                self.escape(literal)?;

                                self.output.write_all(b"</code></pre>\n")?
                            }
                            Some(highlighter) => {
                                highlighter.write_pre_tag(self.output, pre_attributes)?;
                                highlighter.write_code_tag(self.output, code_attributes)?;

                                highlighter.write_highlighted(
                                    self.output,
                                    match std::str::from_utf8(&info[..first_tag]) {
                                        Ok(lang) => Some(lang),
                                        Err(_) => None,
                                    },
                                    &ncb.literal,
                                )?;

                                self.output.write_all(b"</code></pre>\n")?
                            }
                        }
                    }
                }

                Ok(true)
            }

            fn render_document<'a>(&mut self, _node: &'a AstNode<'a>, _entering: bool) -> io::Result<bool> {
                Ok(true)
            }

            fn render_emph<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<em")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</em>")?;
                }

                Ok(true)
            }

            fn render_heading<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Heading(ref nch) = node.data.borrow().value else {
                    panic!()
                };

                match self.plugins.render.heading_adapter {
                    None => {
                        if entering {
                            self.cr()?;
                            write!(self.output, "<h{}", nch.level)?;
                            self.render_sourcepos(node)?;
                            self.output.write_all(b">")?;

                            if let Some(ref prefix) = self.options.extension.header_ids {
                                let mut text_content = Vec::with_capacity(20);
                                Self::collect_text(node, &mut text_content);

                                let mut id = String::from_utf8(text_content).unwrap();
                                id = self.anchorizer.anchorize(id);
                                write!(
                                    self.output,
                                    "<a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}{}\"></a>",
                                    id,
                                    prefix,
                                    id
                                )?;
                            }
                        } else {
                            writeln!(self.output, "</h{}>", nch.level)?;
                        }
                    }
                    Some(adapter) => {
                        let mut text_content = Vec::with_capacity(20);
                        Self::collect_text(node, &mut text_content);
                        let content = String::from_utf8(text_content).unwrap();
                        let heading = crate::adapters::HeadingMeta {
                            level: nch.level,
                            content,
                        };

                        if entering {
                            self.cr()?;
                            adapter.enter(
                                self.output,
                                &heading,
                                if self.options.render.sourcepos {
                                    Some(node.data.borrow().sourcepos)
                                } else {
                                    None
                                },
                            )?;
                        } else {
                            adapter.exit(self.output, &heading)?;
                        }
                    }
                }

                Ok(true)
            }

            fn render_html_block<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::HtmlBlock(ref nhb) = node.data.borrow().value else {
                    panic!()
                };

                // No sourcepos.
                if entering {
                    self.cr()?;
                    let literal = nhb.literal.as_bytes();
                    if self.options.render.escape {
                        self.escape(literal)?;
                    } else if !self.options.render.unsafe_ {
                        self.output.write_all(b"<!-- raw HTML omitted -->")?;
                    } else if self.options.extension.tagfilter {
                        crate::html::tagfilter_block(literal, &mut self.output)?;
                    } else {
                        self.output.write_all(literal)?;
                    }
                    self.cr()?;
                }

                Ok(true)
            }

            fn render_html_inline<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                let crate::nodes::NodeValue::HtmlInline(ref literal) = node.data.borrow().value else {
                    panic!()
                };

                // No sourcepos.
                if entering {
                    let literal = literal.as_bytes();
                    if self.options.render.escape {
                        self.escape(literal)?;
                    } else if !self.options.render.unsafe_ {
                        self.output.write_all(b"<!-- raw HTML omitted -->")?;
                    } else if self.options.extension.tagfilter && crate::html::tagfilter(literal) {
                        self.output.write_all(b"&lt;")?;
                        self.output.write_all(&literal[1..])?;
                    } else {
                        self.output.write_all(literal)?;
                    }
                }

                Ok(true)
            }

            fn render_image<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Image(ref nl) = node.data.borrow().value else {
                    panic!()
                };

                // Unreliable sourcepos.
                if entering {
                    if self.options.render.figure_with_caption {
                        self.output.write_all(b"<figure>")?;
                    }
                    self.output.write_all(b"<img")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b" src=\"")?;
                    let url = nl.url.as_bytes();
                    if self.options.render.unsafe_ || !crate::html::dangerous_url(url) {
                        if let Some(rewriter) = &self.options.extension.image_url_rewriter {
                            self.escape_href(rewriter.to_html(&nl.url).as_bytes())?;
                        } else {
                            self.escape_href(url)?;
                        }
                    }
                    self.output.write_all(b"\" alt=\"")?;
                    return Ok(false);
                } else {
                    if !nl.title.is_empty() {
                        self.output.write_all(b"\" title=\"")?;
                        self.escape(nl.title.as_bytes())?;
                    }
                    self.output.write_all(b"\" />")?;
                    if self.options.render.figure_with_caption {
                        if !nl.title.is_empty() {
                            self.output.write_all(b"<figcaption>")?;
                            self.escape(nl.title.as_bytes())?;
                            self.output.write_all(b"</figcaption>")?;
                        }
                        self.output.write_all(b"</figure>")?;
                    };
                }

                Ok(true)
            }

            fn render_item<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<li")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</li>\n")?;
                }

                Ok(true)
            }

            fn render_line_break<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<br")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b" />\n")?;
                }

                Ok(true)
            }

            fn render_link<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Link(ref nl) = node.data.borrow().value else {
                    panic!()
                };

                // Unreliable sourcepos.
                let parent_node = node.parent();

                if !self.options.parse.relaxed_autolinks
                    || (parent_node.is_none()
                        || !matches!(
                            parent_node.unwrap().data.borrow().value,
                            crate::nodes::NodeValue::Link(..)
                        ))
                {
                    if entering {
                        self.output.write_all(b"<a")?;
                        if self.options.render.experimental_inline_sourcepos {
                            self.render_sourcepos(node)?;
                        }
                        self.output.write_all(b" href=\"")?;
                        let url = nl.url.as_bytes();
                        if self.options.render.unsafe_ || !crate::html::dangerous_url(url) {
                            if let Some(rewriter) = &self.options.extension.link_url_rewriter {
                                self.escape_href(rewriter.to_html(&nl.url).as_bytes())?;
                            } else {
                                self.escape_href(url)?;
                            }
                        }
                        if !nl.title.is_empty() {
                            self.output.write_all(b"\" title=\"")?;
                            self.escape(nl.title.as_bytes())?;
                        }
                        self.output.write_all(b"\">")?;
                    } else {
                        self.output.write_all(b"</a>")?;
                    }
                }

                Ok(true)
            }

            fn render_list<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::List(ref nl) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    self.cr()?;
                    match nl.list_type {
                        crate::nodes::ListType::Bullet => {
                            self.output.write_all(b"<ul")?;
                            if nl.is_task_list && self.options.render.tasklist_classes {
                                self.output.write_all(b" class=\"contains-task-list\"")?;
                            }
                            self.render_sourcepos(node)?;
                            self.output.write_all(b">\n")?;
                        }
                        crate::nodes::ListType::Ordered => {
                            self.output.write_all(b"<ol")?;
                            if nl.is_task_list && self.options.render.tasklist_classes {
                                self.output.write_all(b" class=\"contains-task-list\"")?;
                            }
                            self.render_sourcepos(node)?;
                            if nl.start == 1 {
                                self.output.write_all(b">\n")?;
                            } else {
                                writeln!(self.output, " start=\"{}\">", nl.start)?;
                            }
                        }
                    }
                } else if nl.list_type == crate::nodes::ListType::Bullet {
                    self.output.write_all(b"</ul>\n")?;
                } else {
                    self.output.write_all(b"</ol>\n")?;
                }

                Ok(true)
            }

            fn render_paragraph<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let tight = match node
                    .parent()
                    .and_then(|n| n.parent())
                    .map(|n| n.data.borrow().value.clone())
                {
                    Some(crate::nodes::NodeValue::List(nl)) => nl.tight,
                    Some(crate::nodes::NodeValue::DescriptionItem(nd)) => nd.tight,
                    _ => false,
                };

                let tight = tight
                    || matches!(
                        node.parent().map(|n| n.data.borrow().value.clone()),
                        Some(crate::nodes::NodeValue::DescriptionTerm)
                    );

                if !tight {
                    if entering {
                        self.cr()?;
                        self.output.write_all(b"<p")?;
                        self.render_sourcepos(node)?;
                        self.output.write_all(b">")?;
                    } else {
                        if let crate::nodes::NodeValue::FootnoteDefinition(nfd) =
                            &node.parent().unwrap().data.borrow().value
                        {
                            if node.next_sibling().is_none() {
                                self.output.write_all(b" ")?;
                                self.put_footnote_backref(nfd)?;
                            }
                        }
                        self.output.write_all(b"</p>\n")?;
                    }
                }

                Ok(true)
            }

            fn render_soft_break<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    if self.options.render.hardbreaks {
                        self.output.write_all(b"<br")?;
                        if self.options.render.experimental_inline_sourcepos {
                            self.render_sourcepos(node)?;
                        }
                        self.output.write_all(b" />\n")?;
                    } else {
                        self.output.write_all(b"\n")?;
                    }
                }

                Ok(true)
            }

            fn render_strong<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                let parent_node = node.parent();
                if !self.options.render.gfm_quirks
                    || (parent_node.is_none()
                        || !matches!(parent_node.unwrap().data.borrow().value, crate::nodes::NodeValue::Strong))
                {
                    if entering {
                        self.output.write_all(b"<strong")?;
                        if self.options.render.experimental_inline_sourcepos {
                            self.render_sourcepos(node)?;
                        }
                        self.output.write_all(b">")?;
                    } else {
                        self.output.write_all(b"</strong>")?;
                    }
                }

                Ok(true)
            }

            fn render_text<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Text(ref literal) = node.data.borrow().value else {
                    panic!()
                };

                // Nowhere to put sourcepos.
                if entering {
                    self.escape(literal.as_bytes())?;
                }

                Ok(true)
            }

            fn render_thematic_break<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<hr")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b" />\n")?;
                }

                Ok(true)
            }

            // GFM

            fn render_footnote_definition<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                let crate::nodes::NodeValue::FootnoteDefinition(ref nfd) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    if self.footnote_ix == 0 {
                        self.output.write_all(b"<section")?;
                        self.render_sourcepos(node)?;
                        self.output
                            .write_all(b" class=\"footnotes\" data-footnotes>\n<ol>\n")?;
                    }
                    self.footnote_ix += 1;
                    self.output.write_all(b"<li")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b" id=\"fn-")?;
                    self.escape_href(nfd.name.as_bytes())?;
                    self.output.write_all(b"\">")?;
                } else {
                    if self.put_footnote_backref(nfd)? {
                        self.output.write_all(b"\n")?;
                    }
                    self.output.write_all(b"</li>\n")?;
                }

                Ok(true)
            }

            fn render_footnote_reference<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                let crate::nodes::NodeValue::FootnoteReference(ref nfr) = node.data.borrow().value else {
                    panic!()
                };

                // Unreliable sourcepos.
                if entering {
                    let mut ref_id = format!("fnref-{}", nfr.name);
                    if nfr.ref_num > 1 {
                        ref_id = format!("{}-{}", ref_id, nfr.ref_num);
                    }

                    self.output.write_all(b"<sup")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output
                        .write_all(b" class=\"footnote-ref\"><a href=\"#fn-")?;
                    self.escape_href(nfr.name.as_bytes())?;
                    self.output.write_all(b"\" id=\"")?;
                    self.escape_href(ref_id.as_bytes())?;
                    write!(self.output, "\" data-footnote-ref>{}</a></sup>", nfr.ix)?;
                }

                Ok(true)
            }

            fn render_strikethrough<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<del")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</del>")?;
                }

                Ok(true)
            }

            fn render_table<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<table")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">\n")?;
                } else {
                    if !node
                        .last_child()
                        .unwrap()
                        .same_node(node.first_child().unwrap())
                    {
                        self.cr()?;
                        self.output.write_all(b"</tbody>\n")?;
                    }
                    self.cr()?;
                    self.output.write_all(b"</table>\n")?;
                }

                Ok(true)
            }

            fn render_table_cell<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let row = &node.parent().unwrap().data.borrow().value;
                let in_header = match *row {
                    crate::nodes::NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                let table = &node.parent().unwrap().parent().unwrap().data.borrow().value;
                let alignments = match *table {
                    crate::nodes::NodeValue::Table(crate::nodes::NodeTable { ref alignments, .. }) => alignments,
                    _ => panic!(),
                };

                if entering {
                    self.cr()?;
                    if in_header {
                        self.output.write_all(b"<th")?;
                        self.render_sourcepos(node)?;
                    } else {
                        self.output.write_all(b"<td")?;
                        self.render_sourcepos(node)?;
                    }

                    let mut start = node.parent().unwrap().first_child().unwrap();
                    let mut i = 0;
                    while !start.same_node(node) {
                        i += 1;
                        start = start.next_sibling().unwrap();
                    }

                    match alignments[i] {
                        crate::nodes::TableAlignment::Left => {
                            self.output.write_all(b" align=\"left\"")?;
                        }
                        crate::nodes::TableAlignment::Right => {
                            self.output.write_all(b" align=\"right\"")?;
                        }
                        crate::nodes::TableAlignment::Center => {
                            self.output.write_all(b" align=\"center\"")?;
                        }
                        crate::nodes::TableAlignment::None => (),
                    }

                    self.output.write_all(b">")?;
                } else if in_header {
                    self.output.write_all(b"</th>")?;
                } else {
                    self.output.write_all(b"</td>")?;
                }

                Ok(true)
            }

            fn render_table_row<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::TableRow(header) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    self.cr()?;
                    if header {
                        self.output.write_all(b"<thead>\n")?;
                    } else if let Some(n) = node.previous_sibling() {
                        if let crate::nodes::NodeValue::TableRow(true) = n.data.borrow().value {
                            self.output.write_all(b"<tbody>\n")?;
                        }
                    }
                    self.output.write_all(b"<tr")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">")?;
                } else {
                    self.cr()?;
                    self.output.write_all(b"</tr>")?;
                    if header {
                        self.cr()?;
                        self.output.write_all(b"</thead>")?;
                    }
                }

                Ok(true)
            }

            fn render_task_item<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::TaskItem(symbol) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    self.cr()?;
                    self.output.write_all(b"<li")?;
                    if self.options.render.tasklist_classes {
                        self.output.write_all(b" class=\"task-list-item\"")?;
                    }
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">")?;
                    self.output.write_all(b"<input type=\"checkbox\"")?;
                    if self.options.render.tasklist_classes {
                        self.output
                            .write_all(b" class=\"task-list-item-checkbox\"")?;
                    }
                    if symbol.is_some() {
                        self.output.write_all(b" checked=\"\"")?;
                    }
                    self.output.write_all(b" disabled=\"\" /> ")?;
                } else {
                    self.output.write_all(b"</li>\n")?;
                }

                Ok(true)
            }

            // Extensions

            fn render_alert<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Alert(ref alert) = node.data.borrow().value else {
                    panic!()
                };

                if entering {
                    self.cr()?;
                    self.output.write_all(b"<div class=\"markdown-alert ")?;
                    self.output
                        .write_all(alert.alert_type.css_class().as_bytes())?;
                    self.output.write_all(b"\"")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">\n")?;
                    self.output
                        .write_all(b"<p class=\"markdown-alert-title\">")?;
                    match alert.title {
                        Some(ref title) => self.escape(title.as_bytes())?,
                        None => {
                            self.output
                                .write_all(alert.alert_type.default_title().as_bytes())?;
                        }
                    }
                    self.output.write_all(b"</p>\n")?;
                } else {
                    self.cr()?;
                    self.output.write_all(b"</div>\n")?;
                }

                Ok(true)
            }

            fn render_description_details<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.output.write_all(b"<dd")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</dd>\n")?;
                }

                Ok(true)
            }

            fn render_description_item<'a>(
                &mut self,
                _node: &'a AstNode<'a>,
                _entering: bool,
            ) -> io::Result<bool> {
                Ok(true)
            }

            fn render_description_list<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<dl")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">\n")?;
                } else {
                    self.output.write_all(b"</dl>\n")?;
                }

                Ok(true)
            }

            fn render_description_term<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.output.write_all(b"<dt")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</dt>\n")?;
                }

                Ok(true)
            }

            fn render_escaped<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if self.options.render.escaped_char_spans {
                    if entering {
                        self.output.write_all(b"<span data-escaped-char")?;
                        if self.options.render.experimental_inline_sourcepos {
                            self.render_sourcepos(node)?;
                        }
                        self.output.write_all(b">")?;
                    } else {
                        self.output.write_all(b"</span>")?;
                    }
                }

                Ok(true)
            }

            fn render_escaped_tag<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                _entering: bool,
            ) -> io::Result<bool> {
                let crate::nodes::NodeValue::EscapedTag(ref net) = node.data.borrow().value else {
                    panic!()
                };

                // Nowhere to put sourcepos.
                self.output.write_all(net.as_bytes())?;

                Ok(true)
            }

            fn render_frontmatter<'a>(
                &mut self,
                _node: &'a AstNode<'a>,
                _entering: bool,
            ) -> io::Result<bool> {
                Ok(true)
            }

            // Renders a math dollar inline, `$...$` and `$$...$$` using `<span>` to be similar
            // to other renderers.
            fn render_math<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Math(crate::nodes::NodeMath {
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
                    if self.options.render.experimental_inline_sourcepos && self.options.render.sourcepos {
                        let ast = node.data.borrow();
                        tag_attributes.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
                    }

                    crate::html::write_opening_tag(self.output, tag, tag_attributes)?;
                    self.escape(literal.as_bytes())?;
                    write!(self.output, "</{}>", tag)?;
                }

                Ok(true)
            }

            // Renders a math code block, ```` ```math ```` using `<pre><code>`
            fn render_math_code_block<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                literal: &String,
            ) -> io::Result<bool> {
                self.cr()?;

                // use vectors to ensure attributes always written in the same order,
                // for testing stability
                let mut pre_attributes: Vec<(String, String)> = Vec::new();
                let mut code_attributes: Vec<(String, String)> = Vec::new();
                let lang_str = "math";

                if self.options.render.github_pre_lang {
                    pre_attributes.push((String::from("lang"), lang_str.to_string()));
                    pre_attributes.push((String::from("data-math-style"), String::from("display")));
                } else {
                    let code_attr = format!("language-{}", lang_str);
                    code_attributes.push((String::from("class"), code_attr));
                    code_attributes.push((String::from("data-math-style"), String::from("display")));
                }

                if self.options.render.sourcepos {
                    let ast = node.data.borrow();
                    pre_attributes.push(("data-sourcepos".to_string(), ast.sourcepos.to_string()));
                }

                crate::html::write_opening_tag(self.output, "pre", pre_attributes)?;
                crate::html::write_opening_tag(self.output, "code", code_attributes)?;

                self.escape(literal.as_bytes())?;
                self.output.write_all(b"</code></pre>\n")?;

                Ok(true)
            }

            fn render_multiline_block_quote<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                if entering {
                    self.cr()?;
                    self.output.write_all(b"<blockquote")?;
                    self.render_sourcepos(node)?;
                    self.output.write_all(b">\n")?;
                } else {
                    self.cr()?;
                    self.output.write_all(b"</blockquote>\n")?;
                }

                Ok(true)
            }

            fn render_raw<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::Raw(ref literal) = node.data.borrow().value else {
                    panic!()
                };

                // No sourcepos.
                if entering {
                    self.output.write_all(literal.as_bytes())?;
                }

                Ok(true)
            }

            #[cfg(feature = "shortcodes")]
            fn render_short_code<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let NodeValue::ShortCode(ref nsc) = node.data.borrow().value else {
                    panic!()
                };

                // Nowhere to put sourcepos.
                if entering {
                    self.output.write_all(nsc.emoji.as_bytes())?;
                }

                Ok(true)
            }

            fn render_spoiler_text<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<span")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b" class=\"spoiler\">")?;
                } else {
                    self.output.write_all(b"</span>")?;
                }

                Ok(true)
            }

            fn render_subscript<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<sub")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</sub>")?;
                }

                Ok(true)
            }

            fn render_superscript<'a>(
                &mut self,
                node: &'a AstNode<'a>,
                entering: bool,
            ) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<sup")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</sup>")?;
                }

                Ok(true)
            }

            fn render_underline<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<u")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b">")?;
                } else {
                    self.output.write_all(b"</u>")?;
                }

                Ok(true)
            }

            fn render_wiki_link<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> io::Result<bool> {
                let crate::nodes::NodeValue::WikiLink(ref nl) = node.data.borrow().value else {
                    panic!()
                };

                // Unreliable sourcepos.
                if entering {
                    self.output.write_all(b"<a")?;
                    if self.options.render.experimental_inline_sourcepos {
                        self.render_sourcepos(node)?;
                    }
                    self.output.write_all(b" href=\"")?;
                    let url = nl.url.as_bytes();
                    if self.options.render.unsafe_ || !crate::html::dangerous_url(url) {
                        self.escape_href(url)?;
                    }
                    self.output.write_all(b"\" data-wikilink=\"true")?;
                    self.output.write_all(b"\">")?;
                } else {
                    self.output.write_all(b"</a>")?;
                }

                Ok(true)
            }

            fn render_sourcepos<'a>(&mut self, node: &'a AstNode<'a>) -> io::Result<()> {
                if self.options.render.sourcepos {
                    let ast = node.data.borrow();
                    if ast.sourcepos.start.line > 0 {
                        write!(self.output, " data-sourcepos=\"{}\"", ast.sourcepos)?;
                    }
                }
                Ok(())
            }

            fn put_footnote_backref(&mut self, nfd: &crate::nodes::NodeFootnoteDefinition) -> io::Result<bool> {
                if self.written_footnote_ix >= self.footnote_ix {
                    return Ok(false);
                }

                self.written_footnote_ix = self.footnote_ix;

                let mut ref_suffix = String::new();
                let mut superscript = String::new();

                for ref_num in 1..=nfd.total_references {
                    if ref_num > 1 {
                        ref_suffix = format!("-{}", ref_num);
                        superscript = format!("<sup class=\"footnote-ref\">{}</sup>", ref_num);
                        write!(self.output, " ")?;
                    }

                    self.output.write_all(b"<a href=\"#fnref-")?;
                    self.escape_href(nfd.name.as_bytes())?;
                    write!(
                        self.output,
                        "{}\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"{}{}\" aria-label=\"Back to reference {}{}\">↩{}</a>",
                        ref_suffix, self.footnote_ix, ref_suffix, self.footnote_ix, ref_suffix, superscript
                    )?;
                }
                Ok(true)
            }
        }
    };
}

create_formatter!(HtmlFormatter);
