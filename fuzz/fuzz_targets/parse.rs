#![no_main]
use comrak::{parse_document, Arena, Options};
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    options: Options<'a>,
    markdown: &'a str,
}

fuzz_target!(|input: Input| {
    let arena = Arena::new();
    let _ = parse_document(&arena, input.markdown, &input.options);
});
