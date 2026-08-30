#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{Options, markdown_to_html, options};

// Note that what I'm targeting here isn't exactly the same
// as --gfm, but rather an approximation of what cmark-gfm
// options are routinely used by Commonmarker users.

fuzz_target!(|s: &str| {
    let extension = options::Extension {
        strikethrough: true,
        #[allow(deprecated)]
        tagfilter: true,
        table: true,
        autolink: true,
        ..Default::default()
    };

    let render = options::Render {
        hardbreaks: true,
        github_pre_lang: true,
        r#unsafe: true,
        sourcepos: true,
        ..Default::default()
    };

    markdown_to_html(
        s,
        &Options {
            extension,
            parse: Default::default(),
            render,
        },
    );
});
