use pretty_assertions::assert_eq;
use std::cell::Cell;

use super::*;

fn parse_stopping_on_call<'a>(
    arena: &'a Arena<'a>,
    input: &str,
    options: &Options,
    stop_at: usize,
) -> (Result<Node<'a>, ParseCancelled>, usize) {
    let calls = Cell::new(0);
    let result = try_parse_document(arena, input, options, &|| {
        calls.set(calls.get() + 1);
        calls.get() >= stop_at
    });
    (result, calls.get())
}

fn mixed_input() -> String {
    [
        include_str!("fixtures/alerts.md"),
        include_str!("fixtures/description_lists.md"),
        include_str!("fixtures/math_code.md"),
        include_str!("fixtures/math_dollars.md"),
        include_str!("fixtures/multiline_alerts.md"),
        include_str!("fixtures/multiline_blockquote.md"),
        include_str!("fixtures/wikilinks_title_after_pipe.md"),
        include_str!("fixtures/wikilinks_title_before_pipe.md"),
        "| a | b |\n|---|---|\n| [ ] c | d@e.com |\n\n",
        "- [x] www.example.com[^1] ^[inline *note*]\n  - é **ü** \"smart\" --- ok\n\n",
        "[^1]: The *footnote*.\n\n[ref]: /url 'title'\n\n[ref] and [missing]\n",
    ]
    .concat()
}

fn many_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.header_id_prefix = Some(String::new());
    options.extension.footnotes = true;
    options.extension.inline_footnotes = true;
    options.extension.description_lists = true;
    options.extension.multiline_block_quotes = true;
    options.extension.alerts = true;
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.extension.wikilinks_title_after_pipe = true;
    options.extension.underline = true;
    options.extension.spoiler = true;
    options.parse.smart = true;
    options.parse.relaxed_autolinks = true;
    options
}

#[test]
fn stops_block_parsing() {
    let input = "a\n\n".repeat(10_000);
    let arena = Arena::new();
    let (result, calls) = parse_stopping_on_call(&arena, &input, &Options::default(), 2);
    assert_eq!(result.unwrap_err(), ParseCancelled);
    assert_eq!(calls, 2);
    assert!(arena.len() < 200, "made {} nodes", arena.len());
}

#[test]
fn stops_inline_parsing() {
    let input = "a *b* ".repeat(20_000);
    let arena = Arena::new();
    let (result, calls) = parse_stopping_on_call(&arena, &input, &Options::default(), 2);
    assert_eq!(result.unwrap_err(), ParseCancelled);
    assert_eq!(calls, 2);
    assert!(arena.len() < 1_000, "made {} nodes", arena.len());
}

#[test]
fn stops_on_every_call() {
    let input = mixed_input();

    for sourcepos_chars in [false, true] {
        let mut options = many_options();
        options.parse.sourcepos_chars = sourcepos_chars;

        let arena = Arena::new();
        let (result, total) = parse_stopping_on_call(&arena, &input, &options, usize::MAX);
        result.unwrap();

        for stop_at in 1..=total {
            let arena = Arena::new();
            let (result, calls) = parse_stopping_on_call(&arena, &input, &options, stop_at);
            assert_eq!(
                result.unwrap_err(),
                ParseCancelled,
                "stopping on call {stop_at} of {total}"
            );
            assert_eq!(calls, stop_at);
        }

        let arena = Arena::new();
        let (result, calls) = parse_stopping_on_call(&arena, &input, &options, total + 1);
        result.unwrap();
        assert_eq!(calls, total);
    }
}

#[test]
fn output_unchanged_when_never_stopped() {
    let input = mixed_input();
    let mut options = many_options();
    options.render.sourcepos = true;

    let arena = Arena::new();
    let expected = parse_document(&arena, &input, &options);
    let actual = try_parse_document(&arena, &input, &options, &|| false).unwrap();

    let mut expected_xml = String::new();
    let mut actual_xml = String::new();
    format_xml(expected, &options, &mut expected_xml).unwrap();
    format_xml(actual, &options, &mut actual_xml).unwrap();
    assert_eq!(actual_xml, expected_xml);
}
