#![no_main]
use libfuzzer_sys::fuzz_target;
use comrak::markdown_to_html;
use comrak::ComrakOptions;

fuzz_target!(|markdown: &str, options: &ComrakOptions| {
    markdown_to_html(markdown, options);
    // fuzzed code goes here
});
