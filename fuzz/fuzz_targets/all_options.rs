#![no_main]

use libfuzzer_sys::fuzz_target;

use comrak::{
    markdown_to_html, BrokenLinkReference, ExtensionOptions, ListStyleType, Options, ParseOptions,
    RenderOptions, ResolvedReference,
};
use std::sync::Arc;

fuzz_target!(|s: &str| {
    let url_rewriter = |input: &str| format!("{input}#rewritten");
    let extension = ExtensionOptions {
        strikethrough: true,
        tagfilter: true,
        table: true,
        autolink: true,
        tasklist: true,
        superscript: true,
        header_ids: Some("user-content-".to_string()),
        footnotes: true,
        inline_footnotes: true,
        description_lists: true,
        front_matter_delimiter: Some("---".to_string()),
        multiline_block_quotes: true,
        alerts: true,
        math_dollars: true,
        math_code: true,
        shortcodes: true,
        wikilinks_title_after_pipe: true,
        wikilinks_title_before_pipe: true,
        underline: true,
        subscript: true,
        spoiler: true,
        greentext: true,
        image_url_rewriter: Some(Arc::new(url_rewriter)),
        link_url_rewriter: Some(Arc::new(url_rewriter)),
        cjk_friendly_emphasis: true,
    };

    let cb = |link_ref: BrokenLinkReference| {
        Some(ResolvedReference {
            url: link_ref.normalized.to_string(),
            title: link_ref.original.to_string(),
        })
    };
    let parse = ParseOptions {
        smart: true,
        default_info_string: Some("rust".to_string()),
        relaxed_tasklist_matching: true,
        relaxed_autolinks: true,
        broken_link_callback: Some(Arc::new(cb)),
        ignore_setext: true,
    };

    let render = RenderOptions {
        hardbreaks: true,
        github_pre_lang: true,
        full_info_string: true,
        width: 80,
        unsafe_: true,
        escape: true,
        list_style: ListStyleType::Star,
        sourcepos: true,
        escaped_char_spans: true,
        ignore_empty_links: true,
        gfm_quirks: true,
        prefer_fenced: true,
        figure_with_caption: true,
        tasklist_classes: true,
        ol_width: 3,
        experimental_minimize_commonmark: true,
    };

    markdown_to_html(
        s,
        &Options {
            extension,
            parse,
            render,
        },
    );
});
