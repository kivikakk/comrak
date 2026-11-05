use super::*;

#[test]
fn block_directives() {
    html_opts!(
        [extension.block_directive],
        concat!(
            ":::foo bar\n",
            "Paragraph 1\n",
            "\n",
            "Paragraph 2\n",
            ":::\n"
        ),
        concat!(
            "<div class=\"foo bar\">\n",
            "<p>Paragraph 1</p>\n",
            "<p>Paragraph 2</p>\n",
            "</div>\n",
        ),
    );

    html_opts!(
        [extension.block_directive],
        concat!(
            "- item one\n",
            "\n",
            "  :::foo bar\n",
            "  Paragraph 1\n",
            "\n",
            "  Paragraph 2\n",
            "  :::\n",
            "- item two\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>item one</p>\n",
            "<div class=\"foo bar\">\n",
            "<p>Paragraph 1</p>\n",
            "<p>Paragraph 2</p>\n",
            "</div>\n",
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
        [extension.block_directive],
        "- item one\n"
        "\n"
        "  :::foo bar\n"
        "  Paragraph 1\n"
        "  :::\n"
        "- item two\n",
        (document (1:1-6:10) [
            (list (1:1-6:10) [
                (item (1:1-5:5) [
                    (paragraph (1:3-1:10) [
                        (text (1:3-1:10) "item one")
                    ])
                    (block_directive (3:3-5:5) [
                        (paragraph (4:3-4:13) [
                            (text (4:3-4:13) "Paragraph 1")
                        ])
                    ])
                ])
                (item (6:1-6:10) [
                    (paragraph (6:3-6:10) [
                        (text (6:3-6:10) "item two")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_with_block_quote() {
    assert_ast_match!(
        [extension.block_directive],
        ":::foo bar\n"
        "> a\n"
        ":::\n",
        (document (1:1-3:3) [
            (block_directive (1:1-3:3) [
                (block_quote (2:1-2:3) [
                    (paragraph (2:3-2:3) [
                        (text (2:3-2:3) "a")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_with_list() {
    assert_ast_match!(
        [extension.block_directive],
        ":::foo bar\n"
        "* a\n"
        ":::\n",
        (document (1:1-3:3) [
            (block_directive (1:1-3:3) [
                (list (2:1-2:3) [
                    (item (2:1-2:3) [
                        (paragraph (2:3-2:3) [
                            (text (2:3-2:3) "a")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_in_blockquote() {
    assert_ast_match!(
        [extension.block_directive],
        "> :::foo bar\n"
        "> a\n"
        "> :::\n",
        (document (1:1-3:5) [
            (block_quote (1:1-3:5) [
                (block_directive (1:3-3:5) [
                    (paragraph (2:3-2:3) [
                        (text (2:3-2:3) "a")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_unclosed_in_list() {
    assert_ast_match!(
        [extension.block_directive],
        "- :::foo\n"
        "bar\n",
        (document (1:1-2:3) [
            (list (1:1-1:8) [
                (item (1:1-1:8) [
                    (block_directive (1:3-1:8))
                ])
            ])
            (paragraph (2:1-2:3) [
                (text (2:1-2:3) "bar")
            ])
        ])
    );

    assert_ast_match!(
        [extension.block_directive],
        "- :::foo\n"
        "  a\n"
        "\n"
        "bar\n",
        (document (1:1-4:3) [
            (list (1:1-2:3) [
                (item (1:1-2:3) [
                    (block_directive (1:3-2:3) [
                        (paragraph (2:3-2:3) [
                            (text (2:3-2:3) "a")
                        ])
                    ])
                ])
            ])
            (paragraph (4:1-4:3) [
                (text (4:1-4:3) "bar")
            ])
        ])
    );
}

#[test]
fn list_contents_finalized_on_close() {
    html_opts!(
        [extension.block_directive],
        concat!(":::\n", "- ```rust\n", "  code\n", ":::\n"),
        concat!(
            "<div class=\"\">\n",
            "<ul>\n",
            "<li>\n",
            "<pre><code class=\"language-rust\">code\n",
            "</code></pre>\n",
            "</li>\n",
            "</ul>\n",
            "</div>\n",
        ),
        no_roundtrip,
    );

    html_opts!(
        [extension.block_directive],
        concat!(":::\n", "- [a]: /u\n", ":::\n", "\n", "[a]\n"),
        concat!(
            "<div class=\"\">\n",
            "<ul>\n",
            "<li></li>\n",
            "</ul>\n",
            "</div>\n",
            "<p><a href=\"/u\">a</a></p>\n",
        ),
        no_roundtrip,
    );
}

#[test]
fn sourcepos_list_contents_finalized_on_close() {
    assert_ast_match!(
        [extension.block_directive],
        ":::\n"
        "1. foo\n"
        "2. bar\n"
        ":::\n",
        (document (1:1-4:3) [
            (block_directive (1:1-4:3) [
                (list (2:1-3:6) [
                    (item (2:1-2:6) [
                        (paragraph (2:4-2:6) [
                            (text (2:4-2:6) "foo")
                        ])
                    ])
                    (item (3:1-3:6) [
                        (paragraph (3:4-3:6) [
                            (text (3:4-3:6) "bar")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_table_cells_untouched_on_close() {
    assert_ast_match!(
        [extension.block_directive, extension.table],
        ":::\n"
        "| a |\n"
        "| - |\n"
        ":::\n",
        (document (1:1-4:3) [
            (block_directive (1:1-4:3) [
                (table (2:1-3:5) [
                    (table_row (2:1-2:5) [
                        (table_cell (2:2-2:4) [
                            (text (2:3-2:3) "a")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_unclosed_ends_with_its_contents() {
    assert_ast_match!(
        [extension.block_directive],
        ":::\n"
        "\n",
        (document (1:1-2:0) [
            (block_directive (1:1-1:3))
        ])
    );

    assert_ast_match!(
        [extension.block_directive],
        ":::warn\n"
        "\n",
        (document (1:1-2:0) [
            (block_directive (1:1-1:7))
        ])
    );

    assert_ast_match!(
        [extension.block_directive],
        "> :::\n"
        "> \n",
        (document (1:1-2:2) [
            (block_quote (1:1-2:2) [
                (block_directive (1:3-1:5))
            ])
        ])
    );

    assert_ast_match!(
        [extension.block_directive],
        "* :::\n"
        "\n"
        "z\n",
        (document (1:1-3:1) [
            (list (1:1-1:5) [
                (item (1:1-1:5) [
                    (block_directive (1:3-1:5))
                ])
            ])
            (paragraph (3:1-3:1) [
                (text (3:1-3:1) "z")
            ])
        ])
    );
}
