#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(doc) = std::str::from_utf8(data) {
        comrak_fuzz::sourcepos::assert_clean(doc);
    }
});
