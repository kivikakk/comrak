//! This example shows how to use the bundled syntect plugin.

extern crate comrak;

use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

fn main() {
    let adapter = SyntectAdapter::new("base16-ocean.dark");
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input = concat!("```Rust\n", "fn main<'a>();\n", "```");

    let formatted = markdown_to_html_with_plugins(input, &options, &plugins);

    println!("{}", formatted);
}
