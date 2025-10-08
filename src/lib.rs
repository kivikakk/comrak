//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.
//!
//! Source repository and detailed `README` is at <https://github.com/kivikakk/comrak>.
//!
//! You can use `comrak::markdown_to_html` directly:
//!
//! ```
//! use comrak::{markdown_to_html, Options};
//! assert_eq!(markdown_to_html("Hello, **世界**!", &Options::default()),
//!            "<p>Hello, <strong>世界</strong>!</p>\n");
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```
//! use comrak::{Arena, parse_document, format_html, Options};
//! use comrak::nodes::{AstNode, NodeValue};
//!
//! # fn main() {
//! let arena = Arena::new();
//!
//! let root = parse_document(
//!     &arena,
//!     "This is my input.\n\n1. Also [my](#) input.\n2. Certainly *my* input.\n",
//!     &Options::default());
//!
//! for node in root.descendants() {
//!     if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
//!         *text = text.replace("my", "your");
//!     }
//! }
//!
//! let mut html = String::new();
//! format_html(root, &Options::default(), &mut html).unwrap();
//!
//! assert_eq!(
//!     &html,
//!     "<p>This is your input.</p>\n\
//!      <ol>\n\
//!      <li>Also <a href=\"#\">your</a> input.</li>\n\
//!      <li>Certainly <em>your</em> input.</li>\n\
//!      </ol>\n");
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces
)]
#![allow(
    unknown_lints,
    clippy::doc_markdown,
    cyclomatic_complexity,
    clippy::bool_to_int_with_if,
    clippy::too_many_arguments
)]

pub mod adapters;
mod character_set;
mod cm;
mod ctype;
mod entity;
pub mod html;
pub mod nodes;
mod parser;
pub mod plugins;
mod scanners;
mod strings;
#[cfg(test)]
mod tests;
mod xml;

pub use cm::escape_inline as escape_commonmark_inline;
pub use cm::escape_link_destination as escape_commonmark_link_destination;
pub use cm::format_document as format_commonmark;
pub use cm::format_document_with_plugins as format_commonmark_with_plugins;
pub use html::format_document as format_html;
pub use html::format_document_with_plugins as format_html_with_plugins;
#[doc(inline)]
pub use html::Anchorizer;
pub use indextree::Arena;
#[allow(deprecated)]
pub use parser::parse_document_with_broken_link_callback;
pub use parser::{
    parse_document, BrokenLinkCallback, BrokenLinkReference, ExtensionOptions, ListStyleType,
    Options, ParseOptions, Plugins, RenderOptions, RenderPlugins, ResolvedReference, URLRewriter,
    WikiLinksMode,
};
pub use xml::format_document as format_xml;
pub use xml::format_document_with_plugins as format_xml_with_plugins;

#[cfg(feature = "bon")]
pub use parser::{
    ExtensionOptionsBuilder, ParseOptionsBuilder, PluginsBuilder, RenderOptionsBuilder,
    RenderPluginsBuilder,
};

/// Legacy naming of [`ExtensionOptions`]
pub type ComrakExtensionOptions<'c> = ExtensionOptions<'c>;
/// Legacy naming of [`Options`]
pub type ComrakOptions<'c> = Options<'c>;
/// Legacy naming of [`ParseOptions`]
pub type ComrakParseOptions<'c> = ParseOptions<'c>;
/// Legacy naming of [`Plugins`]
pub type ComrakPlugins<'a> = Plugins<'a>;
/// Legacy naming of [`RenderOptions`]
pub type ComrakRenderOptions = RenderOptions;
/// Legacy naming of [`RenderPlugins`]
pub type ComrakRenderPlugins<'a> = RenderPlugins<'a>;

/// Render Markdown to HTML.
///
/// See the documentation of the crate root for an example.
pub fn markdown_to_html(md: &str, options: &Options) -> String {
    markdown_to_html_with_plugins(md, options, &Plugins::default())
}

/// Render Markdown to HTML using plugins.
///
/// See the documentation of the crate root for an example.
pub fn markdown_to_html_with_plugins(md: &str, options: &Options, plugins: &Plugins) -> String {
    let mut arena = Arena::new();
    let root = parse_document(&mut arena, md, options);
    let mut out = String::new();
    format_html_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
    out
}

/// Return the version of the crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Render Markdown back to CommonMark.
pub fn markdown_to_commonmark(md: &str, options: &Options) -> String {
    let mut arena = Arena::new();
    let root = parse_document(&mut arena, md, options);
    let mut out = String::new();
    format_commonmark(&arena, root, options, &mut out).unwrap();
    out
}

/// Render Markdown to CommonMark XML.
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
pub fn markdown_to_commonmark_xml(md: &str, options: &Options) -> String {
    markdown_to_commonmark_xml_with_plugins(md, options, &Plugins::default())
}

/// Render Markdown to CommonMark XML using plugins.
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
pub fn markdown_to_commonmark_xml_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins,
) -> String {
    let mut arena = Arena::new();
    let root = parse_document(&mut arena, md, options);
    let mut out = String::new();
    format_xml_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
    out
}
