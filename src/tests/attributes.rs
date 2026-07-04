#![cfg(feature = "attributes")]

use crate::tests::assert_ast_match;

#[test]
fn heading_with_attrs() {
    assert_ast_match!(
        [extension.header_attributes],
        "# Hi! {#greeting}\n",
        (document (1:1-1:17) [
            (heading (1:1-1:17) {"#greeting"} [
                (text (1:3-1:5) "Hi!")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "## Yeww {yeww=\"\\\"true\\\"\"} ##\n",
        (document (1:1-1:28) [
            (heading (1:1-1:28) {"yeww=\"true\""} [
                (text (1:4-1:7) "Yeww")
            ])
        ])
    );
}
