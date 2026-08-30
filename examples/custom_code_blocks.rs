use comrak::{
    Options, adapters::CodefenceBlockAdapter, markdown_to_html_with_plugins, nodes::Sourcepos,
    options::Plugins,
};
use std::fmt::{self, Write};

fn main() {
    let adapter = CustomCodeBlockAdapter;
    let options = Options::default();
    let mut plugins = Plugins::default();
    plugins.render.codefence_block_renderer = Some(&adapter);

    print_html(
        "Some prose.\n\n```rust\nfn main() {}\n```\n\nMore prose.",
        &options,
        &plugins,
    );

    print_html(
        "```rust title=\"hello.rs\"\nfn greet() {\n    println!(\"hi\");\n}\n```",
        &options,
        &plugins,
    );

    print_html("```\njust some plain code\n```", &options, &plugins);

    print_html("```sh\necho 'hello world'\n```", &options, &plugins);
}

struct CustomCodeBlockAdapter;

impl CodefenceBlockAdapter for CustomCodeBlockAdapter {
    fn render(
        &self,
        output: &mut dyn Write,
        lang: &str,
        meta: &str,
        literal: &str,
        sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let frame = match lang {
            "sh" | "bash" | "zsh" | "shell" | "console" => "terminal",
            _ => "editor",
        };

        write!(output, "<figure class=\"code-block code-block-{frame}\"")?;
        if let Some(sp) = sourcepos {
            write!(output, " data-sourcepos=\"{sp}\"")?;
        }
        output.write_str(">")?;

        let title = extract_title(meta).unwrap_or(lang);
        if !title.is_empty() {
            write!(output, "<figcaption class=\"code-block-title\">")?;
            escape(output, title)?;
            output.write_str("</figcaption>")?;
        }

        output.write_str("<pre><code")?;
        if !lang.is_empty() {
            write!(output, " class=\"language-{lang}\"")?;
        }
        output.write_str(">")?;
        escape(output, literal)?;
        output.write_str("</code></pre>")?;

        output.write_str("</figure>")
    }
}

fn extract_title(meta: &str) -> Option<&str> {
    let after = meta.strip_prefix("title=\"")?;
    let end = after.find('"')?;
    Some(&after[..end])
}

fn escape(output: &mut dyn Write, text: &str) -> fmt::Result {
    for ch in text.chars() {
        match ch {
            '&' => output.write_str("&amp;")?,
            '<' => output.write_str("&lt;")?,
            '>' => output.write_str("&gt;")?,
            '"' => output.write_str("&quot;")?,
            c => output.write_char(c)?,
        }
    }
    Ok(())
}

fn print_html(document: &str, options: &Options, plugins: &Plugins) {
    let html = markdown_to_html_with_plugins(document, options, plugins);
    println!("{}", html);
}
