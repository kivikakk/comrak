use std::sync::{Arc, Mutex};

use super::*;
use pretty_assertions::assert_eq;

const URL: &str = "https://example.com/some/long/address";

fn doc(uses: usize, definition: &str) -> String {
    format!("{}\n\n[a]: {definition}\n", vec!["[a]"; uses].join(" "))
}

fn links(md: &str) -> usize {
    markdown_to_html(md, &Options::default())
        .matches("<a href=")
        .count()
}

#[test]
fn short_document_is_not_limited_to_its_own_size() {
    html(
        &doc(2, URL),
        &format!("<p><a href=\"{URL}\">a</a> <a href=\"{URL}\">a</a></p>\n"),
    );
}

#[test]
fn short_document_is_limited_to_floor() {
    assert_eq!(links(&doc(3000, URL)), 2702);
}

#[test]
fn long_document_is_limited_to_its_own_size() {
    assert_eq!(links(&doc(50000, URL)), 5406);
}

#[test]
fn front_matter_does_not_count_towards_document_size() {
    let front_matter = format!("---\n{}\n---\n\n", "x".repeat(200_000));
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".to_string());

    let output = markdown_to_html(&format!("{front_matter}{}", doc(50000, URL)), &options);
    assert_eq!(output.matches("<a href=").count(), 5406);

    let output = markdown_to_html(&format!("{front_matter}{}", doc(3000, URL)), &options);
    assert_eq!(output.matches("<a href=").count(), 2702);
}

#[test]
fn exact_fit_of_floor_is_allowed() {
    assert_eq!(links(&doc(2, &"u".repeat(50_000))), 2);
    assert_eq!(links(&doc(2, &"u".repeat(50_001))), 1);
    assert_eq!(links(&doc(11, &"u".repeat(9_091))), 10);
}

#[test]
fn exact_fit_of_long_document_is_allowed() {
    let sized = |total: usize| {
        let definition = format!("\n\n[a]: {}\n", "u".repeat(13));
        let uses = "[a] ".repeat((total - definition.len()) / 4);
        let padding = "x".repeat(total - definition.len() - uses.len());
        format!("{uses}{padding}{definition}")
    };

    assert_eq!(links(&sized(130_000)), 10_000);
    assert_eq!(links(&sized(129_999)), 9_999);
}

#[test]
fn nul_in_url_is_counted_as_replacement_character() {
    assert_eq!(links(&doc(158, &"\0".repeat(632))), 52);
    assert_eq!(links(&doc(2, &"\0".repeat(33_333))), 1);
    assert_eq!(links(&doc(2, &"\0".repeat(33_334))), 0);
}

#[test]
fn nul_in_title_is_counted_as_replacement_character() {
    assert_eq!(links(&doc(158, &format!("u '{}'", "\0".repeat(632)))), 52);
    assert_eq!(links(&doc(2, &format!("u '{}'", "\0".repeat(33_333)))), 1);
    assert_eq!(links(&doc(2, &format!("u '{}'", "\0".repeat(33_334)))), 0);
}

#[test]
fn defined_reference_over_limit_does_not_reach_broken_link_callback() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let cb = {
        let seen = seen.clone();
        move |link_ref: options::BrokenLinkReference| {
            seen.lock().unwrap().push(link_ref.normalized.to_string());
            Some(ResolvedReference {
                url: "/callback".to_string(),
                title: String::new(),
            })
        }
    };
    let mut options = Options::default();
    options.parse.broken_link_callback = Some(Arc::new(cb));

    let url = "u".repeat(40_000);
    let output = markdown_to_html(&format!("[a] [a] [a] [a] [b]\n\n[a]: {url}\n"), &options);

    assert_eq!(
        output,
        format!(
            "<p><a href=\"{url}\">a</a> <a href=\"{url}\">a</a> [a] [a] <a href=\"/callback\">b</a></p>\n"
        )
    );
    assert_eq!(*seen.lock().unwrap(), ["b"]);
}
