use crate::cm::{escape_inline, escape_link_destination};
use crate::{entity, markdown_to_html, Options};

/// Assert that the input text escapes to the expected result in inline context,
/// and that the expected result renders to HTML which displays the input text.
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

/// Assert that the URL is escaped as expected, and that the result is rendered
/// into HTML in such a way that preserves the meaning of the input.
///
/// [link destination]: https://spec.commonmark.org/0.31.2/#link-destination
#[test]
fn escape_link_target() {
    let url = "rabbits) <cup\rcakes\n> [hyacinth](";
    let escaped = r#"<rabbits) \<cup%0Dcakes%0A\> [hyacinth](>"#;

    assert_eq!(escaped, escape_link_destination(url));

    let md = format!("[link]({escaped})");
    let mut html = markdown_to_html(&md, &Options::default());
    html = html
        .strip_prefix("<p><a href=\"")
        .expect("html should be one anchor in a paragraph")
        .to_string();
    html = html
        .strip_suffix("\">link</a></p>\n")
        .expect("html should be one anchor in a paragraph")
        .to_string();

    assert_eq!("rabbits)%20%3Ccup%0Dcakes%0A%3E%20%5Bhyacinth%5D(", html);
    assert_eq!(
        url,
        percent_encoding_rfc3986::percent_decode_str(&html)
            .unwrap()
            .decode_utf8()
            .unwrap()
    );
}
