// Extract the document title by srching for a level-one header at the root level.

use comrak::{html::collect_text, nodes::NodeValue, parse_document, Arena, Options};

fn main() {
    println!("{:?}", get_document_title("# Hello\n"));
    println!("{:?}", get_document_title("## Hello\n"));
    println!("{:?}", get_document_title("# `hi` **there**\n"));
}

fn get_document_title(document: &str) -> String {
    let mut arena = Arena::new();
    let root = parse_document(&mut arena, document, &Options::default());

    for node in root.children(&arena).collect::<Vec<_>>() {
        let header = match node.get(&arena).value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };

        if header.level != 1 {
            continue;
        }

        return collect_text(&arena, node);
    }

    "Untitled Document".to_string()
}
