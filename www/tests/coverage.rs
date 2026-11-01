use std::fs;
use std::path::PathBuf;

use clap::ValueEnum;
use comrak::cli::Extension;

const UNDOCUMENTED: &[&str] = &[
    "strikethrough",
    "table",
    "autolink",
    "tasklist",
    "superscript",
    "footnotes",
    "inline-footnotes",
    "math-latex",
    "underline",
    "subscript",
    "spoiler",
    "greentext",
    "ofm-comments",
    "ofm-callouts",
    "cjk-friendly-emphasis",
    "subtext",
    "highlight",
    "insert",
    "phoenix-heex",
    "block-directive",
    "header-attributes",
    "fenced-code-attributes",
    "inline-code-attributes",
    "link-attributes",
];

#[test]
fn every_extension_has_a_page() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/extensions");
    let names: Vec<String> = Extension::value_variants()
        .iter()
        .map(|e| e.to_possible_value().unwrap().get_name().to_string())
        .collect();

    let mut problems = vec![];

    for name in &names {
        let documented = dir.join(format!("{name}.md")).exists();
        let listed = UNDOCUMENTED.contains(&name.as_str());
        match (documented, listed) {
            (false, false) => problems.push(format!(
                "extension `{name}` has no page: add docs/extensions/{name}.md"
            )),
            (true, true) => problems.push(format!(
                "extension `{name}` has a page: remove it from UNDOCUMENTED in {}",
                file!()
            )),
            _ => {}
        }
    }

    for name in UNDOCUMENTED {
        if !names.iter().any(|n| n == name) {
            problems.push(format!(
                "`{name}` in UNDOCUMENTED isn't an extension: remove it from {}",
                file!()
            ));
        }
    }

    for entry in fs::read_dir(&dir).unwrap() {
        let path = PathBuf::from(entry.unwrap().file_name());
        let stem = path.file_stem().unwrap().to_str().unwrap();
        if path.extension().is_none_or(|ext| ext != "md") || !names.iter().any(|n| n == stem) {
            problems.push(format!(
                "docs/extensions/{} doesn't correspond to an extension",
                path.display()
            ));
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
