//! Adapter traits for plugins.
//!
//! Each plugin implements one or more of the traits available in this module.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

use crate::nodes::Sourcepos;

/// Implement this adapter for custom rendering of codefence blocks.
pub trait CodefenceRendererAdapter: Send + Sync {
    /// Render a codefence block.
    fn write(&self, output: &mut dyn fmt::Write, codefence: &CodefenceMeta<'_>) -> fmt::Result;
}

/// Information about a codefence block given to the adapter implementation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CodefenceMeta<'a> {
    /// Name of the programming language (the first token of the info string).
    pub lang: &'a str,

    /// The remaining codefence info string after the language token, trimmed.
    pub meta: &'a str,

    /// The block content.
    pub code: &'a str,

    /// Source position information for the code fence, if any.
    pub sourcepos: Option<Sourcepos>,
}

impl<'a> CodefenceMeta<'a> {
    /// Create a new `CodefenceMeta`.
    pub fn new(lang: &'a str, meta: &'a str, code: &'a str, sourcepos: Option<Sourcepos>) -> Self {
        CodefenceMeta {
            lang,
            meta,
            code,
            sourcepos,
        }
    }
}

/// Implement this adapter for creating a plugin for custom syntax highlighting of codefence blocks.
pub trait SyntaxHighlighterAdapter: Send + Sync {
    /// Generates a syntax highlighted HTML output.
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        highlight: &SyntaxHighlightMeta<'_>,
    ) -> fmt::Result;

    /// Generates the opening `<pre>` tag. Some syntax highlighter libraries might include their own
    /// `<pre>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by Comrak.
    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result;

    /// Generates the opening `<code>` tag. Some syntax highlighter libraries might include their own
    /// `<code>` tag possibly with some HTML attribute pre-filled.
    ///
    /// `attributes`: A map of HTML attributes provided by Comrak.
    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        highlight: &SyntaxHighlightMeta<'_>,
        attributes: BTreeMap<&str, Cow<'_, str>>,
    ) -> fmt::Result;
}

/// Information about a code block given to the adapter implementation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SyntaxHighlightMeta<'a> {
    /// Name of the programming language (the first token of the info string).
    pub lang: &'a str,

    /// The remaining info string after the language token, trimmed.
    pub meta: &'a str,

    /// The source code to be syntax highlighted.
    pub code: &'a str,
}

impl<'a> SyntaxHighlightMeta<'a> {
    /// Create a new `SyntaxHighlightMeta`.
    pub fn new(lang: &'a str, meta: &'a str, code: &'a str) -> Self {
        SyntaxHighlightMeta { lang, meta, code }
    }
}

/// The struct passed to the [`HeadingAdapter`] for custom heading implementations.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HeadingMeta {
    /// The level of the heading; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u8,

    /// The content of the heading as a "flattened" string&mdash;flattened in the sense that any
    /// `<strong>` or other tags are removed. In the Markdown heading `## This is **bold**`, for
    /// example, the would be the string `"This is bold"`.
    pub content: String,

    /// Source position information for the heading, if any.
    pub sourcepos: Option<Sourcepos>,
}

impl HeadingMeta {
    /// Create a new `HeadingMeta`.
    pub fn new(level: u8, content: impl Into<String>, sourcepos: Option<Sourcepos>) -> Self {
        HeadingMeta {
            level,
            content: content.into(),
            sourcepos,
        }
    }
}

/// Implement this adapter for creating a plugin for custom headings (`h1`, `h2`, etc.). The `enter`
/// method defines what's rendered prior the AST content of the heading while the `exit` method
/// defines what's rendered after it. Both methods provide access to a [`HeadingMeta`] struct and
/// leave the AST content of the heading unchanged.
pub trait HeadingAdapter: Send + Sync {
    /// Render the opening tag.
    fn enter(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result;

    /// Render the closing tag.
    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result;
}
