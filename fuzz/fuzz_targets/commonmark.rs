#![no_main]
use comrak::{markdown_to_commonmark, markdown_to_commonmark_xml, Options};
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    options: Options<'a>,
    markdown: &'a str,
}

fuzz_target!(|input: Input| {
    if input.options.render.width > 100 || input.options.render.ol_width > 100 {
        return;
    }
    let _ = markdown_to_commonmark(input.markdown, &input.options);
    let _ = markdown_to_commonmark_xml(input.markdown, &input.options);
});
