use super::*;

// html_opts! does a roundtrip check unless sourcepos is set.
// These cases don't work roundtrip, because converting to commonmark
// automatically escapes certain characters.
#[test]
fn wikilinks_does_not_unescape_html_entities_in_link_text() {
    html_opts!(
        [extension.wikilinks_title_after_pipe, render.sourcepos],
        concat!("This is [[&lt;script&gt;alert(0)&lt;/script&gt;|a &lt;link]]",),
        concat!("<p data-sourcepos=\"1:1-1:60\">This is <a data-sourcepos=\"1:9-1:60\" href=\"%3Cscript%3Ealert(0)%3C/script%3E\" data-wikilink=\"true\">a &lt;link</a></p>\n"),
    );

    html_opts!(
        [extension.wikilinks_title_before_pipe, render.sourcepos],
        concat!("This is [[a &lt;link|&lt;script&gt;alert(0)&lt;/script&gt;]]",),
        concat!("<p data-sourcepos=\"1:1-1:60\">This is <a data-sourcepos=\"1:9-1:60\" href=\"%3Cscript%3Ealert(0)%3C/script%3E\" data-wikilink=\"true\">a &lt;link</a></p>\n"),
    );
}

#[test]
fn wikilinks_sanitizes_the_href_attribute_case_1() {
    html_opts!(
        [extension.wikilinks_title_after_pipe],
        concat!("[[http:\'\"injected=attribute&gt;&lt;img/src=\"0\"onerror=\"alert(0)\"&gt;https://example.com|a]]",),
        concat!("<p><a href=\"http:&#x27;%22injected=attribute%3E%3Cimg/src=%220%22onerror=%22alert(0)%22%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );

    html_opts!(
        [extension.wikilinks_title_before_pipe],
        concat!("[[a|http:\'\"injected=attribute&gt;&lt;img/src=\"0\"onerror=\"alert(0)\"&gt;https://example.com]]",),
        concat!("<p><a href=\"http:&#x27;%22injected=attribute%3E%3Cimg/src=%220%22onerror=%22alert(0)%22%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );
}

#[test]
fn wikilinks_sanitizes_the_href_attribute_case_2() {
    html_opts!(
        [extension.wikilinks_title_after_pipe],
        concat!("<i>[[\'\"&gt;&lt;svg&gt;&lt;i/class=gl-show-field-errors&gt;&lt;input/title=\"&lt;script&gt;alert(0)&lt;/script&gt;\"/&gt;&lt;/svg&gt;https://example.com|a]]",),
        concat!("<p><!-- raw HTML omitted --><a href=\"&#x27;%22%3E%3Csvg%3E%3Ci/class=gl-show-field-errors%3E%3Cinput/title=%22%3Cscript%3Ealert(0)%3C/script%3E%22/%3E%3C/svg%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );

    html_opts!(
        [extension.wikilinks_title_before_pipe],
        concat!("<i>[[a|\'\"&gt;&lt;svg&gt;&lt;i/class=gl-show-field-errors&gt;&lt;input/title=\"&lt;script&gt;alert(0)&lt;/script&gt;\"/&gt;&lt;/svg&gt;https://example.com]]",),
        concat!("<p><!-- raw HTML omitted --><a href=\"&#x27;%22%3E%3Csvg%3E%3Ci/class=gl-show-field-errors%3E%3Cinput/title=%22%3Cscript%3Ealert(0)%3C/script%3E%22/%3E%3C/svg%3Ehttps://example.com\" data-wikilink=\"true\">a</a></p>\n"),
    );
}

#[test]
fn wikilinks_supercedes_relaxed_autolinks() {
    html_opts!(
        [
            extension.wikilinks_title_after_pipe,
            parse.relaxed_autolinks
        ],
        concat!("[[http://example.com]]",),
        concat!(
            "<p><a href=\"http://example.com\" data-wikilink=\"true\">http://example.com</a></p>\n"
        ),
    );

    html_opts!(
        [
            extension.wikilinks_title_before_pipe,
            parse.relaxed_autolinks
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
        [extension.wikilinks_title_after_pipe],
        "This [[http://example.com|link text]] that\n",
        (document (1:1-1:42) [
            (paragraph (1:1-1:42) [
                (text (1:1-1:5) "This ")
                (link (1:6-1:37) [
                    (text (1:6-1:37) "link text")
                ])
                (text (1:38-1:42) " that")
            ])
        ])
    );

    assert_ast_match!(
        [extension.wikilinks_title_before_pipe],
        "This [[link text|http://example.com]] that\n",
        (document (1:1-1:42) [
            (paragraph (1:1-1:42) [
                (text (1:1-1:5) "This ")
                (link (1:6-1:37) [
                    (text (1:6-1:37) "link text")
                ])
                (text (1:38-1:42) " that")
            ])
        ])
    );
}
