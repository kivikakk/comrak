use super::model::{snapshot, Source};
use super::{all_checks, generate, invariants, options, relations, run_check};
use super::{MAX_DOC_LEN, STATIC_CHECKS};
use comrak::nodes::NodeValue;
use comrak::Options;
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const DEFAULT_GENERATED_DOCS: usize = 2000;
const CASES_PER_BUCKET: usize = 8;
const MINIMISE_FUEL: usize = 1500;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative)
}

fn env_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn examples_in(text: &str) -> Vec<String> {
    let mut examples = vec![];
    let mut current: Option<String> = None;
    for line in text.split_inclusive('\n') {
        match &mut current {
            None if line.starts_with("``````````") && line.contains(" example") => {
                current = Some(String::new());
            }
            Some(_) if line.trim_end_matches(['\n', '\r']) == "." => {
                examples.push(current.take().unwrap().replace('\u{2192}', "\t"));
            }
            Some(example) => example.push_str(line),
            None => {}
        }
    }
    examples
}

fn markdown_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            markdown_files_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

fn corpus() -> Vec<String> {
    let mut docs = generate::documents(
        env_or("SOURCEPOS_SWEEP_DOCS", DEFAULT_GENERATED_DOCS),
        env_or("SOURCEPOS_SWEEP_SEED", 1),
    );
    let mut spec_files = vec![];
    markdown_files_under(&repo_path("docs"), &mut spec_files);
    for name in ["spec", "extensions", "regression", "smart_punct"] {
        spec_files.push(repo_path(&format!("vendor/cmark-gfm/test/{name}.txt")));
    }
    for file in spec_files {
        if let Ok(text) = std::fs::read_to_string(file) {
            docs.extend(examples_in(&text));
        }
    }
    if let Ok(dirs) = std::env::var("SOURCEPOS_SWEEP_DIRS") {
        for dir in dirs.split(',') {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let bytes = std::fs::read(entry.path()).unwrap();
                docs.push(String::from_utf8_lossy(&bytes).into_owned());
            }
        }
    }
    docs.retain(|doc| doc.len() <= MAX_DOC_LEN);
    docs.sort();
    docs.dedup();
    docs
}

fn bucket_still_fires(bucket: &str, check: &str, doc: &str, options: &Options<'static>) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        run_check(check, doc, options, &snapshot(doc, options))
            .iter()
            .any(|finding| finding.bucket() == bucket)
    }))
    .unwrap_or(false)
}

fn shrink_by_removing<T: Clone>(
    items: Vec<T>,
    fuel: &mut usize,
    holds: &impl Fn(&[T]) -> bool,
) -> Vec<T> {
    let mut items = items;
    let mut width = items.len() / 2;
    while width >= 1 && *fuel > 0 {
        let mut at = 0;
        while at + width <= items.len() && *fuel > 0 {
            let mut candidate = items.clone();
            candidate.drain(at..at + width);
            *fuel -= 1;
            if holds(&candidate) {
                items = candidate;
            } else {
                at += width;
            }
        }
        width /= 2;
    }
    items
}

fn minimise(doc: &str, bucket: &str, check: &str, options: &Options<'static>) -> String {
    let holds = |candidate: &str| {
        !candidate.is_empty() && bucket_still_fires(bucket, check, candidate, options)
    };
    let mut fuel = MINIMISE_FUEL;
    let mut current = doc.to_string();
    loop {
        let before = current.len();
        let lines: Vec<String> = current.split_inclusive('\n').map(str::to_string).collect();
        current = shrink_by_removing(lines, &mut fuel, &|lines: &[String]| holds(&lines.concat()))
            .concat();
        let chars: Vec<char> = current.chars().collect();
        current = shrink_by_removing(chars, &mut fuel, &|chars: &[char]| {
            holds(&chars.iter().collect::<String>())
        })
        .into_iter()
        .collect();
        if current.len() >= before || fuel == 0 {
            return current;
        }
    }
}

fn doc_class(doc: &str) -> String {
    let mut class = String::new();
    for c in doc.chars() {
        let c = if c.is_alphanumeric() { 'a' } else { c };
        if !(c == 'a' && class.ends_with('a')) {
            class.push(c);
        }
    }
    class
}

fn parallel<T: Send>(count: usize, work: impl Fn(usize) -> Option<T> + Sync) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(vec![]);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= count {
                    return;
                }
                if let Some(result) = work(index) {
                    results.lock().unwrap().push(result);
                }
            });
        }
    });
    results.into_inner().unwrap()
}

#[derive(Default)]
struct Bucket {
    docs: usize,
    cases: BTreeSet<(usize, String, &'static str)>,
    check: String,
}

fn report(buckets: BTreeMap<String, Bucket>) -> String {
    let mut work = vec![];
    for (name, bucket) in &buckets {
        for (_, doc, set) in &bucket.cases {
            work.push((name, bucket, doc, *set));
        }
    }
    let minimised = parallel(work.len(), |index| {
        let (name, bucket, doc, set) = work[index];
        let options = options::named(set);
        let doc = minimise(doc, name, &bucket.check, &options);
        let detail = catch_unwind(AssertUnwindSafe(|| {
            run_check(&bucket.check, &doc, &options, &snapshot(&doc, &options))
                .into_iter()
                .find(|finding| finding.bucket() == *name)
        }))
        .unwrap_or_default();
        Some((name.clone(), set, doc, detail))
    });
    let mut groups: BTreeMap<(&str, String), Vec<_>> = BTreeMap::new();
    for row in &minimised {
        groups
            .entry((row.1, doc_class(&row.2)))
            .or_default()
            .push(row);
    }
    let mut out = format!(
        "{} failing buckets, {} minimised groups\n",
        buckets.len(),
        groups.len()
    );
    for ((set, _), rows) in &groups {
        let smallest = rows.iter().min_by_key(|row| row.2.len()).unwrap();
        out.push_str(&format!("\n[{set}] {:?}\n", smallest.2));
        if let Some(finding) = &smallest.3 {
            out.push_str(&format!("    {}\n", finding.detail));
            if let Some(edited) = &finding.edited {
                out.push_str(&format!("    edited: {edited:?}\n"));
            }
        }
        let names: BTreeSet<String> = rows
            .iter()
            .map(|row| format!("{} ({} docs)", row.0, buckets[&row.0].docs))
            .collect();
        out.push_str(&format!(
            "    via: {}\n",
            names.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    out
}

#[test]
fn sweep() {
    let docs = corpus();
    let sets: Vec<(&'static str, Options<'static>)> = options::all_names()
        .into_iter()
        .map(|name| (name, options::named(name)))
        .collect();
    let buckets = Mutex::new(BTreeMap::<String, Bucket>::new());
    let effective = Mutex::new(BTreeMap::<&str, usize>::new());
    let panics = parallel(docs.len() * sets.len(), |index| {
        let doc = &docs[index / sets.len()];
        let (set, options) = &sets[index % sets.len()];
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let recs = snapshot(doc, options);
            let mut findings = vec![];
            let mut trials = vec![];
            for check in all_checks() {
                if STATIC_CHECKS.contains(&check) {
                    findings.extend(run_check(check, doc, options, &recs));
                } else {
                    let outcome = relations::run(check, doc, options, &recs);
                    trials.push((check, outcome.tried - outcome.failed));
                    findings.extend(outcome.findings);
                }
            }
            (findings, trials)
        }));
        let Ok((findings, trials)) = outcome else {
            return Some(format!("[{set}] {doc:?}"));
        };
        {
            let mut effective = effective.lock().unwrap();
            for (check, count) in trials {
                *effective.entry(check).or_default() += count;
            }
        }
        let mut seen = BTreeSet::new();
        for finding in findings {
            let name = finding.bucket();
            let mut buckets = buckets.lock().unwrap();
            let bucket = buckets.entry(name.clone()).or_default();
            bucket.check = finding.check;
            if seen.insert(name) {
                bucket.docs += 1;
                bucket.cases.insert((doc.len(), doc.clone(), set));
                if bucket.cases.len() > CASES_PER_BUCKET {
                    let largest = bucket.cases.iter().next_back().cloned().unwrap();
                    bucket.cases.remove(&largest);
                }
            }
        }
        None
    });
    assert!(panics.is_empty(), "comrak panicked on: {:#?}", panics);
    let buckets = buckets.into_inner().unwrap();
    assert!(buckets.is_empty(), "{}", report(buckets));
    let effective = effective.into_inner().unwrap();
    for family in relations::FAMILIES {
        assert!(
            effective.get(family).copied().unwrap_or(0) > 0,
            "relation {} never cleared condition",
            family
        );
    }
}

#[test]
fn conventions_are_not_findings() {
    for doc in [
        "^[\t\n;\n\0]\n",
        "*\n\t[a]: b\n",
        "[\t\na]()\n",
        "![ \na]()\n",
        "- [x]\t\n  a\n",
        "| a |\n| - |\n|||\\|r||\n",
        "jjj|",
        "+---+---+---+\n| AAAAA | B |\n+---+---+ B +\n| D | E | B |\n+ D +---+---+\n| D | CCCCC |\n+---+---+---+\n",
        "+-----+\n| a\\  |\n| b   |\n+-----+\n",
        "+-----+\n| ^[a |\n| b]  |\n+-----+\n",
        "+---+---+\n+---+---+\n",
    ] {
        super::assert_clean(doc);
    }
}

#[test]
fn inline_footnote_docs_stay_comparable() {
    let doc = "^[a]\n";
    let options = options::named("ofm");
    let recs = snapshot(doc, &options);
    for family in [
        "blockquote",
        "blockquote-nested",
        "bullet-item",
        "ordered-item",
        "blockquote-item",
        "heading",
        "table-header-cell",
        "table-body-cell",
        "table-preface",
    ] {
        let outcome = relations::run(family, doc, &options, &recs);
        assert_eq!((outcome.tried, outcome.failed), (1, 0), "{family}");
    }
}

#[test]
fn checks_catch_corrupted_positions() {
    let doc = "para *emph* `code`\nmore\n\n- item\n";
    let options = options::named("cm");
    let clean = snapshot(doc, &options);
    let source = Source::new(doc);
    assert!(invariants::structure(&clean, &source).is_empty());
    assert!(invariants::kind(&clean, &source).is_empty());
    assert!(invariants::tiling(&clean, &source).is_empty());
    for index in 1..clean.len() {
        let mut corrupted = clean.clone();
        corrupted[index].sp.start.column += 1;
        corrupted[index].sp.end.column += 1;
        let caught = !invariants::structure(&corrupted, &source).is_empty()
            || !invariants::kind(&corrupted, &source).is_empty()
            || !invariants::tiling(&corrupted, &source).is_empty();
        assert!(caught, "didn't catch shifted {}", clean[index].kind);
    }
}

#[test]
fn checks_catch_breaks_short_of_crlf() {
    let doc = "a\r\nb  \r\nc\\\r\nd\r\n";
    let options = options::named("cm");
    let clean = snapshot(doc, &options);
    let source = Source::new(doc);
    assert!(invariants::kind(&clean, &source).is_empty());
    let breaks: Vec<usize> = (1..clean.len())
        .filter(|&index| {
            matches!(
                clean[index].value,
                NodeValue::SoftBreak | NodeValue::LineBreak
            )
        })
        .collect();
    assert_eq!(breaks.len(), 3);
    for index in breaks {
        let mut corrupted = clean.clone();
        corrupted[index].sp.end.column -= 1;
        assert!(
            !invariants::kind(&corrupted, &source).is_empty(),
            "didn't catch short {}",
            clean[index].kind
        );
    }
}
