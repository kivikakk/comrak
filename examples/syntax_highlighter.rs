//! This example shows how to implement a syntax highlighter plugin.

extern crate comrak;
extern crate syntect;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::{
    markdown_to_html_with_plugins, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakPlugins, ComrakRenderOptions,
};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct PotatoSyntaxAdapter {
    potato_size: i32,
}

impl PotatoSyntaxAdapter {
    pub fn new(potato_size: i32) -> Self {
        PotatoSyntaxAdapter { potato_size }
    }
}

impl SyntaxHighlighterAdapter for PotatoSyntaxAdapter {
    fn highlight(&self, lang: Option<&str>, code: &str) -> String {
        format!(
            "<span class=\"potato-{}\">{}</span><span class=\"size-{}\">potato</span>",
            lang.unwrap(),
            code,
            self.potato_size
        )
    }

    fn build_pre_tag(&self, attributes: &HashMap<String, String>) -> String {
        println!("build_pre_tag: {:?}", attributes);

        if attributes.contains_key("lang") {
            format!("<pre language=\"{}\">", attributes["lang"])
        } else {
            String::from("<pre>")
        }
    }

    fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String {
        println!("build_code_tag: {:?}", attributes);

        if attributes.contains_key("class") {
            format!("<code class=\"{}\">", attributes["class"])
        } else {
            String::from("<code>")
        }
    }
}

fn main() {
    let adapter = PotatoSyntaxAdapter::new(42);
    let options = ComrakOptions {
        extension: ComrakExtensionOptions::default(),
        parse: ComrakParseOptions::default(),
        render: ComrakRenderOptions {
            pre_lang_and_meta: true,
            ..Default::default()
        },
    };
    let mut plugins = ComrakPlugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input = concat!(
        "```rust showLineNumbers {1-3,5,12}\n",
        "fn main<'a>();\n",
        "```"
    );

    let formatted = markdown_to_html_with_plugins(input, &options, &plugins);

    println!("{}", formatted);
}
