use super::*;

#[test]
fn basic_inline_footnote() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Here is an inline footnote^[This is the footnote content].\n",
        concat!(
            "<p>Here is an inline footnote<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>This is the footnote content <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn inline_footnote_with_emphasis() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[With *emphasis* and **strong**].\n",
        concat!(
            "<p>Text<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>With <em>emphasis</em> and <strong>strong</strong> <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn inline_footnote_with_link() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[See [example](https://example.com)].\n",
        concat!(
            "<p>Text<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>See <a href=\"https://example.com\">example</a> <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn multiple_inline_footnotes() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "First^[First note] and second^[Second note].\n",
        concat!(
            "<p>First<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup> ",
            "and second<sup class=\"footnote-ref\"><a href=\"#fn-__inline_2\" id=\"fnref-__inline_2\" data-footnote-ref>2</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>First note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-__inline_2\">\n",
            "<p>Second note <a href=\"#fnref-__inline_2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn mixed_inline_and_regular_footnotes() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        concat!(
            "Regular[^1] and inline^[Inline note] and another regular[^2].\n",
            "\n",
            "[^1]: First regular footnote.\n",
            "[^2]: Second regular footnote.\n"
        ),
        concat!(
            "<p>Regular<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\" data-footnote-ref>1</a></sup> ",
            "and inline<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>2</a></sup> ",
            "and another regular<sup class=\"footnote-ref\"><a href=\"#fn-2\" id=\"fnref-2\" data-footnote-ref>3</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>First regular footnote. <a href=\"#fnref-1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>Inline note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-2\">\n",
            "<p>Second regular footnote. <a href=\"#fnref-2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"3\" aria-label=\"Back to reference 3\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn empty_inline_footnote_not_parsed() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[] should not parse.\n",
        "<p>Text^[] should not parse.</p>\n",
    );
}

#[test]
fn inline_footnote_without_footnotes_enabled() {
    html_opts!(
        [extension.inline_footnotes],
        "Text^[note] should not parse.\n",
        "<p>Text^[note] should not parse.</p>\n",
    );
}

#[test]
fn inline_footnote_with_code() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[With `code` inline].\n",
        concat!(
            "<p>Text<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>With <code>code</code> inline <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

