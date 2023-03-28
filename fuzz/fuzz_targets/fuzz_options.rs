#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html, ComrakOptions};

#[derive(Debug, arbitrary::Arbitrary)]
struct FuzzInput<'s> {
    s: &'s str,
    opts: ComrakOptions,
}

fuzz_target!(|i: FuzzInput| {
    markdown_to_html(i.s, &i.opts);
});
