use crate::nodes::{NodeCode, NodeValue};

use super::*;

#[test]
fn basic() {
    html(
        concat!(
            "My **document**.\n",
            "\n",
            "It's mine.\n",
            "\n",
            "> Yes.\n",
            "\n",
            "## Hi!\n",
            "\n",
            "Okay.\n"
        ),
        concat!(
            "<p>My <strong>document</strong>.</p>\n",
            "<p>It's mine.</p>\n",
            "<blockquote>\n",
            "<p>Yes.</p>\n",
            "</blockquote>\n",
            "<h2>Hi!</h2>\n",
            "<p>Okay.</p>\n"
        ),
    );
}

#[test]
fn codefence() {
    html(
        concat!("``` rust yum\n", "fn main<'a>();\n", "```\n"),
        concat!(
            "<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n",
            "</code></pre>\n"
        ),
    );
}

#[test]
fn lists() {
    html(
        concat!("2. Hello.\n", "3. Hi.\n"),
        concat!(
            "<ol start=\"2\">\n",
            "<li>Hello.</li>\n",
            "<li>Hi.</li>\n",
            "</ol>\n"
        ),
    );

    html(
        concat!("- Hello.\n", "- Hi.\n"),
        concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"),
    );
}
#[test]
fn thematic_breaks() {
    html(
        concat!("---\n", "\n", "- - -\n", "\n", "\n", "_        _   _\n"),
        concat!("<hr />\n", "<hr />\n", "<hr />\n"),
    );
}

#[test]
fn setext_heading() {
    html(
        concat!("Hi\n", "==\n", "\n", "Ok\n", "-----\n"),
        concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"),
    );
}

#[test]
fn html_block_1() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<script>\n",
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
            "*ok*\n"
        ),
        concat!(
            "<script>\n",
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
            "<p><em>ok</em></p>\n"
        ),
    );
}

#[test]
fn html_block_2() {
    html_opts!(
        [render.unsafe_],
        concat!("   <!-- abc\n", "\n", "ok --> *hi*\n", "*hi*\n"),
        concat!(
            "   <!-- abc\n",
            "\n",
            "ok --> *hi*\n",
            "<p><em>hi</em></p>\n"
        ),
    );
}

#[test]
fn html_block_3() {
    html_opts!(
        [render.unsafe_],
        concat!(" <? o\n", "k ?> *a*\n", "*a*\n"),
        concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"),
    );
}

#[test]
fn html_block_4() {
    html_opts!(
        [render.unsafe_],
        concat!("<!X >\n", "ok\n", "<!X\n", "um > h\n", "ok\n"),
        concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"),
    );
}

#[test]
fn html_block_5() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<![CDATA[\n",
            "\n",
            "hm >\n",
            "*ok*\n",
            "]]> *ok*\n",
            "*ok*\n"
        ),
        concat!(
            "<![CDATA[\n",
            "\n",
            "hm >\n",
            "*ok*\n",
            "]]> *ok*\n",
            "<p><em>ok</em></p>\n"
        ),
    );
}

#[test]
fn html_block_6() {
    html_opts!(
        [render.unsafe_],
        concat!(" </table>\n", "*x*\n", "\n", "ok\n", "\n", "<li\n", "*x*\n"),
        concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"),
    );
}

#[test]
fn html_block_7() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<a b >\n",
            "ok\n",
            "\n",
            "<a b=>\n",
            "ok\n",
            "\n",
            "<a b \n",
            "<a b> c\n",
            "ok\n"
        ),
        concat!(
            "<a b >\n",
            "ok\n",
            "<p>&lt;a b=&gt;\n",
            "ok</p>\n",
            "<p>&lt;a b\n",
            "<a b> c\n",
            "ok</p>\n"
        ),
    );

    html_opts!(
        [render.unsafe_],
        concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "\n", "ok\n"),
        concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "<p>ok</p>\n"),
    );
}

#[test]
fn backticks() {
    html(
        "Some `code\\` yep.\n",
        "<p>Some <code>code\\</code> yep.</p>\n",
    );
}

#[test]
fn backticks_empty_with_newline_should_be_space() {
    html("`\n`", "<p><code> </code></p>\n");
}

#[test]
fn blockquote_hard_linebreak_space() {
    html(">\\\n A", "<blockquote>\n<p><br />\nA</p>\n</blockquote>\n");
}

#[test]
fn blockquote_hard_linebreak_nonlazy_space() {
    html(
        "> A\\\n> B",
        "<blockquote>\n<p>A<br />\nB</p>\n</blockquote>\n",
    );
}

#[test]
fn backticks_num() {
    let input = "Some `code1`. More ``` code2 ```.\n";

    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, input, &options);

    let code1 = NodeValue::Code(NodeCode {
        num_backticks: 1,
        literal: "code1".to_string(),
    });
    asssert_node_eq(root, &[0, 1], &code1);

    let code2 = NodeValue::Code(NodeCode {
        num_backticks: 3,
        literal: "code2".to_string(),
    });
    asssert_node_eq(root, &[0, 3], &code2);
}

#[test]
fn backslashes() {
    html(
        concat!(
            "Some \\`fake code\\`.\n",
            "\n",
            "Some fake linebreaks:\\\n",
            "Yes.\\\n",
            "See?\n",
            "\n",
            "Ga\\rbage.\n"
        ),
        concat!(
            "<p>Some `fake code`.</p>\n",
            "<p>Some fake linebreaks:<br />\n",
            "Yes.<br />\n",
            "See?</p>\n",
            "<p>Ga\\rbage.</p>\n"
        ),
    );
}

#[test]
fn entities() {
    html(
        concat!(
            "This is &amp;, &copy;, &trade;, \\&trade;, &xyz;, &NotEqualTilde;.\n",
            "\n",
            "&#8734; &#x221e;\n"
        ),
        concat!(
            "<p>This is &amp;, ©, ™, &amp;trade;, &amp;xyz;, \u{2242}\u{338}.</p>\n",
            "<p>∞ ∞</p>\n"
        ),
    );
}

#[test]
fn links() {
    html(
        concat!(
            "Where are you [going](https://microsoft.com (today))?\n",
            "\n",
            "[Where am I?](/here)\n"
        ),
        concat!(
            "<p>Where are you <a href=\"https://microsoft.com\" \
             title=\"today\">going</a>?</p>\n",
            "<p><a href=\"/here\">Where am I?</a></p>\n"
        ),
    );
}

#[test]
fn images() {
    html(
        concat!("I am ![eating [things](/url)](http://i.imgur.com/QqK1vq7.png).\n"),
        concat!(
            "<p>I am <img src=\"http://i.imgur.com/QqK1vq7.png\" alt=\"eating things\" \
             />.</p>\n"
        ),
    );
}

#[test]
fn reference_links() {
    html(
        concat!(
            "This [is] [legit], [very][honestly] legit.\n",
            "\n",
            "[legit]: ok\n",
            "[honestly]: sure \"hm\"\n"
        ),
        concat!(
            "<p>This [is] <a href=\"ok\">legit</a>, <a href=\"sure\" title=\"hm\">very</a> \
             legit.</p>\n"
        ),
    );
}

#[test]
fn safety() {
    html(
        concat!(
            "[data:image/png](data:image/png/x)\n\n",
            "[data:image/gif](data:image/gif/x)\n\n",
            "[data:image/jpeg](data:image/jpeg/x)\n\n",
            "[data:image/webp](data:image/webp/x)\n\n",
            "[data:malicious](data:malicious/x)\n\n",
            "[javascript:malicious](javascript:malicious)\n\n",
            "[vbscript:malicious](vbscript:malicious)\n\n",
            "[file:malicious](file:malicious)\n\n",
        ),
        concat!(
            "<p><a href=\"data:image/png/x\">data:image/png</a></p>\n",
            "<p><a href=\"data:image/gif/x\">data:image/gif</a></p>\n",
            "<p><a href=\"data:image/jpeg/x\">data:image/jpeg</a></p>\n",
            "<p><a href=\"data:image/webp/x\">data:image/webp</a></p>\n",
            "<p><a href=\"\">data:malicious</a></p>\n",
            "<p><a href=\"\">javascript:malicious</a></p>\n",
            "<p><a href=\"\">vbscript:malicious</a></p>\n",
            "<p><a href=\"\">file:malicious</a></p>\n",
        ),
    )
}

#[test]
fn link_backslash_requires_punct() {
    // Test should probably be in the spec.
    html("[a](\\ b)", "<p>[a](\\ b)</p>\n");
}

#[test]
fn nul_replacement_1() {
    html("a\0b", "<p>a\u{fffd}b</p>\n");
}

#[test]
fn nul_replacement_2() {
    html("a\0b\0c", "<p>a\u{fffd}b\u{fffd}c</p>\n");
}

#[test]
fn nul_replacement_3() {
    html("a\0\nb", "<p>a\u{fffd}\nb</p>\n");
}

#[test]
fn nul_replacement_4() {
    html("a\0\r\nb", "<p>a\u{fffd}\nb</p>\n");
}

#[test]
fn nul_replacement_5() {
    html("a\r\n\0b", "<p>a\n\u{fffd}b</p>\n");
}

#[test]
fn case_insensitive_safety() {
    html(
        "[a](javascript:a) [b](Javascript:b) [c](jaVascript:c) [d](data:xyz) [e](Data:xyz) [f](vbscripT:f) [g](FILE:g)\n",
        "<p><a href=\"\">a</a> <a href=\"\">b</a> <a href=\"\">c</a> <a href=\"\">d</a> <a href=\"\">e</a> <a href=\"\">f</a> <a href=\"\">g</a></p>\n",
    );
}
