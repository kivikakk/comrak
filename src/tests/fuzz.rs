use super::html;

#[test]
fn pointy_brace_open() {
    html("<!-", "<p>&lt;!-</p>\n");
}
