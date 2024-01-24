use super::*;

#[test]
fn multiline_block_quotes() {
    html_opts!(
        [extension.multiline_block_quotes],
        concat!(">>>\n", "Paragraph 1\n", "\n", "Paragraph 2\n", ">>>\n",),
        concat!(
            "<blockquote>\n",
            "<p>Paragraph 1</p>\n",
            "<p>Paragraph 2</p>\n",
            "</blockquote>\n",
        ),
    );

    html_opts!(
        [extension.multiline_block_quotes],
        concat!(
            "- item one\n",
            "\n",
            "  >>>\n",
            "  Paragraph 1\n",
            "\n",
            "  Paragraph 2\n",
            "  >>>\n",
            "- item two\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>item one</p>\n",
            "<blockquote>\n",
            "<p>Paragraph 1</p>\n",
            "<p>Paragraph 2</p>\n",
            "</blockquote>\n",
            "</li>\n",
            "<li>\n",
            "<p>item two</p>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.multiline_block_quotes],
        "- item one\n"
        "\n"
        "  >>>\n"
        "  Paragraph 1\n"
        "  >>>\n"
        "- item two\n",
        (document (1:1-6:10) [
            (list (1:1-6:10) [
                (item (1:1-5:5) [      // (description_item (1:1-3:4) [
                    (paragraph (1:3-1:10) [
                        (text (1:3-1:10) "item one")
                    ])
                    (multiline_block_quote (3:3-5:5) [
                        (paragraph (4:3-4:13) [
                            (text (4:3-4:13) "Paragraph 1")
                        ])
                    ])
                ])
                (item (6:1-6:10) [      // (description_item (5:1-7:6) [
                    (paragraph (6:3-6:10) [
                        (text (6:3-6:10) "item two")
                    ])
                ])
            ])
        ])
    );
}
