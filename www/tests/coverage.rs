use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{CommandFactory, ValueEnum};
use comrak::cli::{Extension, OptionArgs};
use comrak_www::Doc;

fn check_pages(subdir: &str, kind: &str, names: &[String]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs");

    let mut problems = vec![];
    let mut covered: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    for entry in fs::read_dir(root.join(subdir)).unwrap() {
        let path = PathBuf::from(entry.unwrap().file_name());
        if path.extension().is_none_or(|ext| ext != "md") {
            problems.push(format!("docs/{subdir}/{} isn't a page", path.display()));
        }
    }

    let docs = Doc::load_all(&root);
    for doc in docs
        .iter()
        .filter(|doc| doc.path.parent() == Some(Path::new(subdir)))
    {
        for name in &doc.covers {
            if names.contains(name) {
                covered
                    .entry(name)
                    .or_default()
                    .push(doc.path.display().to_string());
            } else {
                problems.push(format!(
                    "docs/{} covers `{name}`, which isn't {kind}",
                    doc.path.display()
                ));
            }
        }
    }

    for name in names {
        match covered.get(name.as_str()) {
            None => problems.push(format!(
                "{kind} `{name}` has no page: add docs/{subdir}/{name}.md"
            )),
            Some(paths) if paths.len() > 1 => problems.push(format!(
                "{kind} `{name}` is covered by more than one page: {}",
                paths.join(", ")
            )),
            Some(_) => {}
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
