use super::*;
use ntest::test_case;

#[test_case("\\@user", "<p><span data-escaped-char>@</span>user</p>\n")]
#[test_case("This\\@that", "<p>This<span data-escaped-char>@</span>that</p>\n")]
fn escaped_char_spans(markdown: &str, html: &str) {
    html_opts!([render.escaped_char_spans], markdown, html, no_roundtrip);
}

#[test_case("\\@user", "<p>@user</p>\n")]
#[test_case("This\\@that", "<p>This@that</p>\n")]
fn disabled_escaped_char_spans(markdown: &str, expected: &str) {
    html(markdown, expected);
}
