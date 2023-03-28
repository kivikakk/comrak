#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        markdown_to_html(s, &Default::default());
    }
});
