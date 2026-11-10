#![no_main]
use comrak::nodes::NodeValue;
use comrak::{markdown_to_commonmark_xml, parse_document, Arena, Options};
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    options: Options<'a>,
    markdown: &'a str,
}

fuzz_target!(|input: Input| {
    let mut on = input.options;
    on.extension.grid_table = true;
    on.render.sourcepos = true;

    let arena = Arena::new();
    let root = parse_document(&arena, input.markdown, &on);
    if root
        .descendants()
        .any(|n| matches!(n.data().value, NodeValue::GridTable(..)))
    {
        return;
    }

    let mut off = on.clone();
    off.extension.grid_table = false;
    assert_eq!(
        markdown_to_commonmark_xml(input.markdown, &on),
        markdown_to_commonmark_xml(input.markdown, &off),
    );
});
