use crate::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    nodes::Sourcepos,
};

use super::*;

#[test]
fn syntax_highlighter_plugin() {
    pub struct MockAdapter {}

    impl SyntaxHighlighterAdapter for MockAdapter {
        fn write_highlighted(
            &self,
            output: &mut dyn Write,
            lang: Option<&str>,
            code: &str,
        ) -> io::Result<()> {
            write!(output, "<!--{}--><span>{}</span>", lang.unwrap(), code)
        }

        fn write_pre_tag(
            &self,
            output: &mut dyn Write,
            attributes: HashMap<String, String>,
        ) -> io::Result<()> {
            html::write_opening_tag(output, "pre", attributes)
        }

        fn write_code_tag(
            &self,
            output: &mut dyn Write,
            attributes: HashMap<String, String>,
        ) -> io::Result<()> {
            html::write_opening_tag(output, "code", attributes)
        }
    }

    let input = concat!("``` rust yum\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre><code class=\"language-rust\"><!--rust--><span>fn main<'a>();\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = Plugins::default();
    let adapter = MockAdapter {};
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn heading_adapter_plugin() {
    struct MockAdapter;

    impl HeadingAdapter for MockAdapter {
        fn enter(
            &self,
            output: &mut dyn Write,
            heading: &HeadingMeta,
            _sourcepos: Option<Sourcepos>,
        ) -> io::Result<()> {
            write!(output, "<h{} data-heading=\"true\">", heading.level + 1)
        }

        fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> io::Result<()> {
            write!(output, "</h{}>", heading.level + 1)
        }
    }

    let mut plugins = Plugins::default();
    let adapter = MockAdapter {};
    plugins.render.heading_adapter = Some(&adapter);

    let cases: Vec<(&str, &str)> = vec![
        ("# Simple heading", "<h2 data-heading=\"true\">Simple heading</h2>"),
        (
            "## Heading with **bold text** and `code`",
            "<h3 data-heading=\"true\">Heading with <strong>bold text</strong> and <code>code</code></h3>",
        ),
        ("###### Whoa, an h7!", "<h7 data-heading=\"true\">Whoa, an h7!</h7>"),
        ("####### This is not a heading", "<p>####### This is not a heading</p>\n")
    ];
    for (input, expected) in cases {
        html_plugins(input, expected, &plugins);
    }
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin_with_base16_ocean_dark_theme() {
    let adapter = crate::plugins::syntect::SyntectAdapter::new(Some("base16-ocean.dark"));

    let input = concat!("```rust\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre style=\"background-color:#2b303b;\"><code class=\"language-rust\">",
        "<span style=\"color:#b48ead;\">fn </span><span style=\"color:#8fa1b3;\">main</span><span style=\"color:#c0c5ce;\">",
        "&lt;</span><span style=\"color:#b48ead;\">&#39;a</span><span style=\"color:#c0c5ce;\">&gt;();\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin_with_css_classes() {
    let adapter = crate::plugins::syntect::SyntectAdapter::new(None);

    let input = concat!("```rust\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre class=\"syntax-highlighting\"><code class=\"language-rust\">",
        "<span class=\"source rust\"><span class=\"meta function rust\"><span class=\"meta function rust\"><span class=\"storage type function rust\">fn</span> </span><span class=\"entity name function rust\">main</span></span><span class=\"meta generic rust\"><span class=\"punctuation definition generic begin rust\">&lt;</span>",
        "<span class=\"storage modifier lifetime rust\">&#39;a</span><span class=\"punctuation definition generic end rust\">&gt;</span></span><span class=\"meta function rust\"><span class=\"meta function parameters rust\"><span class=\"punctuation section parameters begin rust\">(</span></span><span class=\"meta function rust\">",
        "<span class=\"meta function parameters rust\"><span class=\"punctuation section parameters end rust\">)</span></span></span></span><span class=\"punctuation terminator rust\">;</span>\n</span>",
        "</code></pre>\n",
    );

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin_with_prefixed_css_classes() {
    let prefix = Box::leak(Box::new("prefix-"));
    let adapter = crate::plugins::syntect::SyntectAdapterBuilder::new()
        .css_with_class_prefix(prefix)
        .build();

    let input = concat!("```rust\n", "fn main<'a>();\n", "```\n");

    let expected = concat!(
        "<pre class=\"syntax-highlighting\"><code class=\"language-rust\">",
        "<span class=\"prefix-source prefix-rust\"><span class=\"prefix-meta prefix-function prefix-rust\"><span class=\"prefix-meta prefix-function prefix-rust\"><span class=\"prefix-storage prefix-type prefix-function prefix-rust\">fn</span> </span><span class=\"prefix-entity prefix-name prefix-function prefix-rust\">main</span></span><span class=\"prefix-meta prefix-generic prefix-rust\"><span class=\"prefix-punctuation prefix-definition prefix-generic prefix-begin prefix-rust\">&lt;</span>",
        "<span class=\"prefix-storage prefix-modifier prefix-lifetime prefix-rust\">&#39;a</span><span class=\"prefix-punctuation prefix-definition prefix-generic prefix-end prefix-rust\">&gt;</span></span><span class=\"prefix-meta prefix-function prefix-rust\"><span class=\"prefix-meta prefix-function prefix-parameters prefix-rust\"><span class=\"prefix-punctuation prefix-section prefix-parameters prefix-begin prefix-rust\">(</span></span><span class=\"prefix-meta prefix-function prefix-rust\">",
        "<span class=\"prefix-meta prefix-function prefix-parameters prefix-rust\"><span class=\"prefix-punctuation prefix-section prefix-parameters prefix-end prefix-rust\">)</span></span></span></span><span class=\"prefix-punctuation prefix-terminator prefix-rust\">;</span>\n</span>",
        "</code></pre>\n",
    );

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}
