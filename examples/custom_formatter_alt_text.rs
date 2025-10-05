// Example provided by https://github.com/slonkazoid --- thank you!
// (https://github.com/kivikakk/comrak/issues/557)
//
// Defaults image title text to alt text, if provided.

use comrak::nodes::{Ast, AstNode, NodeLink, NodeValue};
use comrak::{parse_document, Arena};

fn autotitle_images<'a>(
    arena: &mut Arena<Ast>,
    nl: &mut NodeLink,
    _context: &mut comrak::html::Context,
    node: AstNode,
    entering: bool,
) {
    if !entering || !nl.title.is_empty() {
        return;
    }

    let mut s = String::new();

    for child in node.children(arena).collect::<Vec<_>>() {
        if let Some(text) = child.get_mut(arena).value.text() {
            s += text;
        }
    }

    nl.title = s;
}

fn formatter<'a, 'o, 'c>(
    context: &mut comrak::html::Context<'a, 'o, 'c>,
    node: comrak::nodes::AstNode,
    entering: bool,
) -> Result<comrak::html::ChildRendering, std::fmt::Error> {
    let borrow = node.get(context.arena);
    // XXX: we don't allow formatters to modify the arena. Should we?
    // if let NodeValue::Image(ref mut nl) = borrow.value {
    //     autotitle_images(nl, context, node, entering);
    // }
    drop(borrow);
    comrak::html::format_node_default(context, node, entering)
}

fn main() {
    let mut arena = Arena::new();
    let parsed = parse_document(
        &mut arena,
        "![my epic image](/img.png)",
        &Default::default(),
    );

    let mut out = String::new();
    comrak::html::format_document_with_formatter(
        &arena,
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
