//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.
//!
//! The design is based on [cmark](https://github.com/github/cmark), so familiarity with that will
//! help.
//!
//! ```
//! use comrak::{markdown_to_html, ComrakOptions};
//! assert_eq!(markdown_to_html("Hello, **世界**!", &ComrakOptions::default()),
//!            "<p>Hello, <strong>世界</strong>!</p>\n");
//! ```

#![warn(missing_docs)]

extern crate unicode_categories;
extern crate typed_arena;
extern crate arena_tree;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod parser;
mod scanners;
mod html;
mod cm;
mod ctype;
pub mod nodes;
mod entity;
mod entity_data;
mod strings;
mod inlines;
#[cfg(test)]
mod tests;

use typed_arena::Arena;

pub use parser::{parse_document, ComrakOptions};
pub use html::format_document as format_html;
pub use cm::format_document as format_commonmark;

pub fn markdown_to_html(md: &str, options: &ComrakOptions) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    format_html(root, options)
}
