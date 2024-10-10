mod autolink;
mod inlines;
#[cfg(feature = "shortcodes")]
pub mod shortcodes;
mod table;

pub mod math;
pub mod multiline_block_quote;

use crate::adapters::SyntaxHighlighterAdapter;
use crate::arena_tree::Node;
use crate::ctype::{isdigit, isspace};
use crate::entity;
use crate::nodes::{self, NodeFootnoteDefinition, Sourcepos};
use crate::nodes::{
    Ast, AstNode, ListDelimType, ListType, NodeCodeBlock, NodeDescriptionItem, NodeHeading,
    NodeHtmlBlock, NodeList, NodeValue,
};
use crate::scanners::{self, SetextChar};
use crate::strings::{self, split_off_front_matter, Case};
use std::cell::RefCell;
use std::cmp::min;
use bon::Builder;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::str;
use std::sync::{Arc, Mutex};
use typed_arena::Arena;

use crate::adapters::HeadingAdapter;
use crate::parser::multiline_block_quote::NodeMultilineBlockQuote;

use self::inlines::RefMap;

const TAB_STOP: usize = 4;
const CODE_INDENT: usize = 4;

// Very deeply nested lists can cause quadratic performance issues.
// This constant is used in open_new_blocks() to limit the nesting
// depth. It is unlikely that a non-contrived markdown document will
// be nested this deeply.
const MAX_LIST_DEPTH: usize = 100;

macro_rules! node_matches {
    ($node:expr, $( $pat:pat )|+) => {{
        matches!(
            $node.data.borrow().value,
            $( $pat )|+
        )
    }};
}

/// Parse a Markdown document to an AST.
///
/// See the documentation of the crate root for an example.
pub fn parse_document<'a>(
    arena: &'a Arena<AstNode<'a>>,
    buffer: &str,
    options: &Options,
) -> &'a AstNode<'a> {
    let root: &'a AstNode<'a> = arena.alloc(Node::new(RefCell::new(Ast {
        value: NodeValue::Document,
        content: String::new(),
        sourcepos: (1, 1, 1, 1).into(),
        internal_offset: 0,
        open: true,
        last_line_blank: false,
        table_visited: false,
        line_offsets: Vec::with_capacity(0),
    })));
    let mut parser = Parser::new(arena, root, options);
    let mut linebuf = Vec::with_capacity(buffer.len());
    parser.feed(&mut linebuf, buffer, true);
    parser.finish(linebuf)
}

/// Parse a Markdown document to an AST, specifying
/// [`ParseOptions::broken_link_callback`].
#[deprecated(
    since = "0.25.0",
    note = "The broken link callback has been moved into ParseOptions<'c>."
)]
pub fn parse_document_with_broken_link_callback<'a, 'c>(
    arena: &'a Arena<AstNode<'a>>,
    buffer: &str,
    options: &Options<'c>,
    callback: Option<BrokenLinkCallback<'c>>,
) -> &'a AstNode<'a> {
    let mut options_with_callback = options.clone();
    options_with_callback.parse.broken_link_callback = callback.map(|cb| Arc::new(Mutex::new(cb)));
    parse_document(arena, buffer, &options_with_callback)
}

/// The type of the callback used when a reference link is encountered with no
/// matching reference.
///
/// The details of the broken reference are passed in the
/// [`BrokenLinkReference`] argument. If a [`ResolvedReference`] is returned, it
/// is used as the link; otherwise, no link is made and the reference text is
/// preserved in its entirety.
pub type BrokenLinkCallback<'c> =
    &'c mut dyn FnMut(BrokenLinkReference) -> Option<ResolvedReference>;

/// Struct to the broken link callback, containing details on the link reference
/// which failed to find a match.
#[derive(Debug)]
pub struct BrokenLinkReference<'l> {
    /// The normalized reference link label. Unicode case folding is applied;
    /// see <https://github.com/commonmark/commonmark-spec/issues/695> for a
    /// discussion on the details of what this exactly means.
    pub normalized: &'l str,

    /// The original text in the link label.
    pub original: &'l str,
}

pub struct Parser<'a, 'o, 'c> {
    arena: &'a Arena<AstNode<'a>>,
    refmap: RefMap,
    root: &'a AstNode<'a>,
    current: &'a AstNode<'a>,
    line_number: usize,
    offset: usize,
    column: usize,
    thematic_break_kill_pos: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    partially_consumed_tab: bool,
    curline_len: usize,
    curline_end_col: usize,
    last_line_length: usize,
    last_buffer_ended_with_cr: bool,
    total_size: usize,
    options: &'o Options<'c>,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
/// Umbrella options struct. `'c` represents the lifetime of any callback
/// closure options may take.
pub struct Options<'c> {
    /// Enable CommonMark extensions.
    pub extension: ExtensionOptions,

    /// Configure parse-time options.
    pub parse: ParseOptions<'c>,

    /// Configure render-time options.
    pub render: RenderOptions,
}

#[non_exhaustive]
#[derive(Default, Debug, Clone, Builder)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
/// Options to select extensions.
pub struct ExtensionOptions {
    /// Enables the
    /// [strikethrough extension](https://github.github.com/gfm/#strikethrough-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.strikethrough = true;
    /// assert_eq!(markdown_to_html("Hello ~world~ there.\n", &options),
    ///            "<p>Hello <del>world</del> there.</p>\n");
    /// ```
    #[builder(default)]
    pub strikethrough: bool,

    /// Enables the
    /// [tagfilter extension](https://github.github.com/gfm/#disallowed-raw-html-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.tagfilter = true;
    /// options.render.unsafe_ = true;
    /// assert_eq!(markdown_to_html("Hello <xmp>.\n\n<xmp>", &options),
    ///            "<p>Hello &lt;xmp>.</p>\n&lt;xmp>\n");
    /// ```
    #[builder(default)]
    pub tagfilter: bool,

    /// Enables the [table extension](https://github.github.com/gfm/#tables-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.table = true;
    /// assert_eq!(markdown_to_html("| a | b |\n|---|---|\n| c | d |\n", &options),
    ///            "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n\
    ///             <tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n");
    /// ```
    #[builder(default)]
    pub table: bool,

    /// Enables the [autolink extension](https://github.github.com/gfm/#autolinks-extension-)
    /// from the GFM spec.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.autolink = true;
    /// assert_eq!(markdown_to_html("Hello www.github.com.\n", &options),
    ///            "<p>Hello <a href=\"http://www.github.com\">www.github.com</a>.</p>\n");
    /// ```
    #[builder(default)]
    pub autolink: bool,

    /// Enables the
    /// [task list items extension](https://github.github.com/gfm/#task-list-items-extension-)
    /// from the GFM spec.
    ///
    /// Note that the spec does not define the precise output, so only the bare essentials are
    /// rendered.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.tasklist = true;
    /// options.render.unsafe_ = true;
    /// assert_eq!(markdown_to_html("* [x] Done\n* [ ] Not done\n", &options),
    ///            "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> Done</li>\n\
    ///            <li><input type=\"checkbox\" disabled=\"\" /> Not done</li>\n</ul>\n");
    /// ```
    #[builder(default)]
    pub tasklist: bool,

    /// Enables the superscript Comrak extension.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.superscript = true;
    /// assert_eq!(markdown_to_html("e = mc^2^.\n", &options),
    ///            "<p>e = mc<sup>2</sup>.</p>\n");
    /// ```
    #[builder(default)]
    pub superscript: bool,

    /// Enables the header IDs Comrak extension.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
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
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.footnotes = true;
    /// assert_eq!(markdown_to_html("Hi[^x].\n\n[^x]: A greeting.\n", &options),
    ///            "<p>Hi<sup class=\"footnote-ref\"><a href=\"#fn-x\" id=\"fnref-x\" data-footnote-ref>1</a></sup>.</p>\n<section class=\"footnotes\" data-footnotes>\n<ol>\n<li id=\"fn-x\">\n<p>A greeting. <a href=\"#fnref-x\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n</li>\n</ol>\n</section>\n");
    /// ```
    #[builder(default)]
    pub footnotes: bool,

    /// Enables the description lists extension.
    ///
    /// Each term must be defined in one paragraph, followed by a blank line,
    /// and then by the details.  Details begins with a colon.
    ///
    /// Not (yet) compatible with render.sourcepos.
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
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.description_lists = true;
    /// assert_eq!(markdown_to_html("Term\n\n: Definition", &options),
    ///            "<dl><dt>Term</dt>\n<dd>\n<p>Definition</p>\n</dd>\n</dl>\n");
    /// ```
    #[builder(default)]
    pub description_lists: bool,

    /// Enables the front matter extension.
    ///
    /// Front matter, which begins with the delimiter string at the beginning of the file and ends
    /// at the end of the next line that contains only the delimiter, is passed through unchanged
    /// in markdown output and omitted from HTML output.
    ///
    /// ``` md
    /// ---
    /// layout: post
    /// title: Formatting Markdown with Comrak
    /// ---
    ///
    /// # Shorter Title
    ///
    /// etc.
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.front_matter_delimiter = Some("---".to_owned());
    /// assert_eq!(
    ///     markdown_to_html("---\nlayout: post\n---\nText\n", &options),
    ///     markdown_to_html("Text\n", &Options::default()));
    /// ```
    ///
    /// ```
    /// # use comrak::{format_commonmark, Arena, Options};
    /// use comrak::parse_document;
    /// let mut options = Options::default();
    /// options.extension.front_matter_delimiter = Some("---".to_owned());
    /// let arena = Arena::new();
    /// let input ="---\nlayout: post\n---\nText\n";
    /// let root = parse_document(&arena, input, &options);
    /// let mut buf = Vec::new();
    /// format_commonmark(&root, &options, &mut buf);
    /// assert_eq!(&String::from_utf8(buf).unwrap(), input);
    /// ```
    pub front_matter_delimiter: Option<String>,

    /// Enables the multiline block quote extension.
    ///
    /// Place `>>>` before and after text to make it into
    /// a block quote.
    ///
    /// ``` md
    /// Paragraph one
    ///
    /// >>>
    /// Paragraph two
    ///
    /// - one
    /// - two
    /// >>>
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.multiline_block_quotes = true;
    /// assert_eq!(markdown_to_html(">>>\nparagraph\n>>>", &options),
    ///            "<blockquote>\n<p>paragraph</p>\n</blockquote>\n");
    /// ```
    #[builder(default)]
    pub multiline_block_quotes: bool,

    /// Enables math using dollar syntax.
    ///
    /// ``` md
    /// Inline math $1 + 2$ and display math $$x + y$$
    ///
    /// $$
    /// x^2
    /// $$
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.math_dollars = true;
    /// assert_eq!(markdown_to_html("$1 + 2$ and $$x = y$$", &options),
    ///            "<p><span data-math-style=\"inline\">1 + 2</span> and <span data-math-style=\"display\">x = y</span></p>\n");
    /// assert_eq!(markdown_to_html("$$\nx^2\n$$\n", &options),
    ///            "<p><span data-math-style=\"display\">\nx^2\n</span></p>\n");
    /// ```
    #[builder(default)]
    pub math_dollars: bool,

    /// Enables math using code syntax.
    ///
    /// ```` md
    /// Inline math $`1 + 2`$
    ///
    /// ```math
    /// x^2
    /// ```
    /// ````
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.math_code = true;
    /// assert_eq!(markdown_to_html("$`1 + 2`$", &options),
    ///            "<p><code data-math-style=\"inline\">1 + 2</code></p>\n");
    /// assert_eq!(markdown_to_html("```math\nx^2\n```\n", &options),
    ///            "<pre><code class=\"language-math\" data-math-style=\"display\">x^2\n</code></pre>\n");
    /// ```
    #[builder(default)]
    pub math_code: bool,

    #[cfg(feature = "shortcodes")]
    #[cfg_attr(docsrs, doc(cfg(feature = "shortcodes")))]
    /// Phrases wrapped inside of ':' blocks will be replaced with emojis.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("Happy Friday! :smile:", &options),
    ///            "<p>Happy Friday! :smile:</p>\n");
    ///
    /// options.extension.shortcodes = true;
    /// assert_eq!(markdown_to_html("Happy Friday! :smile:", &options),
    ///            "<p>Happy Friday! 😄</p>\n");
    /// ```
    #[builder(default)]
    pub shortcodes: bool,

    /// Enables wikilinks using title after pipe syntax
    ///
    /// ```` md
    /// [[url|link label]]
    /// ````
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.wikilinks_title_after_pipe = true;
    /// assert_eq!(markdown_to_html("[[url|link label]]", &options),
    ///            "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n");
    /// ```
    #[builder(default)]
    pub wikilinks_title_after_pipe: bool,

    /// Enables wikilinks using title before pipe syntax
    ///
    /// ```` md
    /// [[link label|url]]
    /// ````
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.wikilinks_title_before_pipe = true;
    /// assert_eq!(markdown_to_html("[[link label|url]]", &options),
    ///            "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n");
    /// ```
    #[builder(default)]
    pub wikilinks_title_before_pipe: bool,

    /// Enables underlines using double underscores
    ///
    /// ```md
    /// __underlined text__
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.underline = true;
    ///
    /// assert_eq!(markdown_to_html("__underlined text__", &options),
    ///            "<p><u>underlined text</u></p>\n");
    /// ```
    #[builder(default)]
    pub underline: bool,

    /// Enables spoilers using double vertical bars
    ///
    /// ```md
    /// Darth Vader is ||Luke's father||
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.spoiler = true;
    ///
    /// assert_eq!(markdown_to_html("Darth Vader is ||Luke's father||", &options),
    ///            "<p>Darth Vader is <span class=\"spoiler\">Luke's father</span></p>\n");
    /// ```
    #[builder(default)]
    pub spoiler: bool,

    /// Requires at least one space after a `>` character to generate a blockquote,
    /// and restarts blockquote nesting across unique lines of input
    ///
    /// ```md
    /// >implying implications
    ///
    /// > one
    /// > > two
    /// > three
    /// ```
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.greentext = true;
    ///
    /// assert_eq!(markdown_to_html(">implying implications", &options),
    ///            "<p>&gt;implying implications</p>\n");
    ///
    /// assert_eq!(markdown_to_html("> one\n> > two\n> three", &options),
    ///            concat!(
    ///             "<blockquote>\n",
    ///             "<p>one</p>\n",
    ///             "<blockquote>\n<p>two</p>\n</blockquote>\n",
    ///             "<p>three</p>\n",
    ///             "</blockquote>\n"));
    /// ```
    #[builder(default)]
    pub greentext: bool,
}

#[non_exhaustive]
#[derive(Default, Clone, Builder)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
/// Options for parser functions.
pub struct ParseOptions<'c> {
    /// Punctuation (quotes, full-stops and hyphens) are converted into 'smart' punctuation.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("'Hello,' \"world\" ...", &options),
    ///            "<p>'Hello,' &quot;world&quot; ...</p>\n");
    ///
    /// options.parse.smart = true;
    /// assert_eq!(markdown_to_html("'Hello,' \"world\" ...", &options),
    ///            "<p>‘Hello,’ “world” …</p>\n");
    /// ```
    #[builder(default)]
    pub smart: bool,

    /// The default info string for fenced code blocks.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("```\nfn hello();\n```\n", &options),
    ///            "<pre><code>fn hello();\n</code></pre>\n");
    ///
    /// options.parse.default_info_string = Some("rust".into());
    /// assert_eq!(markdown_to_html("```\nfn hello();\n```\n", &options),
    ///            "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n");
    /// ```
    pub default_info_string: Option<String>,

    /// Whether or not a simple `x` or `X` is used for tasklist or any other symbol is allowed.
    #[builder(default)]
    pub relaxed_tasklist_matching: bool,

    /// Relax parsing of autolinks, allow links to be detected inside brackets
    /// and allow all url schemes. It is intended to allow a very specific type of autolink
    /// detection, such as `[this http://and.com that]` or `{http://foo.com}`, on a best can basis.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.extension.autolink = true;
    /// assert_eq!(markdown_to_html("[https://foo.com]", &options),
    ///            "<p>[https://foo.com]</p>\n");
    ///
    /// options.parse.relaxed_autolinks = true;
    /// assert_eq!(markdown_to_html("[https://foo.com]", &options),
    ///            "<p>[<a href=\"https://foo.com\">https://foo.com</a>]</p>\n");
    /// ```
    #[builder(default)]
    pub relaxed_autolinks: bool,

    /// In case the parser encounters any potential links that have a broken
    /// reference (e.g `[foo]` when there is no `[foo]: url` entry at the
    /// bottom) the provided callback will be called with the reference name,
    /// both in normalized form and unmodified, and the returned pair will be
    /// used as the link destination and title if not [`None`].
    ///
    /// ```
    /// # use std::{str, sync::{Arc, Mutex}};
    /// # use comrak::{Arena, ResolvedReference, parse_document, format_html, Options, BrokenLinkReference, ParseOptions};
    /// # use comrak::nodes::{AstNode, NodeValue};
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// let arena = Arena::new();
    /// let mut cb = |link_ref: BrokenLinkReference| match link_ref.normalized {
    ///     "foo" => Some(ResolvedReference {
    ///         url: "https://www.rust-lang.org/".to_string(),
    ///         title: "The Rust Language".to_string(),
    ///     }),
    ///     _ => None,
    /// };
    /// let options = Options {
    ///     parse: ParseOptions::builder()
    ///         .broken_link_callback(Arc::new(Mutex::new(&mut cb)))
    ///         .build(),
    ///     ..Default::default()
    /// };
    ///
    /// let root = parse_document(
    ///     &arena,
    ///     "# Cool input!\nWow look at this cool [link][foo]. A [broken link] renders as text.",
    ///     &options,
    /// );
    ///
    /// let mut output = Vec::new();
    /// format_html(root, &Options::default(), &mut output)?;
    /// assert_eq!(str::from_utf8(&output).unwrap(),
    ///            "<h1>Cool input!</h1>\n<p>Wow look at this cool \
    ///            <a href=\"https://www.rust-lang.org/\" title=\"The Rust Language\">link</a>. \
    ///            A [broken link] renders as text.</p>\n");
    /// # Ok(())
    /// # }
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub broken_link_callback: Option<Arc<Mutex<BrokenLinkCallback<'c>>>>,
}

impl<'c> fmt::Debug for ParseOptions<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut struct_fmt = f.debug_struct("ParseOptions");
        struct_fmt.field("smart", &self.smart);
        struct_fmt.field("default_info_string", &self.default_info_string);
        struct_fmt.field("relaxed_tasklist_matching", &self.relaxed_tasklist_matching);
        struct_fmt.field("relaxed_autolinks", &self.relaxed_autolinks);
        struct_fmt.field(
            "broken_link_callback.is_some()",
            &self.broken_link_callback.is_some(),
        );
        struct_fmt.finish()
    }
}

#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, Builder)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
/// Options for formatter functions.
pub struct RenderOptions {
    /// [Soft line breaks](http://spec.commonmark.org/0.27/#soft-line-breaks) in the input
    /// translate into hard line breaks in the output.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("Hello.\nWorld.\n", &options),
    ///            "<p>Hello.\nWorld.</p>\n");
    ///
    /// options.render.hardbreaks = true;
    /// assert_eq!(markdown_to_html("Hello.\nWorld.\n", &options),
    ///            "<p>Hello.<br />\nWorld.</p>\n");
    /// ```
    #[builder(default)]
    pub hardbreaks: bool,

    /// GitHub-style `<pre lang="xyz">` is used for fenced code blocks with info tags.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("``` rust\nfn hello();\n```\n", &options),
    ///            "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n");
    ///
    /// options.render.github_pre_lang = true;
    /// assert_eq!(markdown_to_html("``` rust\nfn hello();\n```\n", &options),
    ///            "<pre lang=\"rust\"><code>fn hello();\n</code></pre>\n");
    /// ```
    #[builder(default)]
    pub github_pre_lang: bool,

    /// Enable full info strings for code blocks
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// assert_eq!(markdown_to_html("``` rust extra info\nfn hello();\n```\n", &options),
    ///            "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n");
    ///
    /// options.render.full_info_string = true;
    /// let html = markdown_to_html("``` rust extra info\nfn hello();\n```\n", &options);
    /// let re = regex::Regex::new(r#"data-meta="extra info""#).unwrap();
    /// assert!(re.is_match(&html));
    /// ```
    #[builder(default)]
    pub full_info_string: bool,

    /// The wrap column when outputting CommonMark.
    ///
    /// ```
    /// # use comrak::{parse_document, Options, format_commonmark};
    /// # fn main() {
    /// # let arena = typed_arena::Arena::new();
    /// let mut options = Options::default();
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
    #[builder(default)]
    pub width: usize,

    /// Allow rendering of raw HTML and potentially dangerous links.
    ///
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
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
    #[builder(default)]
    pub unsafe_: bool,

    /// Escape raw HTML instead of clobbering it.
    /// ```
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "<i>italic text</i>";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><!-- raw HTML omitted -->italic text<!-- raw HTML omitted --></p>\n");
    ///
    /// options.render.escape = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p>&lt;i&gt;italic text&lt;/i&gt;</p>\n");
    /// ```
    #[builder(default)]
    pub escape: bool,

    /// Set the type of [bullet list marker](https://spec.commonmark.org/0.30/#bullet-list-marker) to use. Options are:
    ///
    /// * [`ListStyleType::Dash`] to use `-` (default)
    /// * [`ListStyleType::Plus`] to use `+`
    /// * [`ListStyleType::Star`] to use `*`
    ///
    /// ```rust
    /// # use comrak::{markdown_to_commonmark, Options, ListStyleType};
    /// let mut options = Options::default();
    /// let input = "- one\n- two\n- three";
    /// assert_eq!(markdown_to_commonmark(input, &options),
    ///            "- one\n- two\n- three\n"); // default is Dash
    ///
    /// options.render.list_style = ListStyleType::Plus;
    /// assert_eq!(markdown_to_commonmark(input, &options),
    ///            "+ one\n+ two\n+ three\n");
    ///
    /// options.render.list_style = ListStyleType::Star;
    /// assert_eq!(markdown_to_commonmark(input, &options),
    ///            "* one\n* two\n* three\n");
    /// ```
    #[builder(default)]
    pub list_style: ListStyleType,

    /// Include source position attributes in HTML and XML output.
    ///
    /// Sourcepos information is reliable for all core block items, and most
    /// extensions. The description lists extension still has issues; see
    /// <https://github.com/kivikakk/comrak/blob/3bb6d4ce/src/tests/description_lists.rs#L60-L125>.
    ///
    /// Sourcepos information is **not** reliable for inlines, and is not
    /// included in HTML without also setting [`experimental_inline_sourcepos`].
    /// See <https://github.com/kivikakk/comrak/pull/439> for a discussion.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_commonmark_xml, Options};
    /// let mut options = Options::default();
    /// options.render.sourcepos = true;
    /// let input = "## Hello world!";
    /// let xml = markdown_to_commonmark_xml(input, &options);
    /// assert!(xml.contains("<text sourcepos=\"1:4-1:15\" xml:space=\"preserve\">"));
    /// ```
    #[builder(default)]
    pub sourcepos: bool,

    /// Include inline sourcepos in HTML output, which is known to have issues.
    /// See <https://github.com/kivikakk/comrak/pull/439> for a discussion.
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// options.render.sourcepos = true;
    /// let input = "Hello *world*!";
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p data-sourcepos=\"1:1-1:14\">Hello <em>world</em>!</p>\n");
    /// options.render.experimental_inline_sourcepos = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p data-sourcepos=\"1:1-1:14\">Hello <em data-sourcepos=\"1:7-1:13\">world</em>!</p>\n");
    /// ```
    #[builder(default)]
    pub experimental_inline_sourcepos: bool,

    /// Wrap escaped characters in a `<span>` to allow any
    /// post-processing to recognize them.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "Notify user \\@example";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p>Notify user @example</p>\n");
    ///
    /// options.render.escaped_char_spans = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p>Notify user <span data-escaped-char>@</span>example</p>\n");
    /// ```
    #[builder(default)]
    pub escaped_char_spans: bool,

    /// Ignore setext headings in input.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "setext heading\n---";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<h2>setext heading</h2>\n");
    ///
    /// options.render.ignore_setext = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p>setext heading</p>\n<hr />\n");
    /// ```
    #[builder(default)]
    pub ignore_setext: bool,

    /// Ignore empty links in input.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "[]()";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><a href=\"\"></a></p>\n");
    ///
    /// options.render.ignore_empty_links = true;
    /// assert_eq!(markdown_to_html(input, &options), "<p>[]()</p>\n");
    /// ```
    #[builder(default)]
    pub ignore_empty_links: bool,

    /// Enables GFM quirks in HTML output which break CommonMark compatibility.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "****abcd**** *_foo_*";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><strong><strong>abcd</strong></strong> <em><em>foo</em></em></p>\n");
    ///
    /// options.render.gfm_quirks = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><strong>abcd</strong> <em><em>foo</em></em></p>\n");
    /// ```
    #[builder(default)]
    pub gfm_quirks: bool,

    /// Prefer fenced code blocks when outputting CommonMark.
    ///
    /// ```rust
    /// # use std::str;
    /// # use comrak::{Arena, Options, format_commonmark, parse_document};
    /// let arena = Arena::new();
    /// let mut options = Options::default();
    /// let input = "```\nhello\n```\n";
    /// let root = parse_document(&arena, input, &options);
    ///
    /// let mut buf = Vec::new();
    /// format_commonmark(&root, &options, &mut buf);
    /// assert_eq!(str::from_utf8(&buf).unwrap(), "    hello\n");
    ///
    /// buf.clear();
    /// options.render.prefer_fenced = true;
    /// format_commonmark(&root, &options, &mut buf);
    /// assert_eq!(str::from_utf8(&buf).unwrap(), "```\nhello\n```\n");
    /// ```
    #[builder(default)]
    pub prefer_fenced: bool,

    /// Render the image as a figure element with the title as its caption.
    ///
    /// ```rust
    /// # use comrak::{markdown_to_html, Options};
    /// let mut options = Options::default();
    /// let input = "![image](https://example.com/image.png \"this is an image\")";
    ///
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><img src=\"https://example.com/image.png\" alt=\"image\" title=\"this is an image\" /></p>\n");
    ///
    /// options.render.figure_with_caption = true;
    /// assert_eq!(markdown_to_html(input, &options),
    ///            "<p><figure><img src=\"https://example.com/image.png\" alt=\"image\" title=\"this is an image\" /><figcaption>this is an image</figcaption></figure></p>\n");
    /// ```
    #[builder(default)]
    pub figure_with_caption: bool,
}

#[non_exhaustive]
#[derive(Default, Debug, Clone, Builder)]
/// Umbrella plugins struct.
pub struct Plugins<'p> {
    /// Configure render-time plugins.
    #[builder(default)]
    pub render: RenderPlugins<'p>,
}

#[non_exhaustive]
#[derive(Default, Clone, Builder)]
/// Plugins for alternative rendering.
pub struct RenderPlugins<'p> {
    /// Provide a syntax highlighter adapter implementation for syntax
    /// highlighting of codefence blocks.
    /// ```
    /// # use comrak::{markdown_to_html, Options, Plugins, markdown_to_html_with_plugins};
    /// # use comrak::adapters::SyntaxHighlighterAdapter;
    /// use std::collections::HashMap;
    /// use std::io::{self, Write};
    /// let options = Options::default();
    /// let mut plugins = Plugins::default();
    /// let input = "```rust\nfn main<'a>();\n```";
    ///
    /// assert_eq!(markdown_to_html_with_plugins(input, &options, &plugins),
    ///            "<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n</code></pre>\n");
    ///
    /// pub struct MockAdapter {}
    /// impl SyntaxHighlighterAdapter for MockAdapter {
    ///     fn write_highlighted(&self, output: &mut dyn Write, lang: Option<&str>, code: &str) -> io::Result<()> {
    ///         write!(output, "<span class=\"lang-{}\">{}</span>", lang.unwrap(), code)
    ///     }
    ///
    ///     fn write_pre_tag(&self, output: &mut dyn Write, _attributes: HashMap<String, String>) -> io::Result<()> {
    ///         output.write_all(b"<pre lang=\"rust\">")
    ///     }
    ///
    ///     fn write_code_tag(&self, output: &mut dyn Write, _attributes: HashMap<String, String>) -> io::Result<()> {
    ///         output.write_all(b"<code class=\"language-rust\">")
    ///     }
    /// }
    ///
    /// let adapter = MockAdapter {};
    /// plugins.render.codefence_syntax_highlighter = Some(&adapter);
    ///
    /// assert_eq!(markdown_to_html_with_plugins(input, &options, &plugins),
    ///            "<pre lang=\"rust\"><code class=\"language-rust\"><span class=\"lang-rust\">fn main<'a>();\n</span></code></pre>\n");
    /// ```
    pub codefence_syntax_highlighter: Option<&'p dyn SyntaxHighlighterAdapter>,

    /// Optional heading adapter
    pub heading_adapter: Option<&'p dyn HeadingAdapter>,
}

impl Debug for RenderPlugins<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPlugins")
            .field(
                "codefence_syntax_highlighter",
                &"impl SyntaxHighlighterAdapter",
            )
            .finish()
    }
}

/// A reference link's resolved details.
#[derive(Clone, Debug)]
pub struct ResolvedReference {
    /// The destination URL of the reference link.
    pub url: String,

    /// The text of the link.
    pub title: String,
}

struct FootnoteDefinition<'a> {
    ix: Option<u32>,
    node: &'a AstNode<'a>,
    name: String,
    total_references: u32,
}

impl<'a, 'o, 'c: 'o> Parser<'a, 'o, 'c> {
    fn new(arena: &'a Arena<AstNode<'a>>, root: &'a AstNode<'a>, options: &'o Options<'c>) -> Self {
        Parser {
            arena,
            refmap: RefMap::new(),
            root,
            current: root,
            line_number: 0,
            offset: 0,
            column: 0,
            thematic_break_kill_pos: 0,
            first_nonspace: 0,
            first_nonspace_column: 0,
            indent: 0,
            blank: false,
            partially_consumed_tab: false,
            curline_len: 0,
            curline_end_col: 0,
            last_line_length: 0,
            last_buffer_ended_with_cr: false,
            total_size: 0,
            options,
        }
    }

    fn feed(&mut self, linebuf: &mut Vec<u8>, mut s: &str, eof: bool) {
        if let (0, Some(delimiter)) = (
            self.total_size,
            &self.options.extension.front_matter_delimiter,
        ) {
            if let Some((front_matter, rest)) = split_off_front_matter(s, delimiter) {
                let node = self.add_child(
                    self.root,
                    NodeValue::FrontMatter(front_matter.to_string()),
                    1,
                );
                s = rest;
                self.finalize(node).unwrap();
            }
        }

        let s = s.as_bytes();

        if s.len() > usize::MAX - self.total_size {
            self.total_size = usize::MAX;
        } else {
            self.total_size += s.len();
        }

        let mut buffer = 0;
        if self.last_buffer_ended_with_cr && !s.is_empty() && s[0] == b'\n' {
            buffer += 1;
        }
        self.last_buffer_ended_with_cr = false;

        let end = s.len();

        while buffer < end {
            let mut process = false;
            let mut eol = buffer;
            while eol < end {
                if strings::is_line_end_char(s[eol]) {
                    process = true;
                    break;
                }
                if s[eol] == 0 {
                    break;
                }
                eol += 1;
            }

            if eol >= end && eof {
                process = true;
            }

            if process {
                if !linebuf.is_empty() {
                    linebuf.extend_from_slice(&s[buffer..eol]);
                    self.process_line(linebuf);
                    linebuf.truncate(0);
                } else {
                    self.process_line(&s[buffer..eol]);
                }
            } else if eol < end && s[eol] == b'\0' {
                linebuf.extend_from_slice(&s[buffer..eol]);
                linebuf.extend_from_slice(&"\u{fffd}".to_string().into_bytes());
            } else {
                linebuf.extend_from_slice(&s[buffer..eol]);
            }

            buffer = eol;
            if buffer < end {
                if s[buffer] == b'\0' {
                    buffer += 1;
                } else {
                    if s[buffer] == b'\r' {
                        buffer += 1;
                        if buffer == end {
                            self.last_buffer_ended_with_cr = true;
                        }
                    }
                    if buffer < end && s[buffer] == b'\n' {
                        buffer += 1;
                    }
                }
            }
        }
    }

    fn scan_thematic_break_inner(&mut self, line: &[u8]) -> (usize, bool) {
        let mut i = self.first_nonspace;

        if i >= line.len() {
            return (i, false);
        }

        let c = line[i];
        if c != b'*' && c != b'_' && c != b'-' {
            return (i, false);
        }

        let mut count = 1;
        let mut nextc;
        loop {
            i += 1;
            if i >= line.len() {
                return (i, false);
            }
            nextc = line[i];

            if nextc == c {
                count += 1;
            } else if nextc != b' ' && nextc != b'\t' {
                break;
            }
        }

        if count >= 3 && (nextc == b'\r' || nextc == b'\n') {
            ((i - self.first_nonspace) + 1, true)
        } else {
            (i, false)
        }
    }

    fn scan_thematic_break(&mut self, line: &[u8]) -> Option<usize> {
        let (offset, found) = self.scan_thematic_break_inner(line);
        if !found {
            self.thematic_break_kill_pos = offset;
            None
        } else {
            Some(offset)
        }
    }

    fn find_first_nonspace(&mut self, line: &[u8]) {
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        if self.first_nonspace <= self.offset {
            self.first_nonspace = self.offset;
            self.first_nonspace_column = self.column;

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

        self.curline_len = line.len();
        self.curline_end_col = line.len();
        if self.curline_end_col > 0 && line[self.curline_end_col - 1] == b'\n' {
            self.curline_end_col -= 1;
        }
        if self.curline_end_col > 0 && line[self.curline_end_col - 1] == b'\r' {
            self.curline_end_col -= 1;
        }

        self.offset = 0;
        self.column = 0;
        self.first_nonspace = 0;
        self.first_nonspace_column = 0;
        self.indent = 0;
        self.thematic_break_kill_pos = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        if self.line_number == 0
            && line.len() >= 3
            && unsafe { str::from_utf8_unchecked(line) }.starts_with('\u{feff}')
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

        self.last_line_length = self.curline_end_col;

        self.curline_len = 0;
        self.curline_end_col = 0;
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
                NodeValue::BlockQuote => {
                    if !self.parse_block_quote_prefix(line) {
                        return (false, container, should_continue);
                    }
                }
                NodeValue::Item(ref nl) => {
                    if !self.parse_node_item_prefix(line, container, nl) {
                        return (false, container, should_continue);
                    }
                }
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
                NodeValue::HtmlBlock(ref nhb) => {
                    if !self.parse_html_block_prefix(nhb.block_type) {
                        return (false, container, should_continue);
                    }
                }
                NodeValue::Paragraph => {
                    if self.blank {
                        return (false, container, should_continue);
                    }
                }
                NodeValue::Table(..) => {
                    if !table::matches(&line[self.first_nonspace..], self.options.extension.spoiler)
                    {
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
                NodeValue::MultilineBlockQuote(..) => {
                    if !self.parse_multiline_block_quote_prefix(
                        line,
                        container,
                        ast,
                        &mut should_continue,
                    ) {
                        return (false, container, should_continue);
                    }
                }
                _ => {}
            }
        }

        (true, container, should_continue)
    }

    fn is_not_greentext(&mut self, line: &[u8]) -> bool {
        !self.options.extension.greentext || strings::is_space_or_tab(line[self.first_nonspace + 1])
    }

    fn setext_heading_line(&mut self, s: &[u8]) -> Option<SetextChar> {
        match self.options.render.ignore_setext {
            false => scanners::setext_heading_line(s),
            true => None,
        }
    }

    fn open_new_blocks(&mut self, container: &mut &'a AstNode<'a>, line: &[u8], all_matched: bool) {
        let mut matched: usize = 0;
        let mut nl: NodeList = NodeList::default();
        let mut sc: scanners::SetextChar = scanners::SetextChar::Equals;
        let mut maybe_lazy = node_matches!(self.current, NodeValue::Paragraph);
        let mut depth = 0;

        while !node_matches!(
            container,
            NodeValue::CodeBlock(..) | NodeValue::HtmlBlock(..)
        ) {
            depth += 1;
            self.find_first_nonspace(line);
            let indented = self.indent >= CODE_INDENT;

            if !indented
                && self.options.extension.multiline_block_quotes
                && unwrap_into(
                    scanners::open_multiline_block_quote_fence(&line[self.first_nonspace..]),
                    &mut matched,
                )
            {
                let first_nonspace = self.first_nonspace;
                let offset = self.offset;
                let nmbc = NodeMultilineBlockQuote {
                    fence_length: matched,
                    fence_offset: first_nonspace - offset,
                };
                *container = self.add_child(
                    container,
                    NodeValue::MultilineBlockQuote(nmbc),
                    self.first_nonspace + 1,
                );
                self.advance_offset(line, first_nonspace + matched - offset, false);
            } else if !indented && line[self.first_nonspace] == b'>' && self.is_not_greentext(line)
            {
                let blockquote_startpos = self.first_nonspace;

                let offset = self.first_nonspace + 1 - self.offset;
                self.advance_offset(line, offset, false);
                if strings::is_space_or_tab(line[self.offset]) {
                    self.advance_offset(line, 1, true);
                }
                *container =
                    self.add_child(container, NodeValue::BlockQuote, blockquote_startpos + 1);
            } else if !indented
                && unwrap_into(
                    scanners::atx_heading_start(&line[self.first_nonspace..]),
                    &mut matched,
                )
            {
                let heading_startpos = self.first_nonspace;
                let offset = self.offset;
                self.advance_offset(line, heading_startpos + matched - offset, false);
                *container = self.add_child(
                    container,
                    NodeValue::Heading(NodeHeading::default()),
                    heading_startpos + 1,
                );

                let mut hashpos = line[self.first_nonspace..]
                    .iter()
                    .position(|&c| c == b'#')
                    .unwrap()
                    + self.first_nonspace;
                let mut level = 0;
                while line[hashpos] == b'#' {
                    level += 1;
                    hashpos += 1;
                }

                let container_ast = &mut container.data.borrow_mut();
                container_ast.value = NodeValue::Heading(NodeHeading {
                    level,
                    setext: false,
                });
                container_ast.internal_offset = matched;
            } else if !indented
                && unwrap_into(
                    scanners::open_code_fence(&line[self.first_nonspace..]),
                    &mut matched,
                )
            {
                let first_nonspace = self.first_nonspace;
                let offset = self.offset;
                let ncb = NodeCodeBlock {
                    fenced: true,
                    fence_char: line[first_nonspace],
                    fence_length: matched,
                    fence_offset: first_nonspace - offset,
                    info: String::with_capacity(10),
                    literal: String::new(),
                };
                *container = self.add_child(
                    container,
                    NodeValue::CodeBlock(ncb),
                    self.first_nonspace + 1,
                );
                self.advance_offset(line, first_nonspace + matched - offset, false);
            } else if !indented
                && (unwrap_into(
                    scanners::html_block_start(&line[self.first_nonspace..]),
                    &mut matched,
                ) || (!node_matches!(container, NodeValue::Paragraph)
                    && unwrap_into(
                        scanners::html_block_start_7(&line[self.first_nonspace..]),
                        &mut matched,
                    )))
            {
                let nhb = NodeHtmlBlock {
                    block_type: matched as u8,
                    literal: String::new(),
                };

                *container = self.add_child(
                    container,
                    NodeValue::HtmlBlock(nhb),
                    self.first_nonspace + 1,
                );
            } else if !indented
                && node_matches!(container, NodeValue::Paragraph)
                && unwrap_into(
                    self.setext_heading_line(&line[self.first_nonspace..]),
                    &mut sc,
                )
            {
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
            } else if !indented
                && !matches!(
                    (&container.data.borrow().value, all_matched),
                    (&NodeValue::Paragraph, false)
                )
                && self.thematic_break_kill_pos <= self.first_nonspace
                && unwrap_into(self.scan_thematic_break(line), &mut matched)
            {
                *container =
                    self.add_child(container, NodeValue::ThematicBreak, self.first_nonspace + 1);
                let adv = line.len() - 1 - self.offset;
                self.advance_offset(line, adv, false);
            } else if !indented
                && self.options.extension.footnotes
                && depth < MAX_LIST_DEPTH
                && unwrap_into(
                    scanners::footnote_definition(&line[self.first_nonspace..]),
                    &mut matched,
                )
            {
                let mut c = &line[self.first_nonspace + 2..self.first_nonspace + matched];
                c = c.split(|&e| e == b']').next().unwrap();
                let offset = self.first_nonspace + matched - self.offset;
                self.advance_offset(line, offset, false);
                *container = self.add_child(
                    container,
                    NodeValue::FootnoteDefinition(NodeFootnoteDefinition {
                        name: str::from_utf8(c).unwrap().to_string(),
                        total_references: 0,
                    }),
                    self.first_nonspace + 1,
                );
                container.data.borrow_mut().internal_offset = matched;
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
            } else if (!indented || node_matches!(container, NodeValue::List(..)))
                && self.indent < 4
                && depth < MAX_LIST_DEPTH
                && unwrap_into_2(
                    parse_list_marker(
                        line,
                        self.first_nonspace,
                        node_matches!(container, NodeValue::Paragraph),
                    ),
                    &mut matched,
                    &mut nl,
                )
            {
                let offset = self.first_nonspace + matched - self.offset;
                self.advance_offset(line, offset, false);
                let (save_partially_consumed_tab, save_offset, save_column) =
                    (self.partially_consumed_tab, self.offset, self.column);

                while self.column - save_column <= 5 && strings::is_space_or_tab(line[self.offset])
                {
                    self.advance_offset(line, 1, true);
                }

                let i = self.column - save_column;
                if !(1..5).contains(&i) || strings::is_line_end_char(line[self.offset]) {
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
                    *container =
                        self.add_child(container, NodeValue::List(nl), self.first_nonspace + 1);
                }

                *container =
                    self.add_child(container, NodeValue::Item(nl), self.first_nonspace + 1);
            } else if indented && !maybe_lazy && !self.blank {
                self.advance_offset(line, CODE_INDENT, true);
                let ncb = NodeCodeBlock {
                    fenced: false,
                    fence_char: 0,
                    fence_length: 0,
                    fence_offset: 0,
                    info: String::new(),
                    literal: String::new(),
                };
                *container = self.add_child(container, NodeValue::CodeBlock(ncb), self.offset + 1);
            } else {
                let new_container = if !indented && self.options.extension.table {
                    table::try_opening_block(self, container, line)
                } else {
                    None
                };

                match new_container {
                    Some((new_container, replace, mark_visited)) => {
                        if replace {
                            container.insert_after(new_container);
                            container.detach();
                            *container = new_container;
                        } else {
                            *container = new_container;
                        }
                        if mark_visited {
                            container.data.borrow_mut().table_visited = true;
                        }
                    }
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
        if indent <= 3 && line[self.first_nonspace] == b'>' && self.is_not_greentext(line) {
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
            1..=5 => true,
            6 | 7 => !self.blank,
            _ => unreachable!(),
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
            let last_child_sourcepos = last_child.data.borrow().sourcepos;

            // TODO: description list sourcepos has issues.
            //
            // DescriptionItem:
            //   For all but the last, the end line/col is wrong.
            //   Where it should be l:c, it gives (l+1):0.
            //
            // DescriptionTerm:
            //   All are incorrect; they all give the start line/col of
            //   the DescriptionDetails, and the end line/col is completely off.
            //
            // descriptionDetails:
            //   Same as the DescriptionItem.  All but last, the end line/col
            //   is (l+1):0.
            //
            // See crate::tests::description_lists::sourcepos.
            let list = match container.last_child() {
                Some(lc) if node_matches!(lc, NodeValue::DescriptionList) => {
                    reopen_ast_nodes(lc);
                    lc
                }
                _ => {
                    let list = self.add_child(
                        container,
                        NodeValue::DescriptionList,
                        self.first_nonspace + 1,
                    );
                    list.data.borrow_mut().sourcepos.start = last_child_sourcepos.start;
                    list
                }
            };

            let metadata = NodeDescriptionItem {
                marker_offset: self.indent,
                padding: 2,
            };

            let item = self.add_child(
                list,
                NodeValue::DescriptionItem(metadata),
                self.first_nonspace + 1,
            );
            item.data.borrow_mut().sourcepos.start = last_child_sourcepos.start;
            let term = self.add_child(item, NodeValue::DescriptionTerm, self.first_nonspace + 1);
            let details =
                self.add_child(item, NodeValue::DescriptionDetails, self.first_nonspace + 1);

            term.append(last_child);

            *container = details;

            true
        } else {
            false
        }
    }

    fn parse_multiline_block_quote_prefix(
        &mut self,
        line: &[u8],
        container: &'a AstNode<'a>,
        ast: &mut Ast,
        should_continue: &mut bool,
    ) -> bool {
        let (fence_length, fence_offset) = match ast.value {
            NodeValue::MultilineBlockQuote(ref node_value) => {
                (node_value.fence_length, node_value.fence_offset)
            }
            _ => unreachable!(),
        };

        let matched = if self.indent <= 3 && line[self.first_nonspace] == b'>' {
            scanners::close_multiline_block_quote_fence(&line[self.first_nonspace..]).unwrap_or(0)
        } else {
            0
        };

        if matched >= fence_length {
            *should_continue = false;
            self.advance_offset(line, matched, false);

            // The last child, like an indented codeblock, could be left open.
            // Make sure it's finalized.
            if nodes::last_child_is_open(container) {
                let child = container.last_child().unwrap();
                let child_ast = &mut *child.data.borrow_mut();

                self.finalize_borrowed(child, child_ast).unwrap();
            }

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

    fn add_child(
        &mut self,
        mut parent: &'a AstNode<'a>,
        value: NodeValue,
        start_column: usize,
    ) -> &'a AstNode<'a> {
        while !nodes::can_contain_type(parent, &value) {
            parent = self.finalize(parent).unwrap();
        }

        assert!(start_column > 0);

        let child = Ast::new(value, (self.line_number, start_column).into());
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
                        || container.data.borrow().sourcepos.start.line != self.line_number
                }
                NodeValue::MultilineBlockQuote(..) => false,
                _ => true,
            };

        let mut tmp = container;
        while let Some(parent) = tmp.parent() {
            parent.data.borrow_mut().last_line_blank = false;
            tmp = parent;
        }

        if !self.current.same_node(last_matched_container)
            && container.same_node(last_matched_container)
            && !self.blank
            && (!self.options.extension.greentext
                || !matches!(
                    container.data.borrow().value,
                    NodeValue::BlockQuote | NodeValue::Document
                ))
            && node_matches!(self.current, NodeValue::Paragraph)
        {
            self.add_line(self.current, line);
        } else {
            while !self.current.same_node(last_matched_container) {
                self.current = self.finalize(self.current).unwrap();
            }

            let add_text_result = match container.data.borrow().value {
                NodeValue::CodeBlock(..) => AddTextResult::LiteralText,
                NodeValue::HtmlBlock(ref nhb) => AddTextResult::HtmlBlock(nhb.block_type),
                _ => AddTextResult::Otherwise,
            };

            match add_text_result {
                AddTextResult::LiteralText => {
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
                        container = self.add_child(
                            container,
                            NodeValue::Paragraph,
                            self.first_nonspace + 1,
                        );
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
                ast.content.push(' ');
            }
        }
        if self.offset < line.len() {
            // since whitespace is stripped off the beginning of lines, we need to keep
            // track of how much was stripped off. This allows us to properly calculate
            // inline sourcepos during inline processing.
            ast.line_offsets.push(self.offset);

            ast.content
                .push_str(str::from_utf8(&line[self.offset..]).unwrap());
        }
    }

    fn finish(&mut self, remaining: Vec<u8>) -> &'a AstNode<'a> {
        if !remaining.is_empty() {
            self.process_line(&remaining);
        }

        self.finalize_document();
        self.postprocess_text_nodes(self.root);
        self.root
    }

    fn finalize_document(&mut self) {
        while !self.current.same_node(self.root) {
            self.current = self.finalize(self.current).unwrap();
        }

        self.finalize(self.root);

        self.refmap.max_ref_size = if self.total_size > 100000 {
            self.total_size
        } else {
            100000
        };

        self.process_inlines();
        if self.options.extension.footnotes {
            self.process_footnotes();
        }
    }

    fn finalize(&mut self, node: &'a AstNode<'a>) -> Option<&'a AstNode<'a>> {
        self.finalize_borrowed(node, &mut node.data.borrow_mut())
    }

    fn resolve_reference_link_definitions(&mut self, content: &mut String) -> bool {
        let mut seeked = 0;
        {
            let mut pos = 0;
            let mut seek: &[u8] = content.as_bytes();
            while !seek.is_empty()
                && seek[0] == b'['
                && unwrap_into(self.parse_reference_inline(seek), &mut pos)
            {
                seek = &seek[pos..];
                seeked += pos;
            }
        }

        if seeked != 0 {
            *content = content[seeked..].to_string();
        }

        !strings::is_blank(content.as_bytes())
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

        if self.curline_len == 0 {
            ast.sourcepos.end = (self.line_number, self.last_line_length).into();
        } else if match ast.value {
            NodeValue::Document => true,
            NodeValue::CodeBlock(ref ncb) => ncb.fenced,
            NodeValue::MultilineBlockQuote(..) => true,
            _ => false,
        } {
            ast.sourcepos.end = (self.line_number, self.curline_end_col).into();
        } else {
            ast.sourcepos.end = (self.line_number - 1, self.last_line_length).into();
        }

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
                    content.push('\n');
                } else {
                    let mut pos = 0;
                    while pos < content.len() {
                        if strings::is_line_end_char(content.as_bytes()[pos]) {
                            break;
                        }
                        pos += 1;
                    }
                    assert!(pos < content.len());

                    let mut tmp = entity::unescape_html(&content.as_bytes()[..pos]);
                    strings::trim(&mut tmp);
                    strings::unescape(&mut tmp);
                    if tmp.is_empty() {
                        ncb.info = self
                            .options
                            .parse
                            .default_info_string
                            .as_ref()
                            .map_or(String::new(), |s| s.clone());
                    } else {
                        ncb.info = String::from_utf8(tmp).unwrap();
                    }

                    if content.as_bytes()[pos] == b'\r' {
                        pos += 1;
                    }
                    if content.as_bytes()[pos] == b'\n' {
                        pos += 1;
                    }

                    content.drain(..pos);
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
                        if (item.next_sibling().is_some() || subitem.next_sibling().is_some())
                            && nodes::ends_with_blank_line(subitem)
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
        let content = strings::rtrim_slice(node_data.content.as_bytes());
        let mut subj = inlines::Subject::new(
            self.arena,
            self.options,
            content,
            node_data.sourcepos.start.line,
            &mut self.refmap,
            &delimiter_arena,
        );

        while subj.parse_inline(node) {}

        subj.process_emphasis(0);

        while subj.pop_bracket() {}
    }

    fn process_footnotes(&mut self) {
        let mut map = HashMap::new();
        Self::find_footnote_definitions(self.root, &mut map);

        let mut ix = 0;
        Self::find_footnote_references(self.root, &mut map, &mut ix);

        if !map.is_empty() {
            // In order for references to be found inside footnote definitions,
            // such as `[^1]: another reference[^2]`,
            // the node needed to remain in the AST. Now we can remove them.
            Self::cleanup_footnote_definitions(self.root);
        }

        if ix > 0 {
            let mut v = map.into_values().collect::<Vec<_>>();
            v.sort_unstable_by(|a, b| a.ix.cmp(&b.ix));
            for f in v {
                if f.ix.is_some() {
                    match f.node.data.borrow_mut().value {
                        NodeValue::FootnoteDefinition(ref mut nfd) => {
                            nfd.name = f.name.to_string();
                            nfd.total_references = f.total_references;
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
        map: &mut HashMap<String, FootnoteDefinition<'a>>,
    ) {
        match node.data.borrow().value {
            NodeValue::FootnoteDefinition(ref nfd) => {
                map.insert(
                    strings::normalize_label(&nfd.name, Case::Fold),
                    FootnoteDefinition {
                        ix: None,
                        node,
                        name: strings::normalize_label(&nfd.name, Case::Preserve),
                        total_references: 0,
                    },
                );
            }
            _ => {
                for n in node.children() {
                    Self::find_footnote_definitions(n, map);
                }
            }
        }
    }

    fn find_footnote_references(
        node: &'a AstNode<'a>,
        map: &mut HashMap<String, FootnoteDefinition>,
        ixp: &mut u32,
    ) {
        let mut ast = node.data.borrow_mut();
        let mut replace = None;
        match ast.value {
            NodeValue::FootnoteReference(ref mut nfr) => {
                let normalized = strings::normalize_label(&nfr.name, Case::Fold);
                if let Some(ref mut footnote) = map.get_mut(&normalized) {
                    let ix = match footnote.ix {
                        Some(ix) => ix,
                        None => {
                            *ixp += 1;
                            footnote.ix = Some(*ixp);
                            *ixp
                        }
                    };
                    footnote.total_references += 1;
                    nfr.ref_num = footnote.total_references;
                    nfr.ix = ix;
                    nfr.name = strings::normalize_label(&footnote.name, Case::Preserve);
                } else {
                    replace = Some(nfr.name.clone());
                }
            }
            _ => {
                for n in node.children() {
                    Self::find_footnote_references(n, map, ixp);
                }
            }
        }

        if let Some(mut label) = replace {
            label.insert_str(0, "[^");
            label.push(']');
            ast.value = NodeValue::Text(label);
        }
    }

    fn cleanup_footnote_definitions(node: &'a AstNode<'a>) {
        match node.data.borrow().value {
            NodeValue::FootnoteDefinition(_) => {
                node.detach();
            }
            _ => {
                for n in node.children() {
                    Self::cleanup_footnote_definitions(n);
                }
            }
        }
    }

    fn postprocess_text_nodes(&mut self, node: &'a AstNode<'a>) {
        let mut stack = vec![node];
        let mut children = vec![];

        while let Some(node) = stack.pop() {
            let mut nch = node.first_child();

            while let Some(n) = nch {
                let mut this_bracket = false;
                let n_ast = &mut n.data.borrow_mut();
                let mut sourcepos = n_ast.sourcepos;

                loop {
                    match n_ast.value {
                        // Join adjacent text nodes together
                        NodeValue::Text(ref mut root) => {
                            let ns = match n.next_sibling() {
                                Some(ns) => ns,
                                _ => {
                                    // Post-process once we are finished joining text nodes
                                    self.postprocess_text_node(n, root, &mut sourcepos);
                                    break;
                                }
                            };

                            match ns.data.borrow().value {
                                NodeValue::Text(ref adj) => {
                                    root.push_str(adj);
                                    sourcepos.end.column = ns.data.borrow().sourcepos.end.column;
                                    ns.detach();
                                }
                                _ => {
                                    // Post-process once we are finished joining text nodes
                                    self.postprocess_text_node(n, root, &mut sourcepos);
                                    break;
                                }
                            }
                        }
                        NodeValue::Link(..) | NodeValue::Image(..) | NodeValue::WikiLink(..) => {
                            this_bracket = true;
                            break;
                        }
                        _ => break,
                    }
                }

                n_ast.sourcepos = sourcepos;

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

    fn postprocess_text_node(
        &mut self,
        node: &'a AstNode<'a>,
        text: &mut String,
        sourcepos: &mut Sourcepos,
    ) {
        if self.options.extension.tasklist {
            self.process_tasklist(node, text, sourcepos);
        }

        if self.options.extension.autolink {
            autolink::process_autolinks(
                self.arena,
                node,
                text,
                self.options.parse.relaxed_autolinks,
            );
        }
    }

    fn process_tasklist(
        &mut self,
        node: &'a AstNode<'a>,
        text: &mut String,
        sourcepos: &mut Sourcepos,
    ) {
        let (end, symbol) = match scanners::tasklist(text.as_bytes()) {
            Some(p) => p,
            None => return,
        };

        let symbol = symbol as char;

        if !self.options.parse.relaxed_tasklist_matching && !matches!(symbol, ' ' | 'x' | 'X') {
            return;
        }

        let parent = node.parent().unwrap();
        if node.previous_sibling().is_some() || parent.previous_sibling().is_some() {
            return;
        }

        if !node_matches!(parent, NodeValue::Paragraph) {
            return;
        }

        if !node_matches!(parent.parent().unwrap(), NodeValue::Item(..)) {
            return;
        }

        text.drain(..end);

        // These are sound only because the exact text that we've matched and
        // the count thereof (i.e. "end") will precisely map to characters in
        // the source document.
        sourcepos.start.column += end;
        parent.data.borrow_mut().sourcepos.start.column += end;

        parent.parent().unwrap().data.borrow_mut().value =
            NodeValue::TaskItem(if symbol == ' ' { None } else { Some(symbol) });
    }

    fn parse_reference_inline(&mut self, content: &[u8]) -> Option<usize> {
        // In this case reference inlines rarely have delimiters
        // so we often just need the minimal case
        let delimiter_arena = Arena::with_capacity(0);
        let mut subj = inlines::Subject::new(
            self.arena,
            self.options,
            content,
            0, // XXX -1 in upstream; never used?
            &mut self.refmap,
            &delimiter_arena,
        );

        let mut lab: String = match subj.link_label() {
            Some(lab) if !lab.is_empty() => lab.to_string(),
            _ => return None,
        };

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
        let title_search = if subj.pos == beforetitle {
            None
        } else {
            scanners::link_title(&subj.input[subj.pos..])
        };
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

        lab = strings::normalize_label(&lab, Case::Fold);
        if !lab.is_empty() {
            subj.refmap.map.entry(lab).or_insert(ResolvedReference {
                url: String::from_utf8(strings::clean_url(url)).unwrap(),
                title: String::from_utf8(strings::clean_title(&title)).unwrap(),
            });
        }
        Some(subj.pos)
    }
}

enum AddTextResult {
    LiteralText,
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
                start,
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
    Uri,
    Email,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
/// Options for bulleted list redering in markdown. See `link_style` in [`RenderOptions`] for more details.
pub enum ListStyleType {
    /// The `-` character
    #[default]
    Dash = 45,
    /// The `+` character
    Plus = 43,
    /// The `*` character
    Star = 42,
}
