use super::*;

#[test]
fn fenced_codeblock_closed_and_unclosed_root() {
    let arena = Arena::new();
    let options = Options::default();

    let md_closed = "```\nfn this_has_a_closing_fence() {}\n```\n";
    let root = parse_document(&arena, md_closed, &options);
    let mut found = false;
    for n in root.descendants() {
        match &n.data().value {
            NodeValue::CodeBlock(ncb) => {
                assert!(ncb.fenced, "expected fenced code block");
                assert!(ncb.closed, "expected closed code block");
                found = true;
                break;
            }
            _ => {}
        }
    }
    assert!(found, "expected a code block node");

    let md_unclosed = "```\nfn this_does_not() {}\n";
    let root2 = parse_document(&arena, md_unclosed, &options);
    let mut found2 = false;
    for n in root2.descendants() {
        match &n.data().value {
            NodeValue::CodeBlock(ncb) => {
                assert!(ncb.fenced, "expected fenced code block");
                assert!(!ncb.closed, "expected unclosed code block");
                found2 = true;
                break;
            }
            _ => {}
        }
    }
    assert!(found2, "expected a code block node");
}

#[test]
fn fenced_codeblock_closed_and_unclosed_in_blockquote() {
    let arena = Arena::new();
    let options = Options::default();

    let md_closed = "> ```\n> fn this_has_a_closing_fence() {}\n> ```\n";
    let root = parse_document(&arena, md_closed, &options);
    let mut found = false;
    for n in root.descendants() {
        match &n.data().value {
            NodeValue::CodeBlock(ncb) => {
                assert!(ncb.fenced, "expected fenced code block in blockquote");
                assert!(ncb.closed, "expected closed code block in blockquote");
                found = true;
                break;
            }
            _ => {}
        }
    }
    assert!(found, "expected a code block node");

    let md_unclosed = "> ```\n> paragraph\n";
    let root2 = parse_document(&arena, md_unclosed, &options);
    let mut found2 = false;
    for n in root2.descendants() {
        match &n.data().value {
            NodeValue::CodeBlock(ncb) => {
                assert!(ncb.fenced, "expected fenced code block in blockquote");
                assert!(!ncb.closed, "expected unclosed code block in blockquote");
                found2 = true;
                break;
            }
            _ => {}
        }
    }
    assert!(found2, "expected a code block node");
}

#[test]
fn fenced_codeblock_closed_sourcepos() {
    assert_ast_match!(
        [],
        "```\ncode\n```",
        (document (1:1-3:3) [
            (code_block (1:1-3:3) "code\n")
        ])
    );
}

#[test]
fn fenced_codeblock_unclosed_sourcepos() {
    assert_ast_match!(
        [],
        "```\ncode\n",
        (document (1:1-2:4) [
            (code_block (1:1-2:4) "code\n")
        ])
    );

    assert_ast_match!(
        [],
        "```\n> code\n",
        (document (1:1-2:6) [
            (code_block (1:1-2:6) "> code\n")
        ])
    );

    assert_ast_match!(
        [],
        "> ```\nparagraph\n",
        (document (1:1-2:9) [
            (block_quote (1:1-1:5) [
                (code_block (1:3-1:5) "")
            ])
            (paragraph (2:1-2:9) [
                (text (2:1-2:9) "paragraph")
            ])
        ])
    );

    assert_ast_match!(
        [],
        "> ```\n> code\n",
        (document (1:1-2:6) [
            (block_quote (1:1-2:6) [
                (code_block (1:3-2:6) "code\n")
            ])
        ])
    );

    assert_ast_match!(
        [],
        "* ```\nparagraph\n",
        (document (1:1-2:9) [
            (list (1:1-1:5) [
                (item (1:1-1:5) [
                    (code_block (1:3-1:5) "")
                ])
            ])
            (paragraph (2:1-2:9) [
                (text (2:1-2:9) "paragraph")
            ])
        ])
    );

    assert_ast_match!(
        [],
        "```\n* code\n",
        (document (1:1-2:6) [
            (code_block (1:1-2:6) "* code\n")
        ])
    );

    assert_ast_match!(
        [],
        "* ```\n* paragraph\n",
        (document (1:1-2:11) [
            (list (1:1-2:11) [
                (item (1:1-1:5) [
                    (code_block (1:3-1:5) "")
                ])
                (item (2:1-2:11) [
                    (paragraph (2:3-2:11) [
                        (text (2:3-2:11) "paragraph")
                    ])
                ])
            ])
        ])
    );
}
