// Update the "comrak --help" text in Comrak's own README.

use std::error::Error;
use std::fmt::Write;
use std::str;
use toml::Table;

use comrak::nodes::NodeValue;
use comrak::{format_commonmark, parse_document, Arena, Options};

const DEPENDENCIES: &str = "[dependencies]\ncomrak = ";
const HELP: &str = "$ comrak --help\n";
const HELP_START: &str =
    "A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter\n";

fn main() -> Result<(), Box<dyn Error>> {
    let mut arena = Arena::new();

    let readme = std::fs::read_to_string("README.md")?;
    let doc = parse_document(&mut arena, &readme, &Options::default());

    let cargo_toml = std::fs::read_to_string("Cargo.toml")?.parse::<Table>()?;
    let msrv = cargo_toml["package"].as_table().unwrap()["rust-version"]
        .as_str()
        .unwrap();

    let mut in_msrv = false;
    let mut next_block_is_help_body = false;

    let mut it = doc.descendants_free();
    while let Some(node) = it.next(&arena) {
        match node.data_mut(&mut arena).value {
            NodeValue::CodeBlock(ref mut ncb) => {
                // Look for the Cargo.toml example block.
                if ncb.info == "toml" && ncb.literal.starts_with(DEPENDENCIES) {
                    let mut content = DEPENDENCIES.to_string();
                    let mut version_parts = comrak::version().split('.').collect::<Vec<&str>>();
                    version_parts.pop();
                    write!(content, "\"{}\"", version_parts.join(".")).unwrap();
                    ncb.literal = content;
                    continue;
                }

                // Look for a console code block whose contents starts with the HELP string.
                // The *next* code block contains our help, minus the starting string.
                if ncb.info == "console" && ncb.literal.starts_with(HELP) {
                    next_block_is_help_body = true;
                    continue;
                }

                if next_block_is_help_body {
                    next_block_is_help_body = false;
                    assert!(ncb.info.is_empty() && ncb.literal.starts_with(HELP_START));
                    let mut content = String::new();
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
                    continue;
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
                    std::mem::swap(t, &mut msrv.to_string().into());
                }
            }
            _ => {}
        }
    }

    let mut options = Options::default();
    options.render.prefer_fenced = true;
    options.render.experimental_minimize_commonmark = true;

    let mut out = String::new();
    format_commonmark(&arena, doc, &options, &mut out)?;
    std::fs::write("README.md", &out)?;

    Ok(())
}
