// Extract the document title by srching for a level-one header at the root level.

use comrak::{html::collect_text, nodes::NodeValue, parse_document, Arena, Options};

fn main() {
    println!("{:?}", get_document_title("# Hello\n"));
    println!("{:?}", get_document_title("## Hello\n"));
    println!("{:?}", get_document_title("# `hi` **there**\n"));
}

fn get_document_title(document: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &Options::default());

    for node in root.children() {
        let header = match node.data.clone().into_inner().value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };

        if header.level != 1 {
            continue;
        }

        return collect_text(node);
    }

    "Untitled Document".to_string()
}
