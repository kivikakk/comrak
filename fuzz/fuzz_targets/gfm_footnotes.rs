#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{markdown_to_html, options, Options};

// Note that what I'm targeting here isn't exactly the same
// as --gfm, but rather an approximation of what cmark-gfm
// options are routinely used by Commonmarker users.

fuzz_target!(|s: &str| {
    let extension = options::Extension::builder()
        .strikethrough(true)
        .table(true)
        .autolink(true)
        .footnotes(true)
        .build();

    let render = options::Render::builder()
        .hardbreaks(true)
        .github_pre_lang(true)
        .r#unsafe(true)
        .build();

    markdown_to_html(
        s,
        &Options::builder()
            .extension(extension)
            .render(render)
            .build(),
    );
});
