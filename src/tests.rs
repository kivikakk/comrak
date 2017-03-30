use ::{Arena, parse_document, format_document};

fn compare(input: &[u8], expected: &str) {
    let arena = Arena::new();
    let ast = parse_document(&arena, input, 0);
    let html = format_document(ast);
    if html != expected {
        println!("Got:");
        println!("==============================");
        println!("{}", html);
        println!("==============================");
        println!();
        println!("Expected:");
        println!("==============================");
        println!("{}", expected);
        println!("==============================");
        println!();
    }
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
            concat!("<hr />\n", "<hr />\n", "<hr />\n"));
}

#[test]
fn setext_heading() {
    compare(b"Hi\n==\n\nOk\n-----\n",
            concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"));
}

#[test]
fn html_block_1() {
    compare(b"<script\n*ok* </script> *ok*\n\n*ok*\n\n*ok*\n\n\
<pre x>\n*ok*\n</style>\n*ok*\n<style>\n*ok*\n</style>\n\n*ok*\n",
            concat!("<script\n",
                    "*ok* </script> *ok*\n",
                    "<p><em>ok</em></p>\n",
                    "<p><em>ok</em></p>\n",
                    "<pre x>\n",
                    "*ok*\n",
                    "</style>\n",
                    "<p><em>ok</em></p>\n",
                    "<style>\n",
                    "*ok*\n",
                    "</style>\n",
                    "<p><em>ok</em></p>\n"));
}

#[test]
fn html_block_2() {
    compare(b"   <!-- abc\n\nok --> *hi*\n*hi*\n",
            concat!("   <!-- abc\n",
                    "\n",
                    "ok --> *hi*\n",
                    "<p><em>hi</em></p>\n"));
}

#[test]
fn html_block_3() {
    compare(b" <? o\nk ?> *a*\n*a*\n",
            concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"));
}

#[test]
fn html_block_4() {
    compare(b"<!X >\nok\n<!X\num > h\nok\n",
            concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"));
}

#[test]
fn html_block_5() {
    compare(b"<![CDATA[\n\nhm >\n*ok*\n]]> *ok*\n*ok*\n",
            concat!("<![CDATA[\n",
                    "\n",
                    "hm >\n",
                    "*ok*\n",
                    "]]> *ok*\n",
                    "<p><em>ok</em></p>\n"));
}
