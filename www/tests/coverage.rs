use std::fs;
use std::path::PathBuf;

use clap::{CommandFactory, ValueEnum};
use comrak::cli::{Extension, OptionArgs};

fn check_pages(subdir: &str, kind: &str, names: &[String]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../docs")
        .join(subdir);

    let mut problems = vec![];

    for name in names {
        if !dir.join(format!("{name}.md")).exists() {
            problems.push(format!(
                "{kind} `{name}` has no page: add docs/{subdir}/{name}.md"
            ));
        }
    }

    for entry in fs::read_dir(&dir).unwrap() {
        let path = PathBuf::from(entry.unwrap().file_name());
        let stem = path.file_stem().unwrap().to_str().unwrap();
        if path.extension().is_none_or(|ext| ext != "md") || !names.iter().any(|n| n == stem) {
            problems.push(format!(
                "docs/{subdir}/{} doesn't correspond to {kind}",
                path.display()
            ));
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_extension_has_a_page() {
    let names: Vec<String> = Extension::value_variants()
        .iter()
        .map(|e| e.to_possible_value().unwrap().get_name().to_string())
        .collect();
    check_pages("extensions", "an extension", &names);
}

#[test]
fn every_option_has_a_page() {
    let names: Vec<String> = OptionArgs::command()
        .get_arguments()
        .filter(|arg| arg.get_id() != "extensions")
        .map(|arg| arg.get_long().unwrap().to_string())
        .collect();
    check_pages("options", "an option", &names);
}
