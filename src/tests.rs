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
    compare(concat!("My **document**.\n",
                    "\n",
                    "It's mine.\n",
                    "\n",
                    "> Yes.\n",
                    "\n",
                    "## Hi!\n",
                    "\n",
                    "Okay.\n"),
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
    compare(concat!("``` rust\n", "fn main<'a>();\n", "```\n"),
            concat!("<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n",
                    "</code></pre>\n"));
}

#[test]
fn lists() {
    compare(concat!("2. Hello.\n", "3. Hi.\n"),
            concat!("<ol start=\"2\">\n",
                    "<li>Hello.</li>\n",
                    "<li>Hi.</li>\n",
                    "</ol>\n"));

    compare(concat!("- Hello.\n", "- Hi.\n"),
            concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"));
}

#[test]
fn thematic_breaks() {
    compare(concat!("---\n", "\n", "- - -\n", "\n", "\n", "_        _   _\n"),
            concat!("<hr />\n", "<hr />\n", "<hr />\n"));
}

#[test]
fn setext_heading() {
    compare(concat!("Hi\n", "==\n", "\n", "Ok\n", "-----\n"),
            concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"));
}

#[test]
fn html_block_1() {
    compare(concat!("<script\n",
                    "*ok* </script> *ok*\n",
                    "\n",
                    "*ok*\n",
                    "\n",
                    "*ok*\n",
                    "\n",
                    "<pre x>\n",
                    "*ok*\n",
                    "</style>\n",
                    "*ok*\n",
                    "<style>\n",
                    "*ok*\n",
                    "</style>\n",
                    "\n",
                    "*ok*\n"),
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
    compare(concat!("   <!-- abc\n", "\n", "ok --> *hi*\n", "*hi*\n"),
            concat!("   <!-- abc\n",
                    "\n",
                    "ok --> *hi*\n",
                    "<p><em>hi</em></p>\n"));
}

#[test]
fn html_block_3() {
    compare(concat!(" <? o\n", "k ?> *a*\n", "*a*\n"),
            concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"));
}

#[test]
fn html_block_4() {
    compare(concat!("<!X >\n", "ok\n", "<!X\n", "um > h\n", "ok\n"),
            concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"));
}

#[test]
fn html_block_5() {
    compare(concat!("<![CDATA[\n",
                    "\n",
                    "hm >\n",
                    "*ok*\n",
                    "]]> *ok*\n",
                    "*ok*\n"),
            concat!("<![CDATA[\n",
                    "\n",
                    "hm >\n",
                    "*ok*\n",
                    "]]> *ok*\n",
                    "<p><em>ok</em></p>\n"));
}

#[test]
fn html_block_6() {
    compare(concat!(" </table>\n", "*x*\n", "\n", "ok\n", "\n", "<li\n", "*x*\n"),
            concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"));
}

#[test]
fn html_block_7() {
    compare(concat!("<a b >\n",
                    "ok\n",
                    "\n",
                    "<a b=>\n",
                    "ok\n",
                    "\n",
                    "<a b \n",
                    "<a b> c\n",
                    "ok\n"),
            concat!("<a b >\n",
                    "ok\n",
                    "<p>&lt;a b=&gt;\n",
                    "ok</p>\n",
                    "<p>&lt;a b\n",
                    "<a b> c\n",
                    "ok</p>\n"));

    compare(concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "\n", "ok\n"),
            concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "<p>ok</p>\n"));
}

#[test]
fn backticks() {
    compare("Some `code\\` yep.\n",
            "<p>Some <code>code\\</code> yep.</p>\n");
}

#[test]
fn backslashes() {
    compare(concat!("Some \\`fake code\\`.\n",
                    "\n",
                    "Some fake linebreaks: \\\n",
                    "\\\n",
                    "See?\n",
                    "\n",
                    "Ga\\rbage.\n"),
            concat!("<p>Some `fake code`.</p>\n",
                    "<p>Some fake linebreaks: <br />\n",
                    "<br />\n",
                    "See?</p>\n",
                    "<p>Ga\\rbage.</p>\n"));
}

#[test]
fn entities() {
    compare(concat!("This is &amp;, &copy;, &trade;, \\&trade;, &xyz;, &NotEqualTilde;.\n",
                    "\n",
                    "&#8734; &#x221e;\n"),
            concat!("<p>This is &amp;, ©, ™, &amp;trade;, &amp;xyz;, \u{2242}\u{338}.</p>\n",
                    "<p>∞ ∞</p>\n"));
}

#[test]
fn pointy_brace() {
    compare(concat!("URI autolink: <https://www.pixiv.net>\n",
                    "\n",
                    "Email autolink: <bill@microsoft.com>\n",
                    "\n",
                    "* Inline <em>tag</em> **ha**.\n",
                    "* Inline <!-- comment --> **ha**.\n",
                    "* Inline <? processing instruction ?> **ha**.\n",
                    "* Inline <!DECLARATION OKAY> **ha**.\n",
                    "* Inline <![CDATA[ok]ha **ha** ]]> **ha**.\n"),
            concat!("<p>URI autolink: <a \
                     href=\"https://www.pixiv.net\">https://www.pixiv.net</a></p>\n",
                    "<p>Email autolink: <a \
                     href=\"mailto:bill@microsoft.com\">bill@microsoft.com</a></p>\n",
                    "<ul>\n",
                    "<li>Inline <em>tag</em> <strong>ha</strong>.</li>\n",
                    "<li>Inline <!-- comment --> <strong>ha</strong>.</li>\n",
                    "<li>Inline <? processing instruction ?> <strong>ha</strong>.</li>\n",
                    "<li>Inline <!DECLARATION OKAY> <strong>ha</strong>.</li>\n",
                    "<li>Inline <![CDATA[ok]ha **ha** ]]> <strong>ha</strong>.</li>\n",
                    "</ul>\n"));
}

#[test]
fn links() {
    compare(concat!("Where are you [going](https://microsoft.com (today))?\n",
                    "\n",
                    "[Where am I?](/here)\n"),
            concat!("<p>Where are you <a href=\"https://microsoft.com\" \
                     title=\"today\">going</a>?</p>\n",
                    "<p><a href=\"/here\">Where am I?</a></p>\n"));
}

#[test]
fn images() {
    compare(concat!("I am ![eating [things](/url)](http://i.imgur.com/QqK1vq7.png).\n"),
            concat!("<p>I am <img src=\"http://i.imgur.com/QqK1vq7.png\" alt=\"eating things\" \
                     />.</p>\n"));
}
