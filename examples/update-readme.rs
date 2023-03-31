// Update the "comrak --help" text in Comrak's own README.

use std::fmt::Write;
use std::str;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};

const DEPENDENCIES: &str = "[dependencies]\ncomrak = ";
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
        if let NodeValue::CodeBlock(ref mut ncb) = node.data.borrow_mut().value {
            // Look for the Cargo.toml example block.
            if ncb.info == "toml" && ncb.literal.starts_with(DEPENDENCIES) {
                let mut content = DEPENDENCIES.to_string();
                let mut version_parts = comrak::version().split('.').collect::<Vec<&str>>();
                version_parts.pop();
                write!(content, "\"{}\"", version_parts.join(".")).unwrap();
                ncb.literal = content;
            }

            // Look for a console code block whose contents starts with the HELP string.
            // Replace its contents with the same string and the actual command output.
            if ncb.info == "console" && ncb.literal.starts_with(HELP) {
                let mut content = HELP.to_string();
                let mut cmd = std::process::Command::new("cargo");
                content.push_str(
                    str::from_utf8(
                        &cmd.args(["run", "--all-features", "--", "--help"])
                            .output()
                            .unwrap()
                            .stdout,
                    )
                    .unwrap(),
                );
                ncb.literal = content;
            }
        }
    });

    let mut out = vec![];
    format_commonmark(doc, &ComrakOptions::default(), &mut out).unwrap();

    std::fs::write("README.md", &out)?;

    Ok(())
}
