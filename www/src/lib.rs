use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use clap::Parser;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::cli::OptionArgs;
use comrak::nodes::{NodeHtmlBlock, NodeValue};
use comrak::options::Plugins;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{
    Arena, Node, Options, format_commonmark, format_html_with_plugins, format_xml,
    markdown_to_html_with_plugins, parse_document,
};
use yaml_rust::YamlLoader;

pub const SNAP: &str = "COMRAK_SNAP";

const GROUPS: [&str; 3] = ["extensions", "options", "combinations"];

static HIGHLIGHTER: LazyLock<SyntectAdapter> = LazyLock::new(|| SyntectAdapter::new(None));

#[derive(Debug)]
pub struct Doc {
    pub path: PathBuf,
    pub title: String,
    pub source: String,
    pub examples: Vec<Example>,
}

#[derive(Debug)]
pub struct Example {
    pub line: usize,
    pub flags: String,
    pub input: String,
    pub expected: String,
    expected_lines: Range<usize>,
}

pub fn page_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("---".into());
    options.extension.header_id_prefix = Some("".into());
    options.extension.footnotes = true;
    options.parse.smart = true;
    options
}

pub fn plugins() -> Plugins<'static> {
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&*HIGHLIGHTER);
    plugins
}

fn example_flags(node: Node<'_>) -> Option<String> {
    let NodeValue::CodeBlock(ref cb) = node.data().value else {
        return None;
    };
    let rest = cb.info.strip_prefix("example")?;
    (cb.fenced && (rest.is_empty() || rest.starts_with(' '))).then(|| rest.trim().to_string())
}

impl Doc {
    pub fn load_all(root: &Path) -> Vec<Doc> {
        let mut paths = vec![];
        collect_markdown(root, Path::new(""), &mut paths);
        paths.sort_by_cached_key(|path| (group_rank(path), path.clone()));
        paths
            .into_iter()
            .map(|path| Doc::load(root, path))
            .collect()
    }

    pub fn load(root: &Path, path: PathBuf) -> Doc {
        let source = fs::read_to_string(root.join(&path))
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));

        let arena = Arena::new();
        let document = parse_document(&arena, &source, &page_options());

        let mut title = None;
        let mut examples = vec![];
        for node in document.descendants() {
            if let NodeValue::FrontMatter(ref front_matter) = node.data().value {
                title = Some(parse_title(&path, front_matter));
            } else if let Some(flags) = example_flags(node) {
                examples.push(Example::from_node(&path, node, flags));
            }
        }

        Doc {
            title: title.unwrap_or_else(|| panic!("{}: no front matter", path.display())),
            path,
            source,
            examples,
        }
    }

    pub fn html_path(&self) -> PathBuf {
        self.path.with_extension("html")
    }

    pub fn failures(&self) -> Vec<String> {
        self.examples
            .iter()
            .filter_map(|example| {
                let at = format!("{}:{}", self.path.display(), example.line);
                match example.render() {
                    Ok(actual) if actual == example.expected => None,
                    Ok(actual) => Some(format!(
                        "{at}: output differs\n--- expected\n{}--- actual\n{actual}",
                        example.expected
                    )),
                    Err(e) => Some(format!("{at}: {e}")),
                }
            })
            .collect()
    }

    pub fn snap(&self) -> String {
        let mut lines: Vec<&str> = self.source.split_inclusive('\n').collect();
        let rendered: Vec<_> = self.examples.iter().map(Example::render).collect();
        for (example, actual) in self.examples.iter().zip(&rendered).rev() {
            if let Ok(actual) = actual {
                lines.splice(example.expected_lines.clone(), actual.split_inclusive('\n'));
            }
        }
        lines.concat()
    }

    pub fn render_page(&self) -> String {
        let mut options = page_options();
        options.render.r#unsafe = true;

        let arena = Arena::new();
        let document = parse_document(&arena, &self.source, &options);

        let mut examples = self.examples.iter();
        for node in document.descendants() {
            if example_flags(node).is_some() {
                node.data_mut().value = NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 6,
                    literal: examples.next().unwrap().html(),
                });
            }
        }

        let mut html = String::new();
        format_html_with_plugins(document, &options, &mut html, &plugins()).unwrap();
        html
    }
}

fn group_rank(path: &Path) -> usize {
    GROUPS
        .iter()
        .position(|group| path.parent() == Some(Path::new(group)))
        .unwrap_or_else(|| panic!("{}: directory not in GROUPS", path.display()))
}

fn collect_markdown(root: &Path, dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root.join(dir)).unwrap() {
        let entry = entry.unwrap();
        let path = dir.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            collect_markdown(root, &path, paths);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }
}

fn parse_title(path: &Path, front_matter: &str) -> String {
    let yaml: String = front_matter
        .lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .flat_map(|line| [line, "\n"])
        .collect();
    let docs = YamlLoader::load_from_str(&yaml)
        .unwrap_or_else(|e| panic!("{}: front matter: {e}", path.display()));
    docs[0]["title"]
        .as_str()
        .unwrap_or_else(|| panic!("{}: front matter has no title", path.display()))
        .to_string()
}

impl Example {
    fn from_node(path: &Path, node: Node<'_>, flags: String) -> Example {
        let data = node.data();
        let NodeValue::CodeBlock(ref cb) = data.value else {
            unreachable!()
        };
        let line = data.sourcepos.start.line;
        assert!(
            cb.closed && matches!(node.parent().unwrap().data().value, NodeValue::Document),
            "{}:{line}: examples must be closed, top-level blocks",
            path.display()
        );

        let lines: Vec<&str> = cb.literal.split_inclusive('\n').collect();
        let separator = lines
            .iter()
            .position(|l| *l == ".\n")
            .unwrap_or_else(|| panic!("{}:{line}: example has no `.` line", path.display()));

        Example {
            line,
            flags,
            input: lines[..separator].concat(),
            expected: lines[separator + 1..].concat(),
            expected_lines: line + separator + 1..line + lines.len(),
        }
    }

    fn options(&self) -> Result<Options<'static>, String> {
        let args = shell_words::split(&self.flags).map_err(|e| e.to_string())?;
        let option_args = OptionArgs::try_parse_from(std::iter::once("comrak".into()).chain(args))
            .map_err(|e| e.to_string())?;
        Ok(option_args.to_options())
    }

    pub fn render(&self) -> Result<String, String> {
        let mut html = markdown_to_html_with_plugins(&self.input, &self.options()?, &plugins());
        if !html.is_empty() && !html.ends_with('\n') {
            html.push('\n');
        }
        Ok(html)
    }

    fn formatted(&self, formatter: Formatter) -> String {
        let options = self.options().unwrap();
        let arena = Arena::new();
        let document = parse_document(&arena, &self.input, &options);
        let mut out = String::new();
        formatter(document, &options, &mut out).unwrap();
        out
    }

    fn html(&self) -> String {
        let views = [
            (
                "rendered",
                "",
                format!(
                    r#"<div class="example-rendered"><template shadowrootmode="open">
                        <link rel="stylesheet" href="/katex/katex.min.css">
                        <link rel="stylesheet" href="/reference.css">
                        <link rel="stylesheet" href="/syntax.css">
                        {}
                    </template></div>"#,
                    self.expected
                ),
            ),
            ("html", "", pre(&highlight("html", &self.expected))),
            (
                "commonmark",
                " -t commonmark",
                pre(&highlight("markdown", &self.formatted(format_commonmark))),
            ),
            (
                "xml",
                " -t xml",
                pre(&highlight("xml", &self.formatted(format_xml))),
            ),
        ];

        let mut invocation = String::from("$ comrak");
        if !self.flags.is_empty() {
            invocation.push(' ');
            invocation.push_str(&self.flags);
        }
        let mut invocation = escape(&invocation);
        let mut tabs = String::new();
        let mut panes = String::new();
        for (i, (view, flag, body)) in views.iter().enumerate() {
            let hidden = if i == 0 { "" } else { " hidden" };
            if !flag.is_empty() {
                invocation.push_str(&format!(
                    r#"<span class="example-flag" data-view="{view}" hidden>{flag}</span>"#
                ));
            }
            tabs.push_str(&format!(
                r#"<button type="button" data-view="{view}" aria-pressed="{}">{view}</button>"#,
                i == 0
            ));
            panes.push_str(&format!(
                r#"<div class="example-view" data-view="{view}"{hidden}>{body}</div>"#
            ));
        }

        format!(
            r#"
            <div class="example">
                <div class="example-header">
                    <pre class="example-invocation"><code>{invocation}</code></pre>
                    <div class="example-tabs" hidden>{tabs}</div>
                </div>

                <div class="example-panes">
                    <pre class="example-markdown"><code>{}</code></pre>
                    <div class="example-output">
                        {panes}
                    </div>
                </div>
            </div>"#,
            highlight("markdown", &self.input),
        )
    }
}

type Formatter = for<'a> fn(Node<'a>, &Options, &mut dyn std::fmt::Write) -> std::fmt::Result;

fn pre(code: &str) -> String {
    format!("<pre><code>{code}</code></pre>")
}

fn highlight(lang: &str, code: &str) -> String {
    let mut out = String::new();
    HIGHLIGHTER
        .write_highlighted(&mut out, Some(lang), code)
        .unwrap();
    out
}

pub fn escape(text: &str) -> String {
    let mut out = String::new();
    comrak::html::escape(&mut out, text).unwrap();
    out
}
