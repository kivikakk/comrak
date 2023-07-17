use super::*;

#[test]
fn commonmark_removes_redundant_strong() {
    let options = ComrakOptions::default();

    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";

    commonmark(input, output, Some(&options));
}

#[test]
fn commonmark_wrap_preserve_link() {
    let options = ComrakOptions {
        render: ComrakRenderOptions {
            width: 20,
            ..ComrakRenderOptions::default()
        },
        ..Default::default()
    };

    let input = "This is [a link containing spaces that should not be wrapped](https://example.com) and then the text continues after.";
    let output = "This is\n[a link containing spaces that should not be wrapped](https://example.com)\nand then the text\ncontinues after.\n";
    commonmark(input, output, Some(&options));
}