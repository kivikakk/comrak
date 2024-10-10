//! The `comrak` binary.

use comrak::{
    adapters::SyntaxHighlighterAdapter, plugins::syntect::SyntectAdapter, Arena, ListStyleType,
    Options, Plugins,
};
use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};
use comrak::{ExtensionOptions, ParseOptions, RenderOptions};

const EXIT_SUCCESS: i32 = 0;
const EXIT_PARSE_CONFIG: i32 = 2;
const EXIT_READ_INPUT: i32 = 3;
const EXIT_CHECK_FILE_NUM: i32 = 4;

#[derive(Debug, Parser)]
#[command(about, author, version)]
#[command(after_help = "\
By default, Comrak will attempt to read command-line options from a config file specified by \
--config-file. This behaviour can be disabled by passing --config-file none. It is not an error \
if the file does not exist.\
        ")]
struct Cli {
    /// CommonMark file(s) to parse; or standard input if none passed
    #[arg(value_name = "FILE")]
    files: Option<Vec<PathBuf>>,

    /// Path to config file containing command-line arguments, or 'none'
    #[arg(short, long, value_name = "PATH", default_value = get_default_config_path())]
    config_file: String,

    /// To perform an in-place formatting
    #[arg(short, long, conflicts_with_all(["format", "output"]))]
    inplace: bool,

    /// Treat newlines as hard line breaks
    #[arg(long)]
    hardbreaks: bool,

    /// Use smart punctuation
    #[arg(long)]
    smart: bool,

    /// Use GitHub-style <pre lang> for code blocks
    #[arg(long)]
    github_pre_lang: bool,

    /// Enable full info strings for code blocks
    #[arg(long)]
    full_info_string: bool,

    /// Enable GitHub-flavored markdown extensions: strikethrough, tagfilter,
    /// table, autolink, and tasklist. Also enables --github-pre-lang and
    /// --gfm-quirks.
    #[arg(long)]
    gfm: bool,

    /// Enables GFM-style quirks in output HTML, such as not nesting <strong>
    /// tags, which otherwise breaks CommonMark compatibility.
    #[arg(long)]
    gfm_quirks: bool,

    /// Enable relaxing which character is allowed in a tasklists.
    #[arg(long)]
    relaxed_tasklist_character: bool,

    /// Enable relaxing of autolink parsing, allow links to be recognized when in brackets
    /// and allow all url schemes
    #[arg(long)]
    relaxed_autolinks: bool,

    /// Default value for fenced code block's info strings if none is given
    #[arg(long, value_name = "INFO")]
    default_info_string: Option<String>,

    /// Allow raw HTML and dangerous URLs
    #[arg(long = "unsafe")]
    unsafe_: bool,

    /// Translate gemojis into UTF-8 characters
    #[arg(long)]
    #[cfg(feature = "shortcodes")]
    gemojis: bool,

    /// Escape raw HTML instead of clobbering it
    #[arg(long)]
    escape: bool,

    /// Wrap escaped characters in span tags
    #[arg(long)]
    escaped_char_spans: bool,

    /// Specify extension name(s) to use
    ///
    /// Multiple extensions can be delimited with ",", e.g. --extension strikethrough,table
    #[arg(
        short,
        long = "extension",
        value_name = "EXTENSION",
        value_delimiter = ',',
        value_enum
    )]
    extensions: Vec<Extension>,

    /// Specify output format
    #[arg(short = 't', long = "to", value_enum, default_value_t = Format::Html)]
    format: Format,

    /// Write output to FILE instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Specify wrap width (0 = nowrap)
    #[arg(long, default_value_t = 0)]
    width: usize,

    /// Use the Comrak header IDs extension, with the given ID prefix
    #[arg(long, value_name = "PREFIX")]
    header_ids: Option<String>,

    /// Ignore front-matter that starts and ends with the given string
    #[arg(long, value_name = "DELIMITER", allow_hyphen_values = true)]
    front_matter_delimiter: Option<String>,

    /// Syntax highlighting for codefence blocks. Choose a theme or 'none' for disabling.
    #[arg(long, value_name = "THEME", default_value = "base16-ocean.dark")]
    syntax_highlighting: String,

    /// Specify bullet character for lists (-, +, *) in CommonMark output
    #[arg(long, value_enum, default_value_t = ListStyle::Dash)]
    list_style: ListStyle,

    /// Include source position attribute in HTML and XML output
    #[arg(long)]
    sourcepos: bool,

    /// Include inline sourcepos in HTML output, which is known to have issues.
    #[arg(long)]
    experimental_inline_sourcepos: bool,

    /// Ignore setext headers
    #[arg(long)]
    ignore_setext: bool,

    /// Ignore empty links
    #[arg(long)]
    ignore_empty_links: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Html,

    Xml,

    #[value(name = "commonmark")]
    CommonMark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Extension {
    Strikethrough,
    Tagfilter,
    Table,
    Autolink,
    Tasklist,
    Superscript,
    Footnotes,
    DescriptionLists,
    MultilineBlockQuotes,
    MathDollars,
    MathCode,
    WikilinksTitleAfterPipe,
    WikilinksTitleBeforePipe,
    Underline,
    Spoiler,
    Greentext,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ListStyle {
    Dash,
    Plus,
    Star,
}

impl From<ListStyle> for ListStyleType {
    fn from(style: ListStyle) -> Self {
        match style {
            ListStyle::Dash => Self::Dash,
            ListStyle::Plus => Self::Plus,
            ListStyle::Star => Self::Star,
        }
    }
}

fn cli_with_config() -> Cli {
    let cli = Cli::parse();
    let config_file_path = &cli.config_file;

    if config_file_path == "none" {
        return cli;
    }

    if let Ok(args) = fs::read_to_string(config_file_path) {
        match shell_words::split(&args) {
            Ok(mut args) => {
                for (i, arg) in env::args_os().enumerate() {
                    if let Some(s) = arg.to_str() {
                        args.insert(i, s.into());
                    }
                }

                Cli::parse_from(args)
            }
            Err(e) => {
                eprintln!("failed to parse {}: {}", config_file_path, e);
                process::exit(EXIT_PARSE_CONFIG);
            }
        }
    } else {
        cli
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli_with_config();

    if cli.inplace {
        if let Some(ref files) = cli.files {
            if files.len() != 1 {
                eprintln!("cannot have more than 1 input file with in-place mode");
                process::exit(EXIT_CHECK_FILE_NUM);
            }
        } else {
            eprintln!("no input file specified: cannot use standard input with in-place mode");
            process::exit(EXIT_CHECK_FILE_NUM);
        }
    }

    let exts = &cli.extensions;

    let extension = ExtensionOptions::builder()
        .strikethrough(exts.contains(&Extension::Strikethrough) || cli.gfm)
        .tagfilter(exts.contains(&Extension::Tagfilter) || cli.gfm)
        .table(exts.contains(&Extension::Table) || cli.gfm)
        .autolink(exts.contains(&Extension::Autolink) || cli.gfm)
        .tasklist(exts.contains(&Extension::Tasklist) || cli.gfm)
        .superscript(exts.contains(&Extension::Superscript))
        .maybe_header_ids(cli.header_ids)
        .footnotes(exts.contains(&Extension::Footnotes))
        .description_lists(exts.contains(&Extension::DescriptionLists))
        .multiline_block_quotes(exts.contains(&Extension::MultilineBlockQuotes))
        .math_dollars(exts.contains(&Extension::MathDollars))
        .math_code(exts.contains(&Extension::MathCode))
        .wikilinks_title_after_pipe(exts.contains(&Extension::WikilinksTitleAfterPipe))
        .wikilinks_title_before_pipe(exts.contains(&Extension::WikilinksTitleBeforePipe))
        .underline(exts.contains(&Extension::Underline))
        .spoiler(exts.contains(&Extension::Spoiler))
        .greentext(exts.contains(&Extension::Greentext))
        .maybe_front_matter_delimiter(cli.front_matter_delimiter);

    #[cfg(feature = "shortcodes")]
    let extension = extension.shortcodes(cli.gemojis);

    let extension = extension.build();

    let parse = ParseOptions::builder()
        .smart(cli.smart)
        .maybe_default_info_string(cli.default_info_string)
        .relaxed_tasklist_matching(cli.relaxed_tasklist_character)
        .relaxed_autolinks(cli.relaxed_autolinks)
        .build();

    let render = RenderOptions::builder()
        .hardbreaks(cli.hardbreaks)
        .github_pre_lang(cli.github_pre_lang || cli.gfm)
        .full_info_string(cli.full_info_string)
        .width(cli.width)
        .unsafe_(cli.unsafe_)
        .escape(cli.escape)
        .list_style(cli.list_style.into())
        .sourcepos(cli.sourcepos)
        .experimental_inline_sourcepos(cli.experimental_inline_sourcepos)
        .escaped_char_spans(cli.escaped_char_spans)
        .ignore_setext(cli.ignore_setext)
        .ignore_empty_links(cli.ignore_empty_links)
        .gfm_quirks(cli.gfm_quirks || cli.gfm)
        .build();

    let options = Options {
        extension,
        parse,
        render,
    };

    let syntax_highlighter: Option<&dyn SyntaxHighlighterAdapter>;
    let mut plugins: Plugins = Plugins::default();
    let adapter: SyntectAdapter;

    let theme = cli.syntax_highlighting;
    if theme.is_empty() || theme == "none" {
        syntax_highlighter = None;
    } else {
        adapter = SyntectAdapter::new(Some(&theme));
        syntax_highlighter = Some(&adapter);
    }

    let mut s: Vec<u8> = Vec::with_capacity(2048);

    match cli.files {
        None => {
            std::io::stdin().read_to_end(&mut s)?;
        }
        Some(ref fs) => {
            for f in fs {
                match fs::File::open(f) {
                    Ok(mut io) => {
                        io.read_to_end(&mut s)?;
                    }
                    Err(e) => {
                        eprintln!("failed to read {}: {}", f.display(), e);
                        process::exit(EXIT_READ_INPUT);
                    }
                }
            }
        }
    };

    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &String::from_utf8(s)?, &options);

    let formatter = if cli.inplace {
        comrak::format_commonmark_with_plugins
    } else {
        match cli.format {
            Format::Html => {
                plugins.render.codefence_syntax_highlighter = syntax_highlighter;
                comrak::format_html_with_plugins
            }
            Format::Xml => comrak::format_xml_with_plugins,
            Format::CommonMark => comrak::format_commonmark_with_plugins,
        }
    };

    if let Some(output_filename) = cli.output {
        let mut bw = BufWriter::new(fs::File::create(output_filename)?);
        formatter(root, &options, &mut bw, &plugins)?;
        bw.flush()?;
    } else if cli.inplace {
        let output_filename = cli.files.unwrap().get(0).unwrap().clone();
        let mut bw = BufWriter::new(fs::File::create(output_filename)?);
        formatter(root, &options, &mut bw, &plugins)?;
        bw.flush()?;
    } else {
        let stdout = std::io::stdout();
        let mut bw = BufWriter::new(stdout.lock());
        formatter(root, &options, &mut bw, &plugins)?;
        bw.flush()?;
    };

    process::exit(EXIT_SUCCESS);
}

#[cfg(all(not(windows), not(target_arch = "wasm32")))]
fn get_default_config_path() -> String {
    if let Ok(xdg_dirs) = xdg::BaseDirectories::with_prefix("comrak") {
        if let Ok(path) = xdg_dirs.place_config_file("config") {
            if let Some(path_str) = path.to_str() {
                return path_str.into();
            }
        }
    }

    "comrak.config".into()
}
// If on Windows or compiling to wasm, disable default config file check
#[cfg(any(windows, target_arch = "wasm32"))]
fn get_default_config_path() -> String {
    "none".into()
}
