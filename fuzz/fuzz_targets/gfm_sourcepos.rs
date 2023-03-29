#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions};

// Note that what I'm targetting here isn't exactly the same
// as --gfm, but rather an approximation of what cmark-gfm
// options are routinely used by Commonmarker users.

fuzz_target!(|s: &str| {
    markdown_to_html(
        s,
        &ComrakOptions {
            extension: ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: true,
                table: true,
                autolink: true,
                ..Default::default()
            },
            parse: Default::default(),
            render: ComrakRenderOptions {
                hardbreaks: true,
                github_pre_lang: true,
                unsafe_: true,
                sourcepos: true,
                ..Default::default()
            },
        },
    );
});
