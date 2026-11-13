use std::collections::BTreeSet;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use clap::Parser;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::cli::OptionArgs;
use comrak::nodes::{NodeHtmlBlock, NodeValue};
use comrak::options::{BrokenLinkReference, Plugins};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{
    Anchorizer, Arena, Node, Options, ResolvedReference, format_commonmark,
    format_html_with_plugins, format_xml, markdown_to_html_with_plugins, parse_document,
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
    terms: BTreeSet<String>,
}

#[derive(Debug)]
pub struct Example {
    pub line: usize,
    pub number: usize,
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

fn is_definition(node: Node<'_>) -> bool {
    matches!(node.data().value, NodeValue::Link(ref link) if link.url == "@")
}

fn term_slug(term: &str) -> String {
    Anchorizer::new().anchorize(&term.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn visible_tabs(text: &str) -> String {
    text.replace('\t', "→")
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
        let mut terms = BTreeSet::new();
        let mut headings = Anchorizer::new();
        let mut heading_ids = BTreeSet::new();
        for node in document.descendants() {
            if let NodeValue::FrontMatter(ref front_matter) = node.data().value {
                title = Some(parse_title(&path, front_matter));
            } else if let NodeValue::Heading(_) = node.data().value {
                heading_ids.insert(headings.anchorize(&node.collect_text()));
            } else if is_definition(node) {
                let slug = term_slug(&node.collect_text());
                assert!(
                    terms.insert(slug.clone()),
                    "{}:{}: term `{slug}` defined twice",
                    path.display(),
                    node.data().sourcepos.start.line
                );
            } else if let Some(flags) = example_flags(node) {
                examples.push(Example::from_node(&path, node, examples.len() + 1, flags));
            }
        }

        for id in terms
            .iter()
            .cloned()
            .chain(examples.iter().map(Example::id))
        {
            assert!(
                !heading_ids.contains(&id),
                "{}: `#{id}` is also a heading's id",
                path.display()
            );
        }

        Doc {
            title: title.unwrap_or_else(|| panic!("{}: no front matter", path.display())),
            path,
            source,
            examples,
            terms,
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
        let rendered: Vec<_> = self
            .examples
            .iter()
            .map(|example| example.render().map(|actual| visible_tabs(&actual)))
            .collect();
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
        options.parse.broken_link_callback = Some(Arc::new(|reference: BrokenLinkReference| {
            let slug = term_slug(reference.normalized);
            self.terms.contains(&slug).then(|| ResolvedReference {
                url: format!("#{slug}"),
                title: String::new(),
            })
        }));

        let arena = Arena::new();
        let document = parse_document(&arena, &self.source, &options);

        let mut examples = self.examples.iter();
        let mut definitions = vec![];
        for node in document.descendants() {
            if example_flags(node).is_some() {
                node.data_mut().value = NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 6,
                    literal: examples.next().unwrap().html(),
                });
            } else if is_definition(node) {
                definitions.push(node);
            }
        }

        for node in definitions {
            let slug = term_slug(&node.collect_text());
            match node.data_mut().value {
                NodeValue::Link(ref mut link) => link.url = format!("#{slug}"),
                _ => unreachable!(),
            }
            node.insert_before(
                arena.alloc(NodeValue::HtmlInline(format!(r#"<dfn id="{slug}">"#)).into()),
            );
            node.insert_after(arena.alloc(NodeValue::HtmlInline("</dfn>".into()).into()));
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
    fn from_node(path: &Path, node: Node<'_>, number: usize, flags: String) -> Example {
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
            number,
            flags,
            input: lines[..separator].concat().replace('→', "\t"),
            expected: lines[separator + 1..].concat().replace('→', "\t"),
            expected_lines: line + separator + 1..line + lines.len(),
        }
    }

    pub fn id(&self) -> String {
        format!("example-{}", self.number)
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
            (
                "html",
                "",
                pre(&highlight("html", &visible_tabs(&self.expected))),
            ),
            (
                "commonmark",
                " -t commonmark",
                pre(&highlight(
                    "markdown",
                    &visible_tabs(&self.formatted(format_commonmark)),
                )),
            ),
            (
                "xml",
                " -t xml",
                pre(&highlight(
                    "xml",
                    &visible_tabs(&self.formatted(format_xml)),
                )),
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
            r##"
            <div class="example" id="{id}">
                <div class="example-header">
                    <a class="example-number" href="#{id}">Example {number}</a>
                    <pre class="example-invocation"><code>{invocation}</code></pre>
                    <div class="example-tabs" hidden>{tabs}</div>
                </div>

                <div class="example-panes">
                    <pre class="example-markdown"><code>{}</code></pre>
                    <div class="example-output">
                        {panes}
                    </div>
                </div>
            </div>"##,
            highlight("markdown", &visible_tabs(&self.input)),
            id = self.id(),
            number = self.number,
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
