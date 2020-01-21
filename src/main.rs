//! The `comrak` binary.

extern crate comrak;

#[macro_use]
extern crate clap;

use comrak::{Arena, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};

use std::boxed::Box;
use std::collections::BTreeSet;
use std::error::Error;
use std::io::Read;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            clap::Arg::with_name("file")
                .value_name("FILE")
                .multiple(true)
                .help("The CommonMark file to parse; or standard input if none passed"),
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
        )
        .get_matches();

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
