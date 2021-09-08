//! Adapter for the Syntect syntax highlighter plugin.

use adapters::SyntaxHighlighterAdapter;
use regex::Regex;
use std::collections::HashMap;
use strings::{build_opening_tag, extract_attributes_from_tag};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

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
        highlighted_html_for_string(
            "",
            &self.syntax_set,
            syntax,
            &self.theme_set.themes[self.theme],
        )
    }

    fn remove_pre_tag(&self, highlighted_code: String) -> String {
        let re: Regex = Regex::new("<pre[\\s]+.*?>").unwrap();

        re.replace_all(highlighted_code.as_str(), "")
            .to_string()
            .replace("</pre>", "")
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

        self.remove_pre_tag(highlighted_html_for_string(
            code,
            &self.syntax_set,
            syntax,
            &self.theme_set.themes[self.theme],
        ))
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
