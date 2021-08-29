//! Adapter for the Syntect syntax highlighter plugin.

use adapters::SyntaxHighlighterAdapter;
use regex::Regex;
use std::collections::HashMap;
use strings::{build_opening_tag, extract_attributes_from_tag};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Debug, Copy, Clone)]
/// Syntect syntax highlighter plugin.
pub struct SyntectAdapter<'a> {
    theme: &'a str,
}

impl<'a> SyntectAdapter<'a> {
    /// Construct a new `SyntectAdapter` object and set the syntax highlighting theme.
    pub fn new(theme: &'a str) -> Self {
        SyntectAdapter { theme: &theme }
    }

    fn gen_empty_block(&self) -> String {
        let ss = SyntaxSet::load_defaults_newlines();
        let syntax = ss.find_syntax_by_name("Plain Text").unwrap();
        let ts = ThemeSet::load_defaults();

        highlighted_html_for_string("", &ss, syntax, &ts.themes[self.theme])
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
        let ss = SyntaxSet::load_defaults_newlines();
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

        let syntax = match ss.find_syntax_by_name(lang) {
            None => match ss.find_syntax_by_first_line(code) {
                Some(s) => s,
                None => ss.find_syntax_by_name(fallback_syntax).unwrap(),
            },
            Some(s) => s,
        };

        let ts = ThemeSet::load_defaults();

        self.remove_pre_tag(highlighted_html_for_string(
            code,
            &ss,
            syntax,
            &ts.themes[self.theme],
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
