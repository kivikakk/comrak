#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html, Options};

#[derive(Debug, arbitrary::Arbitrary)]
struct FuzzInput<'s> {
    s: &'s str,
    opts: Options<'s>,
}

fuzz_target!(|i: FuzzInput| {
    markdown_to_html(i.s, &i.opts);
});
