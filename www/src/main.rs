use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use comrak::markdown_to_html_with_plugins;
use comrak_www::{Doc, SNAP, escape, page_options, plugins};
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style};

const ASSETS: &[&str] = &[
    "styles.css",
    "reference.css",
    "examples.js",
    "comrak.png",
    "favicon.png",
];
const DOC_HEAD: &str = r#"
    <link rel="stylesheet" href="/katex/katex.min.css">
    <script defer src="/katex/katex.min.js"></script>
    <script defer src="/examples.js"></script>
    <noscript><style>.example-view[hidden] { display: block; }</style></noscript>
"#;

struct Templates {
    header: String,
    footer: String,
}

impl Templates {
    fn page(&self, title: &str, og_title: &str, path: &Path, body: &str) -> String {
        let header = self
            .header
            .replace("{{title}}", &escape(title))
            .replace("{{og_title}}", &escape(og_title))
            .replace("{{path}}", &escape(path.to_str().unwrap()));
        format!("{header}{body}{}", self.footer)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let [_, docs_dir, www_dir, light_theme, dark_theme, out_dir]: [PathBuf; 6] =
        std::env::args_os()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| {
                eprintln!("usage: comrak-www DOCS WWW LIGHT-THEME DARK-THEME OUT");
                process::exit(2);
            });

    let docs = Doc::load_all(&docs_dir);

    let failures: Vec<String> = docs.iter().flat_map(Doc::failures).collect();
    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("{failure}");
        }
        eprintln!("run `{SNAP}=1 cargo test -p comrak-www` to update expected");
        process::exit(1);
    }

    let templates = Templates {
        header: fs::read_to_string(www_dir.join("header.html"))?,
        footer: fs::read_to_string(www_dir.join("footer.html"))?,
    };

    fs::create_dir_all(&out_dir)?;
    for asset in ASSETS {
        fs::copy(www_dir.join(asset), out_dir.join(asset))?;
    }
    fs::write(
        out_dir.join("syntax.css"),
        syntax_css(&light_theme, &dark_theme)?,
    )?;

    let index = fs::read_to_string(www_dir.join("index.md"))?;
    fs::write(
        out_dir.join("index.html"),
        templates.page(
            "Comrak · Markdown parser and renderer",
            "Comrak",
            Path::new(""),
            &markdown_to_html_with_plugins(&index, &page_options(), &plugins()),
        ),
    )?;

    let landing = fs::read_to_string(www_dir.join("docs.md"))?;
    let title = "Docs · Comrak";
    let body = format!(
        "<nav class=\"breadcrumb\"><a href=\"/\">Comrak</a> / Docs</nav>\n{}{}",
        markdown_to_html_with_plugins(&landing, &page_options(), &plugins()),
        docs_nav(&docs, None)?,
    );
    fs::write(
        out_dir.join("docs.html"),
        templates.page(title, title, Path::new("docs.html"), &body),
    )?;

    for (i, doc) in docs.iter().enumerate() {
        let html_path = doc.html_path();
        let title = format!("{} · Comrak", doc.title);
        let prev = i.checked_sub(1).map(|i| &docs[i]);
        let next = docs.get(i + 1);
        let body = format!(
            r#"{DOC_HEAD}<div class="docs-layout">
{}<main>
<nav class="breadcrumb"><a href="/">Comrak</a> / <a href="/docs.html">Docs</a> / {}</nav>
{}{}</main>
</div>
"#,
            docs_nav(&docs, Some(doc))?,
            escape(&doc.title),
            doc.render_page(),
            pager(prev, next),
        );
        let out = out_dir.join(&html_path);
        fs::create_dir_all(out.parent().unwrap())?;
        fs::write(out, templates.page(&title, &title, &html_path, &body))?;
    }

    Ok(())
}

fn docs_nav(docs: &[Doc], current: Option<&Doc>) -> Result<String, Box<dyn Error>> {
    let mut nav = String::from("<nav class=\"docs-nav\"><ul>\n");
    for group in docs.chunk_by(|a, b| a.path.parent() == b.path.parent()) {
        let dir = group[0].path.parent().unwrap().to_str().unwrap();
        writeln!(nav, "<li>{}<ul>", escape(dir))?;
        for doc in group {
            let aria_current = if current.is_some_and(|current| current.path == doc.path) {
                r#" aria-current="page""#
            } else {
                ""
            };
            writeln!(
                nav,
                r#"<li><a href="{}"{aria_current}>{}</a></li>"#,
                href(doc),
                escape(&doc.title)
            )?;
        }
        writeln!(nav, "</ul></li>")?;
    }
    nav.push_str("</ul></nav>\n");
    Ok(nav)
}

fn pager(prev: Option<&Doc>, next: Option<&Doc>) -> String {
    let prev = prev.map_or(String::new(), |doc| {
        format!(
            r#"<a rel="prev" href="{}">← {}</a>"#,
            href(doc),
            escape(&doc.title)
        )
    });
    let next = next.map_or(String::new(), |doc| {
        format!(
            r#"<a rel="next" href="{}">{} →</a>"#,
            href(doc),
            escape(&doc.title)
        )
    });
    format!("<nav class=\"docs-pager\">{prev}{next}</nav>\n")
}

fn href(doc: &Doc) -> String {
    format!("/{}", doc.html_path().display())
}

fn syntax_css(light: &Path, dark: &Path) -> Result<String, Box<dyn Error>> {
    let css = |path: &Path| -> Result<String, Box<dyn Error>> {
        let css = css_for_theme_with_class_style(&ThemeSet::get_theme(path)?, ClassStyle::Spaced)?;
        let (_, body) = css.split_once("*/\n").unwrap();
        Ok(body.replacen(".code {", ".syntax-highlighting {", 1))
    };
    let (light, dark) = (css(light)?, css(dark)?);
    let (light_text, light_colours) = split_colours(&light);
    let (dark_text, dark_colours) = split_colours(&dark);
    assert_eq!(
        light_text, dark_text,
        "light and dark syntax themes must differ only in colours"
    );

    let mut merged = String::new();
    for ((text, light), dark) in light_text.iter().zip(&light_colours).zip(&dark_colours) {
        write!(merged, "{text}light-dark({light}, {dark})")?;
    }
    merged.push_str(light_text.last().unwrap());
    Ok(merged)
}

fn split_colours(css: &str) -> (Vec<&str>, Vec<&str>) {
    let mut text = vec![];
    let mut colours = vec![];
    let mut rest = css;
    while let Some(start) = rest.find('#') {
        let end = rest[start + 1..]
            .find(|c: char| !c.is_ascii_hexdigit())
            .map_or(rest.len(), |n| start + 1 + n);
        text.push(&rest[..start]);
        colours.push(&rest[start..end]);
        rest = &rest[end..];
    }
    text.push(rest);
    (text, colours)
}
