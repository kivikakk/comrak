// Extract the document title by srching for a level-one header at the root level.

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

        let mut text = String::new();
        collect_text(node, &mut text);

        // The input was already known good UTF-8 (document: &str) so comrak
        // guarantees the output will be too.
        return text;
    }

    "Untitled Document".to_string()
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut String) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.push_str(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
