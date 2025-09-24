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

/// Assert a suitable method to escape [link destination]s.
///
/// [link destination]: https://spec.commonmark.org/0.31.2/#link-destination
#[test]
fn escape_link_target() {
    let url = "rabbits) <cup\rcakes\n> [hyacinth](";
    let escaped = format!(
        "<{}>",
        url.replace("<", "\\<")
            .replace(">", "\\>")
            .replace("\n", "%0A")
            .replace("\r", "%0D")
    );

    let md = format!("A [link]({escaped}).");
    let html = markdown_to_html(&md, &Options::default());

    assert_eq!(
        "<p>A <a href=\"rabbits)%20%3Ccup%0Dcakes%0A%3E%20%5Bhyacinth%5D(\">link</a>.</p>\n",
        html
    );
}
