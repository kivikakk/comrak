use super::*;

#[test]
fn fenced_codeblock_closed_and_unclosed_root() {
    let arena = Arena::new();
    let options = Options::default();

    let md_closed = "```\nfn this_has_a_closing_fence() {}\n```\n";
    let root = parse_document(&arena, md_closed, &options);
    let mut found = false;
    for n in root.descendants() {
        if let NodeValue::CodeBlock(ncb) = &n.data().value {
            assert!(ncb.fenced, "expected fenced code block");
            assert!(ncb.closed, "expected closed code block");
            found = true;
            break;
        }
    }
    assert!(found, "expected a code block node");

    let md_unclosed = "```\nfn this_does_not() {}\n";
    let root2 = parse_document(&arena, md_unclosed, &options);
    let mut found2 = false;
    for n in root2.descendants() {
        if let NodeValue::CodeBlock(ncb) = &n.data().value {
            assert!(ncb.fenced, "expected fenced code block");
            assert!(!ncb.closed, "expected unclosed code block");
            found2 = true;
            break;
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
        if let NodeValue::CodeBlock(ncb) = &n.data().value {
            assert!(ncb.fenced, "expected fenced code block in blockquote");
            assert!(ncb.closed, "expected closed code block in blockquote");
            found = true;
            break;
        }
    }
    assert!(found, "expected a code block node");

    let md_unclosed = "> ```\n> paragraph\n";
    let root2 = parse_document(&arena, md_unclosed, &options);
    let mut found2 = false;
    for n in root2.descendants() {
        if let NodeValue::CodeBlock(ncb) = &n.data().value {
            assert!(ncb.fenced, "expected fenced code block in blockquote");
            assert!(!ncb.closed, "expected unclosed code block in blockquote");
            found2 = true;
            break;
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

#[test]
fn closed_list_between_fenced_codeblocks_sourcepos() {
    assert_ast_match!(
        [],
        "```\n"
        "code\n"
        "```\n"
        "- list\n"
        "```\n"
        "code\n"
        "```\n",
        (document (1:1-7:3) [
            (code_block (1:1-3:3) "code\n")
            (list (4:1-4:6) [
                (item (4:1-4:6) [
                    (paragraph (4:3-4:6) [
                        (text (4:3-4:6) "list")
                    ])
                ])
            ])
            (code_block (5:1-7:3) "code\n")
        ])
    );
}

#[test]
fn closed_list_before_fenced_codeblocks_sourcepos() {
    assert_ast_match!(
        [],
        "- list\n"
        "```\n"
        "code\n"
        "```\n",
        (document (1:1-4:3) [
            (list (1:1-1:6) [
                (item (1:1-1:6) [
                    (paragraph (1:3-1:6) [
                        (text (1:3-1:6) "list")
                    ])
                ])
            ])
            (code_block (2:1-4:3) "code\n")
        ])
    );
}

#[test]
fn closed_list_after_fenced_codeblocks_sourcepos() {
    assert_ast_match!(
        [],
        "```\n"
        "code\n"
        "```\n"
        "- list\n",
        (document (1:1-4:6) [
            (code_block (1:1-3:3) "code\n")
            (list (4:1-4:6) [
                (item (4:1-4:6) [
                    (paragraph (4:3-4:6) [
                        (text (4:3-4:6) "list")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn nested_list_between_fenced_codeblocks_sourcepos() {
    assert_ast_match!(
        [],
        "```\n"
        "code\n"
        "```\n"
        "1. list\n"
        "    * nested list\n"
        "```\n"
        "code\n"
        "```\n",
        (document (1:1-8:3) [
            (code_block (1:1-3:3) "code\n")
            (list (4:1-5:17) [
                (item (4:1-5:17) [
                    (paragraph (4:4-4:7) [
                        (text (4:4-4:7) "list")
                    ])
                    (list (5:5-5:17) [
                        (item (5:5-5:17) [
                            (paragraph (5:7-5:17) [
                                (text (5:7-5:17) "nested list")
                            ])
                        ])
                    ])
                ])
            ])
            (code_block (6:1-8:3) "code\n")
        ])
    );
}

#[test]
fn nested_list_before_fenced_codeblock_sourcepos() {
    assert_ast_match!(
        [],
        "1. list\n"
        "    * nested list\n"
        "```\n"
        "code\n"
        "```\n",
        (document (1:1-5:3) [
            (list (1:1-2:17) [
                (item (1:1-2:17) [
                    (paragraph (1:4-1:7) [
                        (text (1:4-1:7) "list")
                    ])
                    (list (2:5-2:17) [
                        (item (2:5-2:17) [
                            (paragraph (2:7-2:17) [
                                (text (2:7-2:17) "nested list")
                            ])
                        ])
                    ])
                ])
            ])
            (code_block (3:1-5:3) "code\n")
        ])
    );
}

#[test]
fn nested_list_after_fenced_codeblock_sourcepos() {
    assert_ast_match!(
        [],
        "```\n"
        "code\n"
        "```\n"
        "1. list\n"
        "    * nested list\n",
        (document (1:1-5:17) [
            (code_block (1:1-3:3) "code\n")
            (list (4:1-5:17) [
                (item (4:1-5:17) [
                    (paragraph (4:4-4:7) [
                        (text (4:4-4:7) "list")
                    ])
                    (list (5:5-5:17) [
                        (item (5:5-5:17) [
                            (paragraph (5:7-5:17) [
                                (text (5:7-5:17) "nested list")
                            ])
                        ])
                    ])
                ])
            ])
        ])
    );
}
