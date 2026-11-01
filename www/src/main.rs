use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use comrak::{escape_commonmark_inline, markdown_to_html_with_plugins};
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
const DOCS_MARKER: &str = "{{docs}}";

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
    assert_eq!(
        index.matches(DOCS_MARKER).count(),
        1,
        "index.md must contain {DOCS_MARKER} exactly once"
    );
    let index = index.replace(DOCS_MARKER, &docs_list(&docs)?);
    fs::write(
        out_dir.join("index.html"),
        templates.page(
            "Comrak · Markdown parser and renderer",
            "Comrak",
            Path::new(""),
            &markdown_to_html_with_plugins(&index, &page_options(), &plugins()),
        ),
    )?;

    for doc in &docs {
        let html_path = doc.html_path();
        let title = format!("{} · Comrak", doc.title);
        let body = format!(
            "{DOC_HEAD}<nav><a href=\"/\">Comrak</a></nav>\n{}",
            doc.render_page()
        );
        let out = out_dir.join(&html_path);
        fs::create_dir_all(out.parent().unwrap())?;
        fs::write(out, templates.page(&title, &title, &html_path, &body))?;
    }

    Ok(())
}

fn docs_list(docs: &[Doc]) -> Result<String, Box<dyn Error>> {
    let mut list = String::new();
    let mut group = None;
    for doc in docs {
        let dir = doc.path.parent().unwrap();
        if group != Some(dir) {
            writeln!(
                list,
                "* {}",
                escape_commonmark_inline(dir.to_str().unwrap())
            )?;
            group = Some(dir);
        }
        writeln!(
            list,
            "  * [{}](/{})",
            escape_commonmark_inline(&doc.title),
            doc.html_path().display()
        )?;
    }
    Ok(list)
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
