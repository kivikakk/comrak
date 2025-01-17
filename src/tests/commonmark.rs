use std::cell::RefCell;

use self::nodes::{Ast, LineColumn, ListType, NodeList};

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
    let arena = Arena::new();
    let options = Options::default();
    let empty = LineColumn { line: 0, column: 0 };

    let ast = |val: NodeValue| arena.alloc(AstNode::new(RefCell::new(Ast::new(val, empty))));
    let root = ast(NodeValue::Document);

    let p1 = ast(NodeValue::Paragraph);
    p1.append(ast(NodeValue::Text("Line 1".to_owned())));
    p1.append(ast(NodeValue::LineBreak));
    root.append(p1);

    let p2 = ast(NodeValue::Paragraph);
    p2.append(ast(NodeValue::Text("Line 2".to_owned())));
    root.append(p2);

    let mut output = vec![];
    cm::format_document(root, &options, &mut output).unwrap();

    compare_strs(
        &String::from_utf8(output).unwrap(),
        "Line 1\n\nLine 2\n",
        "rendered",
        "<synthetic>",
    );
}

#[test]
fn commonmark_renders_single_list_item() {
    let arena = Arena::new();
    let options = Options::default();
    let empty = LineColumn { line: 0, column: 0 };
    let ast = |val: NodeValue| arena.alloc(AstNode::new(RefCell::new(Ast::new(val, empty))));
    let list_options = NodeList {
        list_type: ListType::Ordered,
        start: 1,
        ..Default::default()
    };
    let list = ast(NodeValue::List(list_options));
    let item = ast(NodeValue::Item(list_options));
    let p = ast(NodeValue::Paragraph);
    p.append(ast(NodeValue::Text("Item 1".to_owned())));
    item.append(p);
    list.append(item);
    let mut output = vec![];
    cm::format_document(item, &options, &mut output).unwrap();
    compare_strs(
        &String::from_utf8(output).unwrap(),
        "1. Item 1\n",
        "rendered",
        "<synthetic>",
    );
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
    options.extension.wikilinks_title_before_pipe = true;

    commonmark(markdown, cm, Some(&options));
}
