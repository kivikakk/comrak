// Samples used in the README.  Wanna make sure they work as advertised.

fn small() {
    use comrak::{markdown_to_html, Options};

    assert_eq!(
        markdown_to_html("Hello, **世界**!", &Options::default()),
        "<p>Hello, <strong>世界</strong>!</p>\n"
    );
}

fn large() {
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
                // If the node is a text node, perform the string replacement.
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
        let html = replace_text(doc, orig, repl);

        println!("{}", html);
        // Output:
        //
        // <p>This is your input.</p>
        // <ol>
        // <li>Also <a href="#">your</a> input.</li>
        // <li>Certainly <em>your</em> input.</li>
        // </ol>
    }

    main()
}

fn main() {
    small();
    large();
}
