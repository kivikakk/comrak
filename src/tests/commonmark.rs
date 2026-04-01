use self::nodes::{Ast, LineColumn, ListType, NodeList};

use super::*;
use ntest::test_case;

fn synthetic_text_paragraph<'a>(arena: &'a Arena<'a>, text: &str) -> Node<'a> {
    let empty = LineColumn { line: 0, column: 0 };
    let ast = |val: NodeValue| arena.alloc(Ast::new(val, empty).into());
    let root = ast(NodeValue::Document);
    let p = ast(NodeValue::Paragraph);
    p.append(ast(NodeValue::Text(text.to_string().into())));
    root.append(p);
    root
}

fn synthetic_softbreak_paragraph<'a>(arena: &'a Arena<'a>, before: &str, after: &str) -> Node<'a> {
    let empty = LineColumn { line: 0, column: 0 };
    let ast = |val: NodeValue| arena.alloc(Ast::new(val, empty).into());
    let root = ast(NodeValue::Document);
    let p = ast(NodeValue::Paragraph);
    p.append(ast(NodeValue::Text(before.to_string().into())));
    p.append(ast(NodeValue::SoftBreak));
    p.append(ast(NodeValue::Text(after.to_string().into())));
    root.append(p);
    root
}

fn assert_synthetic_roundtrip(
    root: Node<'_>,
    options: &Options,
    expected_cm: &str,
    expected_html: &str,
) {
    let mut rendered = String::new();
    cm::format_document(root, options, &mut rendered).unwrap();
    compare_strs(&rendered, expected_cm, "rendered", "<synthetic>");

    let mut original_html = String::new();
    html::format_document(root, options, &mut original_html).unwrap();
    compare_strs(
        &original_html,
        expected_html,
        "original html",
        "<synthetic>",
    );

    let reparsed_arena = Arena::new();
    let reparsed = parse_document(&reparsed_arena, &rendered, options);
    let mut reparsed_html = String::new();
    html::format_document(reparsed, options, &mut reparsed_html).unwrap();
    compare_strs(&reparsed_html, expected_html, "roundtrip html", &rendered);
}

#[test]
fn commonmark_removes_redundant_strong() {
    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";
    commonmark(input, output, None);
}

#[test]
fn commonmark_avoids_spurious_backslash() {
    let arena = Arena::new();
    let options = Options::default();
    let empty = LineColumn { line: 0, column: 0 };

    let ast = |val: NodeValue| arena.alloc(Ast::new(val, empty).into());
    let root = ast(NodeValue::Document);

    let p1 = ast(NodeValue::Paragraph);
    p1.append(ast(NodeValue::Text("Line 1".into())));
    p1.append(ast(NodeValue::LineBreak));
    root.append(p1);

    let p2 = ast(NodeValue::Paragraph);
    p2.append(ast(NodeValue::Text("Line 2".into())));
    root.append(p2);

    let mut output = String::new();
    cm::format_document(root, &options, &mut output).unwrap();

    compare_strs(&output, "Line 1\n\nLine 2\n", "rendered", "<synthetic>");
}

#[test]
fn commonmark_renders_single_list_item() {
    let arena = Arena::new();
    let options = Options::default();
    let empty = LineColumn { line: 0, column: 0 };
    let ast = |val: NodeValue| arena.alloc(Ast::new(val, empty).into());
    let list_options = NodeList {
        list_type: ListType::Ordered,
        start: 1,
        ..Default::default()
    };
    let list = ast(NodeValue::List(list_options));
    let item = ast(NodeValue::Item(list_options));
    let p = ast(NodeValue::Paragraph);
    p.append(ast(NodeValue::Text("Item 1".into())));
    item.append(p);
    list.append(item);
    let mut output = String::new();
    cm::format_document(item, &options, &mut output).unwrap();
    compare_strs(&output, "1. Item 1\n", "rendered", "<synthetic>");
}

#[test_case("$$x^2$$ and $1 + 2$ and $`y^2`$", "$$x^2$$ and $1 + 2$ and $`y^2`$\n")]
#[test_case("$$\nx^2\n$$", "$$\nx^2\n$$\n")]
#[test_case("```math\nx^2\n```", "```math\nx^2\n```\n")]
fn commonmark_math(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    commonmark(markdown, cm, None);
}

#[test_case("This [[url]] that", "This [[url|url]] that\n")]
#[test_case("This [[url|link label]] that", "This [[url|link%20label]] that\n")]
fn commonmark_wikilinks(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.wikilinks_title_before_pipe = true;

    commonmark(markdown, cm, Some(&options));
}
#[test]
fn commonmark_relist() {
    commonmark(
        concat!("3. one\n", "5. two\n",),
        // Note that right now we always include enough room for up to an user
        // defined number of digits. TODO: Ideally we determine the maximum
        // digit length before getting this far.
        concat!("3. one\n", "4. two\n",),
        None,
    );

    let mut options = Options::default();
    options.extension.tasklist = true;
    commonmark(
        concat!("3. [ ] one\n", "5. [ ] two\n",),
        concat!("3. [ ] one\n", "4. [ ] two\n",),
        Some(&options),
    );
}

#[test_case("> [!note]\n> A note", "> [!NOTE]\n> A note\n")]
#[test_case("> [!note] Title\n> A note", "> [!NOTE] Title\n> A note\n")]
fn commonmark_alerts(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.alerts = true;

    commonmark(markdown, cm, Some(&options));
}

#[test]
fn commonmark_experimental_minimize() {
    let input = r#"This is some text.

It contains [brackets] which could be important.

It contains #hashes# and !exclamation marks! and * asterisks *, _ underscores _,
the < works >.

Let's include some *important\* _ones\_ too.
"#;

    let expected = r#"This is some text.

It contains \[brackets\] which could be important.

It contains \#hashes\# and \!exclamation marks\! and \* asterisks \*, \_ underscores \_,
the \< works \>.

Let's include some \*important\* \_ones\_ too.
"#;

    let mut options = Options::default();
    commonmark(input, expected, Some(&options));

    options.render.experimental_minimize_commonmark = true;
    commonmark(input, input, Some(&options));
}

#[test]
fn dont_create_autolinks() {
    let input = "a_b@c.d_";

    let mut options = Options::default();
    options.extension.autolink = true;

    let expected = "a\\_b\\@c.d\\_\n";
    commonmark(input, expected, Some(&options));

    commonmark(expected, expected, Some(&options));
}

#[test]
fn dont_wrap_table_cell() {
    let input = r#"| option | description |
| --- | --- |
| -o | Write output to FILE instead of stdout |
| --gfm | Use GFM-style quirks in output HTML, such as not nesting <strong> tags, which otherwise breaks CommonMark compatibility |
"#;
    let mut options = Options::default();
    options.extension.table = true;
    options.render.width = 80;
    commonmark(input, input, Some(&options));
}

#[test]
fn commonmark_escapes_synthetic_fence_markers() {
    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_text_paragraph(&arena, "~~~");
    assert_synthetic_roundtrip(root, &options, "\\~~~\n", "<p>~~~</p>\n");

    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_text_paragraph(&arena, "```");
    assert_synthetic_roundtrip(root, &options, "\\`\\`\\`\n", "<p>```</p>\n");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.block_directive = true;
    let root = synthetic_text_paragraph(&arena, ":::");
    assert_synthetic_roundtrip(root, &options, "\\:::\n", "<p>:::</p>\n");
}

#[test]
fn commonmark_escapes_synthetic_markers_after_softbreak() {
    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_softbreak_paragraph(&arena, "foo", "~~~");
    assert_synthetic_roundtrip(root, &options, "foo\n\\~~~\n", "<p>foo\n~~~</p>\n");

    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_softbreak_paragraph(&arena, "foo", "```");
    assert_synthetic_roundtrip(root, &options, "foo\n\\`\\`\\`\n", "<p>foo\n```</p>\n");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.description_lists = true;
    let root = synthetic_softbreak_paragraph(&arena, "term", ": details");
    assert_synthetic_roundtrip(
        root,
        &options,
        "term\n\\: details\n",
        "<p>term\n: details</p>\n",
    );

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.description_lists = true;
    let root = synthetic_softbreak_paragraph(&arena, "term", "~ details");
    assert_synthetic_roundtrip(
        root,
        &options,
        "term\n\\~ details\n",
        "<p>term\n~ details</p>\n",
    );
}

#[test]
fn commonmark_does_not_escape_numeric_prefix_before_synthetic_markers() {
    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_text_paragraph(&arena, "1~~~");
    assert_synthetic_roundtrip(root, &options, "1~~~\n", "<p>1~~~</p>\n");

    let arena = Arena::new();
    let options = Options::default();
    let root = synthetic_text_paragraph(&arena, "1```");
    assert_synthetic_roundtrip(root, &options, "1\\`\\`\\`\n", "<p>1```</p>\n");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.block_directive = true;
    let root = synthetic_text_paragraph(&arena, "1:::");
    assert_synthetic_roundtrip(root, &options, "1:::\n", "<p>1:::</p>\n");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.description_lists = true;
    let root = synthetic_text_paragraph(&arena, "1: details");
    assert_synthetic_roundtrip(root, &options, "1: details\n", "<p>1: details</p>\n");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.description_lists = true;
    let root = synthetic_text_paragraph(&arena, "1~ details");
    assert_synthetic_roundtrip(root, &options, "1~ details\n", "<p>1~ details</p>\n");
}
