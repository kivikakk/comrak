#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions, ListStyleType,
};

fuzz_target!(|s: &str| {
    markdown_to_html(
        s,
        &ComrakOptions {
            extension: ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: true,
                table: true,
                autolink: true,
                tasklist: true,
                superscript: true,
                header_ids: Some("user-content-".to_string()),
                footnotes: true,
                description_lists: true,
                front_matter_delimiter: Some("---".to_string()),
                shortcodes: true,
            },
            parse: ComrakParseOptions {
                smart: true,
                default_info_string: Some("rust".to_string()),
                relaxed_tasklist_matching: true,
            },
            render: ComrakRenderOptions {
                hardbreaks: true,
                github_pre_lang: true,
                full_info_string: true,
                width: 80,
                unsafe_: true,
                escape: true,
                list_style: ListStyleType::Star,
                sourcepos: true,
            },
        },
    );
});
