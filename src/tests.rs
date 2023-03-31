use crate::nodes::{AstNode, NodeValue, Sourcepos};
use crate::*;
use std::collections::HashMap;
use std::io::{self, Write};
use std::panic;

mod api;
mod autolink;
mod core;
mod description_lists;
mod footnotes;
mod fuzz;
mod header_ids;
mod options;
mod plugins;
mod propfuzz;
mod regressions;
mod shortcodes;
mod strikethrough;
mod superscript;
mod table;
mod tagfilter;
mod tasklist;
mod xml;

#[track_caller]
fn compare_strs(output: &str, expected: &str, kind: &str) {
    if output != expected {
        println!("Running {} test", kind);
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
fn commonmark(input: &str, expected: &str, opts: Option<&ComrakOptions>) {
    let arena = Arena::new();
    let defaults = ComrakOptions::default();
    let options = opts.unwrap_or(&defaults);

    let root = parse_document(&arena, input, options);
    let mut output = vec![];
    cm::format_document(root, options, &mut output).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");
}

#[track_caller]
pub fn html(input: &str, expected: &str) {
    html_opts_i(input, expected, |_| ());
}

#[track_caller]
fn html_opts_i<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut ComrakOptions),
{
    let mut options = ComrakOptions::default();
    opts(&mut options);

    html_opts_w(input, expected, &options);
}

#[track_caller]
fn html_opts_w(input: &str, expected: &str, options: &ComrakOptions) {
    let arena = Arena::new();

    let root = parse_document(&arena, input, &options);
    let mut output = vec![];
    html::format_document(root, &options, &mut output).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");

    if options.render.sourcepos {
        return;
    }

    let mut md = vec![];
    cm::format_document(root, &options, &mut md).unwrap();
    let root = parse_document(&arena, &String::from_utf8(md).unwrap(), &options);
    let mut output_from_rt = vec![];
    html::format_document(root, &options, &mut output_from_rt).unwrap();
    compare_strs(
        &String::from_utf8(output_from_rt).unwrap(),
        expected,
        "roundtrip",
    );
}

macro_rules! html_opts {
    ([$($optclass:ident.$optname:ident),*], $lhs:expr, $rhs:expr,) => {
        html_opts!([$($optclass.$optname),*], $lhs, $rhs)
    };
    ([$($optclass:ident.$optname:ident),*], $lhs:expr, $rhs:expr) => {
        $crate::tests::html_opts_i($lhs, $rhs, |opts| {
            $(opts.$optclass.$optname = true;)*
        });
    };
    ([all], $lhs:expr, $rhs:expr) => {
        $crate::tests::html_opts_w($lhs, $rhs, &$crate::ComrakOptions {
            extension: $crate::ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: true,
                table: true,
                autolink: true,
                tasklist: true,
                superscript: true,
                header_ids: Some("user-content-".to_string()),
                footnotes: true,
                description_lists: true,
                front_matter_delimiter: Some("---".to_string()),
                shortcodes: true,
            },
            parse: $crate::ComrakParseOptions {
                smart: true,
                default_info_string: Some("rust".to_string()),
                relaxed_tasklist_matching: true,
            },
            render: $crate::ComrakRenderOptions {
                hardbreaks: true,
                github_pre_lang: true,
                full_info_string: true,
                width: 80,
                unsafe_: true,
                escape: true,
                list_style: $crate::ListStyleType::Star,
                sourcepos: true,
            },
        });
    }
}

pub(crate) use html_opts;

#[track_caller]
fn html_plugins(input: &str, expected: &str, plugins: &ComrakPlugins) {
    let arena = Arena::new();
    let options = ComrakOptions::default();

    let root = parse_document(&arena, input, &options);
    let mut output = vec![];
    html::format_document_with_plugins(root, &options, &mut output, plugins).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");

    if options.render.sourcepos {
        return;
    }

    let mut md = vec![];
    cm::format_document(root, &options, &mut md).unwrap();
    let root = parse_document(&arena, &String::from_utf8(md).unwrap(), &options);
    let mut output_from_rt = vec![];
    html::format_document_with_plugins(root, &options, &mut output_from_rt, plugins).unwrap();
    compare_strs(
        &String::from_utf8(output_from_rt).unwrap(),
        expected,
        "roundtrip",
    );
}

#[track_caller]
fn xml(input: &str, expected: &str) {
    xml_opts(input, expected, |_| ());
}

#[track_caller]
fn xml_opts<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut ComrakOptions),
{
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    opts(&mut options);

    let root = parse_document(&arena, input, &options);
    let mut output = vec![];
    crate::xml::format_document(root, &options, &mut output).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");

    if options.render.sourcepos {
        return;
    }

    let mut md = vec![];
    cm::format_document(root, &options, &mut md).unwrap();
    let root = parse_document(&arena, &String::from_utf8(md).unwrap(), &options);
    let mut output_from_rt = vec![];
    crate::xml::format_document(root, &options, &mut output_from_rt).unwrap();
    compare_strs(
        &String::from_utf8(output_from_rt).unwrap(),
        expected,
        "roundtrip",
    );
}

fn asssert_node_eq<'a>(node: &'a AstNode<'a>, location: &[usize], expected: &NodeValue) {
    let node = location
        .iter()
        .fold(node, |node, &n| node.children().nth(n).unwrap());

    let data = node.data.borrow();
    let actual = format!("{:?}", data.value);
    let expected = format!("{:?}", expected);

    compare_strs(&actual, &expected, "ast comparison");
}

macro_rules! sourcepos {
    (($spsl:literal:$spsc:literal-$spel:literal:$spec:literal)) => {
        ($spsl, $spsc, $spel, $spec).into()
    };
    ((XXX)) => {
        (0, 1, 0, 1).into()
    };
}

pub(crate) use sourcepos;

macro_rules! ast {
    (($name:tt $sp:tt)) => {
        ast!(($name $sp []))
    };
    (($name:tt $sp:tt $content:tt)) => {
        AstMatchTree {
            name: stringify!($name).to_string(),
            sourcepos: sourcepos!($sp),
            content: ast!($content),
        }
    };

    ($text:literal) => {AstMatchContent::Text($text.to_string())};
    ([ $( $children:tt )* ]) => {
        AstMatchContent::Children(vec![ $( ast!($children), )* ])
    };
}

pub(crate) use ast;

#[track_caller]
fn assert_ast_match_i<F>(md: &str, amt: AstMatchTree, opts: F)
where
    F: Fn(&mut ComrakOptions),
{
    let mut options = ComrakOptions::default();
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

        let mut output = vec![];
        format_xml(root, &options, &mut output).unwrap();
        eprintln!("{}", std::str::from_utf8(&output).unwrap());

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
    ([ $( $optclass:ident.$optname:ident ),* ], $( $md:literal )+, $amt:tt) => {
        crate::tests::assert_ast_match_i(
            concat!( $( $md ),+ ),
            ast!($amt),
            |#[allow(unused_variables)] opts| {$(opts.$optclass.$optname = true;)*},
        );
    };
}

pub(crate) use assert_ast_match;

struct AstMatchTree {
    name: String,
    sourcepos: Sourcepos,
    content: AstMatchContent,
}

enum AstMatchContent {
    Text(String),
    Children(Vec<AstMatchTree>),
}

impl AstMatchTree {
    #[track_caller]
    fn assert_match<'a>(&self, node: &'a AstNode<'a>) {
        let ast = node.data.borrow();
        assert_eq!(self.name, ast.value.xml_node_name(), "node type matches");
        assert_eq!(self.sourcepos, ast.sourcepos, "sourcepos are equal");

        match &self.content {
            AstMatchContent::Text(text) => {
                assert_eq!(
                    0,
                    node.children().count(),
                    "text node should have no children"
                );
                assert_eq!(
                    text,
                    ast.value.text().unwrap(),
                    "text node content should match"
                );
            }
            AstMatchContent::Children(children) => {
                assert_eq!(
                    children.len(),
                    node.children().count(),
                    "children count should match"
                );
                for (e, a) in children.iter().zip(node.children()) {
                    e.assert_match(a);
                }
            }
        }
    }
}
