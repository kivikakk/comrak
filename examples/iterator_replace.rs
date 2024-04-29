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
            // If the node is a text node, replace `orig_string` with `replacement`.
            *text = text.replace(orig_string, replacement)
        }
    }

    let mut html = vec![];
    format_html(root, &Options::default(), &mut html).unwrap();

    String::from_utf8(html).unwrap()
}

fn main() {
    let doc = "This is my input.\n\n1. Also [my](#) input.\n2. Certainly *my* input.\n";
    let orig = "my";
    let repl = "your";
    let html = replace_text(&doc, &orig, &repl);

    println!("{}", html);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::{assert_false, assert_true};

    #[test]
    fn sample_replace() {
        let doc = "Replace deeply nested *[foo](https://example.com)* with bar.\n\nReplace shallow foo with bar.";
        let orig = "foo";
        let repl = "bar";
        let html = replace_text(&doc, &orig, &repl);
        println!("{:?}", html);
        assert_false!(html.contains("foo"));
        assert_true!(html.contains("bar"));
        assert_true!(html.contains("<a"));
        assert_true!(html.contains("<p"));
    }
}
