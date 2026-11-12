use super::*;
use ntest::test_case;

#[test]
fn description_lists_loose() {
    html_opts!(
        [extension.description_lists],
        concat!(
            "Term 1\n",
            "\n",
            ": Definition 1\n",
            "\n",
            "Term 2 with *inline markup*\n",
            "\n",
            ": Definition 2\n"
        ),
        concat!(
            "<dl>\n",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
        ),
    );

    html_opts!(
        [extension.description_lists],
        concat!(
            "* Nested\n",
            "\n",
            "    Term 1\n\n",
            "    :   Definition 1\n\n",
            "    Term 2 with *inline markup*\n\n",
            "    :   Definition 2\n\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>Nested</p>\n",
            "<dl>\n",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn description_lists_tight() {
    html_opts!(
        [extension.description_lists],
        concat!(
            "Term 1\n",
            ": Definition 1\n",
            "\n",
            "Term 2 with *inline markup*\n",
            ": Definition 2\n"
        ),
        concat!(
            "<dl>\n",
            "<dt>Term 1</dt>\n",
            "<dd>Definition 1</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>Definition 2</dd>\n",
            "</dl>\n",
        ),
        no_roundtrip,
    );

    html_opts!(
        [extension.description_lists],
        concat!(
            "* Nested\n",
            "\n",
            "    Term 1\n",
            "    :   Definition 1\n\n",
            "    Term 2 with *inline markup*\n",
            "    :   Definition 2\n\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>Nested</p>\n",
            "<dl>\n",
            "<dt>Term 1</dt>\n",
            "<dd>Definition 1</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>Definition 2</dd>\n",
            "</dl>\n",
            "</li>\n",
            "</ul>\n",
        ),
        no_roundtrip,
    );
}
#[test]
fn description_lists_edge_cases() {
    html_opts!([extension.description_lists], ":", "<p>:</p>\n",);

    html_opts!([extension.description_lists], ": foo", "<p>: foo</p>\n",);

    html_opts!([extension.description_lists], "a\n:", "<p>a\n:</p>\n",);

    html_opts!(
        [extension.description_lists],
        concat!("- foo\n", "- : bar\n", "  - baz\n",),
        concat!(
            "<ul>\n",
            "<li>foo</li>\n",
            "<li>: bar\n",
            "<ul>\n",
            "<li>baz</li>\n",
            "</ul>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.description_lists],
        "ta\n"
        "\n"
        ": da\n"
        "\n"
        "t*b*\n"
        "\n"
        ": d*b*\n"
        "\n"
        "tc\n"
        "\n"
        ": dc\n",
        (document (1:1-11:4) [
        (description_list (1:1-11:4) [
            (description_item (1:1-3:4) [
                (description_term (1:1-1:2) [
                        (paragraph (1:1-1:2) [
                            (text (1:1-1:2) "ta")
                        ])
                    ])
                    (description_details (3:1-3:4) [
                        (paragraph (3:3-3:4) [
                            (text (3:3-3:4) "da")
                        ])
                    ])
                ])
                (description_item (5:1-7:6) [
                    (description_term (5:1-5:4) [
                        (paragraph (5:1-5:4) [
                            (text (5:1-5:1) "t")
                            (emph (5:2-5:4) [
                                (text (5:3-5:3) "b")
                            ])
                        ])
                    ])
                    (description_details (7:1-7:6) [
                        (paragraph (7:3-7:6) [
                            (text (7:3-7:3) "d")
                            (emph (7:4-7:6) [
                                (text (7:5-7:5) "b")
                            ])
                        ])
                    ])
                ])
                (description_item (9:1-11:4) [
                    (description_term (9:1-9:2) [
                        (paragraph (9:1-9:2) [
                            (text (9:1-9:2) "tc")
                        ])
                    ])
                    (description_details (11:1-11:4) [
                        (paragraph (11:3-11:4) [
                            (text (11:3-11:4) "dc")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_term_directly_before_details() {
    assert_ast_match!(
        [extension.description_lists],
        "abc *d*\n"
        ": details\n",
        (document (1:1-2:9) [
            (description_list (1:1-2:9) [
                (description_item (1:1-2:9) [
                    (description_term (1:1-1:7) [
                        (paragraph (1:1-1:7) [
                            (text (1:1-1:4) "abc ")
                            (emph (1:5-1:7) [
                                (text (1:6-1:6) "d")
                            ])
                        ])
                    ])
                    (description_details (2:1-2:9) [
                        (paragraph (2:3-2:9) [
                            (text (2:3-2:9) "details")
                        ])
                    ])
                ])
            ])
        ])
    );

    assert_ast_match!(
        [extension.description_lists],
        "abc\n"
        "def\n"
        ": details\n",
        (document (1:1-3:9) [
            (description_list (1:1-3:9) [
                (description_item (1:1-3:9) [
                    (description_term (1:1-2:3) [
                        (paragraph (1:1-2:3) [
                            (text (1:1-1:3) "abc")
                            (softbreak (1:4-1:4))
                            (text (2:1-2:3) "def")
                        ])
                    ])
                    (description_details (3:1-3:9) [
                        (paragraph (3:3-3:9) [
                            (text (3:3-3:9) "details")
                        ])
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_list_ends_with_its_contents() {
    assert_ast_match!(
        [extension.description_lists],
        "t\n"
        "\n"
        ": d\n"
        "\n"
        "z\n",
        (document (1:1-5:1) [
            (description_list (1:1-3:3) [
                (description_item (1:1-3:3) [
                    (description_term (1:1-1:1) [
                        (paragraph (1:1-1:1) [
                            (text (1:1-1:1) "t")
                        ])
                    ])
                    (description_details (3:1-3:3) [
                        (paragraph (3:3-3:3) [
                            (text (3:3-3:3) "d")
                        ])
                    ])
                ])
            ])
            (paragraph (5:1-5:1) [
                (text (5:1-5:1) "z")
            ])
        ])
    );
}

#[test]
fn commonmark_keeps_details_tightness() {
    let mut options = Options::default();
    options.extension.description_lists = true;
    commonmark(
        "a\n\n: b\n\nc\n: d\n",
        "a\n\n: b\n\nc\n: d\n",
        Some(&options),
    );
    commonmark(
        "- a\n\n  : b\n- d\n",
        "- a\n  \n  : b\n- d\n",
        Some(&options),
    );
    commonmark("- a\n  : b\n- d\n", "- a\n  : b\n- d\n", Some(&options));
}

#[test_case(
    "a\n: b\n\n  c\n",
    "a\n: b\n  \n  c\n",
    "<dl>\n<dt>a</dt>\n<dd>b\nc</dd>\n</dl>\n"
)]
#[test_case(
    "a\n: b\n\n  - x\n  - y\n",
    "a\n: b\n  \n  - x\n  - y\n",
    "<dl>\n<dt>a</dt>\n<dd>b\n<ul>\n<li>x</li>\n<li>y</li>\n</ul>\n</dd>\n</dl>\n"
)]
#[test_case(
    "> a\n> : b\n>\n>   c\n",
    "> a\n> : b\n>   \n>   c\n",
    "<blockquote>\n<dl>\n<dt>a</dt>\n<dd>b\nc</dd>\n</dl>\n</blockquote>\n"
)]
fn commonmark_indents_details_continuation(input: &str, cm: &str, expected: &str) {
    let mut options = Options::default();
    options.extension.description_lists = true;
    commonmark(input, cm, Some(&options));
    html_opts!([extension.description_lists], input, expected);
}

#[test_case("- i\n: d\n", "- i\n  \\: d\n", "<ul>\n<li>i\n: d</li>\n</ul>\n")]
#[test_case(
    "> k\n: d\n",
    "> k\n> \\: d\n",
    "<blockquote>\n<p>k\n: d</p>\n</blockquote>\n"
)]
fn commonmark_escapes_leading_colon(input: &str, cm: &str, expected: &str) {
    let mut options = Options::default();
    options.extension.description_lists = true;
    commonmark(input, cm, Some(&options));
    html_opts!([extension.description_lists], input, expected);
}

#[test_case(
    "a\n  : b\n\n  <div>\n",
    "a\n:  b\n\n  <div>\n",
    "<dl>\n<dt>a</dt>\n<dd>b</dd>\n</dl>\n  <div>\n"
)]
#[test_case(
    "a\n   : b\n\n   <!-- x -->\n",
    "a\n:   b\n\n   <!-- x -->\n",
    "<dl>\n<dt>a</dt>\n<dd>b</dd>\n</dl>\n   <!-- x -->\n"
)]
#[test_case(
    "a\n: b\n\n  <div>\n",
    "a\n: b\n  \n  <div>\n",
    "<dl>\n<dt>a</dt>\n<dd>b\n<div>\n</dd>\n</dl>\n"
)]
fn commonmark_indented_html_block_stays_after_details(input: &str, cm: &str, expected: &str) {
    let mut options = Options::default();
    options.extension.description_lists = true;
    options.render.r#unsafe = true;
    commonmark(input, cm, Some(&options));
    html_opts!(
        [extension.description_lists, render.r#unsafe],
        input,
        expected
    );
}

#[test]
fn commonmark_code_block_after_details_is_fenced() {
    let mut options = Options::default();
    options.extension.description_lists = true;
    commonmark(
        "a\n: b\n\n```\n- c\n```\n",
        "a\n: b\n\n```\n- c\n```\n",
        Some(&options),
    );
    html_opts!(
        [extension.description_lists],
        "a\n: b\n\n```\n- c\n```\n",
        "<dl>\n<dt>a</dt>\n<dd>b</dd>\n</dl>\n<pre><code>- c\n</code></pre>\n"
    );
}
