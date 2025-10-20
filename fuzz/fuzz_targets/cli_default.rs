#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html_with_plugins, options, plugins::syntect::SyntectAdapter};

// Note that we end up fuzzing Syntect here.

fuzz_target!(|s: &str| {
    let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));

    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    markdown_to_html_with_plugins(s, &Default::default(), &plugins);
});
