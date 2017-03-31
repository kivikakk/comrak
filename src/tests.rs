use ::{Arena, parse_document, format_document};

fn parse(input: &[char]) -> String {
    let arena = Arena::new();
    let ast = parse_document(&arena, input, 0);
    format_document(ast)
}

fn compare(input: &str, expected: &str) {
    let html = parse(&input.chars().collect::<Vec<char>>());
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
    compare("My **document**.\n\nIt's mine.\n\n> Yes.\n\n## Hi!\n\nOkay.\n",
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
    compare("``` rust\nfn main<'a>();\n```\n",
            concat!("<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n",
                    "</code></pre>\n"));
}

#[test]
fn lists() {
    compare("2. Hello.\n3. Hi.\n",
            concat!("<ol start=\"2\">\n",
                    "<li>Hello.</li>\n",
                    "<li>Hi.</li>\n",
                    "</ol>\n"));

    compare("- Hello.\n- Hi.\n",
            concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"));
}

#[test]
fn thematic_breaks() {
    compare("---\n\n- - -\n\n\n_        _   _\n",
            concat!("<hr />\n", "<hr />\n", "<hr />\n"));
}

#[test]
fn setext_heading() {
    compare("Hi\n==\n\nOk\n-----\n",
            concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"));
}

#[test]
fn html_block_1() {
    compare("<script\n*ok* </script> *ok*\n\n*ok*\n\n*ok*\n\n\
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
    compare("   <!-- abc\n\nok --> *hi*\n*hi*\n",
            concat!("   <!-- abc\n",
                    "\n",
                    "ok --> *hi*\n",
                    "<p><em>hi</em></p>\n"));
}

#[test]
fn html_block_3() {
    compare(" <? o\nk ?> *a*\n*a*\n",
            concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"));
}

#[test]
fn html_block_4() {
    compare("<!X >\nok\n<!X\num > h\nok\n",
            concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"));
}

#[test]
fn html_block_5() {
    compare("<![CDATA[\n\nhm >\n*ok*\n]]> *ok*\n*ok*\n",
            concat!("<![CDATA[\n",
                    "\n",
                    "hm >\n",
                    "*ok*\n",
                    "]]> *ok*\n",
                    "<p><em>ok</em></p>\n"));
}

#[test]
fn html_block_6() {
    compare(" </table>\n*x*\n\nok\n\n<li\n*x*\n",
            concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"));
}

#[test]
fn html_block_7() {
    // XXX: relies too much on entity conversion and inlines
    //
    // compare("<a b >\nok\n\n<a b=>\nok\n\n<a b \n<a b> c\nok\n",
    // concat!("<a b >\n",
    // "ok\n",
    // "<p>&lt;a b=&gt;\n",
    // "ok</p>\n",
    // "<p>&lt;a b\n",
    // "<a b> c\n",
    // "ok</p>\n"));
    //


    compare("<a b c=x d='y' z=\"f\" >\nok\n\nok\n",
            concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "<p>ok</p>\n"));
}
