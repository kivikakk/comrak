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
