//! The `comrak` binary.

extern crate comrak;

#[macro_use]
extern crate clap;

use comrak::{Arena, ComrakOptions};

use std::boxed::Box;
use std::collections::BTreeSet;
use std::error::Error;
use std::io::Read;
use std::process;

fn main() -> Result<(), Box<Error>> {
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
        hardbreaks: matches.is_present("hardbreaks"),
        smart: matches.is_present("smart"),
        github_pre_lang: matches.is_present("github-pre-lang"),
        width: matches
            .value_of("width")
            .unwrap_or("0")
            .parse()
            .unwrap_or(0),
        default_info_string: matches
            .value_of("default-info-string")
            .map(|e| e.to_owned()),
        safe: !matches.is_present("unsafe"),
        ext_strikethrough: exts.remove("strikethrough"),
        ext_tagfilter: exts.remove("tagfilter"),
        ext_table: exts.remove("table"),
        ext_autolink: exts.remove("autolink"),
        ext_tasklist: exts.remove("tasklist"),
        ext_superscript: exts.remove("superscript"),
        ext_header_ids: matches.value_of("header-ids").map(|s| s.to_string()),
        ext_footnotes: exts.remove("footnotes"),
        ext_description_lists: exts.remove("description-lists"),
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
