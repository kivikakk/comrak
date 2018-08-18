extern crate comrak;
extern crate xml;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use xml::writer::{EventWriter, XmlEvent};

enum WriteEvent {
    Text(String),
    Element(&'static str),
}

fn event_node(node: &NodeValue) -> WriteEvent {
    use NodeValue::*;
    match node {
        BlockQuote => WriteEvent::Element("block-quote"),
        Code(..) => WriteEvent::Element("code"),
        CodeBlock(..) => WriteEvent::Element("code-block"),
        DescriptionDetails => WriteEvent::Element("desc-details"),
        DescriptionItem => WriteEvent::Element("desc-item"),
        DescriptionList => WriteEvent::Element("desc-list"),
        DescriptionTerm => WriteEvent::Element("desc-term"),
        Document => WriteEvent::Element("document"),
        Emph => WriteEvent::Element("emph"),
        FootnoteDefinition(..) => WriteEvent::Element("footnote-def"),
        FootnoteReference(..) => WriteEvent::Element("footnote-ref"),
        Heading(..) => WriteEvent::Element("heading"),
        HtmlBlock(..) => WriteEvent::Element("html"),
        HtmlInline(..) => WriteEvent::Element("inline-html"),
        Image(..) => WriteEvent::Element("image"),
        Item(..) => WriteEvent::Element("item"),
        LineBreak => WriteEvent::Element("line-break"),
        Link(..) => WriteEvent::Element("link"),
        List(..) => WriteEvent::Element("list"),
        Paragraph => WriteEvent::Element("paragraph"),
        SoftBreak => WriteEvent::Element("soft-break"),
        Strikethrough => WriteEvent::Element("strikethrough"),
        Strong => WriteEvent::Element("strong"),
        Superscript => WriteEvent::Element("superscript"),
        Table(..) => WriteEvent::Element("table"),
        TableCell => WriteEvent::Element("table-cell"),
        TableRow(..) => WriteEvent::Element("table-wrote"),
        Text(ref t) => WriteEvent::Text(String::from_utf8_lossy(&t).to_string()),
        ThematicBreak => WriteEvent::Element("thematic-break"),
    }
}

fn iter_nodes<'a, W: Write>(
    node: &'a AstNode<'a>,
    writer: &mut EventWriter<W>,
) -> Result<(), Box<Error>> {

    let xml_event = event_node(&node.data.borrow().value);

    match xml_event {
        WriteEvent::Element(e) => {
            writer.write(XmlEvent::start_element(e))?;

            for child in node.children() {
                iter_nodes(child, writer)?;
            }

            writer.write(XmlEvent::end_element())?;
        },

        WriteEvent::Text(t) => {
            writer.write(XmlEvent::characters(&t))?;
        }
    };

    Ok(())
}

fn to_xml(source: &str) -> Result<(), Box<Error>> {

    let mut output = io::stdout();
    let mut writer = xml::writer::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut output);

    let arena = Arena::new();

    let opts = ComrakOptions {
        ext_strikethrough: true,
        ext_tagfilter: true,
        ext_table: true,
        ext_autolink: true,
        ext_tasklist: true,
        ext_superscript: true,
        ext_footnotes: true,
        ext_description_lists: true,
        ..ComrakOptions::default()
    };

    let doc = parse_document(&arena, source, &opts);
    iter_nodes(doc, &mut writer)
}

fn main() -> Result<(), Box<Error>> {

    let mut args = env::args_os().skip(1).peekable();
    let mut body = String::new();

    if args.peek().is_none() {
        io::stdin().read_to_string(&mut body)?;
        return to_xml(&body);
    }

    for filename in args {
        body.clear();
        let mut file = File::open(&filename)?;
        file.read_to_string(&mut body)?;
        to_xml(&body)?;
        println!("");
    }

    Ok(())
}
