// Example provided by https://github.com/slonkazoid --- thank you!
// (https://github.com/kivikakk/comrak/issues/557)
//
// Defaults image title text to alt text, if provided.

use std::cell::RefCell;

use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeLink, NodeValue};
use comrak::{parse_document, Arena};

fn autotitle_images<'a>(
    nl: &mut NodeLink,
    _context: &mut comrak::html::Context,
    node: &'a Node<'a, RefCell<Ast>>,
    entering: bool,
) {
    if !entering || !nl.title.is_empty() {
        return;
    }

    let mut s = String::new();

    for child in node.children() {
        if let Some(text) = child.data.borrow().value.text() {
            s += text;
        }
    }

    nl.title = s;
}

fn formatter<'a>(
    context: &mut comrak::html::Context,
    node: &'a comrak::nodes::AstNode<'a>,
    entering: bool,
) -> Result<comrak::html::ChildRendering, std::fmt::Error> {
    let mut borrow = node.data.borrow_mut();
    if let NodeValue::Image(ref mut nl) = borrow.value {
        autotitle_images(nl, context, node, entering);
    }
    drop(borrow);
    comrak::html::format_node_default(context, node, entering)
}

fn main() {
    let arena = Arena::new();
    let parsed = parse_document(&arena, "![my epic image](/img.png)", &Default::default());

    let mut out = String::new();
    comrak::html::format_document_with_formatter(
        parsed,
        &Default::default(),
        &mut out,
        &Default::default(),
        formatter,
        (),
    )
    .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));

    println!("{out}");
}
