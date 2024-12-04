use super::*;
use crate::WikiLinksMode;

#[test]
fn wikilinks_does_not_unescape_html_entities_in_link_label() {
    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::UrlFirst)],
        concat!("This is [[&lt;script&gt;alert(0)&lt;/script&gt;|a &lt;link]]",),
        concat!("<p>This is <a href=\"%3Cscript%3Ealert(0)%3C/script%3E\" data-wikilink=\"true\">a &lt;link</a></p>\n"),
        no_roundtrip,
    );

    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        concat!("This is [[a &lt;link|&lt;script&gt;alert(0)&lt;/script&gt;]]",),
        concat!("<p>This is <a href=\"%3Cscript%3Ealert(0)%3C/script%3E\" data-wikilink=\"true\">a &lt;link</a></p>\n"),
        no_roundtrip,
    );
}

#[test]
fn wikilinks_sanitizes_the_href_attribute_case_1() {
    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::UrlFirst)],
        concat!("[[http:\'\"injected=attribute&gt;&lt;img/src=\"0\"onerror=\"alert(0)\"&gt;https://example.com|a]]",),
        concat!("<p><a href=\"http:&#x27;%22injected=attribute%3E%3Cimg/src=%220%22onerror=%22alert(0)%22%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );

    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        concat!("[[a|http:\'\"injected=attribute&gt;&lt;img/src=\"0\"onerror=\"alert(0)\"&gt;https://example.com]]",),
        concat!("<p><a href=\"http:&#x27;%22injected=attribute%3E%3Cimg/src=%220%22onerror=%22alert(0)%22%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );
}

#[test]
fn wikilinks_sanitizes_the_href_attribute_case_2() {
    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::UrlFirst)],
        concat!("<i>[[\'\"&gt;&lt;svg&gt;&lt;i/class=gl-show-field-errors&gt;&lt;input/title=\"&lt;script&gt;alert(0)&lt;/script&gt;\"/&gt;&lt;/svg&gt;https://example.com|a]]",),
        concat!("<p><!-- raw HTML omitted --><a href=\"&#x27;%22%3E%3Csvg%3E%3Ci/class=gl-show-field-errors%3E%3Cinput/title=%22%3Cscript%3Ealert(0)%3C/script%3E%22/%3E%3C/svg%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );

    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        concat!("<i>[[a|\'\"&gt;&lt;svg&gt;&lt;i/class=gl-show-field-errors&gt;&lt;input/title=\"&lt;script&gt;alert(0)&lt;/script&gt;\"/&gt;&lt;/svg&gt;https://example.com]]",),
        concat!("<p><!-- raw HTML omitted --><a href=\"&#x27;%22%3E%3Csvg%3E%3Ci/class=gl-show-field-errors%3E%3Cinput/title=%22%3Cscript%3Ealert(0)%3C/script%3E%22/%3E%3C/svg%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );
}

#[test]
fn wikilinks_title_escape_chars() {
    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst), render.escaped_char_spans = true],
        concat!("[[Name \\[of\\] page|http://example.com]]",),
        concat!("<p><a href=\"http://example.com\" data-wikilink=\"true\">Name <span data-escaped-char>[</span>of<span data-escaped-char>]</span> page</a></p>\n"),
        no_roundtrip,
    );
}

#[test]
fn wikilinks_supercedes_relaxed_autolinks() {
    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::UrlFirst),
            parse.relaxed_autolinks = true
        ],
        concat!("[[http://example.com]]",),
        concat!(
            "<p><a href=\"http://example.com\" data-wikilink=\"true\">http://example.com</a></p>\n"
        ),
    );

    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::TitleFirst),
            parse.relaxed_autolinks = true
        ],
        concat!("[[http://example.com]]",),
        concat!(
            "<p><a href=\"http://example.com\" data-wikilink=\"true\">http://example.com</a></p>\n"
        ),
    );
}

#[test]
fn wikilinks_only_url_in_tables() {
    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::UrlFirst),
            extension.table = true
        ],
        concat!("| header  |\n", "| ------- |\n", "| [[url]] |\n",),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>header</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td><a href=\"url\" data-wikilink=\"true\">url</a></td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );

    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::TitleFirst),
            extension.table = true
        ],
        concat!("| header  |\n", "| ------- |\n", "| [[url]] |\n",),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>header</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td><a href=\"url\" data-wikilink=\"true\">url</a></td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn wikilinks_full_in_tables_not_supported() {
    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::UrlFirst),
            extension.table = true
        ],
        concat!("| header  |\n", "| ------- |\n", "| [[url|link label]] |\n",),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>header</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>[[url</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );

    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::TitleFirst),
            extension.table = true
        ],
        concat!("| header  |\n", "| ------- |\n", "| [[link label|url]] |\n",),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>header</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>[[link label</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn wikilinks_exceeds_label_limit() {
    let long_label = format!("[[{:b<1100}]]", "a");
    let expected = format!("<p>{}</p>\n", long_label);

    html_opts!(
        [extension.wikilinks = Some(WikiLinksMode::UrlFirst)],
        &long_label,
        &expected,
    );
}

#[test]
fn wikilinks_autolinker_ignored() {
    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::UrlFirst),
            extension.autolink = true
        ],
        concat!("[[http://example.com]]",),
        concat!(
            "<p><a href=\"http://example.com\" data-wikilink=\"true\">http://example.com</a></p>\n"
        ),
    );

    html_opts!(
        [
            extension.wikilinks = Some(WikiLinksMode::TitleFirst),
            extension.autolink = true
        ],
        concat!("[[http://example.com]]",),
        concat!(
            "<p><a href=\"http://example.com\" data-wikilink=\"true\">http://example.com</a></p>\n"
        ),
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.wikilinks = Some(WikiLinksMode::UrlFirst)],
        "This [[http://example.com|link label]] that\n",
        (document (1:1-1:43) [
            (paragraph (1:1-1:43) [
                (text (1:1-1:5) "This ")
                (wikilink (1:6-1:38) [
                    (text (1:27-1:36) "link label")
                ])
                (text (1:39-1:43) " that")
            ])
        ])
    );

    assert_ast_match!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        "This [[link label|http://example.com]] that\n",
        (document (1:1-1:43) [
            (paragraph (1:1-1:43) [
                (text (1:1-1:5) "This ")
                (wikilink (1:6-1:38) [
                    (text (1:8-1:17) "link label")
                ])
                (text (1:39-1:43) " that")
            ])
        ])
    );

    assert_ast_match!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        "This [[http://example.com]] that\n",
        (document (1:1-1:32) [
            (paragraph (1:1-1:32) [
                (text (1:1-1:5) "This ")
                (wikilink (1:6-1:27) [
                    (text (1:8-1:25) "http://example.com")
                ])
                (text (1:28-1:32) " that")
            ])
        ])
    );

    assert_ast_match!(
        [extension.wikilinks = Some(WikiLinksMode::TitleFirst)],
        "This [[link\\[label|http://example.com]] that\n",
        (document (1:1-1:44) [
            (paragraph (1:1-1:44) [
                (text (1:1-1:5) "This ")
                (wikilink (1:6-1:39) [
                    (text (1:8-1:11) "link")
                    (text (1:12-1:13) "[")
                    (text (1:14-1:18) "label")
                ])
                (text (1:40-1:44) " that")
            ])
        ])
    );
}
