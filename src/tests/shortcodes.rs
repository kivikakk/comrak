#![cfg(feature = "shortcodes")]

use super::*;

#[test]
fn emojis() {
    // Test match
    html_opts!(
        [extension.shortcodes],
        concat!("Hello, happy days! :smile:\n"),
        concat!("<p>Hello, happy days! 😄</p>\n"),
    );

    // Test match
    html_opts!(
        [extension.shortcodes],
        concat!(":smile::smile::smile::smile:\n"),
        concat!("<p>😄😄😄😄</p>\n"),
    );

    // Test match
    html_opts!(
        [extension.shortcodes],
        concat!(":smile:::smile:::smile:::smile:\n"),
        concat!("<p>😄:😄:😄:😄</p>\n"),
    );

    // Test no match
    html_opts!(
        [extension.shortcodes],
        concat!("Hello, happy days! :diego:\n"),
        concat!("<p>Hello, happy days! :diego:</p>\n"),
    );
}
