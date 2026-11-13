use std::fs;
use std::path::{Path, PathBuf};

use comrak_www::Doc;

fn load(name: &str, body: &str) -> Doc {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("page");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join(name), format!("---\ntitle: Test\n---\n\n{body}")).unwrap();
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
