use super::*;

#[test]
fn commonmark_removes_redundant_strong() {
    let options = ComrakOptions::default();

    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";

    commonmark(input, output, Some(&options));
}
