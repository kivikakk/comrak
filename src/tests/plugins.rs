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

    let mut plugins = ComrakPlugins::default();
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

    let mut plugins = ComrakPlugins::default();
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
fn syntect_plugin() {
    let adapter = crate::plugins::syntect::SyntectAdapter::new("base16-ocean.dark");

    let input = concat!("```rust\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre style=\"background-color:#2b303b;\"><code class=\"language-rust\">",
        "<span style=\"color:#b48ead;\">fn </span><span style=\"color:#8fa1b3;\">main</span><span style=\"color:#c0c5ce;\">",
        "&lt;</span><span style=\"color:#b48ead;\">&#39;a</span><span style=\"color:#c0c5ce;\">&gt;();\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}
