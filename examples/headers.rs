// Extract the document title by srching for a level-one header at the root level.

extern crate comrak;

use comrak::{
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions,
};

fn main() {
    println!("{:?}", get_document_title("# Hello\n"));
    println!("{:?}", get_document_title("## Hello\n"));
    println!("{:?}", get_document_title("# `hi` **there**\n"));
}

fn get_document_title(document: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &ComrakOptions::default());

    for node in root.children() {
        let header = match node.data.clone().into_inner().value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };

        if header.level != 1 {
            continue;
        }

        let mut text = Vec::new();
        collect_text(node, &mut text);

        // The input was already known good UTF-8 (document: &str) so comrak
        // guarantees the output will be too.
        return String::from_utf8(text).unwrap();
    }

    "Untitled Document".to_string()
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.extend_from_slice(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
