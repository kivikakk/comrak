//! Adapter traits for plugins.
//!
//! Each plugin has to implement one of the traits available in this module.

use std::collections::HashMap;

/// Implement this adapter for creating a plugin for custom syntax highlighting of codefence blocks.
pub trait SyntaxHighlighterAdapter {
    /// Generates a syntax highlighted HTML output.
    ///
    /// lang: Name of the programming language (the info string of the codefence block after the initial "```" part).
    /// code: The source code to be syntax highlighted.
    fn highlight(&self, lang: Option<&str>, code: &str) -> String;

    /// Generates the opening `<pre>` tag. Some syntax highlighter libraries might include their own
    /// `<pre>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by comrak.
    fn build_pre_tag(&self, attributes: &HashMap<String, String>) -> String;

    /// Generates the opening `<code>` tag. Some syntax highlighter libraries might include their own
    /// `<code>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by comrak.
    fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String;
}

/// The struct passed to the `HeadingAdapter` for providing a custom heading implementation.
#[derive(Clone, Debug)]
pub struct HeadingMeta {
    /// The level of the heading; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u32,

    /// The content of the heading as a "flattened" string&mdash;flattened in the sense that any
    /// `<strong>` or other tags are removed. In the Markdown heading `## This is **bold**`, the
    /// `content` would be the string `"This is bold"`.
    pub content: String,
}

/// Implement this adapter for creating a plugin for custom headings (`h1`, `h2`, etc.). The `enter`
/// defines what's rendered prior the AST content of the heading while the `exit` method defines
/// what's rendered after it. Both methods provide access to a [`HeadingMeta`] struct and leave the
/// AST content of the heading unchanged.
pub trait HeadingAdapter {
    /// Called prior to rendering
    fn enter(&self, heading: &HeadingMeta) -> String;

    /// Close tags.
    fn exit(&self, heading: &HeadingMeta) -> String;
}
