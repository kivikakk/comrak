use self::nodes::{ListType, NodeList};

use super::*;
use ntest::test_case;

#[test]
fn commonmark_removes_redundant_strong() {
    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";
    commonmark(input, output, None);
}

#[test]
fn commonmark_avoids_spurious_backslash() {
    let mut arena = Arena::new();
    let options = Options::default();

    let root = Node::with_value(&mut arena, NodeValue::Document);

    let p1 = Node::with_value(&mut arena, NodeValue::Paragraph);
    {
        let node = Node::with_value(&mut arena, NodeValue::Text("Line 1".into()));
        p1.append(&arena, node);
    }
    {
        let node = Node::with_value(&mut arena, NodeValue::LineBreak);
        p1.append(&arena, node);
    }
    root.append(&arena, p1);

    let p2 = Node::with_value(&mut arena, NodeValue::Paragraph);
    {
        let node = Node::with_value(&mut arena, NodeValue::Text("Line 2".into()));
        p2.append(&arena, node);
    }
    root.append(&arena, p2);

    let mut output = String::new();
    cm::format_document(&arena, root, &options, &mut output).unwrap();

    compare_strs(&output, "Line 1\n\nLine 2\n", "rendered", "<synthetic>");
}

#[test]
fn commonmark_renders_single_list_item() {
    let mut arena = Arena::new();
    let options = Options::default();
    let list_options = NodeList {
        list_type: ListType::Ordered,
        start: 1,
        ..Default::default()
    };
    let list = Node::with_value(&mut arena, NodeValue::List(list_options));
    let item = Node::with_value(&mut arena, NodeValue::Item(list_options));
    let p = Node::with_value(&mut arena, NodeValue::Paragraph);
    {
        let node = Node::with_value(&mut arena, NodeValue::Text("Item 1".into()));
        p.append(&arena, node);
    }
    item.append(&arena, p);
    list.append(&arena, item);
    let mut output = String::new();
    cm::format_document(&arena, item, &options, &mut output).unwrap();
    compare_strs(&output, "1. Item 1\n", "rendered", "<synthetic>");
}

#[test_case("$$x^2$$ and $1 + 2$ and $`y^2`$", "$$x^2$$ and $1 + 2$ and $`y^2`$\n")]
#[test_case("$$\nx^2\n$$", "$$\nx^2\n$$\n")]
#[test_case("```math\nx^2\n```", "``` math\nx^2\n```\n")]
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
