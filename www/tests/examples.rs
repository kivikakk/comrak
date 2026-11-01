use std::fs;
use std::path::PathBuf;

use comrak_www::{Doc, SNAP};

fn docs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs")
}

#[test]
fn examples_match_expected_output() {
    let root = docs_root();
    let docs = Doc::load_all(&root);

    if std::env::var_os(SNAP).is_some() {
        for doc in &docs {
            let snap = doc.snap();
            if snap != doc.source {
                fs::write(root.join(&doc.path), snap).unwrap();
            }
        }
    }

    let failures: Vec<String> = Doc::load_all(&root)
        .iter()
        .flat_map(Doc::failures)
        .collect();
    assert!(
        failures.is_empty(),
        "{}\n\nrun with {SNAP}=1 to update expected",
        failures.join("\n")
    );
}
