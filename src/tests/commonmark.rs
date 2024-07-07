use std::cell::RefCell;

use self::nodes::{Ast, LineColumn};

use super::*;
use ntest::test_case;

#[test]
fn commonmark_removes_redundant_strong() {
    let options = Options::default();

    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";

    commonmark(input, output, Some(&options));
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

#[test_case("$$x^2$$ and $1 + 2$ and $`y^2`$", "$$x^2$$ and $1 + 2$ and $`y^2`$\n")]
#[test_case("$$\nx^2\n$$", "$$\nx^2\n$$\n")]
#[test_case("```math\nx^2\n```", "``` math\nx^2\n```\n")]
fn math(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    commonmark(markdown, cm, Some(&options));
}

#[test_case("This [[url]] that", "This [[url|url]] that\n")]
#[test_case("This [[url|link label]] that", "This [[url|link%20label]] that\n")]
fn wikilinks(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.wikilinks_title_before_pipe = true;

    commonmark(markdown, cm, Some(&options));
}
