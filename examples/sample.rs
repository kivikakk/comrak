// Samples used in the README.  Wanna make sure they work as advertised.

fn small() {
    use comrak::{markdown_to_html, Options};

    assert_eq!(
        markdown_to_html("¡Olá, **世界**!", &Options::default()),
        "<p>¡Olá, <strong>世界</strong>!</p>\n"
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

        let mut html = String::new();
        format_html(root, &Options::default(), &mut html).unwrap();

        html
    }

    fn main() {
        let doc = "Hello, pretty world!\n\n1. Do you like [pretty](#) paintings?\n2. Or *pretty* music?\n";
        let orig = "pretty";
        let repl = "beautiful";
        let html = replace_text(doc, orig, repl);

        println!("{}", html);
        // Output:
        //
        // <p>Hello, beautiful world!</p>
        // <ol>
        // <li>Do you like <a href="#">beautiful</a> paintings?</li>
        // <li>Or <em>beautiful</em> music?</li>
        // </ol>
    }

    main()
}

fn main() {
    small();
    large();
}
