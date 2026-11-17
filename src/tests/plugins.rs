use pretty_assertions::assert_eq;
use std::borrow::Cow;

use crate::adapters::{
    CodefenceMeta, CodefenceRendererAdapter, HeadingAdapter, HeadingMeta, SyntaxHighlightMeta,
    SyntaxHighlighterAdapter,
};

use super::*;

#[test]
fn syntax_highlighter_plugin() {
    pub struct MockAdapter {}

    impl SyntaxHighlighterAdapter for MockAdapter {
        fn write_highlighted(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
        ) -> std::fmt::Result {
            let lang = highlight.lang;
            let code = highlight.code;
            write!(output, "<!--{}--><span>{}</span>", lang, code)
        }

        fn write_pre_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "pre", attributes)
        }

        fn write_code_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "code", attributes)
        }
    }

    let input = concat!("``` rust yum\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre><code class=\"language-rust\"><!--rust--><span>fn main<'a>();\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = options::Plugins::default();
    let adapter = MockAdapter {};
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
    html_plugins(
        concat!("```\n", "plain\n", "```\n"),
        "<pre><code><!----><span>plain\n</span></code></pre>\n",
        &plugins,
    );
}

#[test]
fn syntax_highlighter_tags_receive_meta() {
    struct MockAdapter;

    impl SyntaxHighlighterAdapter for MockAdapter {
        fn write_highlighted(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
        ) -> std::fmt::Result {
            output.write_str(highlight.code)
        }

        fn write_pre_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
            _attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            write!(
                output,
                "<pre data-pre=\"{}|{}\">",
                highlight.lang, highlight.meta
            )
        }

        fn write_code_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
            _attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            write!(output, "<code data-code=\"{}\">", highlight.lang)
        }
    }

    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&MockAdapter);

    html_plugins(
        concat!("``` rust yum\n", "x\n", "```\n"),
        "<pre data-pre=\"rust|yum\"><code data-code=\"rust\">x\n</code></pre>\n",
        &plugins,
    );
}

#[test]
fn language_specific_codefence_renderer_plugin() {
    struct MermaidAdapter;

    impl CodefenceRendererAdapter for MermaidAdapter {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            let lang = codefence.lang;
            let meta = codefence.meta;
            let code = codefence.code;
            writeln!(
                output,
                "<figure class=\"{lang}\" data-meta=\"{meta}\">{code}</figure>"
            )
        }
    }

    let input = concat!("``` mermaid theme=dark\n", "graph TD;\n", "```\n");
    let expected = "<figure class=\"mermaid\" data-meta=\"theme=dark\">graph TD;\n</figure>\n";

    let mut plugins = options::Plugins::default();
    let adapter = MermaidAdapter;
    plugins
        .render
        .codefence_renderers
        .insert("mermaid".to_string(), &adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn language_specific_codefence_renderer_precedes_highlighter() {
    struct MermaidAdapter;
    struct HighlighterAdapter;

    impl CodefenceRendererAdapter for MermaidAdapter {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            let code = codefence.code;
            writeln!(output, "<div>{code}</div>")
        }
    }

    impl SyntaxHighlighterAdapter for HighlighterAdapter {
        fn write_highlighted(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
        ) -> std::fmt::Result {
            let code = highlight.code;
            write!(output, "<span>{code}</span>")
        }

        fn write_pre_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "pre", attributes)
        }

        fn write_code_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "code", attributes)
        }
    }

    let input = concat!("```mermaid\n", "graph LR;\n", "```\n");
    let expected = "<div>graph LR;\n</div>\n";

    let mut plugins = options::Plugins::default();
    let codefence_renderer = MermaidAdapter;
    let syntax_highlighter = HighlighterAdapter;
    plugins
        .render
        .codefence_renderers
        .insert("mermaid".to_string(), &codefence_renderer);
    plugins.render.codefence_syntax_highlighter = Some(&syntax_highlighter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn language_specific_codefence_renderer_receives_sourcepos() {
    struct MermaidAdapter;

    impl CodefenceRendererAdapter for MermaidAdapter {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            let code = codefence.code;
            let sourcepos = codefence
                .sourcepos
                .expect("sourcepos should be passed to adapter");
            writeln!(
                output,
                "<figure data-sourcepos=\"{sourcepos}\">{code}</figure>"
            )
        }
    }

    let input = concat!("```mermaid\n", "graph TD;\n", "```\n");
    let expected = "<figure data-sourcepos=\"1:1-3:3\">graph TD;\n</figure>\n";

    let mut plugins = options::Plugins::default();
    let adapter = MermaidAdapter;
    plugins
        .render
        .codefence_renderers
        .insert("mermaid".to_string(), &adapter);

    let arena = Arena::new();
    let mut options = Options::default();
    options.render.sourcepos = true;
    let root = parse_document(&arena, input, &options);
    let mut output = String::new();
    html::format_document_with_plugins(root, &options, &mut output, &plugins).unwrap();
    compare_strs(&output, expected, "regular", input);
}

#[test]
fn syntax_highlighter_still_handles_unmatched_codefence_renderer() {
    struct UnusedCodefenceRenderer;
    struct HighlighterAdapter;

    impl CodefenceRendererAdapter for UnusedCodefenceRenderer {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            _codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            output.write_str("<div>should-not-run</div>\n")
        }
    }

    impl SyntaxHighlighterAdapter for HighlighterAdapter {
        fn write_highlighted(
            &self,
            output: &mut dyn std::fmt::Write,
            highlight: &SyntaxHighlightMeta<'_>,
        ) -> std::fmt::Result {
            let lang = highlight.lang;
            let code = highlight.code;
            write!(output, "<!--{}--><span>{code}</span>", lang)
        }

        fn write_pre_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "pre", attributes)
        }

        fn write_code_tag(
            &self,
            output: &mut dyn std::fmt::Write,
            _highlight: &SyntaxHighlightMeta<'_>,
            attributes: BTreeMap<&str, Cow<str>>,
        ) -> std::fmt::Result {
            html::write_opening_tag(output, "code", attributes)
        }
    }

    let input = concat!("``` mermaid theme=dark\n", "graph TD;\n", "```\n");
    let expected = concat!(
        "<pre><code class=\"language-mermaid\"><!--mermaid--><span>graph TD;\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = options::Plugins::default();
    let unused_renderer = UnusedCodefenceRenderer;
    let syntax_highlighter = HighlighterAdapter;
    plugins
        .render
        .codefence_renderers
        .insert("mermaid theme=dark".to_string(), &unused_renderer);
    plugins.render.codefence_syntax_highlighter = Some(&syntax_highlighter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn math_codefence_renderer_is_not_overridden_by_custom_renderer() {
    struct MathAdapter;

    impl CodefenceRendererAdapter for MathAdapter {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            _codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            output.write_str("<div>custom-math</div>\n")
        }
    }

    let input = concat!("```math\n", "x^2\n", "```\n");
    let expected =
        "<pre><code class=\"language-math\" data-math-style=\"display\">x^2\n</code></pre>\n";

    let mut plugins = options::Plugins::default();
    let adapter = MathAdapter;
    plugins
        .render
        .codefence_renderers
        .insert("math".to_string(), &adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn empty_info_codefence_does_not_use_custom_renderer() {
    struct EmptyLangAdapter;

    impl CodefenceRendererAdapter for EmptyLangAdapter {
        fn write(
            &self,
            output: &mut dyn std::fmt::Write,
            _codefence: &CodefenceMeta<'_>,
        ) -> std::fmt::Result {
            output.write_str("<div>should-not-run</div>\n")
        }
    }

    let input = concat!("```\n", "plain\n", "```\n");
    let expected = "<pre><code>plain\n</code></pre>\n";

    let mut plugins = options::Plugins::default();
    let adapter = EmptyLangAdapter;
    plugins
        .render
        .codefence_renderers
        .insert(String::new(), &adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn heading_adapter_receives_sourcepos() {
    struct MockAdapter;

    impl HeadingAdapter for MockAdapter {
        fn enter(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(
                output,
                "<h{} data-enter=\"{}\">",
                heading.level,
                heading.sourcepos.unwrap()
            )
        }

        fn exit(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(
                output,
                "</h{}><!-- {} -->",
                heading.level,
                heading.sourcepos.unwrap()
            )
        }
    }

    let mut plugins = options::Plugins::default();
    let adapter = MockAdapter;
    plugins.render.heading_adapter = Some(&adapter);
    let mut options = Options::default();
    options.render.sourcepos = true;

    assert_eq!(
        markdown_to_html_with_plugins("text\n\n## Heading\n", &options, &plugins),
        "<p data-sourcepos=\"1:1-1:4\">text</p>\n<h2 data-enter=\"3:1-3:10\">Heading</h2><!-- 3:1-3:10 -->"
    );
}

#[test]
fn heading_adapter_plugin() {
    struct MockAdapter;

    impl HeadingAdapter for MockAdapter {
        fn enter(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(output, "<h{} data-heading=\"true\">", heading.level + 1)
        }

        fn exit(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(output, "</h{}>", heading.level + 1)
        }
    }

    let mut plugins = options::Plugins::default();
    let adapter = MockAdapter {};
    plugins.render.heading_adapter = Some(&adapter);

    let cases: Vec<(&str, &str)> = vec![
        (
            "# Simple heading",
            "<h2 data-heading=\"true\">Simple heading</h2>",
        ),
        (
            "## Heading with **bold text** and `code`",
            "<h3 data-heading=\"true\">Heading with <strong>bold text</strong> and <code>code</code></h3>",
        ),
        (
            "###### Whoa, an h7!",
            "<h7 data-heading=\"true\">Whoa, an h7!</h7>",
        ),
        (
            "####### This is not a heading",
            "<p>####### This is not a heading</p>\n",
        ),
    ];
    for (input, expected) in cases {
        html_plugins(input, expected, &plugins);
    }
}

#[test]
fn heading_adapter_plugin_deep_nesting() {
    struct MockAdapter;

    impl HeadingAdapter for MockAdapter {
        fn enter(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(output, "<h{} data-heading=\"true\">", heading.level)
        }

        fn exit(
            &self,
            output: &mut dyn std::fmt::Write,
            heading: &HeadingMeta,
        ) -> std::fmt::Result {
            write!(output, "</h{}>", heading.level)
        }
    }

    let input = "# ".to_string() + &"*a _".repeat(60_000) + "x" + &"_ b*".repeat(60_000);

    let adapter = Box::leak(Box::new(MockAdapter {}));

    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(move || {
            let options = Options::default();
            let arena = Arena::new();
            let root = parse_document(&arena, &input, &options);
            let mut plugins = options::Plugins::default();
            plugins.render.heading_adapter = Some(adapter);
            let mut output = String::new();
            html::format_document_with_plugins(root, &options, &mut output, &plugins).unwrap();
            assert!(output.contains("<em>"));
        })
        .unwrap();
    handle.join().unwrap();
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin_with_base16_ocean_dark_theme() {
    let adapter = crate::plugins::syntect::SyntectAdapter::new(Some("base16-ocean.dark"));

    let cases: [(&str, &str); 2] = [
        (
            concat!("```rust\n", "fn main<'a>();\n", "```\n"),
            concat!(
                "<pre style=\"background-color:#2b303b;\"><code class=\"language-rust\">",
                "<span style=\"color:#b48ead;\">fn </span><span style=\"color:#8fa1b3;\">main</span><span style=\"color:#c0c5ce;\">",
                "&lt;</span><span style=\"color:#b48ead;\">&#39;a</span><span style=\"color:#c0c5ce;\">&gt;();\n</span>",
                "</code></pre>\n"
            ),
        ),
        (
            concat!("```rust,ignore\n", "fn main() {}\n", "```\n"),
            concat!(
                "<pre style=\"background-color:#2b303b;\"><code class=\"language-rust,ignore\">",
                "<span style=\"color:#b48ead;\">fn </span><span style=\"color:#8fa1b3;\">main</span><span style=\"color:#c0c5ce;\">() {}\n</span>",
                "</code></pre>\n"
            ),
        ),
    ];

    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    for (input, expected) in cases {
        html_plugins(input, expected, &plugins);
    }
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

    let mut plugins = options::Plugins::default();
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

    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin_keeps_pre_attributes() {
    let mut options = Options::default();
    options.render.github_pre_lang = true;
    options.render.full_info_string = true;
    options.render.sourcepos = true;
    let input = "```rust title=x\nfn main() {}\n```\n";

    let cases = [
        (
            crate::plugins::syntect::SyntectAdapter::new(None),
            "<pre class=\"syntax-highlighting\" data-meta=\"title=x\" data-sourcepos=\"1:1-3:3\" lang=\"rust\"><code>",
        ),
        (
            crate::plugins::syntect::SyntectAdapter::new(Some("base16-ocean.dark")),
            "<pre data-meta=\"title=x\" data-sourcepos=\"1:1-3:3\" lang=\"rust\" style=\"background-color:#2b303b;\"><code>",
        ),
    ];

    for (adapter, expected_prefix) in &cases {
        let mut plugins = options::Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(adapter);
        let html = crate::markdown_to_html_with_plugins(input, &options, &plugins);
        assert!(html.starts_with(expected_prefix), "{html}");
    }
}
