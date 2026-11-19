use super::*;

#[test]
fn alerts() {
    html_opts!(
        [extension.alerts],
        concat!("> [!note]\n", "> Pay attention\n",),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">Note</p>\n",
            "<p>Pay attention</p>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn multiline_alerts() {
    html_opts!(
        [extension.alerts, extension.multiline_block_quotes],
        concat!(">>> [!note]\n", "Pay attention\n", ">>>",),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">Note</p>\n",
            "<p>Pay attention</p>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn semantic_alerts() {
    html_opts!(
        [
            extension.alerts = true,
            render.alert_style = options::AlertStyleType::Semantic
        ],
        concat!("> [!note]\n", "> Pay attention\n",),
        concat!(
            "<aside class=\"admonition note\">\n",
            "<p class=\"admonition-title\">Note</p>\n",
            "<p>Pay attention</p>\n",
            "</aside>\n",
        ),
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.alerts],
        "> [!note]\n"
        "> Pay attention\n",
        (document (1:1-2:15) [
            (alert (1:1-2:15) [
                (paragraph (2:3-2:15) [
                    (text (2:3-2:15) "Pay attention")
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_in_list() {
    assert_ast_match!(
        [extension.alerts],
        "- item one\n"
        "\n"
        "  > [!note]\n"
        "  > Pay attention\n",
        (document (1:1-4:17) [
            (list (1:1-4:17) [
                (item (1:1-4:17) [
                    (paragraph (1:3-1:10) [
                        (text (1:3-1:10) "item one")
                    ])
                    (alert (3:3-4:17) [
                        (paragraph (4:5-4:17) [
                            (text (4:5-4:17) "Pay attention")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn multiline_list_contents_finalized_on_close() {
    html_opts!(
        [extension.alerts, extension.multiline_block_quotes],
        concat!(">>> [!note]\n", "- ```rust\n", "  code\n", ">>>\n"),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">Note</p>\n",
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
}

#[test]
fn sourcepos_multiline() {
    assert_ast_match!(
        [extension.alerts, extension.multiline_block_quotes],
        ">>> [!note]\n"
        "Pay attention\n"
        ">>>\n",
        (document (1:1-3:3) [
            (alert (1:1-3:3) [
                (paragraph (2:1-2:13) [
                    (text (2:1-2:13) "Pay attention")
                ])
            ])
        ])
    );
}

#[test]
fn surprise_eof() {
    assert_ast_match!(
        [extension.alerts],
        "> [!note]\n",
        (document (1:1-1:9) [
            (alert (1:1-1:9))
        ])
    );

    assert_ast_match!(
        [extension.alerts],
        "> [!note]",
        (document (1:1-1:9) [
            (alert (1:1-1:9))
        ])
    );
}

#[test]
fn marker_spacing_follows_block_quotes() {
    html_opts!(
        [extension.alerts],
        concat!(
            ">[!note]\n",
            "> a\n",
            "\n",
            ">  [!tip]\n",
            "> b\n",
            "\n",
            ">\t[!important]\n",
            "> c\n",
            "\n",
            ">    [!warning]\n",
            "> d\n",
            "\n",
            ">     [!caution]\n",
            "> e\n",
        ),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">Note</p>\n",
            "<p>a</p>\n",
            "</div>\n",
            "<div class=\"markdown-alert markdown-alert-tip\">\n",
            "<p class=\"markdown-alert-title\">Tip</p>\n",
            "<p>b</p>\n",
            "</div>\n",
            "<div class=\"markdown-alert markdown-alert-important\">\n",
            "<p class=\"markdown-alert-title\">Important</p>\n",
            "<p>c</p>\n",
            "</div>\n",
            "<div class=\"markdown-alert markdown-alert-warning\">\n",
            "<p class=\"markdown-alert-title\">Warning</p>\n",
            "<p>d</p>\n",
            "</div>\n",
            "<blockquote>\n",
            "<pre><code>[!caution]\n",
            "</code></pre>\n",
            "<p>e</p>\n",
            "</blockquote>\n",
        ),
    );
}

#[test]
fn multiline_marker_spacing() {
    html_opts!(
        [extension.alerts, extension.multiline_block_quotes],
        concat!(
            ">>>[!note]\n",
            "a\n",
            ">>>\n",
            "\n",
            ">>>  [!tip] T\n",
            "b\n",
            ">>>\n"
        ),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">Note</p>\n",
            "<p>a</p>\n",
            "</div>\n",
            "<div class=\"markdown-alert markdown-alert-tip\">\n",
            "<p class=\"markdown-alert-title\">T</p>\n",
            "<p>b</p>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn title_escapes_roundtrip() {
    html_opts!(
        [extension.alerts],
        concat!(
            "> [!note] a \\\\* b \\* c &amp;lt; d \\\\[ e & f\n",
            "> x\n"
        ),
        concat!(
            "<div class=\"markdown-alert markdown-alert-note\">\n",
            "<p class=\"markdown-alert-title\">a \\* b * c &amp;lt; d \\[ e &amp; f</p>\n",
            "<p>x</p>\n",
            "</div>\n",
        ),
    );
}
