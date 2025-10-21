use crate::{format_commonmark, parse_document, Arena, Options};

use super::*;

#[test]
fn round_trip_one_field() {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let input = "---\nlayout: post\n---\nText\n";
    let root = parse_document(&arena, input, &options);
    let mut buf = String::new();
    format_commonmark(root, &options, &mut buf).unwrap();
    assert_eq!(buf, input);
}

#[test]
fn round_trip_wide_delimiter() {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("\u{04fc}".to_owned());
    let arena = Arena::new();
    let input = "\u{04fc}\nlayout: post\n\u{04fc}\nText\n";
    let root = parse_document(&arena, input, &options);
    let mut buf = String::new();
    format_commonmark(root, &options, &mut buf).unwrap();
    assert_eq!(buf, input);
}

#[test]
fn ast() {
    assert_ast_match!(
        [extension.front_matter_delimiter = Some("q".to_owned())],
        "q\nlayout: post\nq\nText\n",
        (document (1:1-4:4) [
            (frontmatter (1:1-3:1) "q\nlayout: post\nq\n")
            (paragraph (4:1-4:4) [
                (text (4:1-4:4) "Text")
            ])
        ])
    );
}

#[test]
fn ast_blank_line() {
    assert_ast_match!(
        [extension.front_matter_delimiter = Some("---".to_owned())],
        r#"---
a: b
---

hello world
"#,
        (document (1:1-5:11) [
            (frontmatter (1:1-3:3) "---\na: b\n---\n\n")
            (paragraph (5:1-5:11) [
                (text (5:1-5:11) "hello world")
            ])
        ])
    );
}

#[test]
fn ast_carriage_return() {
    assert_ast_match!(
        [extension.front_matter_delimiter = Some("q".to_owned())],
        "q\r\nlayout: post\r\nq\r\nText\r\n",
        (document (1:1-4:4) [
            (frontmatter (1:1-3:1) "q\r\nlayout: post\r\nq\r\n")
            (paragraph (4:1-4:4) [
                (text (4:1-4:4) "Text")
            ])
        ])
    );
}

#[test]
fn ast_wide_delimiter() {
    assert_ast_match!(
        [extension.front_matter_delimiter = Some("\u{04fc}".to_owned())],
        "\u{04fc}\nlayout: post\n\u{04fc}\nText\n",
        (document (1:1-4:4) [
            (frontmatter (1:1-3:2) "\u{04fc}\nlayout: post\n\u{04fc}\n")
            (paragraph (4:1-4:4) [
                (text (4:1-4:4) "Text")
            ])
        ])
    );
}

#[test]
fn trailing_space_open() {
    let input = "--- \nlayout: post\n---\nText\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_none(), "no FrontMatter expected");
}

#[test]
fn leading_space_open() {
    let input = " ---\nlayout: post\n---\nText\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_none(), "no FrontMatter expected");
}

#[test]
fn leading_space_close() {
    let input = "---\nlayout: post\n ---\nText\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_none(), "no FrontMatter expected");
}

#[test]
fn trailing_space_close() {
    let input = "---\nlayout: post\n--- \nText\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_none(), "no FrontMatter expected");
}

#[test]
fn second_line() {
    let input = "\n---\nlayout: post\n ---\nText\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_none(), "no FrontMatter expected");
}

#[test]
fn fm_only_with_trailing_newline() {
    let input = "---\nfoo: bar\n---\n";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_some(), "front matter expected");
}

#[test]
fn fm_only_without_trailing_newline() {
    let input = "---\nfoo: bar\n---";

    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);

    let found = root
        .descendants()
        .find(|n| node_matches!(n, NodeValue::FrontMatter(..)));

    assert!(found.is_some(), "front matter expected");
}
