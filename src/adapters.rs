//! Adapter traits for plugins.
//!
//! Each plugin has to implement one of the traits available in this module.

use std::collections::HashMap;
use std::io::{self, Write};

use crate::nodes::Sourcepos;

/// Implement this adapter for creating a plugin for custom syntax highlighting of codefence blocks.
pub trait SyntaxHighlighterAdapter {
    /// Generates a syntax highlighted HTML output.
    ///
    /// lang: Name of the programming language (the info string of the codefence block after the initial "```" part).
    /// code: The source code to be syntax highlighted.
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()>;

    /// Generates the opening `<pre>` tag. Some syntax highlighter libraries might include their own
    /// `<pre>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by comrak.
    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()>;

    /// Generates the opening `<code>` tag. Some syntax highlighter libraries might include their own
    /// `<code>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by comrak.
    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()>;
}

/// The struct passed to the [`HeadingAdapter`] for custom heading implementations.
#[derive(Clone, Debug)]
pub struct HeadingMeta {
    /// The level of the heading; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u8,

    /// The content of the heading as a "flattened" string&mdash;flattened in the sense that any
    /// `<strong>` or other tags are removed. In the Markdown heading `## This is **bold**`, for
    /// example, the would be the string `"This is bold"`.
    pub content: String,
}

/// Implement this adapter for creating a plugin for custom headings (`h1`, `h2`, etc.). The `enter`
/// method defines what's rendered prior the AST content of the heading while the `exit` method
/// defines what's rendered after it. Both methods provide access to a [`HeadingMeta`] struct and
/// leave the AST content of the heading unchanged.
pub trait HeadingAdapter {
    /// Render the opening tag.
    fn enter(
        &self,
        output: &mut dyn Write,
        heading: &HeadingMeta,
        sourcepos: Option<Sourcepos>,
    ) -> io::Result<()>;

    /// Render the closing tag.
    fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> io::Result<()>;
}
