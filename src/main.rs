//! The `comrak` binary.

use std::env;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process;
use std::{boxed::Box, io::BufWriter};

use clap::{Parser, ValueEnum};

use comrak::cli::OptionArgs;
use comrak::{Arena, options};
#[cfg(feature = "syntect")]
use comrak::{adapters::SyntaxHighlighterAdapter, plugins::syntect::SyntectAdapter};

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

    /// Reformat CommonMark files in-place
    #[arg(short, long, conflicts_with_all(["format", "output"]))]
    inplace: bool,

    /// Specify output format
    #[arg(short = 't', long = "to", value_enum, default_value_t = Format::Html)]
    format: Format,

    /// Write output to FILE instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Syntax highlighting for fenced code blocks; 'css' for CSS classes (default),
    /// a theme name for inline styles, or 'none' to disable
    #[arg(long, value_name = "THEME", default_value = "css")]
    #[cfg(feature = "syntect")]
    syntax_highlighting: String,

    #[command(flatten)]
    options: OptionArgs,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Html,

    Xml,

    #[value(name = "commonmark")]
    CommonMark,
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

    if cli.inplace && cli.files.is_none() {
        eprintln!("no input file specified: cannot use standard input with in-place mode");
        process::exit(EXIT_CHECK_FILE_NUM);
    }

    let options = cli.options.to_options();

    #[cfg(feature = "syntect")]
    let syntax_highlighter: Option<&dyn SyntaxHighlighterAdapter>;
    #[cfg(feature = "syntect")]
    let adapter: SyntectAdapter;

    #[cfg_attr(not(feature = "syntect"), allow(unused_mut))]
    let mut plugins = options::Plugins::default();

    #[cfg(feature = "syntect")]
    {
        let theme = cli.syntax_highlighting;
        if theme.is_empty() || theme == "none" {
            syntax_highlighter = None;
        } else if theme == "css" {
            adapter = SyntectAdapter::new(None);
            syntax_highlighter = Some(&adapter);
        } else {
            adapter = SyntectAdapter::new(Some(&theme));
            syntax_highlighter = Some(&adapter);
        }
    }

    if cli.inplace {
        for path in cli.files.as_ref().unwrap() {
            let mut input = String::new();
            match fs::File::open(path) {
                Ok(mut io) => {
                    io.read_to_string(&mut input)?;
                }
                Err(e) => {
                    eprintln!("failed to read {}: {}", path.display(), e);
                    process::exit(EXIT_READ_INPUT);
                }
            }

            let arena = Arena::new();
            let root = comrak::parse_document(&arena, &input, &options);
            let mut bw = BufWriter::new(fs::File::create(path)?);
            fmt2io::write(&mut bw, |writer| {
                comrak::format_commonmark_with_plugins(root, &options, writer, &plugins)
            })?;
            std::io::Write::flush(&mut bw)?;
        }
    } else {
        // The stdlib is very good at reserving buffer space based on available
        // information; don't try to one-up it.
        let input = match cli.files {
            None => {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            }
            Some(ref paths) => {
                let mut buf = String::new();
                for path in paths {
                    match fs::File::open(path) {
                        Ok(mut io) => {
                            io.read_to_string(&mut buf)?;
                        }
                        Err(e) => {
                            eprintln!("failed to read {}: {}", path.display(), e);
                            process::exit(EXIT_READ_INPUT);
                        }
                    }
                }
                buf
            }
        };

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &input, &options);

        let formatter = match cli.format {
            Format::Html => {
                #[cfg(feature = "syntect")]
                {
                    plugins.render.codefence_syntax_highlighter = syntax_highlighter;
                }
                comrak::format_html_with_plugins
            }
            Format::Xml => comrak::format_xml_with_plugins,
            Format::CommonMark => comrak::format_commonmark_with_plugins,
        };

        if let Some(output_filename) = cli.output {
            let mut bw = BufWriter::new(fs::File::create(output_filename)?);
            fmt2io::write(&mut bw, |writer| {
                formatter(root, &options, writer, &plugins)
            })?;
            std::io::Write::flush(&mut bw)?;
        } else {
            let stdout = std::io::stdout();
            let mut bw = BufWriter::new(stdout.lock());
            fmt2io::write(&mut bw, |writer| {
                formatter(root, &options, writer, &plugins)
            })?;
            std::io::Write::flush(&mut bw)?;
        };
    }

    process::exit(EXIT_SUCCESS);
}

#[cfg(all(not(windows), not(target_arch = "wasm32")))]
fn get_default_config_path() -> String {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("comrak");
    if let Ok(path) = xdg_dirs.place_config_file("config")
        && let Some(path_str) = path.to_str()
    {
        return path_str.into();
    }

    "comrak.config".into()
}
// If on Windows or compiling to wasm, disable default config file check
#[cfg(any(windows, target_arch = "wasm32"))]
fn get_default_config_path() -> String {
    "none".into()
}
