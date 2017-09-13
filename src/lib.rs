//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.  Source repository is at <https://github.com/kivikakk/comrak>.
//!
//! The design is based on [cmark](https://github.com/github/cmark), so familiarity with that will
//! help.
//!
//! You can use `comrak::markdown_to_html` directly:
//!
//! ```
//! use comrak::{markdown_to_html, ComrakOptions};
//! assert_eq!(markdown_to_html("Hello, **世界**!", &ComrakOptions::default()),
//!            "<p>Hello, <strong>世界</strong>!</p>\n");
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```
//! extern crate comrak;
//! extern crate typed_arena;
//! use typed_arena::Arena;
//! use comrak::{parse_document, format_html, ComrakOptions};
//! use comrak::nodes::{AstNode, NodeValue};
//!
//! # fn main() {
//! // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
//! let arena = Arena::new();
//!
//! let root = parse_document(
//!     &arena,
//!     "This is my input.\n\n1. Also my input.\n2. Certainly my input.\n",
//!     &ComrakOptions::default());
//!
//! fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
//!     where F : Fn(&'a AstNode<'a>) {
//!     f(node);
//!     for c in node.children() {
//!         iter_nodes(c, f);
//!     }
//! }
//!
//! iter_nodes(root, &|node| {
//!     match &mut node.data.borrow_mut().value {
//!         &mut NodeValue::Text(ref mut text) => {
//!             let orig = std::mem::replace(text, vec![]);
//!             *text = String::from_utf8(orig).unwrap().replace("my", "your").as_bytes().to_vec();
//!         }
//!         _ => (),
//!     }
//! });
//!
//! let mut html = vec![];
//! format_html(root, &ComrakOptions::default(), &mut html).unwrap();
//!
//! assert_eq!(
//!     String::from_utf8(html).unwrap(),
//!     "<p>This is your input.</p>\n\
//!      <ol>\n\
//!      <li>Also your input.</li>\n\
//!      <li>Certainly your input.</li>\n\
//!      </ol>\n");
//! # }
//! ```

#![deny(missing_docs,
        missing_debug_implementations,
	missing_copy_implementations,
	trivial_casts,
	trivial_numeric_casts,
	unstable_features,
	unused_import_braces,
	unused_qualifications)]

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![allow(unknown_lints, doc_markdown, cyclomatic_complexity)]

extern crate unicode_categories;
extern crate typed_arena;
extern crate regex;
extern crate entities;
#[macro_use]
extern crate lazy_static;
extern crate twoway;

mod arena_tree;
mod parser;
mod scanners;
mod html;
mod cm;
mod ctype;
pub mod nodes;
mod entity;
mod strings;
#[cfg(test)]
mod tests;

pub use cm::format_document as format_commonmark;
pub use html::format_document as format_html;

pub use parser::{parse_document, ComrakOptions};
use typed_arena::Arena;

/// Render Markdown to HTML.
///
/// See the documentation of the crate root for an example.
pub fn markdown_to_html(md: &str, options: &ComrakOptions) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut s = Vec::new();
    format_html(root, options, &mut s).unwrap();
    String::from_utf8(s).unwrap()
}
