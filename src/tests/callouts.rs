use super::*;
use crate::nodes::{CalloutFold, NodeCallout};

fn callout_assert(
    md: &str,
    expected_type: &str,
    expected_metadata: Option<&str>,
    expected_fold: CalloutFold,
) {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.ofm_callouts = true;
    let root = parse_document(&arena, md, &options);
    let callout = root.children().next().expect("expected a callout child");
    match &callout.data().value {
        NodeValue::Callout(nc) => {
            std::assert_eq!(nc.callout_type, expected_type, "input: {md}");
            std::assert_eq!(nc.metadata.as_deref(), expected_metadata, "input: {md}");
            std::assert_eq!(nc.fold, expected_fold, "input: {md}");
        }
        other => panic!("expected Callout, got {:?} (input: {md})", other),
    }
}

fn cm_roundtrip(md: &str, options: &Options, check: impl FnOnce(crate::nodes::Node<'_>)) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut out = String::new();
    cm::format_document(root, options, &mut out).unwrap();
    let root2 = parse_document(&arena, &out, options);
    check(root2);
    out
}

// === Configuration & priority ===

#[test]
fn disabled_by_default() {
    html(
        "> [!note]\n> Body\n",
        concat!(
            "<blockquote>\n",
            "<p>[!note]\n",
            "Body</p>\n",
            "</blockquote>\n",
        ),
    );
}

#[test]
fn both_enabled_important() {
    html_opts!(
        [extension.alerts, extension.ofm_callouts],
        "> [!important]\n> Body\n",
        concat!(
            "<div class=\"callout\" data-callout=\"important\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">Important</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Body</p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

// === Header recognition (table-driven) ===

#[test]
fn valid_headers() {
    let cases = [
        ("> [!note]\n> Body\n", "note", None, CalloutFold::None),
        ("> [! ]\n> Body\n", "", None, CalloutFold::None),
        ("> [!|]\n> Body\n", "", Some(""), CalloutFold::None),
        (
            "> [!custom|my meta]\n> Body\n",
            "custom",
            Some("my meta"),
            CalloutFold::None,
        ),
        ("> [!note|]\n> Body\n", "note", Some(""), CalloutFold::None),
        ("> [!note]+\n> Body\n", "note", None, CalloutFold::Expanded),
        ("> [!note]-\n> Body\n", "note", None, CalloutFold::Collapsed),
        ("> [!My Type]\n> Body\n", "my-type", None, CalloutFold::None),
    ];
    for (md, ty, meta, fold) in cases {
        callout_assert(md, ty, meta, fold);
    }
}

#[test]
fn invalid_headers() {
    let cases = [
        (
            ">  [!note]\n> Body\n",
            concat!(
                "<blockquote>\n",
                "<p>[!note]\n",
                "Body</p>\n",
                "</blockquote>\n"
            ),
        ),
        (
            "> [!]\n> Body\n",
            concat!(
                "<blockquote>\n",
                "<p>[!]\n",
                "Body</p>\n",
                "</blockquote>\n"
            ),
        ),
        (
            "> [!note\n> Body\n",
            concat!(
                "<blockquote>\n",
                "<p>[!note\n",
                "Body</p>\n",
                "</blockquote>\n"
            ),
        ),
        (
            "> [!note]++\n> Body\n",
            concat!(
                "<blockquote>\n",
                "<p>[!note]++\n",
                "Body</p>\n",
                "</blockquote>\n"
            ),
        ),
    ];
    for (md, expected) in cases {
        html_opts_i(md, expected, true, |opts| {
            opts.extension.ofm_callouts = true
        });
    }
}

// === Title ===

#[test]
fn title_richtext() {
    html_opts!(
        [extension.ofm_callouts],
        "> [!warning] **Bold** and *italic*\n> Body\n",
        concat!(
            "<div class=\"callout\" data-callout=\"warning\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">\n",
            "<strong>Bold</strong> and <em>italic</em>\n",
            "</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Body</p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn title_body_emphasis_boundary() {
    html_opts!(
        [extension.ofm_callouts],
        "> [!note] *Title\n> *body*\n",
        concat!(
            "<div class=\"callout\" data-callout=\"note\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">\n",
            "*Title\n",
            "</div>\n",
            "<div class=\"callout-content\">\n",
            "<p><em>body</em></p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn hidden_comment_title() {
    html_opts!(
        [extension.ofm_callouts, extension.ofm_comments],
        "> [!note] %%hidden%%\n> Body\n",
        concat!(
            "<div class=\"callout\" data-callout=\"note\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">\n",
            "</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Body</p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn no_title_default() {
    html_opts!(
        [extension.ofm_callouts],
        "> [!note]\n> Body\n",
        concat!(
            "<div class=\"callout\" data-callout=\"note\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">Note</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Body</p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn default_title_hyphen_to_space() {
    let nc = NodeCallout {
        callout_type: "custom-type".into(),
        metadata: None,
        fold: CalloutFold::None,
    };
    std::assert_eq!(nc.default_title(), "Custom type");
}

// === Container ===

#[test]
fn nested_in_blockquote() {
    assert_ast_match!(
        [extension.ofm_callouts],
        "> > [!note]\n> > Body\n",
        (document (1:1-2:8) [
            (block_quote (1:1-2:8) [
                (callout (1:3-2:8) [
                    (paragraph (2:5-2:8) [
                        (text (2:5-2:8) "Body")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn lazy_continuation() {
    html_opts!(
        [extension.ofm_callouts],
        "> [!note] Title\nLazy body\n",
        concat!(
            "<div class=\"callout\" data-callout=\"note\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">\n",
            "Title\n",
            "</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Lazy body</p>\n",
            "</div>\n",
            "</div>\n",
        ),
    );
}

#[test]
fn blank_line_terminates() {
    html_opts!(
        [extension.ofm_callouts],
        "> [!note]\n> Body\n\nAfter\n",
        concat!(
            "<div class=\"callout\" data-callout=\"note\" data-callout-fold=\"none\">\n",
            "<div class=\"callout-title\">Note</div>\n",
            "<div class=\"callout-content\">\n",
            "<p>Body</p>\n",
            "</div>\n",
            "</div>\n",
            "<p>After</p>\n",
        ),
    );
}

// === Output & roundtrip ===

#[test]
fn cm_roundtrip_with_title() {
    let mut o = Options::default();
    o.extension.ofm_callouts = true;
    let out = cm_roundtrip("> [!tip|right]- **Read**\n> Body\n", &o, |root2| {
        let callout = root2.children().next().unwrap();
        match &callout.data().value {
            NodeValue::Callout(nc) => {
                std::assert_eq!(nc.callout_type, "tip");
                std::assert_eq!(nc.metadata.as_deref(), Some("right"));
                std::assert_eq!(nc.fold, CalloutFold::Collapsed);
            }
            other => panic!("expected Callout, got {:?}", other),
        }
    });
    std::assert_eq!(out, "> [!tip|right]- **Read**\n> Body\n");
}

#[test]
fn cm_roundtrip_no_title() {
    let mut o = Options::default();
    o.extension.ofm_callouts = true;
    let out = cm_roundtrip("> [!note]\n> Body\n", &o, |root2| {
        let callout = root2.children().next().unwrap();
        match &callout.data().value {
            NodeValue::Callout(nc) => {
                std::assert_eq!(nc.callout_type, "note");
                std::assert_eq!(nc.metadata, None);
                std::assert_eq!(nc.fold, CalloutFold::None);
            }
            other => panic!("expected Callout, got {:?}", other),
        }
    });
    std::assert_eq!(out, "> [!note]\n> Body\n");
}

/// Table-driven: code-fence info round-trips in the title.
/// Each case: (input, expected CM output, expected re-parsed info).
#[test]
fn cm_roundtrip_code_info() {
    let cases = [
        (
            "> [!note] ```rust\n> Body\n",
            "> [!note] ```rust\n> Body\n",
            "rust",
        ),
        (
            // `&amp;amp;` → parser decodes to `&amp;` → formatter re-escapes.
            "> [!note] ```&amp;amp;\n> Body\n",
            "> [!note] ```&amp;amp;\n> Body\n",
            "&amp;",
        ),
        (
            // `\\` → parser unescapes to `\` → formatter re-escapes.
            "> [!note] ```a\\\\*b\n> Body\n",
            "> [!note] ```a\\\\*b\n> Body\n",
            "a\\*b",
        ),
        (
            // Newline encoded as &#10; to stay single-line.
            "> [!note] ```a&#10;b\n> Body\n",
            "> [!note] ```a&#10;b\n> Body\n",
            "a\nb",
        ),
    ];
    let mut o = Options::default();
    o.extension.ofm_callouts = true;
    for (md, expected_cm, expected_info) in cases {
        let out = cm_roundtrip(md, &o, |root2| {
            let callout = root2.children().next().unwrap();
            let title = callout.children().next().unwrap();
            let cb = title.children().next().unwrap();
            match &cb.data().value {
                NodeValue::CodeBlock(ncb) => {
                    std::assert_eq!(ncb.info, expected_info, "input: {md}");
                }
                other => panic!("expected CodeBlock, got {:?} (input: {md})", other),
            }
        });
        std::assert_eq!(out, expected_cm, "input: {md}");
    }
}

#[test]
fn cm_roundtrip_code_indent_content() {
    // Four-space indent in the title produces an indented code block.
    // Verify the content is preserved through the round-trip.
    let mut o = Options::default();
    o.extension.ofm_callouts = true;
    let out = cm_roundtrip("> [!note]     x\n> Body\n", &o, |root2| {
        let callout = root2.children().next().unwrap();
        let title = callout.children().next().unwrap();
        let cb = title.children().next().unwrap();
        match &cb.data().value {
            NodeValue::CodeBlock(ncb) => {
                std::assert_eq!(ncb.literal, "x\n");
            }
            other => panic!("expected CodeBlock, got {:?}", other),
        }
    });
    std::assert_eq!(out, "> [!note]     x\n> Body\n");
}

#[test]
fn cm_roundtrip_directive_colon() {
    // The fence scanner rejects a literal `:` in the info field, so the
    // formatter encodes it as &#58;. Verify the re-parsed node is still a
    // BlockDirective with the original info.
    let mut o = Options::default();
    o.extension.ofm_callouts = true;
    o.extension.block_directive = true;
    let out = cm_roundtrip("> [!note] :::a&#58;b\n> Body\n", &o, |root2| {
        let callout = root2.children().next().unwrap();
        let title = callout.children().next().unwrap();
        let bd = title.children().next().unwrap();
        match &bd.data().value {
            NodeValue::BlockDirective(nbd) => std::assert_eq!(nbd.info, "a:b"),
            other => panic!("expected BlockDirective, got {:?}", other),
        }
    });
    std::assert_eq!(out, "> [!note] :::a&#58;b\n> Body\n");
}

// === XML ===

#[test]
fn xml_output() {
    // Full structure: title, paragraph, strong, body.
    // Metadata contains a TAB to verify &#9; escaping.
    xml_opts(
        "> [!tip|a\tb]- **Read**\n> Body\n",
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <callout type=\"tip\" metadata=\"a&#9;b\" fold=\"collapsed\">\n",
            "    <callout_title>\n",
            "      <paragraph>\n",
            "        <strong>\n",
            "          <text xml:space=\"preserve\">Read</text>\n",
            "        </strong>\n",
            "      </paragraph>\n",
            "    </callout_title>\n",
            "    <paragraph>\n",
            "      <text xml:space=\"preserve\">Body</text>\n",
            "    </paragraph>\n",
            "  </callout>\n",
            "</document>\n",
        ),
        |opts| {
            opts.extension.ofm_callouts = true;
        },
    );
}

#[test]
fn xml_no_metadata_attr() {
    // Focused attribute check: no metadata attribute present.
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.ofm_callouts = true;
    let root = parse_document(&arena, "> [!note]\n> Body\n", &options);
    let mut out = String::new();
    crate::xml::format_document(root, &options, &mut out).unwrap();
    std::assert!(
        out.contains(r#"<callout type="note" fold="none">"#),
        "expected callout tag without metadata, got: {out}"
    );
}

// === Sourcepos ===

#[test]
fn sourcepos_utf8_crlf() {
    // UTF-8 title + CRLF line endings in one case.
    assert_ast_match!(
        [extension.ofm_callouts],
        "> [!note] Héllo\r\n> Body\r\n",
        (document (1:1-2:6) [
            (callout (1:1-2:6) [
                (callout_title (1:11-1:16) [
                    (paragraph (1:11-1:16) [
                        (text (1:11-1:16) "Héllo")
                    ])
                ])
                (paragraph (2:3-2:6) [
                    (text (2:3-2:6) "Body")
                ])
            ])
        ])
    );
}
