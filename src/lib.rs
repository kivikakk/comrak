extern crate clap;
extern crate unicode_categories;
extern crate typed_arena;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod parser;
mod arena_tree;
mod scanners;
pub mod html;
pub mod cm;
mod ctype;
mod node;
mod entity;
mod entity_data;
mod strings;
mod inlines;
#[cfg(test)]
mod tests;

pub use typed_arena::Arena;
pub use arena_tree::Node;
pub use node::{AstCell, Ast, NodeValue};

pub use parser::{parse_document, ComrakOptions};

pub fn markdown_to_html(md: &str, options: &ComrakOptions) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    html::format_document(root, options)
}
