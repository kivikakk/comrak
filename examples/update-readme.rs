// Update the "comrak --help" text in Comrak's own README.

extern crate comrak;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};

const HELP: &str = "$ comrak --help\n";

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let arena = Arena::new();

    let readme = std::fs::read_to_string("README.md")?;
    let doc = parse_document(&arena, &readme, &ComrakOptions::default());

    fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where
        F: Fn(&'a AstNode<'a>),
    {
        f(node);
        for c in node.children() {
            iter_nodes(c, f);
        }
    }

    iter_nodes(doc, &|node| {
        // Look for a code block whose contents starts with the HELP string.
        // Replace its contents with the same string and the actual command output.
        if let NodeValue::CodeBlock(ref mut ncb) = node.data.borrow_mut().value {
            if ncb.literal.starts_with(&HELP.as_bytes()) {
                let mut content = HELP.as_bytes().to_vec();
                let mut cmd = std::process::Command::new("cargo");
                content.extend(cmd.args(&["run", "--", "--help"]).output().unwrap().stdout);
                ncb.literal = content;
            }
        }
    });

    let mut out = vec![];
    format_commonmark(doc, &ComrakOptions::default(), &mut out).unwrap();

    std::fs::write("README.md", &out)?;

    Ok(())
}
