//! The `comrak` binary.

extern crate comrak;

#[macro_use]
extern crate clap;

extern crate xdg;
extern crate shell_words;

use comrak::{Arena, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};

use std::boxed::Box;
use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let default_config_path = get_default_config_path();

    let app = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .after_help("\
By default, Comrak will attempt to read command-line options from a config file specified by \
--config-file.  This behaviour can be disabled with --no-config-file.  It is not an error if the \
file does not exist.\
        ")
        .arg(
            clap::Arg::with_name("file")
                .value_name("FILE")
                .multiple(true)
                .help("The CommonMark file to parse; or standard input if none passed"),
        )
        .arg(
            clap::Arg::with_name("config-file")
                .short("c")
                .long("config-file")
                .help("Path to config file containing command-line arguments")
                .value_name("PATH")
                .takes_value(true)
                .default_value(&default_config_path),
        )
        .arg(
            clap::Arg::with_name("no-config-file")
                .long("no-config-file")
                .help("Do not read config file"),
        )
        .arg(
            clap::Arg::with_name("hardbreaks")
                .long("hardbreaks")
                .help("Treat newlines as hard line breaks"),
        )
        .arg(
            clap::Arg::with_name("smart")
                .long("smart")
                .help("Use smart punctuation"),
        )
        .arg(
            clap::Arg::with_name("github-pre-lang")
                .long("github-pre-lang")
                .help("Use GitHub-style <pre lang> for code blocks"),
        )
        .arg(
            clap::Arg::with_name("gfm")
                .long("gfm")
                .help("Enable GitHub-flavored markdown extensions strikethrough, tagfilter, table, autolink, and tasklist. It also enables --github-pre-lang.")
        )
        .arg(
            clap::Arg::with_name("default-info-string")
                .long("default-info-string")
                .help("Default value for fenced code block's info strings if none is given")
                .value_name("INFO")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("unsafe")
                .long("unsafe")
                .help("Allow raw HTML and dangerous URLs"),
        )
        .arg(
            clap::Arg::with_name("escape")
                .long("escape")
                .help("Escape raw HTML instead of clobbering it"),
        )
        .arg(
            clap::Arg::with_name("extension")
                .short("e")
                .long("extension")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .possible_values(&[
                    "strikethrough",
                    "tagfilter",
                    "table",
                    "autolink",
                    "tasklist",
                    "superscript",
                    "footnotes",
                    "description-lists",
                ])
                .value_name("EXTENSION")
                .help("Specify an extension name to use"),
        )
        .arg(
            clap::Arg::with_name("format")
                .short("t")
                .long("to")
                .takes_value(true)
                .possible_values(&["html", "commonmark"])
                .default_value("html")
                .value_name("FORMAT")
                .help("Specify output format"),
        )
        .arg(
            clap::Arg::with_name("width")
                .long("width")
                .takes_value(true)
                .value_name("WIDTH")
                .default_value("0")
                .help("Specify wrap width (0 = nowrap)"),
        )
        .arg(
            clap::Arg::with_name("header-ids")
                .long("header-ids")
                .takes_value(true)
                .value_name("PREFIX")
                .help("Use the Comrak header IDs extension, with the given ID prefix"),
        );

    let mut matches = app.clone().get_matches();

    if !matches.is_present("no-config-file") {
        let config_file_path = matches.value_of("config-file").unwrap();
        if let Ok(args) = fs::read_to_string(config_file_path) {
            match shell_words::split(&args) {
                Ok(mut args) => {
                    for (i, arg) in env::args_os().enumerate() {
                        if let Some(s) = arg.to_str() {
                            args.insert(i, s.into());
                        }
                    }
                    matches = app.get_matches_from(args);
                },
                Err(e) => {
                    eprintln!("failed to parse {} -- {}", config_file_path, e);
                    process::exit(2);
                },
            }
        }
    }

    let mut exts = matches
        .values_of("extension")
        .map_or(BTreeSet::new(), |vals| vals.collect());

    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: exts.remove("strikethrough") || matches.is_present("gfm"),
            tagfilter: exts.remove("tagfilter") || matches.is_present("gfm"),
            table: exts.remove("table") || matches.is_present("gfm"),
            autolink: exts.remove("autolink") || matches.is_present("gfm"),
            tasklist: exts.remove("tasklist") || matches.is_present("gfm"),
            superscript: exts.remove("superscript"),
            header_ids: matches.value_of("header-ids").map(|s| s.to_string()),
            footnotes: exts.remove("footnotes"),
            description_lists: exts.remove("description-lists"),
        },
        parse: ComrakParseOptions {
            smart: matches.is_present("smart"),
            default_info_string: matches
                .value_of("default-info-string")
                .map(|e| e.to_owned()),
        },
        render: ComrakRenderOptions {
            hardbreaks: matches.is_present("hardbreaks"),
            github_pre_lang: matches.is_present("github-pre-lang") || matches.is_present("gfm"),
            width: matches
                .value_of("width")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0),
            unsafe_: matches.is_present("unsafe"),
            escape: matches.is_present("escape"),
        },
    };

    if !exts.is_empty() {
        eprintln!("unknown extensions: {:?}", exts);
        process::exit(1);
    }

    let mut s: Vec<u8> = Vec::with_capacity(2048);

    match matches.values_of("file") {
        None => {
            std::io::stdin().read_to_end(&mut s)?;
        }
        Some(fs) => for f in fs {
            let mut io = std::fs::File::open(f)?;
            io.read_to_end(&mut s)?;
        },
    };

    let arena = Arena::new();
    let root = comrak::parse_document(&arena, &String::from_utf8(s)?, &options);

    let formatter = match matches.value_of("format") {
        Some("html") => comrak::format_html,
        Some("commonmark") => comrak::format_commonmark,
        _ => panic!("unknown format"),
    };

    formatter(root, &options, &mut std::io::stdout())?;

    process::exit(0);
}

fn get_default_config_path() -> String {
    if let Ok(xdg_dirs) = xdg::BaseDirectories::with_prefix("comrak") {
        if let Ok(path) = xdg_dirs.place_config_file("config") {
            if let Some(path_str) = path.to_str() {
                return path_str.into();
            }
        }
    }

    return "comrak.config".into();
}
