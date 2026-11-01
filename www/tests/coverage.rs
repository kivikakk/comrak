use std::fs;
use std::path::PathBuf;

use clap::ValueEnum;
use comrak::cli::Extension;

#[test]
fn every_extension_has_a_page() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/extensions");
    let names: Vec<String> = Extension::value_variants()
        .iter()
        .map(|e| e.to_possible_value().unwrap().get_name().to_string())
        .collect();

    let mut problems = vec![];

    for name in &names {
        if !dir.join(format!("{name}.md")).exists() {
            problems.push(format!(
                "extension `{name}` has no page: add docs/extensions/{name}.md"
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
