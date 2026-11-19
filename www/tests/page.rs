use std::fs;
use std::path::{Path, PathBuf};

use comrak_www::{Doc, render_with_slots};

fn load(name: &str, body: &str) -> Doc {
    load_with(name, "", body)
}

fn load_with(name: &str, front_matter: &str, body: &str) -> Doc {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("page");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join(name),
        format!("---\ntitle: Test\n{front_matter}---\n\n{body}"),
    )
    .unwrap();
    Doc::load(&root, PathBuf::from(name))
}

#[test]
fn terms_link_to_their_definition() {
    let html = load(
        "terms.md",
        "A [widget](@) is a thing.\n\nTwo [widgets][widget] are\n[Widget]s too.\n",
    )
    .render_page();

    assert!(html.contains(r##"<dfn id="widget"><a href="#widget">widget</a></dfn>"##));
    assert!(html.contains(r##"<a href="#widget">widgets</a>"##));
    assert!(html.contains(r##"<a href="#widget">Widget</a>s"##));
}

#[test]
fn undefined_terms_stay_text() {
    let html = load("undefined.md", "A [widget](@) and a [gadget].\n").render_page();

    assert!(html.contains("a [gadget]."));
}

#[test]
#[should_panic(expected = "term `widget` defined twice")]
fn terms_are_defined_once() {
    load("twice.md", "A [widget](@) or a [Widget](@).\n");
}

#[test]
#[should_panic(expected = "`#widget` is also a heading's id")]
fn terms_do_not_shadow_headings() {
    load("shadow.md", "## Widget\n\nA [widget](@).\n");
}

#[test]
fn examples_are_numbered_and_show_tabs() {
    let fence = "`".repeat(32);
    let example = format!("{fence} example\na→b\n.\n<p>a→b</p>\n{fence}\n");
    let doc = load("examples.md", &format!("{example}\n{example}"));

    assert_eq!(doc.examples[1].input, "a\tb\n");
    assert!(doc.failures().is_empty(), "{:?}", doc.failures());

    let html = doc.render_page();
    assert!(html.contains(r##"<div class="example" id="example-1">"##));
    assert!(html.contains(r##"<a class="example-number" href="#example-2">Example 2</a>"##));
    assert!(html.contains("<p>a\tb</p>"));
    assert!(html.contains("a→b"));
}

#[test]
fn snapping_writes_tabs_as_arrows() {
    let fence = "`".repeat(32);
    let doc = load("snap.md", &format!("{fence} example\na→b\n.\n{fence}\n"));

    assert!(doc.snap().contains("a→b\n.\n<p>a→b</p>\n"));
}

#[test]
fn lede_is_opening_paragraph_as_text() {
    let doc = load(
        "lede.md",
        "A [widget](@) is a `thing`.  It\n*really* is.\n\nMore.\n",
    );

    assert_eq!(
        doc.lede.as_deref(),
        Some("A widget is a thing. It really is.")
    );
}

#[test]
fn lede_must_open_page() {
    let fence = "`".repeat(32);
    let example = format!("{fence} example\na\n.\n<p>a</p>\n{fence}\n");

    assert_eq!(load("headed.md", "## Widget\n\nA widget.\n").lede, None);
    assert_eq!(
        load("example.md", &format!("{example}\nA widget.\n")).lede,
        None
    );
}

#[test]
fn page_covers_its_name_by_default() {
    assert_eq!(load("widget.md", "Text.\n").covers, ["widget"]);
}

#[test]
fn front_matter_lists_covered_extensions() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("page");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("widgets.md"),
        "---\ntitle: Test\nextensions: [widget-a, widget-b]\n---\n\nText.\n",
    )
    .unwrap();

    assert_eq!(
        Doc::load(&root, PathBuf::from("widgets.md")).covers,
        ["widget-a", "widget-b"]
    );
}

#[test]
fn pages_are_finished_by_default() {
    let doc = load("finished.md", "Text.\n");

    assert!(!doc.draft);
    assert_eq!(doc.display_title(), "Test");
}

#[test]
fn front_matter_marks_drafts() {
    let doc = load_with("draft.md", "draft: true\n", "Text.\n");

    assert!(doc.draft);
    assert_eq!(doc.title, "Test");
    assert_eq!(doc.display_title(), "Test (draft)");
}

#[test]
#[should_panic(expected = "`draft` must be a boolean")]
fn draft_is_boolean() {
    load_with("bad-draft.md", "draft: yes please\n", "Text.\n");
}

#[test]
fn slots_fill_their_paragraphs() {
    let html = render_with_slots(
        "Before.\n\n{{a}}\n\nBetween.\n\n{{b}}\n\nAfter.\n",
        &[("a", "<nav>A</nav>\n"), ("b", "<nav>B</nav>\n")],
    );

    assert_eq!(
        html,
        "<p>Before.</p>\n<nav>A</nav>\n<p>Between.</p>\n<nav>B</nav>\n<p>After.</p>\n"
    );
}

#[test]
#[should_panic(expected = "`{{a}}` must be its own paragraph exactly once")]
fn slots_are_filled_once() {
    render_with_slots("{{a}}\n\n{{a}}\n", &[("a", "")]);
}

#[test]
#[should_panic(expected = "`{{a}}` must be its own paragraph exactly once")]
fn slots_are_whole_paragraphs() {
    render_with_slots("See {{a}}.\n", &[("a", "")]);
}
