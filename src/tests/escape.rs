use crate::{cm::escape_inline, entity, markdown_to_html, Options};

/// Assert that the input escapes to the expected result, and that the expected
/// result renders to HTML which displays the input text.
#[track_caller]
fn assert_escape_inline(input: &str, expected: &str) {
    let actual = escape_inline(input);
    assert_eq!(expected, actual);
    let mut html = markdown_to_html(expected, &Options::default());
    html = html
        .strip_prefix("<p>")
        .expect("html should be one paragraph")
        .to_string();
    html = html
        .strip_suffix("</p>\n")
        .expect("html should be one paragraph")
        .to_string();
    assert_eq!(
        input,
        std::str::from_utf8(&entity::unescape_html(html.as_bytes())).unwrap()
    );
}

#[test]
fn escape_inline_baseline() {
    assert_escape_inline("abcdefg", "abcdefg");
    assert_escape_inline("*hello*", r#"\*hello\*"#);
    assert_escape_inline(
        "[A link](https://link.com)",
        r#"\[A link\]\(https://link\.com\)"#,
    );
    assert_escape_inline(
        r#"some <"complicated"> & '/problematic\' input"#,
        r#"some \<\"complicated\"\> \& '/problematic\\' input"#,
    );
}
