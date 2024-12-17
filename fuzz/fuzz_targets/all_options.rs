#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{
    markdown_to_html, BrokenLinkReference, ExtensionOptions, ListStyleType, Options, ParseOptions,
    RenderOptions, ResolvedReference,
};
use std::sync::Arc;

fuzz_target!(|s: &str| {
    let mut extension = ExtensionOptions::default();
    extension.strikethrough = true;
    extension.tagfilter = true;
    extension.table = true;
    extension.autolink = true;
    extension.tasklist = true;
    extension.superscript = true;
    extension.header_ids = Some("user-content-".to_string());
    extension.footnotes = true;
    extension.description_lists = true;
    extension.front_matter_delimiter = Some("---".to_string());
    extension.multiline_block_quotes = true;
    extension.math_dollars = true;
    extension.math_code = true;
    extension.shortcodes = true;
    extension.wikilinks_title_after_pipe = true;
    extension.wikilinks_title_before_pipe = true;
    extension.underline = true;
    extension.spoiler = true;
    extension.greentext = true;

    let mut parse = ParseOptions::default();
    parse.smart = true;
    parse.default_info_string = Some("rust".to_string());
    parse.relaxed_tasklist_matching = true;
    parse.relaxed_autolinks = true;
    let cb = |link_ref: BrokenLinkReference| {
        Some(ResolvedReference {
            url: link_ref.normalized.to_string(),
            title: link_ref.original.to_string(),
        })
    };
    parse.broken_link_callback = Some(Arc::new(cb));

    let mut render = RenderOptions::default();
    render.hardbreaks = true;
    render.github_pre_lang = true;
    render.full_info_string = true;
    render.width = 80;
    render.unsafe_ = true;
    render.escape = true;
    render.list_style = ListStyleType::Star;
    render.sourcepos = true;
    render.escaped_char_spans = true;
    render.ignore_setext = true;
    render.ignore_empty_links = true;
    render.gfm_quirks = true;
    render.prefer_fenced = true;
    render.tasklist_classes = true;

    markdown_to_html(
        s,
        &Options {
            extension,
            parse,
            render,
        },
    );
});
