use comrak::{
    arena_tree::NodeEdge,
    nodes::{Node, NodeValue},
    parse_document, Arena, Options,
};

// `node.traverse()`` creates an iterator that will traverse
// the current node and all descendants in order.
// The iterator yields `NodeEdges`. `NodeEdges` can have the
// following values:
//
// `NodeEdge::Start(node)` Start of node.
// `NodeEdge::End(node)` End of node.
// `None` End of iterator at bottom of last branch.
//
// This example extracts plain text ignoring nested
// markup.

// Note: root can be any Node, not just document root.

fn extract_text_traverse(arena: &Arena, root: Node) -> String {
    let mut output_text = String::new();

    // Use `traverse` to get an iterator of `NodeEdge` and process each.
    for edge in root.traverse(&arena) {
        if let NodeEdge::Start(node) = edge {
            // Handle the Start edge to process the node's value.
            if let NodeValue::Text(ref text) = node.data(arena).value {
                // If the node is a text node, append its text to `output_text`.
                output_text.push_str(text);
            }
        }
    }

    output_text
}

fn main() {
    let markdown_input = "Hello, *worl[d](https://example.com/)*";
    // Nested inline markup. Equivalent html should look like this:
    //"<p>Hello, <em>worl<a href="https://example.com">d</a></em></p>

    println!("INPUT:  {}", markdown_input);

    // setup parser
    let mut arena = Arena::new();
    let options = Options::default();

    // parse document and return root.
    let root = parse_document(&mut arena, markdown_input, &options);

    // extract text and print
    println!("OUTPUT: {}", extract_text_traverse(&arena, root).as_str())
}

#[cfg(test)]
mod tests {
    // Import everything from the outer module to make it available for tests
    use super::*;

    #[test]
    fn extract_text_traverse_test() {
        let markdown_input = "Hello, *worl[d](https://example.com/)*";
        let mut arena = Arena::new();
        let options = Options::default();
        let root = parse_document(&mut arena, markdown_input, &options);
        assert_eq!("Hello, world", extract_text_traverse(&arena, root));
    }
}
