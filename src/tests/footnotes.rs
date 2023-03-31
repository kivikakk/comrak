use super::*;

#[test]
fn footnotes() {
    html_opts!(
        [extension.footnotes],
        concat!(
            "Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]\n",
            "\n",
            "This is another note.[^note]\n",
            "\n",
            "[^note]: Hi.\n",
            "\n",
            "[^1]: Here is the footnote.\n",
            "\n",
            "[^longnote]: Here's one with multiple blocks.\n",
            "\n",
            "    Subsequent paragraphs are indented.\n",
            "\n",
            "        code\n",
            "\n",
            "This is regular content.\n",
            "\n",
            "[^unused]: This is not used.\n"
        ),
        concat!(
            "<p>Here is a[^nowhere] footnote reference,<sup class=\"footnote-ref\"><a href=\"#fn-1\" \
             id=\"fnref-1\" data-footnote-ref>1</a></sup> and another.<sup class=\"footnote-ref\"><a \
             href=\"#fn-2\" id=\"fnref-2\" data-footnote-ref>2</a></sup></p>\n",
            "<p>This is another note.<sup class=\"footnote-ref\"><a href=\"#fn-3\" \
             id=\"fnref-3\" data-footnote-ref>3</a></sup></p>\n",
            "<p>This is regular content.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>Here is the footnote. <a href=\"#fnref-1\" \
             class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-2\">\n",
            "<p>Here's one with multiple blocks.</p>\n",
            "<p>Subsequent paragraphs are indented.</p>\n",
            "<pre><code>code\n",
            "</code></pre>\n",
            "<a href=\"#fnref-2\" class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a>\n",
            "</li>\n",
            "<li id=\"fn-3\">\n",
            "<p>Hi. <a href=\"#fnref-3\" \
             class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn footnote_does_not_eat_exclamation() {
    html_opts!(
        [extension.footnotes],
        concat!("Here's my footnote![^a]\n", "\n", "[^a]: Yep.\n"),
        concat!(
            "<p>Here's my footnote!<sup class=\"footnote-ref\"><a href=\"#fn-1\" \
             id=\"fnref-1\" data-footnote-ref>1</a></sup></p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>Yep. <a href=\"#fnref-1\" class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn footnote_in_table() {
    html_opts!(
        [extension.table, extension.footnotes],
        concat!(
            "A footnote in a paragraph[^1]\n",
            "\n",
            "| Column1   | Column2 |\n",
            "| --------- | ------- |\n",
            "| foot [^1] | note    |\n",
            "\n",
            "[^1]: a footnote\n",
        ), concat!(
            "<p>A footnote in a paragraph<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\" data-footnote-ref>1</a></sup></p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>Column1</th>\n",
            "<th>Column2</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>foot <sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\" data-footnote-ref>1</a></sup></td>\n",
            "<td>note</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>a footnote <a href=\"#fnref-1\" class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n",
        ));
}

#[test]
fn footnote_with_superscript() {
    html_opts!(
        [extension.superscript, extension.footnotes],
        concat!(
            "Here is a footnote reference.[^1]\n",
            "\n",
            "Here is a longer footnote reference.[^ref]\n",
            "\n",
            "e = mc^2^.\n",
            "\n",
            "[^1]: Here is the footnote.\n",
            "[^ref]: Here is another footnote.\n",
        ),
        concat!(
            "<p>Here is a footnote reference.<sup class=\"footnote-ref\"><a href=\"#fn-1\" \
             id=\"fnref-1\" data-footnote-ref>1</a></sup></p>\n",
            "<p>Here is a longer footnote reference.<sup class=\"footnote-ref\"><a href=\"#fn-2\" \
             id=\"fnref-2\" data-footnote-ref>2</a></sup></p>\n",
            "<p>e = mc<sup>2</sup>.</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>Here is the footnote. <a href=\"#fnref-1\" \
             class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn-2\">\n",
            "<p>Here is another footnote. <a href=\"#fnref-2\" \
             class=\"footnote-backref\" data-footnote-backref aria-label=\"Back to content\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.footnotes],
        "Here is a footnote reference.[^1]\n"
        "\n"
        "Here is a longer footnote reference.[^ref]\n"
        "\n"
        "[^1]: Here is the footnote.\n"
        "[^ref]: Here is another footnote.\n",
        (document (1:1-6:33) [
            (paragraph (1:1-1:33) [
                (text (1:1-1:29) "Here is a footnote reference.")
                (footnote_reference (1:30-1:33))
            ])
            (paragraph (3:1-3:42) [
                (text (3:1-3:36) "Here is a longer footnote reference.")
                (footnote_reference (3:37-3:42))
            ])
            (footnote_definition (5:1-5:27) [
                (paragraph (5:7-5:27) [
                    (text (5:7-5:27) "Here is the footnote.")
                ])
            ])
            (footnote_definition (6:1-6:33) [
                (paragraph (6:9-6:33) [
                    (text (6:9-6:33) "Here is another footnote.")
                ])
            ])
        ])
    );
}
