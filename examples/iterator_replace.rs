extern crate comrak;
use comrak::nodes::NodeValue;
use comrak::{format_html, parse_document, Arena, Options};

fn replace_text(document: &str, orig_string: &str, replacement: &str) -> String {
    // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
    let arena = Arena::new();

    // Parse the document into a root `AstNode`
    let root = parse_document(&arena, document, &Options::default());

    // Iterate over all the descendants of root.
    for node in root.descendants() {
        if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
            // If the node is a text node, append its text to `output_text`.
            *text = text.replace(orig_string, replacement)
        }
    }

    let mut html = vec![];
    format_html(root, &Options::default(), &mut html).unwrap();

    String::from_utf8(html).unwrap()
}

fn main() {
    let doc = "This is my input.\n\n1. Also [my](#) input.\n2. Certainly *my* input.\n".to_string();
    let orig = "my".to_string();
    let repl = "your".to_string();
    let html = replace_text(&doc, &orig, &repl);

    println!("{}", html);
}
