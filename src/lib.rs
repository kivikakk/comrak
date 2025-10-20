//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.
//!
//! Source repository and detailed `README` is at
//! [github.com/kivikakk/comrak](https://github.com/kivikakk/comrak).
//!
//! If you're reading this in HTML, you're reading Markdown processed by Comrak.
//!
//! You can use `comrak::markdown_to_html` directly:
//!
//! ```rust
//! use comrak::{markdown_to_html, Options};
//! assert_eq!(
//!     markdown_to_html("Olá, **世界**!", &Options::default()),
//!     "<p>Olá, <strong>世界</strong>!</p>\n"
//! );
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```rust
//! use comrak::{Arena, parse_document, format_html, Options};
//! use comrak::nodes::{NodeValue};
//!
//! # fn main() {
//! let arena = Arena::new();
//!
//! let root = parse_document(
//!     &arena,
//!     "Hello, pretty world!\n\n1. Do you like [pretty](#) paintings?\n2. Or *pretty* music?\n",
//!     &Options::default());
//!
//! for node in root.descendants() {
//!     if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
//!         *text = text.to_mut().replace("pretty", "beautiful").into()
//!     }
//! }
//!
//! let mut html = String::new();
//! format_html(root, &Options::default(), &mut html).unwrap();
//!
//! assert_eq!(
//!     &html,
//!     "<p>Hello, beautiful world!</p>\n\
//!      <ol>\n\
//!      <li>Do you like <a href=\"#\">beautiful</a> paintings?</li>\n\
//!      <li>Or <em>beautiful</em> music?</li>\n\
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
pub mod arena_tree;
pub mod html;
pub mod nodes;
pub mod plugins;

mod character_set;
mod cm;
mod ctype;
mod entity;
mod parser;
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
pub use parser::options;
pub use parser::{parse_document, Options, ResolvedReference};
pub use typed_arena::Arena;
pub use xml::format_document as format_xml;
pub use xml::format_document_with_plugins as format_xml_with_plugins;

#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::Extension` instead of `comrak::ExtensionOptions`"
)]
/// Deprecated alias: use [`options::Extension`] instead of [`ExtensionOptions`].
pub type ExtensionOptions<'c> = parser::options::Extension<'c>;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::Parse` instead of `comrak::ParseOptions`"
)]
/// Deprecated alias: use [`options::Parse`] instead of [`ParseOptions`].
pub type ParseOptions<'c> = parser::options::Parse<'c>;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::Render` instead of `comrak::RenderOptions `"
)]
/// Deprecated alias: use [`options::Render`] instead of [`RenderOptions ]`.
pub type RenderOptions = parser::options::Render;

#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::BrokenLinkReference` instead of `comrak::BrokenLinkReference`"
)]
/// Deprecated alias: use [`options::BrokenLinkReference`] instead of [`BrokenLinkReference`].
pub type BrokenLinkReference<'l> = parser::options::BrokenLinkReference<'l>;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::ListStyleType` instead of `comrak::ListStyleType `"
)]
/// Deprecated alias: use [`options::ListStyleType`] instead of [`ListStyleType ]`.
pub type ListStyleType = parser::options::ListStyleType;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::Plugins` instead of `comrak::Plugins`"
)]
/// Deprecated alias: use [`options::Plugins`] instead of [`Plugins`].
pub type Plugins<'p> = parser::options::Plugins<'p>;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::RenderPlugins` instead of `comrak::RenderPlugins`"
)]
/// Deprecated alias: use [`options::RenderPlugins`] instead of [`RenderPlugins`].
pub type RenderPlugins<'p> = parser::options::RenderPlugins<'p>;
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::WikiLinksMode` instead of `comrak::WikiLinksMode `"
)]
/// Deprecated alias: use [`options::WikiLinksMode`] instead of [`WikiLinksMode ]`.
pub type WikiLinksMode = parser::options::WikiLinksMode;

#[cfg(feature = "bon")]
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::ExtensionBuilder` instead of `comrak::ExtensionOptionsBuilder`"
)]
/// Deprecated alias: use [`options::ExtensionBuilder`] instead of [`ExtensionOptionsBuilder`].
pub type ExtensionOptionsBuilder<'c> = parser::options::ExtensionBuilder<'c>;
#[cfg(feature = "bon")]
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::ParseBuilder` instead of `comrak::ParseOptionsBuilder`"
)]
/// Deprecated alias: use [`options::ParseBuilder`] instead of [`ParseOptionsBuilder`].
pub type ParseOptionsBuilder<'c> = parser::options::ParseBuilder<'c>;
#[cfg(feature = "bon")]
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::RenderBuilder` instead of `comrak::RenderOptionsBuilder `"
)]
/// Deprecated alias: use [`options::RenderBuilder`] instead of [`RenderOptionsBuilder ]`.
pub type RenderOptionsBuilder = parser::options::RenderBuilder;
#[cfg(feature = "bon")]
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::PluginsBuilder` instead of `comrak::PluginsBuilder`"
)]
/// Deprecated alias: use [`options::PluginsBuilder`] instead of [`PluginsBuilder`].
pub type PluginsBuilder<'p> = parser::options::PluginsBuilder<'p>;
#[cfg(feature = "bon")]
#[deprecated(
    since = "0.45.0",
    note = "use `comrak::options::RenderPluginsBuilder` instead of `comrak::RenderPluginsBuilder`"
)]
/// Deprecated alias: use [`options::RenderPluginsBuilder`] instead of [`RenderPluginsBuilder`].
pub type RenderPluginsBuilder<'p> = parser::options::RenderPluginsBuilder<'p>;

/// Render Markdown to HTML.
///
/// See the documentation of the crate root for an example.
pub fn markdown_to_html(md: &str, options: &Options) -> String {
    markdown_to_html_with_plugins(md, options, &options::Plugins::default())
}

/// Render Markdown to HTML using plugins.
///
/// See the documentation of the crate root for an example.
pub fn markdown_to_html_with_plugins(
    md: &str,
    options: &Options,
    plugins: &options::Plugins,
) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut out = String::new();
    format_html_with_plugins(root, options, &mut out, plugins).unwrap();
    out
}

/// Return the version of the crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Render Markdown back to CommonMark.
pub fn markdown_to_commonmark(md: &str, options: &Options) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut out = String::new();
    format_commonmark(root, options, &mut out).unwrap();
    out
}

/// Render Markdown to CommonMark XML.
///
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
pub fn markdown_to_commonmark_xml(md: &str, options: &Options) -> String {
    markdown_to_commonmark_xml_with_plugins(md, options, &options::Plugins::default())
}

/// Render Markdown to CommonMark XML using plugins.
///
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
pub fn markdown_to_commonmark_xml_with_plugins(
    md: &str,
    options: &Options,
    plugins: &options::Plugins,
) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut out = String::new();
    format_xml_with_plugins(root, options, &mut out, plugins).unwrap();
    out
}
