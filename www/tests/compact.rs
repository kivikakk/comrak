use std::path::PathBuf;

use comrak::markdown_to_html_with_plugins;
use comrak_www::{Doc, plugins};

const BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "legend",
    "li",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

fn only_newlines_removed(full: &str, compact: &str) -> bool {
    let mut compact = compact.chars().peekable();
    for c in full.chars() {
        if compact.peek() == Some(&c) {
            compact.next();
        } else if c != '\n' {
            return false;
        }
    }
    compact.next().is_none()
}

fn rendered_text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = if rest.starts_with("<!--") {
            rest.find("-->").unwrap() + 3
        } else {
            rest.find('>').unwrap() + 1
        };
        let name: String = rest[1..end]
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if BLOCK_TAGS.contains(&name.as_str()) {
            out.push(' ');
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn compact_output_only_removes_layout_newlines() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs");
    let mut failures = vec![];
    for doc in Doc::load_all(&root) {
        for example in &doc.examples {
            let mut options = example.options().unwrap();
            options.render.compact_html = false;
            let full = markdown_to_html_with_plugins(&example.input, &options, &plugins());
            options.render.compact_html = true;
            let compact = markdown_to_html_with_plugins(&example.input, &options, &plugins());
            if !only_newlines_removed(&full, &compact)
                || rendered_text(&full) != rendered_text(&compact)
            {
                failures.push(format!(
                    "{}:{}\n--- full\n{full}--- compact\n{compact}\n",
                    doc.path.display(),
                    example.line
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
