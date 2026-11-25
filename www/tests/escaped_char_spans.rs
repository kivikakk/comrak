use std::path::PathBuf;

use comrak::markdown_to_html_with_plugins;
use comrak_www::{Doc, plugins};

const OPEN: &str = "<span data-escaped-char";
const CLOSE: &str = "</span>";

fn without_spans(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        rest = &rest[rest.find('>').unwrap() + 1..];
        let end = rest.find(CLOSE).unwrap();
        out.push_str(&rest[..end]);
        rest = &rest[end + CLOSE.len()..];
    }
    out.push_str(rest);
    out
}

#[test]
fn escaped_char_spans_only_add_spans() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs");
    let mut failures = vec![];
    for doc in Doc::load_all(&root) {
        for example in &doc.examples {
            let mut options = example.options().unwrap();
            options.render.escaped_char_spans = false;
            let plain = markdown_to_html_with_plugins(&example.input, &options, &plugins());
            options.render.escaped_char_spans = true;
            let spans = markdown_to_html_with_plugins(&example.input, &options, &plugins());
            if without_spans(&spans) != plain {
                failures.push(format!(
                    "{}:{}\n--- without\n{plain}--- with\n{spans}\n",
                    doc.path.display(),
                    example.line
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
