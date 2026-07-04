#![cfg(feature = "attributes")]

use crate::tests::assert_ast_match;

#[test]
fn heading_with_attrs() {
    assert_ast_match!(
        [extension.header_attributes],
        "# Hi! {#greeting}\n",
        (document (1:1-1:17) [
            (heading (1:1-1:17) {#greeting} [
                (text (1:3-1:5) "Hi!")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "## Yeww {.ok yeww=\"\\\"true\\\"\"} ##\n",
        (document (1:1-1:32) [
            (heading (1:1-1:32) {.ok yeww="\"true\""} [
                (text (1:4-1:7) "Yeww")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "## Yeww ## {x=y x=y #a #b}\n",
        (document (1:1-1:26) [
            (heading (1:1-1:26) {#b x="y" x="y"} [
                (text (1:4-1:10) "Yeww ##")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "Yeww {x=y x=y #a #b}\n"
        "====================\n",
        (document (1:1-2:20) [
            (heading (1:1-2:20) {#b x="y" x="y"} [
                (text (1:1-1:4) "Yeww")
            ])
        ])
    );
}

#[test]
fn fenced_code_with_attrs() {
    assert_ast_match!(
        [extension.fenced_code_attributes],
        "```rust {#example}\n"
        "const fn dogs() -> ! { yay }\n"
        "```\n",
        (document (1:1-3:3) [
            (code_block (1:1-3:3) info:"rust" {#example} "const fn dogs() -> ! { yay }\n")
        ])
    );

    // Pandoc does some weird language/first-class interpretative dance,
    // but I'm not yet convinced I want to do that. Let the formatter decide.
    assert_ast_match!(
        [extension.fenced_code_attributes],
        "```{.zig #truism}\n"
        "fn cats() noreturn { yay_too }\n"
        "```\n",
        (document (1:1-3:3) [
            (code_block (1:1-3:3) info:"" {#truism .zig} "fn cats() noreturn { yay_too }\n")
        ])
    );
}

#[test]
fn inline_code_with_attrs() {
    assert_ast_match!(
        [extension.inline_code_attributes],
        "Uhh `totally`{yea=so} hm.",
        (document (1:1-1:25) [
            (paragraph (1:1-1:25) [
                (text (1:1-1:4) "Uhh ")
                // There's probably someone out there depending on sourcepos not being this way.
                // But then again, if they enable attributes, it's on them!
                (code (1:5-1:21) {yea="so"} "totally")
                (text (1:22-1:25) " hm.")
            ])
        ])
    );
}
