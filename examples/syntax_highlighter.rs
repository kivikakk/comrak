//! This example shows how to implement a syntax highlighter plugin.

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
use std::collections::HashMap;
use std::io::{self, Write};

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
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        write!(
            output,
            "<span class=\"potato-{}\">{}</span><span class=\"size-{}\">potato</span>",
            lang.unwrap(),
            code,
            self.potato_size
        )
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("lang") {
            write!(output, "<pre lang=\"{}\">", attributes["lang"])
        } else {
            output.write_all(b"<pre>")
        }
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("class") {
            write!(output, "<code class=\"{}\">", attributes["class"])
        } else {
            output.write_all(b"<code>")
        }
    }
}

fn main() {
    let adapter = PotatoSyntaxAdapter::new(42);
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input = concat!("```Rust\n", "fn main<'a>();\n", "```");

    let formatted = markdown_to_html_with_plugins(input, &options, &plugins);

    println!("{}", formatted);
}
