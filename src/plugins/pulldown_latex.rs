//! Adapter for the pulldown-latex MathML rendering plugin.

use std::fmt;

use crate::adapters::MathAdapter;
use crate::nodes::Sourcepos;

#[derive(Debug, Clone, Copy)]
/// A MathML renderer using [`pulldown_latex`](https://crates.io/crates/pulldown-latex).
///
/// Converts LaTeX math content to MathML using the pulldown-latex crate.
///
/// When rendering fails (e.g., invalid LaTeX), the adapter falls back to
/// rendering the raw LaTeX source wrapped in `<span class="math-error">`.
///
/// # Example
///
/// ```rust
/// use comrak::plugins::pulldown_latex::PulldownLatexAdapter;
/// use comrak::{Options, markdown_to_html_with_plugins, options};
///
/// let adapter = PulldownLatexAdapter::default();
/// let mut options = Options::default();
/// options.extension.math_dollars = true;
/// let mut plugins = options::Plugins::default();
/// plugins.render.math_renderer = Some(&adapter);
///
/// let result = markdown_to_html_with_plugins("$E=mc^2$", &options, &plugins);
/// assert!(result.contains("<math"));
/// ```
pub struct PulldownLatexAdapter {
    /// Whether to include a `<annotation>` element with the original LaTeX source.
    pub include_annotation: bool,
}

impl Default for PulldownLatexAdapter {
    fn default() -> Self {
        Self {
            include_annotation: true,
        }
    }
}

impl MathAdapter for PulldownLatexAdapter {
    fn render(
        &self,
        output: &mut dyn fmt::Write,
        latex: &str,
        display_math: bool,
        _dollar_math: bool,
        sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let display_mode = if display_math {
            ::pulldown_latex::config::DisplayMode::Block
        } else {
            ::pulldown_latex::config::DisplayMode::Inline
        };

        let config = ::pulldown_latex::config::RenderConfig {
            display_mode,
            annotation: if self.include_annotation {
                Some(latex)
            } else {
                None
            },
            ..Default::default()
        };

        let storage = ::pulldown_latex::Storage::new();
        let parser = ::pulldown_latex::Parser::new(latex, &storage);

        let mut mathml = String::new();
        match ::pulldown_latex::mathml::push_mathml(&mut mathml, parser, config) {
            Ok(()) => {
                if let Some(sp) = sourcepos {
                    // pulldown-latex always starts its output with `<math`. We inject
                    // the data-sourcepos attribute right after that opening token.
                    // If the output format ever changes, we fall back to writing
                    // the MathML without sourcepos rather than panicking.
                    if let Some(rest) = mathml.strip_prefix("<math") {
                        write!(output, "<math data-sourcepos=\"{sp}\"{rest}")
                    } else {
                        output.write_str(&mathml)
                    }
                } else {
                    output.write_str(&mathml)
                }
            }
            Err(_) => {
                // Graceful fallback: render as escaped text in a <span> with error class.
                output.write_str("<span class=\"math-error\">")?;
                crate::html::escape(output, latex)?;
                output.write_str("</span>")
            }
        }
    }
}
