#![allow(rustdoc::invalid_html_tags)]

use clap::{Parser, ValueEnum};

use crate::{Options, options};

#[derive(Debug, Parser)]
pub struct OptionArgs {
    /// Treat newlines as hard line breaks
    #[arg(long)]
    pub hardbreaks: bool,

    /// Replace punctuation like "this" with smart punctuation like “this”
    #[arg(long)]
    pub smart: bool,

    /// Use GitHub-style "<pre lang>" for code blocks
    #[arg(long)]
    pub github_pre_lang: bool,

    /// Include words following the code block info string in a data-meta attribute
    #[arg(long)]
    pub full_info_string: bool,

    /// Enable GitHub-flavored markdown extensions: strikethrough, table,
    /// autolink, and tasklist. Also enables --github-pre-lang and --gfm-quirks.
    #[arg(long)]
    pub gfm: bool,

    /// Use GFM-style quirks in output HTML, such as not nesting <strong>
    /// tags, which otherwise breaks CommonMark compatibility.
    #[arg(long)]
    pub gfm_quirks: bool,

    /// Permit any character inside a tasklist item, not just " ", "x", or "X"
    #[arg(long)]
    pub relaxed_tasklist_character: bool,

    /// Allow task list items in table cells
    #[arg(long)]
    pub tasklist_in_table: bool,

    /// Relax autolink parsing: allows links to be recognised when in brackets,
    /// permits all URL schemes, and permits domains without a TLD (like "http://localhost")
    #[arg(long)]
    pub relaxed_autolinks: bool,

    /// Include "task-list-item" and "task-list-item-checkbox" classes on
    // tasklist "<li>" and "<input>" elements respectively
    #[arg(long)]
    pub tasklist_classes: bool,

    /// Default value for fenced code block's info strings if none is given
    #[arg(long, value_name = "INFO")]
    pub default_info_string: Option<String>,

    /// Allow inline and block HTML (unless --escape is given), and permit
    // dangerous URLs (like "javascript:" and non-image "data:" URLs)
    #[arg(long = "unsafe")]
    pub r#unsafe: bool,

    /// Translate gemoji like ":thumbsup:" into Unicode emoji like "👍"
    #[arg(long)]
    pub gemoji: bool,

    /// Escape raw HTML, instead of clobbering it; takes precedence over --unsafe
    #[arg(long)]
    pub escape: bool,

    /// Wrap escaped Markdown characters in "<span data-escaped-char>" in HTML
    #[arg(long)]
    pub escaped_char_spans: bool,

    /// Specify extensions to use
    ///
    /// Multiple extensions can be delimited with ",", e.g. '--extension
    /// strikethrough,table', or you can pass --extension/-e multiple times
    #[arg(
        short,
        long = "extension",
        value_name = "EXTENSION",
        value_delimiter = ',',
        value_enum
    )]
    pub extensions: Vec<Extension>,

    /// Specify wrap width for output CommonMark, or '0' to disable wrapping
    #[arg(long, default_value_t = 0)]
    pub width: usize,

    /// Use the Comrak header IDs extension, with the given ID prefix
    #[arg(long, value_name = "PREFIX", required = false)]
    pub header_id_prefix: Option<String>,

    /// Apply the header ID prefix to the href anchor as well
    #[arg(long)]
    pub header_id_prefix_in_href: bool,

    /// Detect frontmatter that starts and ends with the given string, and do
    /// not include it in the resulting document
    #[arg(long, value_name = "DELIMITER", allow_hyphen_values = true)]
    pub front_matter_delimiter: Option<String>,

    /// Specify bullet character for lists ("-", "+", "*") in CommonMark output
    #[arg(long, value_enum, default_value_t = ListStyle::Dash)]
    pub list_style: ListStyle,

    /// Specify alert style (<div> vs <aside>) in HTML output
    #[arg(long, value_enum, default_value_t = AlertStyle::Specific)]
    pub alert_style: AlertStyle,

    /// Include source position attributes in HTML and XML output
    #[arg(long)]
    pub sourcepos: bool,

    /// Do not parse setext headers
    #[arg(long)]
    pub ignore_setext: bool,

    /// Do not parse empty links
    #[arg(long)]
    pub ignore_empty_links: bool,

    /// Report column positions in sourcepos as a Unicode character count
    /// rather than UTF-8 byte offsets
    #[arg(long)]
    pub sourcepos_chars: bool,

    /// Minimise escapes in CommonMark output using a trial-and-error algorithm
    #[arg(long)]
    pub experimental_minimize_commonmark: bool,

    /// Suppress pretty-printing newlines between block-level HTML elements
    #[arg(long)]
    pub compact: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Extension {
    Strikethrough,
    Table,
    Autolink,
    Tasklist,
    Superscript,
    Footnotes,
    InlineFootnotes,
    DescriptionLists,
    MultilineBlockQuotes,
    MathDollars,
    MathLatex,
    MathCode,
    WikilinksTitleAfterPipe,
    WikilinksTitleBeforePipe,
    Underline,
    Subscript,
    Spoiler,
    Greentext,
    OfmComments,
    Alerts,
    OfmCallouts,
    CjkFriendlyEmphasis,
    Subtext,
    Highlight,
    Insert,
    PhoenixHeex,
    BlockDirective,
    HeaderAttributes,
    FencedCodeAttributes,
    InlineCodeAttributes,
    LinkAttributes,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ListStyle {
    Dash,
    Plus,
    Star,
}

impl From<ListStyle> for options::ListStyleType {
    fn from(style: ListStyle) -> Self {
        match style {
            ListStyle::Dash => Self::Dash,
            ListStyle::Plus => Self::Plus,
            ListStyle::Star => Self::Star,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AlertStyle {
    Specific,
    Semantic,
}

impl From<AlertStyle> for options::AlertStyleType {
    fn from(style: AlertStyle) -> Self {
        match style {
            AlertStyle::Specific => Self::Specific,
            AlertStyle::Semantic => Self::Semantic,
        }
    }
}

impl OptionArgs {
    pub fn to_options(&self) -> Options<'static> {
        let ext = |e| self.extensions.contains(&e);

        let extension = options::Extension::builder()
            .strikethrough(ext(Extension::Strikethrough) || self.gfm)
            .table(ext(Extension::Table) || self.gfm)
            .autolink(ext(Extension::Autolink) || self.gfm)
            .tasklist(ext(Extension::Tasklist) || self.gfm)
            .superscript(ext(Extension::Superscript))
            .maybe_header_id_prefix(self.header_id_prefix.clone())
            .header_id_prefix_in_href(self.header_id_prefix_in_href)
            .footnotes(ext(Extension::Footnotes))
            .inline_footnotes(ext(Extension::InlineFootnotes))
            .description_lists(ext(Extension::DescriptionLists))
            .multiline_block_quotes(ext(Extension::MultilineBlockQuotes))
            .math_dollars(ext(Extension::MathDollars))
            .math_latex(ext(Extension::MathLatex))
            .math_code(ext(Extension::MathCode))
            .wikilinks_title_after_pipe(ext(Extension::WikilinksTitleAfterPipe))
            .wikilinks_title_before_pipe(ext(Extension::WikilinksTitleBeforePipe))
            .underline(ext(Extension::Underline))
            .subscript(ext(Extension::Subscript))
            .spoiler(ext(Extension::Spoiler))
            .greentext(ext(Extension::Greentext))
            .ofm_comments(ext(Extension::OfmComments))
            .alerts(ext(Extension::Alerts))
            .ofm_callouts(ext(Extension::OfmCallouts))
            .maybe_front_matter_delimiter(self.front_matter_delimiter.clone())
            .cjk_friendly_emphasis(ext(Extension::CjkFriendlyEmphasis))
            .subtext(ext(Extension::Subtext))
            .highlight(ext(Extension::Highlight))
            .insert(ext(Extension::Insert))
            .phoenix_heex(ext(Extension::PhoenixHeex))
            .block_directive(ext(Extension::BlockDirective))
            .header_attributes(ext(Extension::HeaderAttributes))
            .fenced_code_attributes(ext(Extension::FencedCodeAttributes))
            .inline_code_attributes(ext(Extension::InlineCodeAttributes))
            .link_attributes(ext(Extension::LinkAttributes))
            .shortcodes(self.gemoji)
            .build();

        let parse = options::Parse::builder()
            .smart(self.smart)
            .maybe_default_info_string(self.default_info_string.clone())
            .relaxed_tasklist_matching(self.relaxed_tasklist_character)
            .tasklist_in_table(self.tasklist_in_table)
            .relaxed_autolinks(self.relaxed_autolinks)
            .ignore_setext(self.ignore_setext)
            .sourcepos_chars(self.sourcepos_chars)
            .build();

        let render = options::Render::builder()
            .hardbreaks(self.hardbreaks)
            .github_pre_lang(self.github_pre_lang || self.gfm)
            .full_info_string(self.full_info_string)
            .width(self.width)
            .r#unsafe(self.r#unsafe)
            .escape(self.escape)
            .list_style(self.list_style.into())
            .alert_style(self.alert_style.into())
            .sourcepos(self.sourcepos)
            .experimental_minimize_commonmark(self.experimental_minimize_commonmark)
            .compact_html(self.compact)
            .escaped_char_spans(self.escaped_char_spans)
            .ignore_empty_links(self.ignore_empty_links)
            .gfm_quirks(self.gfm_quirks || self.gfm)
            .tasklist_classes(self.tasklist_classes)
            .build();

        Options {
            extension,
            parse,
            render,
        }
    }
}
