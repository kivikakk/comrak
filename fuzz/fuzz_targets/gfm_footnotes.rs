#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html, ExtensionOptions, Options, RenderOptions};

// Note that what I'm targetting here isn't exactly the same
// as --gfm, but rather an approximation of what cmark-gfm
// options are routinely used by Commonmarker users.

fuzz_target!(|s: &str| {
    let mut extension = ExtensionOptions::default();
    extension.strikethrough = true;
    extension.tagfilter = true;
    extension.table = true;
    extension.autolink = true;
    extension.footnotes = true;

    let mut render = RenderOptions::default();
    render.hardbreaks = true;
    render.github_pre_lang = true;
    render.unsafe_ = true;

    markdown_to_html(
        s,
        &Options {
            extension,
            parse: Default::default(),
            render,
        },
    );
});
