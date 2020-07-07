mod autolink;
mod inlines;
mod table;

use arena_tree::Node;
use ctype::{isdigit, isspace};
use entity;
use nodes;
use nodes::{
    Ast, AstNode, ListDelimType, ListType, NodeCodeBlock, NodeDescriptionItem,
    NodeHeading, NodeHtmlBlock, NodeList, NodeValue,
};
use regex::bytes::Regex;
use scanners;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::mem;
use std::str;
use strings;
use typed_arena::Arena;

const TAB_STOP: usize = 4;
const CODE_INDENT: usize = 4;

macro_rules! node_matches {
    ($node:expr, $pat:pat) => {{
        match $node.data.borrow().value {
            $pat => true,
            _ => false,
        }
    }};
}

/// Parse a Markdown document to an AST.
///
/// See the documentation of the crate root for an example.
pub fn parse_document<'a>(
    arena: &'a Arena<AstNode<'a>>,
    buffer: &str,
    options: &ComrakOptions,
) -> &'a AstNode<'a> {
    parse_document_with_broken_link_callback(arena, buffer, options, None)
}

/// Parse a Markdown document to an AST.
///
/// In case the parser encounters any potential links that have a broken reference (e.g `[foo]`
/// when there is no `[foo]: url` entry at the bottom) the provided callback will be called with
/// the reference name, and the returned pair will be used as the link destination and title if not
/// None.
///
/// **Note:** The label provided to the callback is the normalized representation of the label as
/// described in the [GFM spec](https://github.github.com/gfm/#matches).
///
/// ```
/// extern crate comrak;
/// use comrak::{Arena, parse_document_with_broken_link_callback, format_html, ComrakOptions};
/// use comrak::nodes::{AstNode, NodeValue};
///
/// # fn main() -> std::io::Result<()> {
/// // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
/// let arena = Arena::new();
///
/// let root = parse_document_with_broken_link_callback(
///     &arena,
///     "# Cool input!\nWow look at this cool [link][foo]. A [broken link] renders as text.",
///     &ComrakOptions::default(),
///     Some(&mut |link_ref: &[u8]| match link_ref {
///         b"foo" => Some((
///             b"https://www.rust-lang.org/".to_vec(),
///             b"The Rust Language".to_vec(),
///         )),
///         _ => None,
///     }),
/// );
///
/// let mut output = Vec::new();
/// format_html(root, &ComrakOptions::default(), &mut output)?;
/// let output_str = std::str::from_utf8(&output).expect("invalid UTF-8");
/// assert_eq!(output_str, "<h1>Cool input!</h1>\n<p>Wow look at this cool \
///                 <a href=\"https://www.rust-lang.org/\" title=\"The Rust Language\">link</a>. \
///                 A [broken link] renders as text.</p>\n");
/// # Ok(())
/// # }
/// ```
pub fn parse_document_with_broken_link_callback<'a, 'c>(
    arena: &'a Arena<AstNode<'a>>,
    buffer: &str,
    options: &ComrakOptions,
    callback: Option<&'c mut dyn FnMut(&[u8]) -> Option<(Vec<u8>, Vec<u8>)>>,
) -> &'a AstNode<'a> {
    let root: &'a AstNode<'a> = arena.alloc(Node::new(RefCell::new(Ast {
        value: NodeValue::Document,
        content: vec![],
        start_line: 0,
        open: true,
        last_line_blank: false,
    })));
    let mut parser = Parser::new(arena, root, options, callback);
    parser.feed(buffer);
    parser.finish()
}

pub struct Parser<'a, 'o, 'c> {
    arena: &'a Arena<AstNode<'a>>,
    refmap: HashMap<Vec<u8>, Reference>,
    root: &'a AstNode<'a>,
    current: &'a AstNode<'a>,
    line_number: u32,
    offset: usize,
    column: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    partially_consumed_tab: bool,
    last_line_length: usize,
    options: &'o ComrakOptions,
    callback: Option<&'c mut dyn FnMut(&[u8]) -> Option<(Vec<u8>, Vec<u8>)>>,
}

#[derive(Default, Debug, Clone)]
/// Umbrella options struct.
pub struct ComrakOptions {
    /// Enable CommonMark extensions.
    pub extension: ComrakExtensionOptions,

    /// Configure parse-time options.
    pub parse: ComrakParseOptions,

    /// Configure render-time options.
    pub render: ComrakRenderOptions,
}

#[derive(Default, Debug, Clone)]
/// Options to select extensions.
pub struct ComrakExtensionOptions {
    /// Enables the
    /// [strikethrough extension](https://github.github.com/gfm/#strikethrough-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.strikethrough = true;
    /// assert_eq!(markdown_to_html("Hello ~world~ there.\n", &options),
    ///            "<p>Hello <del>world</del> there.</p>\n");
    /// ```
    pub strikethrough: bool,

    /// Enables the
    /// [tagfilter extension](https://github.github.com/gfm/#disallowed-raw-html-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.tagfilter = true;
    /// options.render.unsafe_ = true;
    /// assert_eq!(markdown_to_html("Hello <xmp>.\n\n<xmp>", &options),
    ///            "<p>Hello &lt;xmp>.</p>\n&lt;xmp>\n");
    /// ```
    pub tagfilter: bool,

    /// Enables the [table extension](https://github.github.com/gfm/#tables-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.table = true;
    /// assert_eq!(markdown_to_html("| a | b |\n|---|---|\n| c | d |\n", &options),
    ///            "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n\
    ///             <tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n");
    /// ```
    pub table: bool,

    /// Enables the [autolink extension](https://github.github.com/gfm/#autolinks-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.autolink = true;
    /// assert_eq!(markdown_to_html("Hello www.github.com.\n", &options),
    ///            "<p>Hello <a href=\"http://www.github.com\">www.github.com</a>.</p>\n");
    /// ```
    pub autolink: bool,

    /// Enables the
    /// [task list items extension](https://github.github.com/gfm/#task-list-items-extension-)
    /// from the GFM spec.
    ///
    /// Note that the spec does not define the precise output, so only the bare essentials are
    /// rendered.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.tasklist = true;
    /// options.render.unsafe_ = true;
    /// assert_eq!(markdown_to_html("* [x] Done\n* [ ] Not done\n", &options),
    ///            "<ul>\n<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Done</li>\n\
    ///            <li><input type=\"checkbox\" disabled=\"\" /> Not done</li>\n</ul>\n");
    /// ```
    pub tasklist: bool,

    /// Enables the superscript Comrak extension.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.superscript = true;
    /// assert_eq!(markdown_to_html("e = mc^2^.\n", &options),
    ///            "<p>e = mc<sup>2</sup>.</p>\n");
    /// ```
    pub superscript: bool,

    /// Enables the header IDs Comrak extension.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.header_ids = Some("user-content-".to_string());
    /// assert_eq!(markdown_to_html("# README\n", &options),
    ///            "<h1><a href=\"#readme\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-readme\"></a>README</h1>\n");
    /// ```
    pub header_ids: Option<String>,

    /// Enables the footnotes extension per `cmark-gfm`.
    ///
    /// For usage, see `src/tests.rs`.  The extension is modelled after
    /// [Kramdown](https://kramdown.gettalong.org/syntax.html#footnotes).
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.footnotes = true;
    /// assert_eq!(markdown_to_html("Hi[^x].\n\n[^x]: A greeting.\n", &options),
    ///            "<p>Hi<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup>.</p>\n<section class=\"footnotes\">\n<ol>\n<li id=\"fn1\">\n<p>A greeting. <a href=\"#fnref1\" class=\"footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n");
    /// ```
    pub footnotes: bool,

    /// Enables the description lists extension.
    ///
    /// Each term must be defined in one paragraph, followed by a blank line,
    /// and then by the details.  Details begins with a colon.
    ///
    /// ``` md
    /// First term
    ///
    /// : Details for the **first term**
    ///
    /// Second term
    ///
    /// : Details for the **second term**
    ///
    ///     More details in second paragraph.
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// options.extension.description_lists = true;
    /// assert_eq!(markdown_to_html("Term\n\n: Definition", &options),
    ///            "<dl><dt>Term</dt>\n<dd>\n<p>Definition</p>\n</dd>\n</dl>\n");
    /// ```
    pub description_lists: bool,
}

#[derive(Default, Debug, Clone)]
/// Options for parser functions.
pub struct ComrakParseOptions {
    /// Punctuation (quotes, full-stops and hyphens) are converted into 'smart' punctuation.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// assert_eq!(markdown_to_html("'Hello,' \"world\" ...", &options),
    ///            "<p>'Hello,' &quot;world&quot; ...</p>\n");
    ///
    /// options.parse.smart = true;
    /// assert_eq!(markdown_to_html("'Hello,' \"world\" ...", &options),
    ///            "<p>‘Hello,’ “world” …</p>\n");
    /// ```
    pub smart: bool,

    /// The default info string for fenced code blocks.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// assert_eq!(markdown_to_html("```\nfn hello();\n```\n", &options),
    ///            "<pre><code>fn hello();\n</code></pre>\n");
    ///
    /// options.parse.default_info_string = Some("rust".into());
    /// assert_eq!(markdown_to_html("```\nfn hello();\n```\n", &options),
    ///            "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n");
    /// ```
    pub default_info_string: Option<String>,
}

#[derive(Default, Debug, Clone, Copy)]
/// Options for formatter functions.
pub struct ComrakRenderOptions {
    /// [Soft line breaks](http://spec.commonmark.org/0.27/#soft-line-breaks) in the input
    /// translate into hard line breaks in the output.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// assert_eq!(markdown_to_html("Hello.\nWorld.\n", &options),
    ///            "<p>Hello.\nWorld.</p>\n");
    ///
    /// options.render.hardbreaks = true;
    /// assert_eq!(markdown_to_html("Hello.\nWorld.\n", &options),
    ///            "<p>Hello.<br />\nWorld.</p>\n");
    /// ```
    pub hardbreaks: bool,

    /// GitHub-style `<pre lang="xyz">` is used for fenced code blocks with info tags.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// assert_eq!(markdown_to_html("``` rust\nfn hello();\n```\n", &options),
    ///            "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n");
    ///
    /// options.render.github_pre_lang = true;
    /// assert_eq!(markdown_to_html("``` rust\nfn hello();\n```\n", &options),
    ///            "<pre lang=\"rust\"><code>fn hello();\n</code></pre>\n");
    /// ```
    pub github_pre_lang: bool,

    /// The wrap column when outputting CommonMark.
    ///
    /// ```
    /// # extern crate typed_arena;
    /// # extern crate comrak;
    /// # use comrak::{parse_document, ComrakOptions, format_commonmark};
    /// # fn main() {
    /// # let arena = typed_arena::Arena::new();
    /// let mut options = ComrakOptions::default();
    /// let node = parse_document(&arena, "hello hello hello hello hello hello", &options);
    /// let mut output = vec![];
    /// format_commonmark(node, &options, &mut output).unwrap();
    /// assert_eq!(String::from_utf8(output).unwrap(),
    ///            "hello hello hello hello hello hello\n");
    ///
    /// options.render.width = 20;
    /// let mut output = vec![];
    /// format_commonmark(node, &options, &mut output).unwrap();
    /// assert_eq!(String::from_utf8(output).unwrap(),
    ///            "hello hello hello\nhello hello hello\n");
    /// # }
    /// ```
    pub width: usize,

    /// Allow rendering of raw HTML and potentially dangerous links.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, ComrakOptions};
    /// let mut options = ComrakOptions::default();
    /// let input = "<script>\nalert('xyz');\n</script>\n\n\
    ///              Possibly <marquee>annoying</marquee>.\n\n\
    ///              [Dangerous](javascript:alert(document.cookie)).\n\n\
    ///              [Safe](http://commonmark.org).\n";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<!-- raw HTML omitted -->\n\
    ///             <p>Possibly <!-- raw HTML omitted -->annoying<!-- raw HTML omitted -->.</p>\n\
    ///             <p><a href=\"\">Dangerous</a>.</p>\n\
    ///             <p><a href=\"http://commonmark.org\">Safe</a>.</p>\n");
    ///
    /// options.render.unsafe_ = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<script>\nalert(\'xyz\');\n</script>\n\
    ///             <p>Possibly <marquee>annoying</marquee>.</p>\n\
    ///             <p><a href=\"javascript:alert(document.cookie)\">Dangerous</a>.</p>\n\
    ///             <p><a href=\"http://commonmark.org\">Safe</a>.</p>\n");
    /// ```
    pub unsafe_: bool,
}

#[derive(Clone)]
pub struct Reference {
    pub url: Vec<u8>,
    pub title: Vec<u8>,
}

struct FootnoteDefinition<'a> {
    ix: Option<u32>,
    node: &'a AstNode<'a>,
}

impl<'a, 'o, 'c> Parser<'a, 'o, 'c> {
    fn new(
        arena: &'a Arena<AstNode<'a>>,
        root: &'a AstNode<'a>,
        options: &'o ComrakOptions,
        callback: Option<&'c mut dyn FnMut(&[u8]) -> Option<(Vec<u8>, Vec<u8>)>>,
    ) -> Self {
        Parser {
            arena: arena,
            refmap: HashMap::new(),
            root: root,
            current: root,
            line_number: 0,
            offset: 0,
            column: 0,
            first_nonspace: 0,
            first_nonspace_column: 0,
            indent: 0,
            blank: false,
            partially_consumed_tab: false,
            last_line_length: 0,
            options: options,
            callback: callback,
        }
    }

    fn feed(&mut self, s: &str) {
        let s = s.as_bytes();
        let mut i = 0;
        let sz = s.len();
        let mut linebuf = vec![];

        while i < sz {
            let mut process = true;
            let mut eol = i;
            while eol < sz {
                if strings::is_line_end_char(s[eol]) {
                    break;
                }
                if s[eol] == 0 {
                    process = false;
                    break;
                }
                eol += 1;
            }

            if process {
                if !linebuf.is_empty() {
                    linebuf.extend_from_slice(&s[i..eol]);
                    self.process_line(&linebuf);
                    linebuf.truncate(0);
                } else if sz > eol && s[eol] == b'\n' {
                    self.process_line(&s[i..eol + 1]);
                } else {
                    self.process_line(&s[i..eol]);
                }

                i = eol;
                if i < sz && s[i] == b'\r' {
                    i += 1;
                }
                if i < sz && s[i] == b'\n' {
                    i += 1;
                }
            } else {
                debug_assert!(eol < sz && s[eol] == b'\0');
                linebuf.extend_from_slice(&s[i..eol]);
                linebuf.extend_from_slice(&"\u{fffd}".to_string().into_bytes());
                i = eol + 1;
            }
        }
    }

    fn find_first_nonspace(&mut self, line: &[u8]) {
        self.first_nonspace = self.offset;
        self.first_nonspace_column = self.column;
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        loop {
            if self.first_nonspace >= line.len() {
                break;
            }
            match line[self.first_nonspace] {
                32 => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += 1;
                    chars_to_tab -= 1;
                    if chars_to_tab == 0 {
                        chars_to_tab = TAB_STOP;
                    }
                }
                9 => {
                    self.first_nonspace += 1;
                    self.first_nonspace_column += chars_to_tab;
                    chars_to_tab = TAB_STOP;
                }
                _ => break,
            }
        }

        self.indent = self.first_nonspace_column - self.column;
        self.blank = self.first_nonspace < line.len()
            && strings::is_line_end_char(line[self.first_nonspace]);
    }

    fn process_line(&mut self, line: &[u8]) {
        let mut new_line: Vec<u8>;
        let line = if line.is_empty() || !strings::is_line_end_char(*line.last().unwrap()) {
            new_line = line.into();
            new_line.push(b'\n');
            &new_line
        } else {
            line
        };

        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        if self.line_number == 0 && line.len() >= 3
            && unsafe { str::from_utf8_unchecked(line) }
                .chars()
                .next()
                .unwrap() == '\u{feff}'
        {
            self.offset += 3;
        }

        self.line_number += 1;

        let mut all_matched = true;
        if let Some(last_matched_container) = self.check_open_blocks(line, &mut all_matched) {
            let mut container = last_matched_container;
            let current = self.current;
            self.open_new_blocks(&mut container, line, all_matched);

            if current.same_node(self.current) {
                self.add_text_to_container(container, last_matched_container, line);
            }
        }

        self.last_line_length = line.len();
        if self.last_line_length > 0 && line[self.last_line_length - 1] == b'\n' {
            self.last_line_length -= 1;
        }
        if self.last_line_length > 0 && line[self.last_line_length - 1] == b'\r' {
            self.last_line_length -= 1;
        }
    }

    fn check_open_blocks(
        &mut self,
        line: &[u8],
        all_matched: &mut bool,
    ) -> Option<&'a AstNode<'a>> {
        let (new_all_matched, mut container, should_continue) =
            self.check_open_blocks_inner(self.root, line);

        *all_matched = new_all_matched;
        if !*all_matched {
            container = container.parent().unwrap();
        }

        if !should_continue {
            None
        } else {
            Some(container)
        }
    }

    fn check_open_blocks_inner(
        &mut self,
        mut container: &'a AstNode<'a>,
        line: &[u8],
    ) -> (bool, &'a AstNode<'a>, bool) {
        let mut should_continue = true;

        while nodes::last_child_is_open(container) {
            container = container.last_child().unwrap();
            let ast = &mut *container.data.borrow_mut();

            self.find_first_nonspace(line);

            match ast.value {
                NodeValue::BlockQuote => if !self.parse_block_quote_prefix(line) {
                    return (false, container, should_continue);
                },
                NodeValue::Item(ref nl) => if !self.parse_node_item_prefix(line, container, nl) {
                    return (false, container, should_continue);
                },
                NodeValue::DescriptionItem(ref di) => {
                    if !self.parse_description_item_prefix(line, container, di) {
                        return (false, container, should_continue);
                    }
                }
                NodeValue::CodeBlock(..) => {
                    if !self.parse_code_block_prefix(line, container, ast, &mut should_continue) {
                        return (false, container, should_continue);
                    }
                }
                NodeValue::HtmlBlock(ref nhb) => if !self.parse_html_block_prefix(nhb.block_type) {
                    return (false, container, should_continue);
                },
                NodeValue::Paragraph => if self.blank {
                    return (false, container, should_continue);
                },
                NodeValue::Table(..) => {
                    if !table::matches(&line[self.first_nonspace..]) {
                        return (false, container, should_continue);
                    }
                    continue;
                }
                NodeValue::Heading(..) | NodeValue::TableRow(..) | NodeValue::TableCell => {
                    return (false, container, should_continue);
                }
                NodeValue::FootnoteDefinition(..) => {
                    if !self.parse_footnote_definition_block_prefix(line) {
                        return (false, container, should_continue);
                    }
                }
                _ => {}
            }
        }

        (true, container, should_continue)
    }

    fn open_new_blocks(&mut self, container: &mut &'a AstNode<'a>, line: &[u8], all_matched: bool) {
        let mut matched: usize = 0;
        let mut nl: NodeList = NodeList::default();
        let mut sc: scanners::SetextChar = scanners::SetextChar::Equals;
        let mut maybe_lazy = match self.current.data.borrow().value {
            NodeValue::Paragraph => true,
            _ => false,
        };

        while match container.data.borrow().value {
            NodeValue::CodeBlock(..) | NodeValue::HtmlBlock(..) => false,
            _ => true,
        } {
            self.find_first_nonspace(line);
            let indented = self.indent >= CODE_INDENT;

            if !indented && line[self.first_nonspace] == b'>' {
                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if strings::is_space_or_tab(line[self.offset]) {
                    self.advance_offset(line, 1, true);
                }
                *container = self.add_child(*container, NodeValue::BlockQuote);
            } else if !indented
                && unwrap_into(
                    scanners::atx_heading_start(&line[self.first_nonspace..]),
                    &mut matched,
                ) {
                let heading_startpos = self.first_nonspace;
                let offset = self.offset;
                self.advance_offset(line, heading_startpos + matched - offset, false);
                *container = self.add_child(*container, NodeValue::Heading(NodeHeading::default()));

                let mut hashpos = line[self.first_nonspace..]
                    .iter()
                    .position(|&c| c == b'#')
                    .unwrap() + self.first_nonspace;
                let mut level = 0;
                while line[hashpos] == b'#' {
                    level += 1;
                    hashpos += 1;
                }

                container.data.borrow_mut().value = NodeValue::Heading(NodeHeading {
                    level: level,
                    setext: false,
                });
            } else if !indented
                && unwrap_into(
                    scanners::open_code_fence(&line[self.first_nonspace..]),
                    &mut matched,
                ) {
                let first_nonspace = self.first_nonspace;
                let offset = self.offset;
                let ncb = NodeCodeBlock {
                    fenced: true,
                    fence_char: line[first_nonspace],
                    fence_length: matched,
                    fence_offset: first_nonspace - offset,
                    info: Vec::with_capacity(10),
                    literal: Vec::new(),
                };
                *container = self.add_child(*container, NodeValue::CodeBlock(ncb));
                self.advance_offset(line, first_nonspace + matched - offset, false);
            } else if !indented
                && (unwrap_into(
                    scanners::html_block_start(&line[self.first_nonspace..]),
                    &mut matched,
                ) || match container.data.borrow().value {
                    NodeValue::Paragraph => false,
                    _ => unwrap_into(
                        scanners::html_block_start_7(&line[self.first_nonspace..]),
                        &mut matched,
                    ),
                }) {
                let nhb = NodeHtmlBlock {
                    block_type: matched as u8,
                    literal: Vec::new(),
                };

                *container = self.add_child(*container, NodeValue::HtmlBlock(nhb));
            } else if !indented && match container.data.borrow().value {
                NodeValue::Paragraph => unwrap_into(
                    scanners::setext_heading_line(&line[self.first_nonspace..]),
                    &mut sc,
                ),
                _ => false,
            } {
                let has_content = {
                    let mut ast = container.data.borrow_mut();
                    self.resolve_reference_link_definitions(&mut ast.content)
                };
                if has_content {
                    container.data.borrow_mut().value = NodeValue::Heading(NodeHeading {
                        level: match sc {
                            scanners::SetextChar::Equals => 1,
                            scanners::SetextChar::Hyphen => 2,
                        },
                        setext: true,
                    });
                    let adv = line.len() - 1 - self.offset;
                    self.advance_offset(line, adv, false);
                }
            } else if !indented && match (&container.data.borrow().value, all_matched) {
                (&NodeValue::Paragraph, false) => false,
                _ => unwrap_into(
                    scanners::thematic_break(&line[self.first_nonspace..]),
                    &mut matched,
                ),
            } {
                *container = self.add_child(*container, NodeValue::ThematicBreak);
                let adv = line.len() - 1 - self.offset;
                self.advance_offset(line, adv, false);
            } else if !indented && self.options.extension.footnotes
                && unwrap_into(
                    scanners::footnote_definition(&line[self.first_nonspace..]),
                    &mut matched,
                ) {
                let mut c = &line[self.first_nonspace + 2..self.first_nonspace + matched];
                c = c.split(|&e| e == b']').next().unwrap();
                let offset = self.first_nonspace + matched - self.offset;
                self.advance_offset(line, offset, false);
                *container = self.add_child(*container, NodeValue::FootnoteDefinition(c.to_vec()));
            } else if !indented
                && self.options.extension.description_lists
                && line[self.first_nonspace] == b':'
                && self.parse_desc_list_details(container)
            {
                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if strings::is_space_or_tab(line[self.offset]) {
                    self.advance_offset(line, 1, true);
                }
            } else if (!indented || match container.data.borrow().value {
                NodeValue::List(..) => true,
                _ => false,
            }) && self.indent < 4
                && unwrap_into_2(
                    parse_list_marker(
                        line,
                        self.first_nonspace,
                        match container.data.borrow().value {
                            NodeValue::Paragraph => true,
                            _ => false,
                        },
                    ),
                    &mut matched,
                    &mut nl,
                ) {
                let offset = self.first_nonspace + matched - self.offset;
                self.advance_offset(line, offset, false);
                let (save_partially_consumed_tab, save_offset, save_column) =
                    (self.partially_consumed_tab, self.offset, self.column);

                while self.column - save_column <= 5 && strings::is_space_or_tab(line[self.offset])
                {
                    self.advance_offset(line, 1, true);
                }

                let i = self.column - save_column;
                if i >= 5 || i < 1 || strings::is_line_end_char(line[self.offset]) {
                    nl.padding = matched + 1;
                    self.offset = save_offset;
                    self.column = save_column;
                    self.partially_consumed_tab = save_partially_consumed_tab;
                    if i > 0 {
                        self.advance_offset(line, 1, true);
                    }
                } else {
                    nl.padding = matched + i;
                }

                nl.marker_offset = self.indent;

                if match container.data.borrow().value {
                    NodeValue::List(ref mnl) => !lists_match(&nl, mnl),
                    _ => true,
                } {
                    *container = self.add_child(*container, NodeValue::List(nl));
                }

                *container = self.add_child(*container, NodeValue::Item(nl));
            } else if indented && !maybe_lazy && !self.blank {
                self.advance_offset(line, CODE_INDENT, true);
                let ncb = NodeCodeBlock {
                    fenced: false,
                    fence_char: 0,
                    fence_length: 0,
                    fence_offset: 0,
                    info: vec![],
                    literal: Vec::new(),
                };
                *container = self.add_child(*container, NodeValue::CodeBlock(ncb));
            } else {
                let new_container = if !indented && self.options.extension.table {
                    table::try_opening_block(self, *container, line)
                } else {
                    None
                };

                match new_container {
                    Some((new_container, replace)) => if replace {
                        container.insert_after(new_container);
                        container.detach();
                        *container = new_container;
                    } else {
                        *container = new_container;
                    },
                    _ => break,
                }
            }

            if container.data.borrow().value.accepts_lines() {
                break;
            }

            maybe_lazy = false;
        }
    }

    fn advance_offset(&mut self, line: &[u8], mut count: usize, columns: bool) {
        while count > 0 {
            match line[self.offset] {
                9 => {
                    let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
                    if columns {
                        self.partially_consumed_tab = chars_to_tab > count;
                        let chars_to_advance = min(count, chars_to_tab);
                        self.column += chars_to_advance;
                        self.offset += if self.partially_consumed_tab { 0 } else { 1 };
                        count -= chars_to_advance;
                    } else {
                        self.partially_consumed_tab = false;
                        self.column += chars_to_tab;
                        self.offset += 1;
                        count -= 1;
                    }
                }
                _ => {
                    self.partially_consumed_tab = false;
                    self.offset += 1;
                    self.column += 1;
                    count -= 1;
                }
            }
        }
    }

    fn parse_block_quote_prefix(&mut self, line: &[u8]) -> bool {
        let indent = self.indent;
        if indent <= 3 && line[self.first_nonspace] == b'>' {
            self.advance_offset(line, indent + 1, true);

            if strings::is_space_or_tab(line[self.offset]) {
                self.advance_offset(line, 1, true);
            }

            return true;
        }

        false
    }

    fn parse_footnote_definition_block_prefix(&mut self, line: &[u8]) -> bool {
        if self.indent >= 4 {
            self.advance_offset(line, 4, true);
            true
        } else {
            line == b"\n" || line == b"\r\n"
        }
    }

    fn parse_node_item_prefix(
        &mut self,
        line: &[u8],
        container: &'a AstNode<'a>,
        nl: &NodeList,
    ) -> bool {
        if self.indent >= nl.marker_offset + nl.padding {
            self.advance_offset(line, nl.marker_offset + nl.padding, true);
            true
        } else if self.blank && container.first_child().is_some() {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            true
        } else {
            false
        }
    }

    fn parse_description_item_prefix(
        &mut self,
        line: &[u8],
        container: &'a AstNode<'a>,
        di: &NodeDescriptionItem,
    ) -> bool {
        if self.indent >= di.marker_offset + di.padding {
            self.advance_offset(line, di.marker_offset + di.padding, true);
            true
        } else if self.blank && container.first_child().is_some() {
            let offset = self.first_nonspace - self.offset;
            self.advance_offset(line, offset, false);
            true
        } else {
            false
        }
    }

    fn parse_code_block_prefix(
        &mut self,
        line: &[u8],
        container: &'a AstNode<'a>,
        ast: &mut Ast,
        should_continue: &mut bool,
    ) -> bool {
        let (fenced, fence_char, fence_length, fence_offset) = match ast.value {
            NodeValue::CodeBlock(ref ncb) => (
                ncb.fenced,
                ncb.fence_char,
                ncb.fence_length,
                ncb.fence_offset,
            ),
            _ => unreachable!(),
        };

        if !fenced {
            if self.indent >= CODE_INDENT {
                self.advance_offset(line, CODE_INDENT, true);
                return true;
            } else if self.blank {
                let offset = self.first_nonspace - self.offset;
                self.advance_offset(line, offset, false);
                return true;
            }
            return false;
        }

        let matched = if self.indent <= 3 && line[self.first_nonspace] == fence_char {
            scanners::close_code_fence(&line[self.first_nonspace..]).unwrap_or(0)
        } else {
            0
        };

        if matched >= fence_length {
            *should_continue = false;
            self.advance_offset(line, matched, false);
            self.current = self.finalize_borrowed(container, ast).unwrap();
            return false;
        }

        let mut i = fence_offset;
        while i > 0 && strings::is_space_or_tab(line[self.offset]) {
            self.advance_offset(line, 1, true);
            i -= 1;
        }
        true
    }

    fn parse_html_block_prefix(&mut self, t: u8) -> bool {
        match t {
            1 | 2 | 3 | 4 | 5 => true,
            6 | 7 => !self.blank,
            _ => {
                assert!(false);
                false
            }
        }
    }

    fn parse_desc_list_details(&mut self, container: &mut &'a AstNode<'a>) -> bool {
        let last_child = match container.last_child() {
            Some(lc) => lc,
            None => return false,
        };

        if node_matches!(last_child, NodeValue::Paragraph) {
            // We have found the details after the paragraph for the term.
            //
            // This paragraph is moved as a child of a new DescriptionTerm node.
            //
            // If the node before the paragraph is a description list, the item
            // is added to it. If not, create a new list.

            last_child.detach();

            let list = match container.last_child() {
                Some(lc) if node_matches!(lc, NodeValue::DescriptionList) => {
                    reopen_ast_nodes(lc);
                    lc
                }
                _ => self.add_child(container, NodeValue::DescriptionList),
            };

            let metadata = NodeDescriptionItem {
                marker_offset: self.indent,
                padding: 2,
            };

            let item = self.add_child(list, NodeValue::DescriptionItem(metadata));
            let term = self.add_child(item, NodeValue::DescriptionTerm);
            let details = self.add_child(item, NodeValue::DescriptionDetails);

            term.append(last_child);

            *container = details;

            true
        } else {
            false
        }
    }

    fn add_child(&mut self, mut parent: &'a AstNode<'a>, value: NodeValue) -> &'a AstNode<'a> {
        while !nodes::can_contain_type(parent, &value) {
            parent = self.finalize(parent).unwrap();
        }

        let mut child = Ast::new(value);
        child.start_line = self.line_number;
        let node = self.arena.alloc(Node::new(RefCell::new(child)));
        parent.append(node);
        node
    }

    fn add_text_to_container(
        &mut self,
        mut container: &'a AstNode<'a>,
        last_matched_container: &'a AstNode<'a>,
        line: &[u8],
    ) {
        self.find_first_nonspace(line);

        if self.blank {
            if let Some(last_child) = container.last_child() {
                last_child.data.borrow_mut().last_line_blank = true;
            }
        }

        container.data.borrow_mut().last_line_blank = self.blank
            && match container.data.borrow().value {
                NodeValue::BlockQuote | NodeValue::Heading(..) | NodeValue::ThematicBreak => false,
                NodeValue::CodeBlock(ref ncb) => !ncb.fenced,
                NodeValue::Item(..) => {
                    container.first_child().is_some()
                        || container.data.borrow().start_line != self.line_number
                }
                _ => true,
            };

        let mut tmp = container;
        while let Some(parent) = tmp.parent() {
            parent.data.borrow_mut().last_line_blank = false;
            tmp = parent;
        }

        if !self.current.same_node(last_matched_container)
            && container.same_node(last_matched_container)
            && !self.blank && match self.current.data.borrow().value {
            NodeValue::Paragraph => true,
            _ => false,
        } {
            self.add_line(self.current, line);
        } else {
            while !self.current.same_node(last_matched_container) {
                self.current = self.finalize(self.current).unwrap();
            }

            let add_text_result = match container.data.borrow().value {
                NodeValue::CodeBlock(..) => AddTextResult::CodeBlock,
                NodeValue::HtmlBlock(ref nhb) => AddTextResult::HtmlBlock(nhb.block_type),
                _ => AddTextResult::Otherwise,
            };

            match add_text_result {
                AddTextResult::CodeBlock => {
                    self.add_line(container, line);
                }
                AddTextResult::HtmlBlock(block_type) => {
                    self.add_line(container, line);

                    let matches_end_condition = match block_type {
                        1 => scanners::html_block_end_1(&line[self.first_nonspace..]),
                        2 => scanners::html_block_end_2(&line[self.first_nonspace..]),
                        3 => scanners::html_block_end_3(&line[self.first_nonspace..]),
                        4 => scanners::html_block_end_4(&line[self.first_nonspace..]),
                        5 => scanners::html_block_end_5(&line[self.first_nonspace..]),
                        _ => false,
                    };

                    if matches_end_condition {
                        container = self.finalize(container).unwrap();
                    }
                }
                _ => {
                    if self.blank {
                        // do nothing
                    } else if container.data.borrow().value.accepts_lines() {
                        let mut line: Vec<u8> = line.into();
                        if let NodeValue::Heading(ref nh) = container.data.borrow().value {
                            if !nh.setext {
                                strings::chop_trailing_hashtags(&mut line);
                            }
                        };
                        let count = self.first_nonspace - self.offset;

                        // In a rare case the above `chop` operation can leave
                        // the line shorter than the recorded `first_nonspace`
                        // This happens with ATX headers containing no header
                        // text, multiple spaces and trailing hashes, e.g
                        //
                        // ###     ###
                        //
                        // In this case `first_nonspace` indexes into the second
                        // set of hashes, while `chop_trailing_hashtags` truncates
                        // `line` to just `###` (the first three hashes).
                        // In this case there's no text to add, and no further
                        // processing to be done.
                        let have_line_text = self.first_nonspace <= line.len();

                        if have_line_text {
                            self.advance_offset(&line, count, false);
                            self.add_line(container, &line);
                        }
                    } else {
                        container = self.add_child(container, NodeValue::Paragraph);
                        let count = self.first_nonspace - self.offset;
                        self.advance_offset(line, count, false);
                        self.add_line(container, line);
                    }
                }
            }

            self.current = container;
        }
    }

    fn add_line(&mut self, node: &'a AstNode<'a>, line: &[u8]) {
        let mut ast = node.data.borrow_mut();
        assert!(ast.open);
        if self.partially_consumed_tab {
            self.offset += 1;
            let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
            for _ in 0..chars_to_tab {
                ast.content.push(b' ');
            }
        }
        if self.offset < line.len() {
            ast.content.extend_from_slice(&line[self.offset..]);
        }
    }

    fn finish(&mut self) -> &'a AstNode<'a> {
        self.finalize_document();
        self.postprocess_text_nodes(self.root);
        self.root
    }

    fn finalize_document(&mut self) {
        while !self.current.same_node(self.root) {
            self.current = self.finalize(self.current).unwrap();
        }

        self.finalize(self.root);
        self.process_inlines();
        if self.options.extension.footnotes {
            self.process_footnotes();
        }
    }

    fn finalize(&mut self, node: &'a AstNode<'a>) -> Option<&'a AstNode<'a>> {
        self.finalize_borrowed(node, &mut *node.data.borrow_mut())
    }

    fn resolve_reference_link_definitions(&mut self, content: &mut Vec<u8>) -> bool {
        let mut seeked = 0;
        {
            let mut pos = 0;
            let mut seek: &[u8] = &*content;
            while !seek.is_empty()
                && seek[0] == b'['
                && unwrap_into(self.parse_reference_inline(seek), &mut pos)
            {
                seek = &seek[pos..];
                seeked += pos;
            }
        }

        if seeked != 0 {
            *content = content[seeked..].to_vec();
        }

        !strings::is_blank(content)
    }

    fn finalize_borrowed(
        &mut self,
        node: &'a AstNode<'a>,
        ast: &mut Ast,
    ) -> Option<&'a AstNode<'a>> {
        assert!(ast.open);
        ast.open = false;

        let content = &mut ast.content;
        let parent = node.parent();

        match ast.value {
            NodeValue::Paragraph => {
                let has_content = self.resolve_reference_link_definitions(content);
                if !has_content {
                    node.detach();
                }
            }
            NodeValue::CodeBlock(ref mut ncb) => {
                if !ncb.fenced {
                    strings::remove_trailing_blank_lines(content);
                    content.push(b'\n');
                } else {
                    let mut pos = 0;
                    while pos < content.len() {
                        if strings::is_line_end_char(content[pos]) {
                            break;
                        }
                        pos += 1;
                    }
                    assert!(pos < content.len());

                    let mut tmp = entity::unescape_html(&content[..pos]);
                    strings::trim(&mut tmp);
                    strings::unescape(&mut tmp);
                    if tmp.is_empty() {
                        ncb.info = self
                            .options
                            .parse
                            .default_info_string
                            .as_ref()
                            .map_or(vec![], |s| s.as_bytes().to_vec());
                    } else {
                        ncb.info = tmp;
                    }

                    if content[pos] == b'\r' {
                        pos += 1;
                    }
                    if content[pos] == b'\n' {
                        pos += 1;
                    }

                    *content = content[pos..].to_vec();
                }
                mem::swap(&mut ncb.literal, content);
            }
            NodeValue::HtmlBlock(ref mut nhb) => {
                mem::swap(&mut nhb.literal, content);
            }
            NodeValue::List(ref mut nl) => {
                nl.tight = true;
                let mut ch = node.first_child();

                while let Some(item) = ch {
                    if item.data.borrow().last_line_blank && item.next_sibling().is_some() {
                        nl.tight = false;
                        break;
                    }

                    let mut subch = item.first_child();
                    while let Some(subitem) = subch {
                        if nodes::ends_with_blank_line(subitem)
                            && (item.next_sibling().is_some() || subitem.next_sibling().is_some())
                        {
                            nl.tight = false;
                            break;
                        }
                        subch = subitem.next_sibling();
                    }

                    if !nl.tight {
                        break;
                    }

                    ch = item.next_sibling();
                }
            }
            _ => (),
        }

        parent
    }

    fn process_inlines(&mut self) {
        self.process_inlines_node(self.root);
    }

    fn process_inlines_node(&mut self, node: &'a AstNode<'a>) {
        for node in node.descendants() {
            if node.data.borrow().value.contains_inlines() {
                self.parse_inlines(node);
            }
        }
    }

    fn parse_inlines(&mut self, node: &'a AstNode<'a>) {
        let delimiter_arena = Arena::new();
        let node_data = node.data.borrow();
        let content = strings::rtrim_slice(&node_data.content);
        let mut subj = inlines::Subject::new(
            self.arena,
            self.options,
            content,
            &mut self.refmap,
            &delimiter_arena,
            self.callback.as_mut(),
        );

        while subj.parse_inline(node) {}

        subj.process_emphasis(None);

        while subj.pop_bracket() {}
    }

    fn process_footnotes(&mut self) {
        let mut map = HashMap::new();
        Self::find_footnote_definitions(self.root, &mut map);

        let mut ix = 0;
        Self::find_footnote_references(self.root, &mut map, &mut ix);

        if ix > 0 {
            let mut v = map.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
            v.sort_unstable_by(|a, b| a.ix.cmp(&b.ix));
            for f in v {
                if f.ix.is_some() {
                    match f.node.data.borrow_mut().value {
                        NodeValue::FootnoteDefinition(ref mut name) => {
                            *name = format!("{}", f.ix.unwrap()).into_bytes();
                        }
                        _ => unreachable!(),
                    }
                    self.root.append(f.node);
                }
            }
        }
    }

    fn find_footnote_definitions(
        node: &'a AstNode<'a>,
        map: &mut HashMap<Vec<u8>, FootnoteDefinition<'a>>,
    ) {
        match node.data.borrow().value {
            NodeValue::FootnoteDefinition(ref name) => {
                node.detach();
                map.insert(
                    strings::normalize_label(name),
                    FootnoteDefinition {
                        ix: None,
                        node: node,
                    },
                );
            }
            _ => for n in node.children() {
                Self::find_footnote_definitions(n, map);
            },
        }
    }

    fn find_footnote_references(
        node: &'a AstNode<'a>,
        map: &mut HashMap<Vec<u8>, FootnoteDefinition>,
        ix: &mut u32,
    ) {
        let mut ast = node.data.borrow_mut();
        let mut replace = None;
        match ast.value {
            NodeValue::FootnoteReference(ref mut name) => {
                if let Some(ref mut footnote) = map.get_mut(name) {
                    if footnote.ix.is_none() {
                        *ix += 1;
                        footnote.ix = Some(*ix);
                    }
                    *name = format!("{}", footnote.ix.unwrap()).into_bytes();
                } else {
                    replace = Some(name.clone());
                }
            }
            _ => for n in node.children() {
                Self::find_footnote_references(n, map, ix);
            },
        }

        if let Some(mut label) = replace {
            label.insert(0, b'[');
            label.insert(1, b'^');
            let len = label.len();
            label.insert(len, b']');
            ast.value = NodeValue::Text(label);
        }
    }

    fn postprocess_text_nodes(&mut self, node: &'a AstNode<'a>) {
        let mut stack = vec![node];
        let mut children = vec![];

        while let Some(node) = stack.pop() {
            let mut nch = node.first_child();

            while let Some(n) = nch {
                let mut this_bracket = false;
                loop {
                    match n.data.borrow_mut().value {
                        // Join adjacent text nodes together
                        NodeValue::Text(ref mut root) => {
                            let ns = match n.next_sibling() {
                                Some(ns) => ns,
                                _ => {
                                    // Post-process once we are finished joining text nodes
                                    self.postprocess_text_node(n, root);
                                    break;
                                }
                            };

                            match ns.data.borrow().value {
                                NodeValue::Text(ref adj) => {
                                    root.extend_from_slice(adj);
                                    ns.detach();
                                }
                                _ => {
                                    // Post-process once we are finished joining text nodes
                                    self.postprocess_text_node(n, root);
                                    break;
                                }
                            }
                        }
                        NodeValue::Link(..) | NodeValue::Image(..) => {
                            this_bracket = true;
                            break;
                        }
                        _ => break,
                    }
                }

                if !this_bracket {
                    children.push(n);
                }

                nch = n.next_sibling();
            }

            // Push children onto work stack in reverse order so they are
            // traversed in order
            stack.extend(children.drain(..).rev());
        }
    }

    fn postprocess_text_node(&mut self, node: &'a AstNode<'a>, text: &mut Vec<u8>) {
        if self.options.extension.tasklist {
            self.process_tasklist(node, text);
        }

        if self.options.extension.autolink {
            autolink::process_autolinks(self.arena, node, text);
        }
    }

    fn process_tasklist(&mut self, node: &'a AstNode<'a>, text: &mut Vec<u8>) {
        lazy_static! {
            static ref TASKLIST: Regex = Regex::new(r"\A(\s*\[([xX ])\])(?:\z|\s)").unwrap();
        }

        let (active, end) = match TASKLIST.captures(text) {
            None => return,
            Some(c) => (
                c.get(2).unwrap().as_bytes() != b" ",
                c.get(0).unwrap().end(),
            ),
        };

        let parent = node.parent().unwrap();
        if node.previous_sibling().is_some() || parent.previous_sibling().is_some() {
            return;
        }

        match parent.data.borrow().value {
            NodeValue::Paragraph => (),
            _ => return,
        }

        match parent.parent().unwrap().data.borrow().value {
            NodeValue::Item(..) => (),
            _ => return,
        }

        *text = text[end..].to_vec();
        let checkbox = inlines::make_inline(self.arena, NodeValue::TaskItem(active));
        node.insert_before(checkbox);
    }

    fn parse_reference_inline(&mut self, content: &[u8]) -> Option<usize> {
        // In this case reference inlines rarely have delimiters
        // so we often just need the minimal case
        let delimiter_arena = Arena::with_capacity(0);
        let mut subj = inlines::Subject::new(
            self.arena,
            self.options,
            content,
            &mut self.refmap,
            &delimiter_arena,
            self.callback.as_mut(),
        );

        let mut lab: Vec<u8> = match subj.link_label() {
            Some(lab) => if lab.is_empty() {
                return None;
            } else {
                lab
            },
            None => return None,
        }.to_vec();

        if subj.peek_char() != Some(&(b':')) {
            return None;
        }

        subj.pos += 1;
        subj.spnl();
        let (url, matchlen) = match inlines::manual_scan_link_url(&subj.input[subj.pos..]) {
            Some((url, matchlen)) => (url, matchlen),
            None => return None,
        };
        subj.pos += matchlen;

        let beforetitle = subj.pos;
        subj.spnl();
        let title_search = if subj.pos == beforetitle { None } else { scanners::link_title(&subj.input[subj.pos..]) };
        let title = match title_search {
            Some(matchlen) => {
                let t = &subj.input[subj.pos..subj.pos + matchlen];
                subj.pos += matchlen;
                t.to_vec()
            }
            _ => {
                subj.pos = beforetitle;
                vec![]
            }
        };

        subj.skip_spaces();
        if !subj.skip_line_end() {
            if !title.is_empty() {
                subj.pos = beforetitle;
                subj.skip_spaces();
                if !subj.skip_line_end() {
                    return None;
                }
            } else {
                return None;
            }
        }

        lab = strings::normalize_label(&lab);
        if !lab.is_empty() {
            subj.refmap.entry(lab.to_vec()).or_insert(Reference {
                url: strings::clean_url(&url),
                title: strings::clean_title(&title),
            });
        }
        Some(subj.pos)
    }
}

enum AddTextResult {
    CodeBlock,
    HtmlBlock(u8),
    Otherwise,
}

fn parse_list_marker(
    line: &[u8],
    mut pos: usize,
    interrupts_paragraph: bool,
) -> Option<(usize, NodeList)> {
    let mut c = line[pos];
    let startpos = pos;

    if c == b'*' || c == b'-' || c == b'+' {
        pos += 1;
        if !isspace(line[pos]) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while strings::is_space_or_tab(line[i]) {
                i += 1;
            }
            if line[i] == b'\n' {
                return None;
            }
        }

        return Some((
            pos - startpos,
            NodeList {
                list_type: ListType::Bullet,
                marker_offset: 0,
                padding: 0,
                start: 1,
                delimiter: ListDelimType::Period,
                bullet_char: c,
                tight: false,
            },
        ));
    } else if isdigit(c) {
        let mut start: usize = 0;
        let mut digits = 0;

        loop {
            start = (10 * start) + (line[pos] - b'0') as usize;
            pos += 1;
            digits += 1;

            if !(digits < 9 && isdigit(line[pos])) {
                break;
            }
        }

        if interrupts_paragraph && start != 1 {
            return None;
        }

        c = line[pos];
        if c != b'.' && c != b')' {
            return None;
        }

        pos += 1;

        if !isspace(line[pos]) {
            return None;
        }

        if interrupts_paragraph {
            let mut i = pos;
            while strings::is_space_or_tab(line[i]) {
                i += 1;
            }
            if strings::is_line_end_char(line[i]) {
                return None;
            }
        }

        return Some((
            pos - startpos,
            NodeList {
                list_type: ListType::Ordered,
                marker_offset: 0,
                padding: 0,
                start: start,
                delimiter: if c == b'.' {
                    ListDelimType::Period
                } else {
                    ListDelimType::Paren
                },
                bullet_char: 0,
                tight: false,
            },
        ));
    }

    None
}

pub fn unwrap_into<T>(t: Option<T>, out: &mut T) -> bool {
    match t {
        Some(v) => {
            *out = v;
            true
        }
        _ => false,
    }
}

pub fn unwrap_into_copy<T: Copy>(t: Option<&T>, out: &mut T) -> bool {
    match t {
        Some(v) => {
            *out = *v;
            true
        }
        _ => false,
    }
}

fn unwrap_into_2<T, U>(tu: Option<(T, U)>, out_t: &mut T, out_u: &mut U) -> bool {
    match tu {
        Some((t, u)) => {
            *out_t = t;
            *out_u = u;
            true
        }
        _ => false,
    }
}

fn lists_match(list_data: &NodeList, item_data: &NodeList) -> bool {
    list_data.list_type == item_data.list_type
        && list_data.delimiter == item_data.delimiter
        && list_data.bullet_char == item_data.bullet_char
}

fn reopen_ast_nodes<'a>(mut ast: &'a AstNode<'a>) {
    loop {
        ast.data.borrow_mut().open = true;
        ast = match ast.parent() {
            Some(p) => p,
            None => return,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutolinkType {
    URI,
    Email,
}
