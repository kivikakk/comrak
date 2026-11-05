pub mod generate;
pub mod invariants;
pub mod model;
pub mod options;
pub mod relations;

#[cfg(test)]
mod sweep;

use comrak::Options;
use model::{snapshot, Finding, Rec, Source};
use std::sync::OnceLock;

pub const MAX_DOC_LEN: usize = 4000;

pub const STATIC_CHECKS: &[&str] = &["structure", "kind", "tiling", "document", "chars"];

pub fn run_check(check: &str, doc: &str, options: &Options<'static>, recs: &[Rec]) -> Vec<Finding> {
    let source = Source::new(doc);
    match check {
        "structure" => invariants::structure(recs, &source),
        "kind" => invariants::kind(recs, &source),
        "tiling" => invariants::tiling(recs, &source),
        "document" => invariants::document_extent(recs, &source),
        "chars" => relations::chars(doc, options, recs),
        family => relations::run(family, doc, options, recs).findings,
    }
}

pub fn all_checks() -> impl Iterator<Item = &'static str> {
    STATIC_CHECKS.iter().chain(relations::FAMILIES).copied()
}

pub fn assert_clean(doc: &str) {
    static SETS: OnceLock<Vec<(&str, Options<'static>)>> = OnceLock::new();
    if doc.len() > MAX_DOC_LEN {
        return;
    }
    let sets = SETS.get_or_init(|| {
        options::composite_names()
            .into_iter()
            .map(|name| (name, options::named(name)))
            .collect()
    });
    for (name, options) in sets {
        let recs = snapshot(doc, options);
        for check in all_checks().filter(|check| *check != "option-toggle") {
            if let Some(finding) = run_check(check, doc, options, &recs).first() {
                panic!(
                    "{} [{name}] {}\ndoc: {doc:?}\nedited: {:?}",
                    finding.bucket(),
                    finding.detail,
                    finding.edited,
                );
            }
        }
    }
}
