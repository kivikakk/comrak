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

    // "Bart, can we stop for newlines?"
    // "I don't see why not."
    assert_ast_match!(
        [extension.inline_code_attributes],
        "`T1 Faker`{role=mid\n"
        "  kda=\"0/0/0\"\n"
        "  cs=120}\n",
        (document (1:1-3:9) [
            (paragraph (1:1-3:9) [
                (code (1:1-3:9) {role="mid" kda="0/0/0" cs="120"} "T1 Faker")
            ])
        ])
    );
}

#[test]
fn links_and_images_with_attrs() {
    assert_ast_match!(
        [extension.link_attributes],
        "Incredible [Jhin](game){lies=true}.\n",
        (document (1:1-1:35) [
            (paragraph (1:1-1:35) [
                (text (1:1-1:11) "Incredible ")
                (link (1:12-1:34) {lies="true"} "game" [
                    (text (1:13-1:16) "Jhin")
                ])
                (text (1:35-1:35) ".")
            ])
        ])
    );

    assert_ast_match!(
        [extension.link_attributes],
        "Incredible ![Jhin](game){lies=true}.\n",
        (document (1:1-1:36) [
            (paragraph (1:1-1:36) [
                (text (1:1-1:11) "Incredible ")
                (image (1:12-1:35) {lies="true"} "game" [
                    (text (1:14-1:17) "Jhin")
                ])
                (text (1:36-1:36) ".")
            ])
        ])
    );
}

#[test]
fn multiline_link_attrs_sourcepos() {
    assert_ast_match!(
        [extension.link_attributes, extension.inline_code_attributes],
        "a\n"
        "b ![](x){w=1\n"
        "   h=2} c `d` e\n"
        "f *g*\n",
        (document (1:1-4:5) [
            (paragraph (1:1-4:5) [
                (text (1:1-1:1) "a")
                (softbreak (1:2-1:2))
                (text (2:1-2:2) "b ")
                (image (2:3-3:7) {w="1" h="2"} "x")
                (text (3:8-3:10) " c ")
                (code (3:11-3:13) "d")
                (text (3:14-3:15) " e")
                (softbreak (3:16-3:16))
                (text (4:1-4:2) "f ")
                (emph (4:3-4:5) [
                    (text (4:4-4:4) "g")
                ])
            ])
        ])
    );
}

#[test]
fn multiline_inline_code_attrs_sourcepos() {
    assert_ast_match!(
        [extension.inline_code_attributes],
        "a\n"
        "`x`{w=1\n"
        "  h=2} b\n",
        (document (1:1-3:8) [
            (paragraph (1:1-3:8) [
                (text (1:1-1:1) "a")
                (softbreak (1:2-1:2))
                (code (2:1-3:6) {w="1" h="2"} "x")
                (text (3:7-3:8) " b")
            ])
        ])
    );
}
