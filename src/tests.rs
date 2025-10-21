use std::collections::HashMap;
use std::panic;

use crate::nodes::{AstNode, Node, NodeValue, Sourcepos};
use crate::options;
use crate::*;

mod alerts;
mod autolink;
mod cjk_friendly_emphasis;
mod commonmark;
mod core;
mod description_lists;
mod empty;
mod escape;
mod escaped_char_spans;
mod footnotes;
mod front_matter;
mod fuzz;
mod greentext;
mod header_ids;
#[path = "tests/html.rs"]
mod html_;
mod inline_footnotes;
mod math;
mod multiline_block_quotes;
#[path = "tests/options.rs"]
mod options_;
mod pathological;
mod plugins;
mod raw;
mod regressions;
mod rewriter;
mod shortcodes;
#[path = "tests/sourcepos.rs"]
mod sourcepos_;
mod spoiler;
mod strikethrough;
mod subscript;
mod supersubscript;
mod table;
mod tagfilter;
mod tasklist;
mod underline;
mod wikilinks;
mod xml;

#[track_caller]
fn compare_strs(output: &str, expected: &str, kind: &str, original_input: &str) {
    if output != expected {
        println!("Running {} test", kind);
        println!("Original Input:");
        println!("==============================");
        println!("{}", original_input);
        println!("==============================");
        println!("Got:");
        println!("==============================");
        println!("{}", output);
        println!("==============================");
        println!();
        println!("Expected:");
        println!("==============================");
        println!("{}", expected);
        println!("==============================");
        println!();
    }
    assert_eq!(output, expected);
}

#[track_caller]
fn commonmark(input: &str, expected: &str, opts: Option<&Options>) {
    let arena = Arena::new();
    let defaults = Options::default();
    let options = opts.unwrap_or(&defaults);

    let root = parse_document(&arena, input, options);
    let mut output = String::new();
    cm::format_document(root, options, &mut output).unwrap();
    compare_strs(&output, expected, "regular", input);
}

#[track_caller]
pub fn html(input: &str, expected: &str) {
    html_opts_i(input, expected, true, |_| ());
}

#[track_caller]
fn html_opts_i<'c, F>(input: &str, expected: &str, roundtrip: bool, opts: F)
where
    F: FnOnce(&mut Options<'c>),
{
    let mut options = Options::default();
    opts(&mut options);

    html_opts_w(input, expected, roundtrip, &options);
}

#[track_caller]
fn html_opts_w(input: &str, expected: &str, roundtrip: bool, options: &Options) {
    let arena = Arena::new();

    let root = parse_document(&arena, input, options);
    let mut output = String::new();
    html::format_document(root, options, &mut output).unwrap();
    compare_strs(&output, expected, "regular", input);

    if !roundtrip {
        return;
    }

    let mut md = String::new();
    cm::format_document(root, options, &mut md).unwrap();

    let root = parse_document(&arena, &md, options);
    let mut output_from_rt = String::new();
    html::format_document(root, options, &mut output_from_rt).unwrap();

    let expected_no_sourcepos = remove_sourcepos(expected);
    let actual_no_sourcepos = remove_sourcepos(&output_from_rt);

    compare_strs(
        &actual_no_sourcepos,
        &expected_no_sourcepos,
        "roundtrip",
        &md,
    );
}

fn remove_sourcepos(i: &str) -> String {
    const S: &str = " data-sourcepos=\"";

    let mut r = i.to_string();
    while let Some(start_ix) = r.find(S) {
        let end_offset = start_ix + S.len();
        let end_ix = r.get(start_ix + S.len()..).unwrap().find('"').unwrap();
        r.replace_range(start_ix..end_offset + end_ix + 1, "");
    }

    r
}

macro_rules! html_opts {
    ([$($optclass:ident.$optname:ident),*], $input:expr, $expected:expr) => {
        html_opts!([$($optclass.$optname),*], $input, $expected,)
    };
    ([$($optclass:ident.$optname:ident = $val:expr),*], $input:expr, $expected:expr) => {
        html_opts!([$($optclass.$optname = $val),*], $input, $expected,)
    };
    ([$($optclass:ident.$optname:ident),*], $input:expr, $expected:expr,) => {
        html_opts!([$($optclass.$optname),*], $input, $expected, roundtrip)
    };
    ([$($optclass:ident.$optname:ident = $val:expr),*], $input:expr, $expected:expr,) => {
        html_opts!([$($optclass.$optname = $val),*], $input, $expected, roundtrip)
    };
    ([$($optclass:ident.$optname:ident),*], $input:expr, $expected:expr, $rt:ident) => {
        html_opts!([$($optclass.$optname),*], $input, $expected, $rt,)
    };
    ([$($optclass:ident.$optname:ident = $val:expr),*], $input:expr, $expected:expr, $rt:ident) => {
        html_opts!([$($optclass.$optname = $val),*], $input, $expected, $rt,)
    };
    ([$($optclass:ident.$optname:ident),*], $input:expr, $expected:expr, roundtrip,) => {
        html_opts!([$($optclass.$optname = true),*], $input, $expected, roundtrip,)
    };
    ([$($optclass:ident.$optname:ident = $val:expr),*], $input:expr, $expected:expr, roundtrip,) => {
        $crate::tests::html_opts_i($input, $expected, true, |opts| {
            $(opts.$optclass.$optname = $val;)*
        });
    };
    ([$($optclass:ident.$optname:ident),*], $input:expr, $expected:expr, no_roundtrip,) => {
        html_opts!([$($optclass.$optname = true),*], $input, $expected, no_roundtrip,)
    };
    ([$($optclass:ident.$optname:ident = $val:expr),*], $input:expr, $expected:expr, no_roundtrip,) => {
        $crate::tests::html_opts_i($input, $expected, false, |opts| {
            $(opts.$optclass.$optname = $val;)*
        });
    };
}

pub(crate) use html_opts;

#[track_caller]
fn html_plugins(input: &str, expected: &str, plugins: &options::Plugins) {
    let arena = Arena::new();
    let options = Options::default();

    let root = parse_document(&arena, input, &options);
    let mut output = String::new();
    html::format_document_with_plugins(root, &options, &mut output, plugins).unwrap();
    compare_strs(&output, expected, "regular", input);

    let mut md = String::new();
    cm::format_document(root, &options, &mut md).unwrap();

    let root = parse_document(&arena, &md, &options);
    let mut output_from_rt = String::new();
    html::format_document_with_plugins(root, &options, &mut output_from_rt, plugins).unwrap();
    compare_strs(&output_from_rt, expected, "roundtrip", &md);
}

#[track_caller]
fn xml(input: &str, expected: &str) {
    xml_opts(input, expected, |_| ());
}

#[track_caller]
fn xml_opts<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut Options),
{
    let arena = Arena::new();
    let mut options = Options::default();
    opts(&mut options);

    let root = parse_document(&arena, input, &options);
    let mut output = String::new();
    crate::xml::format_document(root, &options, &mut output).unwrap();
    compare_strs(&output, expected, "regular", input);

    if options.render.sourcepos {
        return;
    }

    let mut md = String::new();
    cm::format_document(root, &options, &mut md).unwrap();

    let root = parse_document(&arena, &md, &options);
    let mut output_from_rt = String::new();
    crate::xml::format_document(root, &options, &mut output_from_rt).unwrap();
    compare_strs(&output_from_rt, expected, "roundtrip", &md);
}

fn asssert_node_eq<'a>(node: Node<'a>, location: &[usize], expected: &NodeValue) {
    let node = location
        .iter()
        .fold(node, |node, &n| node.children().nth(n).unwrap());

    let data = node.data();
    let actual = format!("{:?}", data.value);
    let expected = format!("{:?}", expected);

    compare_strs(&actual, &expected, "ast comparison", "ast node");
}

macro_rules! sourcepos {
    (($spsl:literal:$spsc:literal-$spel:literal:$spec:literal)) => {
        $crate::nodes::Sourcepos {
            start: $crate::nodes::LineColumn {
                line: $spsl,
                column: $spsc,
            },
            end: $crate::nodes::LineColumn {
                line: $spel,
                column: $spec,
            },
        }
    };
    ((XXX)) => {
        $crate::tests::sourcepos!((0:1-0:1))
    };
}

pub(crate) use sourcepos;

macro_rules! ast {
    (($name:tt $sp:tt $( $content:tt )*)) => {
        AstMatchTree {
            name: stringify!($name).to_string(),
            sourcepos: sourcepos!($sp),
            matches: vec![ $( ast_content!($content), )* ],
        }
    };
}

macro_rules! ast_content {
    ($text:literal) => {AstMatchContent::Text($text.to_string())};
    ([ $( $children:tt )* ]) => {
        AstMatchContent::Children(vec![ $( ast!($children), )* ])
    };
}

pub(crate) use ast;
pub(crate) use ast_content;

#[track_caller]
fn assert_ast_match_i<F>(md: &str, amt: AstMatchTree, opts: F)
where
    F: Fn(&mut Options),
{
    let mut options = Options::default();
    options.render.sourcepos = true;
    opts(&mut options);

    let result = panic::catch_unwind(|| {
        let arena = Arena::new();
        let root = parse_document(&arena, md, &options);

        amt.assert_match(root);
    });

    if let Err(err) = result {
        let arena = Arena::new();
        let root = parse_document(&arena, md, &options);

        let mut output = String::new();
        format_xml(root, &options, &mut output).unwrap();
        eprintln!("{output}");

        panic::resume_unwind(err)
    }
}

macro_rules! assert_ast_match {
    ([ $( $optclass:ident.$optname:ident ),* ], $( $md:literal )+, $amt:tt,) => {
        assert_ast_match!(
            [ $( $optclass.$optname ),* ],
            $( $md )+,
            $amt
        )
    };
    ([ $( $optclass:ident.$optname:ident = $val:expr ),* ], $( $md:literal )+, $amt:tt) => {
        crate::tests::assert_ast_match_i(
            concat!( $( $md ),+ ),
            ast!($amt),
            |#[allow(unused_variables)] opts| {$(opts.$optclass.$optname = $val;)*},
        );
    };
    ([ $( $optclass:ident.$optname:ident ),* ], $( $md:literal )+, $amt:tt) => {
        assert_ast_match!(
            [ $( $optclass.$optname  = true),* ],
            $( $md )+,
            $amt
        )
    };
}

pub(crate) use assert_ast_match;

struct AstMatchTree {
    name: String,
    sourcepos: Sourcepos,
    matches: Vec<AstMatchContent>,
}

enum AstMatchContent {
    Text(String),
    Children(Vec<AstMatchTree>),
}

impl AstMatchTree {
    #[track_caller]
    fn assert_match<'a>(&self, node: Node<'a>) {
        let ast = node.data();
        assert_eq!(self.name, ast.value.xml_node_name(), "node type matches");
        assert_eq!(self.sourcepos, ast.sourcepos, "sourcepos are equal");

        let mut asserted_text = false;
        let mut asserted_children = false;

        for m in &self.matches {
            match m {
                AstMatchContent::Text(text) => match ast.value {
                    NodeValue::Math(ref nm) => {
                        assert_eq!(text, &nm.literal, "Math literal should match");
                        asserted_text = true;
                    }
                    NodeValue::CodeBlock(ref ncb) => {
                        assert_eq!(text, &ncb.literal, "CodeBlock literal should match");
                        asserted_text = true;
                    }
                    NodeValue::Text(ref nt) => {
                        assert_eq!(text, nt, "Text content should match");
                        asserted_text = true;
                    }
                    NodeValue::Link(ref nl) => {
                        assert_eq!(text, &nl.url, "Link destination should match");
                        asserted_text = true;
                    }
                    NodeValue::Image(ref ni) => {
                        assert_eq!(text, &ni.url, "Image source should match");
                        asserted_text = true;
                    }
                    NodeValue::FrontMatter(ref nfm) => {
                        assert_eq!(text, nfm, "Front matter content should match");
                        asserted_text = true;
                    }
                    _ => panic!(
                        "no text content matcher for this node type: {:?}",
                        ast.value
                    ),
                },
                AstMatchContent::Children(children) => {
                    assert_eq!(
                        children.len(),
                        node.children().count(),
                        "children count should match"
                    );
                    for (e, a) in children.iter().zip(node.children()) {
                        e.assert_match(a);
                    }
                    asserted_children = true;
                }
            }
        }

        assert!(
            asserted_children || node.children().count() == 0,
            "children were not asserted"
        );
        assert!(
            asserted_text
                || !matches!(
                    ast.value,
                    NodeValue::Math(_)
                        | NodeValue::CodeBlock(_)
                        | NodeValue::Text(_)
                        | NodeValue::Link(_)
                        | NodeValue::Image(_)
                        | NodeValue::FrontMatter(_)
                ),
            "text wasn't asserted"
        );
    }
}
