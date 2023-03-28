use crate::nodes::{AstNode, NodeValue};
use crate::*;
use std::collections::HashMap;
use std::io::{self, Write};

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
        crate::tests::html_opts_i($lhs, $rhs, |opts| {
            $(opts.$optclass.$optname = true;)*
        });
    };
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
