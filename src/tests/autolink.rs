use super::*;

#[test]
fn autolink_www() {
    html_opts!(
        [extension.autolink],
        concat!("www.autolink.com\n"),
        concat!("<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n"),
    );
}

#[test]
fn autolink_email() {
    html_opts!(
        [extension.autolink],
        concat!("john@smith.com\n"),
        concat!("<p><a href=\"mailto:john@smith.com\">john@smith.com</a></p>\n"),
    );
}

#[test]
fn autolink_scheme() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\n", "rdar://localhost.com/blah"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.com/search</a>\n",
            "rdar://localhost.com/blah</p>\n"
        ),
    );
}

#[test]
fn autolink_scheme_multiline() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\nhttps://www.google.com/maps"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a>\n<a href=\"https://www.google.com/maps\">\
             https://www.google.com/maps</a></p>\n"
        ),
    );
}

#[test]
fn autolink_no_link_bad() {
    html_opts!(
        [extension.autolink],
        concat!("@a.b.c@. x\n", "\n", "n@. x\n"),
        concat!("<p>@a.b.c@. x</p>\n", "<p>n@. x</p>\n"),
    );
}

#[test]
fn autolink_parentheses_balanced() {
    let examples = [
        [
            "http://www.pokemon.com/Pikachu_(Electric)",
            "<p><a href=\"http://www.pokemon.com/Pikachu_(Electric)\">http://www.pokemon.com/Pikachu_(Electric)</a></p>\n",
        ],
        [
            "http://www.pokemon.com/Pikachu_((Electric)",
            "<p><a href=\"http://www.pokemon.com/Pikachu_((Electric)\">http://www.pokemon.com/Pikachu_((Electric)</a></p>\n",
        ],
        [
            "http://www.pokemon.com/Pikachu_(Electric))",
            "<p><a href=\"http://www.pokemon.com/Pikachu_(Electric)\">http://www.pokemon.com/Pikachu_(Electric)</a>)</p>\n",
        ],
        [
            "http://www.pokemon.com/Pikachu_((Electric))",
            "<p><a href=\"http://www.pokemon.com/Pikachu_((Electric))\">http://www.pokemon.com/Pikachu_((Electric))</a></p>\n",
        ],
    ];

    for example in examples {
        html_opts!([extension.autolink], example[0], example[1]);
    }

    for example in examples {
        html_opts!(
            [extension.autolink, parse.relaxed_autolinks],
            example[0],
            example[1]
        );
    }
}

#[test]
fn autolink_brackets_unbalanced() {
    html_opts!(
        [extension.autolink],
        concat!("http://example.com/[abc]]...\n"),
        concat!(
            "<p><a href=\"http://example.com/%5Babc%5D%5D\">http://example.com/[abc]]</a>...</p>\n"
        ),
    );
}

#[test]
fn autolink_ignore_links_in_brackets() {
    let examples = [
        ["[https://foo.com]", "<p>[https://foo.com]</p>\n"],
        ["[[https://foo.com]]", "<p>[[https://foo.com]]</p>\n"],
        [
            "[[Foo|https://foo.com]]",
            "<p>[[Foo|https://foo.com]]</p>\n",
        ],
        [
            "[<https://foo.com>]",
            "<p>[<a href=\"https://foo.com\">https://foo.com</a>]</p>\n",
        ],
    ];

    for example in examples {
        html_opts!([extension.autolink], example[0], example[1], no_roundtrip);
    }
}

#[test]
fn autolink_relaxed_links_in_brackets() {
    let examples = [
        [
            "[https://foo.com]",
            "<p>[<a href=\"https://foo.com\">https://foo.com</a>]</p>\n",
        ],
        [
            "[[https://foo.com]]",
            "<p>[[<a href=\"https://foo.com\">https://foo.com</a>]]</p>\n",
        ],
        [
            "[[Foo|https://foo.com]]",
            "<p>[[Foo|<a href=\"https://foo.com\">https://foo.com</a>]]</p>\n",
        ],
        [
            "[<https://foo.com>]",
            "<p>[<a href=\"https://foo.com\">https://foo.com</a>]</p>\n",
        ],
        [
            "[http://foo.com/](url)",
            "<p><a href=\"url\">http://foo.com/</a></p>\n",
        ],
        ["[http://foo.com/](url", "<p>[http://foo.com/](url</p>\n"],
        [
            "[www.foo.com/](url)",
            "<p><a href=\"url\">www.foo.com/</a></p>\n",
        ],
        [
            "{https://foo.com}",
            "<p>{<a href=\"https://foo.com\">https://foo.com</a>}</p>\n",
        ],
        [
            "[this http://and.com that](url)",
            "<p><a href=\"url\">this http://and.com that</a></p>\n",
        ],
        [
            "[this <http://and.com> that](url)",
            "<p><a href=\"url\">this http://and.com that</a></p>\n",
        ],
        [
            "{this http://and.com that}(url)",
            "<p>{this <a href=\"http://and.com\">http://and.com</a> that}(url)</p>\n",
        ],
        [
            "[http://foo.com](url)\n[http://bar.com]\n\n[http://bar.com]: http://bar.com/extra",
            "<p><a href=\"url\">http://foo.com</a>\n<a href=\"http://bar.com/extra\">http://bar.com</a></p>\n",
        ],
    ];

    for example in examples {
        html_opts!(
            [extension.autolink, parse.relaxed_autolinks],
            example[0],
            example[1]
        );
    }
}

#[test]
fn autolink_relaxed_links_brackets_balanced() {
    html_opts!(
        [extension.autolink, parse.relaxed_autolinks],
        concat!("http://example.com/[abc]]...\n"),
        concat!(
            "<p><a href=\"http://example.com/%5Babc%5D\">http://example.com/[abc]</a>]...</p>\n"
        ),
    );
}

#[test]
fn autolink_relaxed_links_curly_braces_balanced() {
    html_opts!(
        [extension.autolink, parse.relaxed_autolinks],
        concat!("http://example.com/{abc}}...\n"),
        concat!(
            "<p><a href=\"http://example.com/%7Babc%7D\">http://example.com/{abc}</a>}...</p>\n"
        ),
    );
}

#[test]
fn autolink_relaxed_links_curly_parentheses_balanced() {
    html_opts!(
        [extension.autolink, parse.relaxed_autolinks],
        concat!("http://example.com/(abc))...\n"),
        concat!("<p><a href=\"http://example.com/(abc)\">http://example.com/(abc)</a>)...</p>\n"),
    );
}

#[test]
fn autolink_relaxed_links_schemes() {
    let examples = [
        [
            "https://foo.com",
            "<p><a href=\"https://foo.com\">https://foo.com</a></p>\n",
        ],
        [
            "smb:///Volumes/shared/foo.pdf",
            "<p><a href=\"smb:///Volumes/shared/foo.pdf\">smb:///Volumes/shared/foo.pdf</a></p>\n",
        ],
        [
            "irc://irc.freenode.net/git",
            "<p><a href=\"irc://irc.freenode.net/git\">irc://irc.freenode.net/git</a></p>\n",
        ],
        [
            "rdar://localhost.com/blah",
            "<p><a href=\"rdar://localhost.com/blah\">rdar://localhost.com/blah</a></p>\n",
        ],
    ];

    for example in examples {
        html_opts!(
            [extension.autolink, parse.relaxed_autolinks],
            example[0],
            example[1]
        );
    }
}

#[test]
fn sourcepos_correctly_restores_context() {
    assert_ast_match!(
        [],
        "ab _cde_ f@g.ee h*ijklm* n",
        (document (1:1-1:26) [
            (paragraph (1:1-1:26) [
                (text (1:1-1:3) "ab ")
                (emph (1:4-1:8) [
                    (text (1:5-1:7) "cde")
                ])
                (text (1:9-1:17) " f@g.ee h")
                (emph (1:18-1:24) [
                    (text (1:19-1:23) "ijklm")
                ])
                (text (1:25-1:26) " n")
            ])
        ])
    );

    assert_ast_match!(
        [extension.autolink],
        "ab _cde_ f@g.ee h*ijklm* n",
        (document (1:1-1:26) [
            (paragraph (1:1-1:26) [
                (text (1:1-1:3) "ab ")
                (emph (1:4-1:8) [
                    (text (1:5-1:7) "cde")
                ])
                (text (1:9-1:9) " ")
                (link (1:10-1:15) "mailto:f@g.ee" [
                    (text (1:10-1:15) "f@g.ee")
                ])
                (text (1:16-1:17) " h")
                (emph (1:18-1:24) [
                    (text (1:19-1:23) "ijklm")
                ])
                (text (1:25-1:26) " n")
            ])
        ])
    );
}

#[test]
fn autolink_cmark_edge_382() {
    html_opts!(
        [extension.autolink],
        "See &lt;&lt;&lt;http://example.com/&gt;&gt;&gt;",
        "<p>See &lt;&lt;&lt;<a href=\"http://example.com/\">http://example.com/</a>&gt;&gt;&gt;</p>\n",
    );
}

#[test]
fn autolink_cmark_edge_388() {
    html_opts!(
        [extension.autolink],
        "http://example.com/src/_mocks_/vscode.js",
        "<p><a href=\"http://example.com/src/_mocks_/vscode.js\">http://example.com/src/_mocks_/vscode.js</a></p>\n",
    );
}

#[test]
fn autolink_cmark_edge_423() {
    html_opts!(
        [extension.autolink, extension.strikethrough],
        concat!(
            "Here's an autolink: ",
            "https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022",
            " and another one ",
            "https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022",
            ".",
        ),
        concat!(
            "<p>Here's an autolink: ",
            r#"<a href="https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022">"#,
            "https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022",
            "</a> and another one ",
            r#"<a href="https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022">"#,
            "https://www.unicode.org/review/pri453/feedback.html#:~:text=Fri%20Jun%2024%2009:56:01%20CDT%202022",
            "</a>.</p>\n",
        ),
    );
}

#[test]
fn autolink_cmark_edge_58() {
    html_opts!(
        [extension.autolink, extension.superscript],
        "https://www.wolframalpha.com/input/?i=x^2+(y-(x^2)^(1/3))^2=1",
        concat!(
            "<p>",
            r#"<a href="https://www.wolframalpha.com/input/?i=x%5E2+(y-(x%5E2)%5E(1/3))%5E2=1">"#,
            "https://www.wolframalpha.com/input/?i=x^2+(y-(x^2)^(1/3))^2=1",
            "</a></p>\n",
        ),
    );
}

#[test]
fn autolink_failing_spec_image() {
    html_opts!(
        [extension.autolink],
        "![http://inline.com/image](http://inline.com/image)",
        "<p><img src=\"http://inline.com/image\" alt=\"http://inline.com/image\" /></p>\n",
    );
}

#[test]
fn autolink_failing_spec_underscores() {
    html_opts!(
        [extension.autolink],
        "Underscores not allowed in host name www.xxx.yyy._zzz",
        "<p>Underscores not allowed in host name www.xxx.yyy._zzz</p>\n",
    );
}

#[test]
fn autolink_fuzz_leading_colon() {
    html_opts!(
        [extension.autolink, parse.relaxed_autolinks],
        "://-",
        "<p><a href=\"://-\">://-</a></p>\n",
        no_roundtrip,
    );
}

#[test]
fn autolink_fuzz_we() {
    html_opts!(
        [extension.autolink, parse.relaxed_autolinks],
        "we://w",
        "<p><a href=\"we://w\">we://w</a></p>\n",
        no_roundtrip,
    );
}

#[test]
fn autolink_sourcepos() {
    assert_ast_match!(
        [extension.autolink],
        "a  www.com  x\n"
        "\n"
        "b  https://www.com  y\n"
        "\n"
        "c  foo@www.com  z\n"
        ,
        (document (1:1-5:17) [
            (paragraph (1:1-1:13) [
                (text (1:1-1:3) "a  ")
                (link (1:4-1:10) "http://www.com" [
                    (text (1:4-1:10) "www.com")
                ])
                (text (1:11-1:13) "  x")
            ])
            (paragraph (3:1-3:21) [
                (text (3:1-3:3) "b  ")
                (link (3:4-3:18) "https://www.com" [
                    (text (3:4-3:18) "https://www.com")
                ])
                (text (3:19-3:21) "  y")
            ])
            (paragraph (5:1-5:17) [
                (text (5:1-5:3) "c  ")
                (link (5:4-5:14) "mailto:foo@www.com" [
                    (text (5:4-5:14) "foo@www.com")
                ])
                (text (5:15-5:17) "  z")
            ])
        ])
    );
}

#[test]
fn autolink_consecutive_email() {
    assert_ast_match!(
        [extension.autolink],
        "scyther@pokemon.com/beedrill@pokemon.com",
        (document (1:1-1:40) [
            (paragraph (1:1-1:40) [
                (link (1:1-1:19) "mailto:scyther@pokemon.com" [
                    (text (1:1-1:19) "scyther@pokemon.com")
                ])
                (text (1:20-1:20) "/")
                (link (1:21-1:40) "mailto:beedrill@pokemon.com" [
                    (text (1:21-1:40) "beedrill@pokemon.com")
                ])
            ])
        ])
    );
}
