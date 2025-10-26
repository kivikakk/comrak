// Example provided by https://github.com/slonkazoid --- thank you!
// (https://github.com/kivikakk/comrak/issues/557)
//
// Defaults image title text to alt text, if provided.

use comrak::nodes::{Node, NodeLink, NodeValue};
use comrak::{parse_document, Arena};

fn autotitle_images(
    nl: &mut NodeLink,
    context: &mut comrak::html::Context,
    node: Node,
    entering: bool,
) {
    if !entering || !nl.title.is_empty() {
        return;
    }

    let mut s = String::new();

    for child in node.children(context.arena) {
        if let Some(text) = child.data(context.arena).value.text() {
            s += text;
        }
    }

    nl.title = s;
}

fn formatter(
    context: &mut comrak::html::Context,
    node: comrak::nodes::Node,
    entering: bool,
) -> Result<comrak::html::ChildRendering, std::fmt::Error> {
    if let NodeValue::Image(ref mut nl) = node.data_mut(context.arena).value {
        autotitle_images(nl, context, node, entering);
    }
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
