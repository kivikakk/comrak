//! This example shows how to use the bundled syntect plugin.

use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, Options, Plugins};

fn main() {
    let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
    let options = Options::default();
    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input = concat!("```Rust\n", "fn main<'a>();\n", "```");

    let formatted = markdown_to_html_with_plugins(input, &options, &plugins);

    println!("{}", formatted);
}
