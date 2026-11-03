use pretty_assertions::assert_eq;

use super::*;

#[test]
fn ofm_block_ids_html() {
    for (input, expected) in [
        ("hello ^37066d\n", "<p id=\"^37066d\">hello </p>\n"),
        ("# Title ^hdr\n", "<h1 id=\"^hdr\">Title </h1>\n"),
        (
            "> one ^inside\n",
            "<blockquote id=\"^inside\">\n<p>one </p>\n</blockquote>\n",
        ),
        (
            "> one ^inside\n>\n> two\n",
            "<blockquote>\n<p id=\"^inside\">one </p>\n<p>two</p>\n</blockquote>\n",
        ),
        (
            "| A | B |\n| - | - |\n| x | y ^cell |\n",
            concat!(
                "<table id=\"^cell\">\n<thead>\n<tr>\n<th>A</th>\n<th>B</th>\n</tr>\n</thead>\n",
                "<tbody>\n<tr>\n<td>x</td>\n<td>y </td>\n</tr>\n</tbody>\n</table>\n"
            ),
        ),
        (
            "- a ^x\n- b\n",
            "<ul>\n<li id=\"^x\">a </li>\n<li>b</li>\n</ul>\n",
        ),
        (
            "- a ^x\n\n- b\n",
            "<ul>\n<li id=\"^x\">\n<p>a </p>\n</li>\n<li>\n<p>b</p>\n</li>\n</ul>\n",
        ),
        (
            "- outer\n  - inner ^inner\n",
            "<ul>\n<li>outer\n<ul>\n<li id=\"^inner\">inner </li>\n</ul>\n</li>\n</ul>\n",
        ),
        ("*hello ^em*\n", "<p><em>hello ^em</em></p>\n"),
        (
            "[hello ^label](https://example.com)\n",
            "<p><a href=\"https://example.com\">hello ^label</a></p>\n",
        ),
        (
            "one ^first\ntwo ^second\n",
            "<p id=\"^second\">one ^first\ntwo </p>\n",
        ),
        ("hello\n^next\n", "<p>hello\n^next</p>\n"),
        ("hello^tight\n", "<p>hello^tight</p>\n"),
        ("hello ^trail  \n", "<p>hello ^trail</p>\n"),
        ("hello \\^escaped\n", "<p>hello ^escaped</p>\n"),
        ("hello ^bad_id\n", "<p>hello ^bad_id</p>\n"),
        (
            "hello ^Not-Hex-123\n",
            "<p id=\"^Not-Hex-123\">hello </p>\n",
        ),
        ("`code ^id` and\n", "<p><code>code ^id</code> and</p>\n"),
        ("^orphan\n", "<p>^orphan</p>\n"),
        (
            "hello\n\n^abc\n# Heading\n",
            "<p>hello</p>\n<p>^abc</p>\n<h1>Heading</h1>\n",
        ),
    ] {
        html_opts!(
            [
                extension.ofm_block_ids,
                extension.ofm_callouts,
                extension.table
            ],
            input,
            expected
        );
    }
}

#[test]
fn off_by_default() {
    html("hello ^37066d\n", "<p>hello ^37066d</p>\n");
}

#[test]
fn superscript_wins_when_it_closes() {
    html_opts!(
        [extension.ofm_block_ids, extension.superscript],
        "a^b^\n",
        "<p>a<sup>b</sup></p>\n"
    );
    html_opts!(
        [extension.ofm_block_ids, extension.superscript],
        "e = mc^2^ and a ^b\n",
        "<p id=\"^b\">e = mc<sup>2</sup> and a </p>\n"
    );
}

#[test]
fn headings_keep_their_own_anchor() {
    let mut options = Options::default();
    options.extension.ofm_block_ids = true;
    options.extension.header_id_prefix = Some("h-".to_string());

    let arena = Arena::new();
    let root = parse_document(&arena, "# Title ^hdr\n", &options);
    let mut output = String::new();
    html::format_document(root, &options, &mut output).unwrap();
    assert_eq!(output.matches(" id=").count(), 1, "{}", output);
    assert!(output.starts_with("<h1 id=\"h-title-\""), "{}", output);
}

#[test]
fn ast_and_sourcepos() {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.ofm_block_ids = true;
    let root = parse_document(&arena, "- a ^x\n- b\n", &options);
    root.validate().unwrap();

    let markers: Vec<_> = root
        .descendants()
        .filter(|n| node_matches!(n, NodeValue::BlockId(_)))
        .map(|n| {
            let ast = n.data();
            match &ast.value {
                NodeValue::BlockId(b) => (b.name.clone(), ast.sourcepos),
                _ => unreachable!(),
            }
        })
        .collect();

    assert_eq!(markers, vec![("x".to_string(), sourcepos!((1:5-1:6)))]);
    assert_eq!(root.collect_text(), "a b");

    let arena = Arena::new();
    let root = parse_document(&arena, "one ^first\ntwo ^second\n", &options);
    let text: Vec<String> = root
        .descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::Text(t) => Some(t.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(text, vec!["one ^first", "two "]);
}

#[test]
fn commonmark_roundtrip() {
    let mut options = Options::default();
    options.extension.ofm_block_ids = true;

    for (input, expected) in [
        ("hello ^37066d\n", "hello ^37066d\n"),
        ("*hello ^em*\n", "*hello ^em*\n"),
        ("*&#32;\\^id*\n", "*&#32;^id*\n"),
        ("^orphan\n", "^orphan\n"),
    ] {
        commonmark(input, expected, Some(&options));
    }
}

#[test]
fn xml_ofm_block_ids() {
    xml_opts(
        "> q ^inside\n",
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <block_quote>\n",
            "    <paragraph>\n",
            "      <text xml:space=\"preserve\">q </text>\n",
            "      <block_id name=\"inside\" />\n",
            "    </paragraph>\n",
            "  </block_quote>\n",
            "</document>\n",
        ),
        |opts| opts.extension.ofm_block_ids = true,
    );
}
