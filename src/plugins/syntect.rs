//! Adapter for the Syntect syntax highlighter plugin.

use crate::adapters::SyntaxHighlighterAdapter;
use crate::html::build_opening_tag;
use crate::strings::extract_attributes_from_tag;
use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::html::{
    append_highlighted_html_for_styled_line, highlighted_html_for_string, IncludeBackground,
};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use syntect::Error;

#[derive(Debug)]
/// Syntect syntax highlighter plugin.
pub struct SyntectAdapter<'a> {
    theme: &'a str,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl<'a> SyntectAdapter<'a> {
    /// Construct a new `SyntectAdapter` object and set the syntax highlighting theme.
    pub fn new(theme: &'a str) -> Self {
        SyntectAdapter {
            theme: &theme,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    fn gen_empty_block(&self) -> String {
        let syntax = self.syntax_set.find_syntax_by_name("Plain Text").unwrap();
        match highlighted_html_for_string(
            "",
            &self.syntax_set,
            syntax,
            &self.theme_set.themes[self.theme],
        ) {
            Ok(empty_block) => empty_block,
            Err(_) => "".into(),
        }
    }

    fn highlight_html(&self, code: &str, syntax: &SyntaxReference) -> Result<String, Error> {
        // syntect::html::highlighted_html_for_string, without the opening/closing <pre>.
        let theme = &self.theme_set.themes[self.theme];
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut output = String::new();
        let bg = theme.settings.background.unwrap_or(Color::WHITE);

        for line in LinesWithEndings::from(code) {
            let regions = highlighter.highlight_line(line, &self.syntax_set)?;
            append_highlighted_html_for_styled_line(
                &regions[..],
                IncludeBackground::IfDifferent(bg),
                &mut output,
            )?;
        }
        Ok(output)
    }
}

impl SyntaxHighlighterAdapter for SyntectAdapter<'_> {
    fn highlight(&self, lang: Option<&str>, code: &str) -> String {
        let fallback_syntax = "Plain Text";

        let lang: &str = match lang {
            None => fallback_syntax,
            Some(l) => {
                if l.is_empty() {
                    fallback_syntax
                } else {
                    l
                }
            }
        };

        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| {
                self.syntax_set
                    .find_syntax_by_first_line(code)
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            });

        match self.highlight_html(code, syntax) {
            Ok(highlighted_code) => highlighted_code,
            Err(_) => code.into(),
        }
    }

    fn build_pre_tag(&self, attributes: &HashMap<String, String>) -> String {
        let mut syntect_attributes = extract_attributes_from_tag(self.gen_empty_block().as_str());

        for (comrak_attr, val) in attributes {
            let mut combined_attr: String = val.clone();

            if syntect_attributes.contains_key(comrak_attr.as_str()) {
                combined_attr = format!(
                    "{} {}",
                    syntect_attributes.remove(comrak_attr).unwrap(),
                    val
                );
            }

            syntect_attributes.insert(comrak_attr.clone(), combined_attr);
        }

        build_opening_tag("pre", &syntect_attributes)
    }

    fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String {
        build_opening_tag("code", attributes)
    }
}

#[derive(Debug, Default)]
/// A builder for [`SyntectAdapter`].
///
/// Allows customization of `Theme`, [`ThemeSet`], and [`SyntaxSet`].
pub struct SyntectAdapterBuilder<'a> {
    theme: Option<&'a str>,
    syntax_set: Option<SyntaxSet>,
    theme_set: Option<ThemeSet>,
}

impl<'a> SyntectAdapterBuilder<'a> {
    /// Creates a new empty [`SyntectAdapterBuilder`]
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the theme
    pub fn theme(mut self, s: &'a str) -> Self {
        self.theme.replace(s);
        self
    }

    /// Sets the syntax set
    pub fn syntax_set(mut self, s: SyntaxSet) -> Self {
        self.syntax_set.replace(s);
        self
    }

    /// Sets the theme set
    pub fn theme_set(mut self, s: ThemeSet) -> Self {
        self.theme_set.replace(s);
        self
    }

    /// Builds the [`SyntectAdapter`]. Default values:
    /// - `theme`: `InspiredGitHub`
    /// - `syntax_set`: [`SyntaxSet::load_defaults_newlines()`]
    /// - `theme_set`: [`ThemeSet::load_defaults()`]
    pub fn build(self) -> SyntectAdapter<'a> {
        SyntectAdapter {
            theme: self.theme.unwrap_or("InspiredGitHub"),
            syntax_set: self
                .syntax_set
                .unwrap_or_else(SyntaxSet::load_defaults_newlines),
            theme_set: self.theme_set.unwrap_or_else(ThemeSet::load_defaults),
        }
    }
}
