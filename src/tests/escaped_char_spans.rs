use super::*;
use ntest::test_case;

// html_opts! does a roundtrip check unless sourcepos is set.
// These cases don't work roundtrip, because converting to commonmark
// automatically escapes certain characters.
#[test_case("\\@user", "<p data-sourcepos=\"1:1-1:6\"><span data-escaped-char data-sourcepos=\"1:1-1:2\">@</span>user</p>\n")]
#[test_case("This\\@that", "<p data-sourcepos=\"1:1-1:10\">This<span data-escaped-char data-sourcepos=\"1:5-1:6\">@</span>that</p>\n")]
fn escaped_char_spans(markdown: &str, html: &str) {
    html_opts!(
        [render.escaped_char_spans, render.sourcepos],
        markdown,
        html
    );
}

#[test_case("\\@user", "<p>@user</p>\n")]
#[test_case("This\\@that", "<p>This@that</p>\n")]
fn disabled_escaped_char_spans(markdown: &str, expected: &str) {
    html(markdown, expected);
}
