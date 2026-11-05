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
fn whitespace_only_inline_footnote_not_parsed() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[\t\r]more.\n",
        "<p>Text^[\n]more.</p>\n",
    );

    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[   ]more.\n",
        "<p>Text^[   ]more.</p>\n",
    );

    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^[\t\n]more.\n",
        "<p>Text^[\n]more.</p>\n",
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

#[test]
fn inline_footnote_multiline() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        concat!(
            "Here is an inline note.^[Inline notes are easier to write, since\n",
            "you don't have to pick an identifier and move down to type the\n",
            "note.]\n"
        ),
        concat!(
            "<p>Here is an inline note.<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>Inline notes are easier to write, since\n",
            "you don't have to pick an identifier and move down to type the\n",
            "note. <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn escaped_caret_not_parsed() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        r"Escaped: \^[not a note]. Real: ^[yes].
",
        concat!(
            "<p>Escaped: ^[not a note]. Real: <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>yes <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn whitespace_between_caret_and_bracket() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "Text^ [not a note].\n",
        "<p>Text^ [not a note].</p>\n",
    );
}

#[test]
fn escaped_bracket_inside_body() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        r"A ^[square bracket \] inside].
",
        concat!(
            "<p>A <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>square bracket ] inside <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn caret_in_other_contexts() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "2^32 is big. ^[note].\n",
        concat!(
            "<p>2^32 is big. <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn inline_code_protects_marker() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "`^[nope]` and yes ^[ok].\n",
        concat!(
            "<p><code>^[nope]</code> and yes <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>ok <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn back_to_back_inline_footnotes() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "A^[one]^[two].\n",
        concat!(
            "<p>A<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>",
            "<sup class=\"footnote-ref\"><a href=\"#fn-__inline_2\" id=\"fnref-__inline_2\" data-footnote-ref>2</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>one <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-__inline_2\">\n",
            "<p>two <a href=\"#fnref-__inline_2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn no_surrounding_spaces() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "word^[tight]word\n",
        concat!(
            "<p>word<sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>word</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>tight <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn at_paragraph_start() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "^[start] begins.\n",
        concat!(
            "<p><sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup> begins.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>start <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn at_paragraph_end() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "ends here ^[fin]\n",
        concat!(
            "<p>ends here <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>fin <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn inside_emphasis() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "*emph ^[note] text*\n",
        concat!(
            "<p><em>emph <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup> text</em></p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn nested_inline_footnote_supported() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "A ^[outer and ^[inner] literal].\n",
        concat!(
            "<p>A <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>outer and <sup class=\"footnote-ref\"><a href=\"#fn-__inline_2\" id=\"fnref-__inline_2\" data-footnote-ref>2</a></sup> literal <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-__inline_2\">\n",
            "<p>inner <a href=\"#fnref-__inline_2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn with_image() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "X ^[see ![alt](img.png)].\n",
        concat!(
            "<p>X <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>see <img src=\"img.png\" alt=\"alt\" /> <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn with_unicode_and_emoji() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "X ^[π ≈ 3.14159 😊].\n",
        concat!(
            "<p>X <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>π ≈ 3.14159 😊 <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn in_list_item() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "- a ^[note]\n- b\n",
        concat!(
            "<ul>\n",
            "<li>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></li>\n",
            "<li>b</li>\n",
            "</ul>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn in_blockquote() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "> quote ^[q].\n",
        concat!(
            "<blockquote>\n",
            "<p>quote <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup>.</p>\n",
            "</blockquote>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>q <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn in_heading() {
    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        "# H ^[note]\n",
        concat!(
            "<h1>H <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></h1>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>note <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn in_table_cell() {
    html_opts!(
        [
            extension.footnotes,
            extension.inline_footnotes,
            extension.table
        ],
        "| H |\n| - |\n| c ^[cell] |\n",
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>H</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c <sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-__inline_1\">\n",
            "<p>cell <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn inline_footnote_sourcepos() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "a^[b *c* d] e\n",
        (document (1:1-1:13) [
            (paragraph (1:1-1:13) [
                (text (1:1-1:1) "a")
                (footnote_reference (1:2-1:11))
                (text (1:12-1:13) " e")
            ])
            (footnote_definition (1:2-1:11) [
                (paragraph (1:4-1:10) [
                    (text (1:4-1:5) "b ")
                    (emph (1:6-1:8) [
                        (text (1:7-1:7) "c")
                    ])
                    (text (1:9-1:10) " d")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_multiline() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "a^[b\nc] d\n",
        (document (1:1-2:4) [
            (paragraph (1:1-2:4) [
                (text (1:1-1:1) "a")
                (footnote_reference (1:2-2:2))
                (text (2:3-2:4) " d")
            ])
            (footnote_definition (1:2-2:2) [
                (paragraph (1:4-2:1) [
                    (text (1:4-1:4) "b")
                    (softbreak (1:5-1:5))
                    (text (2:1-2:1) "c")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_multiline_indented() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "> a^[b\n> c] d\n",
        (document (1:1-2:6) [
            (block_quote (1:1-2:6) [
                (paragraph (1:3-2:6) [
                    (text (1:3-1:3) "a")
                    (footnote_reference (1:4-2:4))
                    (text (2:5-2:6) " d")
                ])
            ])
            (footnote_definition (1:4-2:4) [
                (paragraph (1:6-2:3) [
                    (text (1:6-1:6) "b")
                    (softbreak (1:7-1:7))
                    (text (2:3-2:3) "c")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_nested_multiline() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "a^[b ^[c\nd] e\nf] g\n",
        (document (1:1-3:4) [
            (paragraph (1:1-3:4) [
                (text (1:1-1:1) "a")
                (footnote_reference (1:2-3:2))
                (text (3:3-3:4) " g")
            ])
            (footnote_definition (1:2-3:2) [
                (paragraph (1:4-3:1) [
                    (text (1:4-1:5) "b ")
                    (footnote_reference (1:6-2:2))
                    (text (2:3-2:4) " e")
                    (softbreak (2:5-2:5))
                    (text (3:1-3:1) "f")
                ])
            ])
            (footnote_definition (1:6-2:2) [
                (paragraph (1:8-2:1) [
                    (text (1:8-1:8) "c")
                    (softbreak (1:9-1:9))
                    (text (2:1-2:1) "d")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_chars() {
    assert_ast_match!(
        [
            extension.footnotes,
            extension.inline_footnotes,
            parse.sourcepos_chars
        ],
        "ä^[b *c* d] e\n",
        (document (1:1-1:13) [
            (paragraph (1:1-1:13) [
                (text (1:1-1:1) "ä")
                (footnote_reference (1:2-1:11))
                (text (1:12-1:13) " e")
            ])
            (footnote_definition (1:2-1:11) [
                (paragraph (1:4-1:10) [
                    (text (1:4-1:5) "b ")
                    (emph (1:6-1:8) [
                        (text (1:7-1:7) "c")
                    ])
                    (text (1:9-1:10) " d")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_table() {
    assert_ast_match!(
        [
            extension.footnotes,
            extension.inline_footnotes,
            extension.table
        ],
        "| H |\n| - |\n| c ^[cell] |\n",
        (document (1:1-3:13) [
            (table (1:1-3:13) [
                (table_row (1:1-1:5) [
                    (table_cell (1:2-1:4) [
                        (text (1:3-1:3) "H")
                    ])
                ])
                (table_row (3:1-3:13) [
                    (table_cell (3:2-3:12) [
                        (text (3:3-3:4) "c ")
                        (footnote_reference (3:5-3:11))
                    ])
                ])
            ])
            (footnote_definition (3:5-3:11) [
                (paragraph (3:7-3:10) [
                    (text (3:7-3:10) "cell")
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_closing_bracket_on_its_own_line() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "^[a\n"
        "] b\n",
        (document (1:1-2:3) [
            (paragraph (1:1-2:3) [
                (footnote_reference (1:1-2:1))
                (text (2:2-2:3) " b")
            ])
            (footnote_definition (1:1-2:1) [
                (paragraph (1:3-1:4) [
                    (text (1:3-1:3) "a")
                    (softbreak (1:4-1:4))
                ])
            ])
        ])
    );

    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "^[a\r\n"
        "] b\n",
        (document (1:1-2:3) [
            (paragraph (1:1-2:3) [
                (footnote_reference (1:1-2:1))
                (text (2:2-2:3) " b")
            ])
            (footnote_definition (1:1-2:1) [
                (paragraph (1:3-1:5) [
                    (text (1:3-1:3) "a")
                    (softbreak (1:4-1:5))
                ])
            ])
        ])
    );
}

#[test]
fn inline_footnote_sourcepos_leading_break() {
    assert_ast_match!(
        [extension.footnotes, extension.inline_footnotes],
        "^[\t\n"
        "a]\n",
        (document (1:1-2:2) [
            (paragraph (1:1-2:2) [
                (footnote_reference (1:1-2:2))
            ])
            (footnote_definition (1:1-2:2) [
                (paragraph (1:3-2:1) [
                    (softbreak (1:4-1:4))
                    (text (2:1-2:1) "a")
                ])
            ])
        ])
    );
}
