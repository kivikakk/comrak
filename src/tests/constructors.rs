use pretty_assertions::assert_eq;

use crate::nodes::{
    ListDelimType, NodeCode, NodeCodeBlock, NodeEmph, NodeHeading, NodeLink, NodeList, NodeStrong,
    NodeThematicBreak, NodeValue,
};
use crate::{Arena, Options, format_commonmark, format_html};

#[test]
fn constructed_document_renders() {
    let arena = Arena::new();
    let node = |value: NodeValue| arena.alloc(value.into());

    let doc = node(NodeValue::Document);

    let heading = node(NodeValue::Heading(NodeHeading::new(2)));
    heading.append(node(NodeValue::Text("Title".into())));
    doc.append(heading);

    let para = node(NodeValue::Paragraph);
    let emph = node(NodeValue::Emph(NodeEmph::default()));
    emph.append(node(NodeValue::Text("a".into())));
    para.append(emph);
    para.append(node(NodeValue::Text(" ".into())));
    let strong = node(NodeValue::Strong(NodeStrong::default()));
    strong.append(node(NodeValue::Text("b".into())));
    para.append(strong);
    para.append(node(NodeValue::Text(" ".into())));
    para.append(node(NodeValue::Code(NodeCode::new("c"))));
    para.append(node(NodeValue::Text(" ".into())));
    let link = node(NodeValue::Link(Box::new(NodeLink::new("/u", "t"))));
    link.append(node(NodeValue::Text("d".into())));
    para.append(link);
    doc.append(para);

    doc.append(node(NodeValue::ThematicBreak(NodeThematicBreak::default())));

    let list = node(NodeValue::List(NodeList::ordered(
        3,
        ListDelimType::Paren,
        true,
    )));
    for text in ["x", "y"] {
        let item = node(NodeValue::Item(NodeList::ordered(
            3,
            ListDelimType::Paren,
            true,
        )));
        let p = node(NodeValue::Paragraph);
        p.append(node(NodeValue::Text(text.to_string().into())));
        item.append(p);
        list.append(item);
    }
    doc.append(list);

    doc.append(node(NodeValue::CodeBlock(Box::new(NodeCodeBlock::fenced(
        "rust",
        "fn f() {}\n",
    )))));
    doc.append(node(NodeValue::CodeBlock(Box::new(
        NodeCodeBlock::indented("g\n"),
    ))));

    let list = NodeList::bullet(b'-', false);
    let bullets = node(NodeValue::List(list.clone()));
    let item = node(NodeValue::Item(list));
    let p = node(NodeValue::Paragraph);
    p.append(node(NodeValue::Text("z".into())));
    item.append(p);
    bullets.append(item);
    doc.append(bullets);

    let options = Options::default();

    let mut html = String::new();
    format_html(doc, &options, &mut html).unwrap();
    assert_eq!(
        html,
        concat!(
            "<h2>Title</h2>\n",
            "<p><em>a</em> <strong>b</strong> <code>c</code> <a href=\"/u\" title=\"t\">d</a></p>\n",
            "<hr />\n",
            "<ol start=\"3\">\n",
            "<li>x</li>\n",
            "<li>y</li>\n",
            "</ol>\n",
            "<pre><code class=\"language-rust\">fn f() {}\n</code></pre>\n",
            "<pre><code>g\n</code></pre>\n",
            "<ul>\n",
            "<li>\n",
            "<p>z</p>\n",
            "</li>\n",
            "</ul>\n",
        )
    );

    let mut cm = String::new();
    format_commonmark(doc, &options, &mut cm).unwrap();
    assert_eq!(
        cm,
        concat!(
            "## Title\n",
            "\n",
            "*a* **b** `c` [d](/u \"t\")\n",
            "\n",
            "-----\n",
            "\n",
            "3) x\n",
            "4) y\n",
            "\n",
            "<!-- end list -->\n",
            "\n",
            "```rust\n",
            "fn f() {}\n",
            "```\n",
            "\n",
            "```\n",
            "g\n",
            "```\n",
            "\n",
            "- z\n",
        )
    );
}
