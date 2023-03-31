use super::*;
use ntest::timeout;

#[test]
fn pointy_brace() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "URI autolink: <https://www.pixiv.net>\n",
            "\n",
            "Email autolink: <bill@microsoft.com>\n",
            "\n",
            "* Inline <em>tag</em> **ha**.\n",
            "* Inline <!-- comment --> **ha**.\n",
            "* Inline <? processing instruction ?> **ha**.\n",
            "* Inline <!DECLARATION OKAY> **ha**.\n",
            "* Inline <![CDATA[ok]ha **ha** ]]> **ha**.\n"
        ),
        concat!(
            "<p>URI autolink: <a \
             href=\"https://www.pixiv.net\">https://www.pixiv.net</a></p>\n",
            "<p>Email autolink: <a \
             href=\"mailto:bill@microsoft.com\">bill@microsoft.com</a></p>\n",
            "<ul>\n",
            "<li>Inline <em>tag</em> <strong>ha</strong>.</li>\n",
            "<li>Inline <!-- comment --> <strong>ha</strong>.</li>\n",
            "<li>Inline <? processing instruction ?> <strong>ha</strong>.</li>\n",
            "<li>Inline <!DECLARATION OKAY> <strong>ha</strong>.</li>\n",
            "<li>Inline <![CDATA[ok]ha **ha** ]]> <strong>ha</strong>.</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn no_control_characters_in_reference_links() {
    html(
        "[A]:\u{1b}\n\nX [A] Y\n",
        "<p>[A]:\u{1b}</p>\n<p>X [A] Y</p>\n",
    )
}

#[test]
fn link_entity_regression() {
    html(
        "[link](&#x6A&#x61&#x76&#x61&#x73&#x63&#x72&#x69&#x70&#x74&#x3A&#x61&#x6C&#x65&#x72&#x74&#x28&#x27&#x58&#x53&#x53&#x27&#x29)",
        "<p><a href=\"&amp;#x6A&amp;#x61&amp;#x76&amp;#x61&amp;#x73&amp;#x63&amp;#x72&amp;#x69&amp;#x70&amp;#x74&amp;#x3A&amp;#x61&amp;#x6C&amp;#x65&amp;#x72&amp;#x74&amp;#x28&amp;#x27&amp;#x58&amp;#x53&amp;#x53&amp;#x27&amp;#x29\">link</a></p>\n",
    );
}

#[test]
fn regression_back_to_back_ranges() {
    html(
        "**bold*****bold+italic***",
        "<p><strong>bold</strong><em><strong>bold+italic</strong></em></p>\n",
    );
}

#[test]
#[timeout(4000)]
fn pathological_emphases() {
    let mut s = String::with_capacity(50000 * 4);
    for _ in 0..50000 {
        s.push_str("*a_ ");
    }

    let mut exp = format!("<p>{}", s);
    // Right-most space is trimmed in output.
    exp.pop();
    exp += "</p>\n";

    html(&s, &exp);
}

#[test]
fn no_panic_on_empty_bookended_atx_headers() {
    html("#  #", "<h1></h1>\n");
}

#[test]
fn no_stack_smash_html() {
    let s: String = ">".repeat(150_000);
    let arena = Arena::new();
    let root = parse_document(&arena, &s, &ComrakOptions::default());
    let mut output = vec![];
    html::format_document(root, &ComrakOptions::default(), &mut output).unwrap()
}

#[test]
fn no_stack_smash_cm() {
    let s: String = ">".repeat(150_000);
    let arena = Arena::new();
    let root = parse_document(&arena, &s, &ComrakOptions::default());
    let mut output = vec![];
    cm::format_document(root, &ComrakOptions::default(), &mut output).unwrap()
}

#[test]
fn cm_autolink_regression() {
    // Testing that the cm renderer handles this case without crashing
    html("<a+c:dd>", "<p><a href=\"a+c:dd\">a+c:dd</a></p>\n");
}

#[test]
fn regression_424() {
    html(
        "*text* [link](#section)",
        "<p><em>text</em> <a href=\"#section\">link</a></p>\n",
    );
}

#[test]
fn example_61() {
    html(
        r##"
`Foo
----
`

<a title="a lot
---
of dashes"/>
"##,
        r##"<h2>`Foo</h2>
<p>`</p>
<h2>&lt;a title=&quot;a lot</h2>
<p>of dashes&quot;/&gt;</p>
"##,
    );
}

#[test]
fn nul_at_eof() {
    html("foo\0", "<p>foo\u{fffd}</p>\n");
    html("foo\0ba", "<p>foo\u{fffd}ba</p>\n");
    html("foo\0ba\0", "<p>foo\u{fffd}ba\u{fffd}</p>\n");
}

#[test]
fn sourcepos_para() {
    html_opts!(
        [render.sourcepos],
        "abc\ndef\n\nghi\n",
        "<p data-sourcepos=\"1:1-2:3\">abc\ndef</p>\n<p data-sourcepos=\"4:1-4:3\">ghi</p>\n",
    );
}

#[test]
#[cfg(feature = "shortcodes")]
fn gemoji() {
    html_opts!([extension.shortcodes], ":x:", "<p>❌</p>\n");
}
