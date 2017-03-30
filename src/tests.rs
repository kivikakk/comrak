use ::{Arena, parse_document, format_document};

fn compare(input: &[u8], expected: &str) {
    let arena = Arena::new();
    let ast = parse_document(&arena, input, 0);
    let html = format_document(ast);
    assert_eq!(html, expected);
}

#[test]
fn basic() {
    compare(b"My **document**.\n\nIt's mine.\n\n> Yes.\n\n## Hi!\n\nOkay.\n",
            concat!("<p>My <strong>document</strong>.</p>\n",
                    "<p>It's mine.</p>\n",
                    "<blockquote>\n",
                    "<p>Yes.</p>\n",
                    "</blockquote>\n",
                    "<h2>Hi!</h2>\n",
                    "<p>Okay.</p>\n"));
}

#[test]
fn codefence() {
    compare(b"``` rust\nfn main();\n```\n",
            concat!("<pre><code class=\"language-rust\">fn main();\n",
                    "</code></pre>\n"));
}

#[test]
fn lists() {
    compare(b"2. Hello.\n3. Hi.\n",
            concat!("<ol start=\"2\">\n",
                    "<li>Hello.</li>\n",
                    "<li>Hi.</li>\n",
                    "</ol>\n"));

    compare(b"- Hello.\n- Hi.\n",
            concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"));
}

#[test]
fn thematic_breaks() {
    compare(b"---\n\n- - -\n\n\n_        _   _\n",
            concat!("<hr />\n",
                    "<hr />\n",
                    "<hr />\n"));
}
