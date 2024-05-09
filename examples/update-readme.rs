// Update the "comrak --help" text in Comrak's own README.

use std::error::Error;
use std::fmt::Write;
use std::str;
use toml::Table;

use comrak::nodes::NodeValue;
use comrak::{format_commonmark, parse_document, Arena, Options};

const DEPENDENCIES: &str = "[dependencies]\ncomrak = ";
const HELP: &str = "$ comrak --help\n";

fn main() -> Result<(), Box<dyn Error>> {
    let arena = Arena::new();

    let readme = std::fs::read_to_string("README.md")?;
    let doc = parse_document(&arena, &readme, &Options::default());

    let cargo_toml = std::fs::read_to_string("Cargo.toml")?.parse::<Table>()?;
    let msrv = cargo_toml["package"].as_table().unwrap()["rust-version"]
        .as_str()
        .unwrap();

    let mut in_msrv = false;
    for node in doc.descendants() {
        match node.data.borrow_mut().value {
            NodeValue::CodeBlock(ref mut ncb) => {
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
            NodeValue::HtmlInline(ref mut s) => {
                if s == "<span class=\"msrv\">" {
                    in_msrv = true;
                } else if in_msrv && s == "</span>" {
                    in_msrv = false;
                }
            }
            NodeValue::Text(ref mut t) => {
                if in_msrv {
                    std::mem::swap(t, &mut msrv.to_string());
                }
            }
            _ => {}
        }
    }

    let mut out = vec![];
    format_commonmark(doc, &Options::default(), &mut out)?;
    std::fs::write("README.md", &out)?;

    Ok(())
}
