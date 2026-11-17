#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

use comrak::{
    markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html, options, Options,
    ResolvedReference,
};

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    options: Options<'a>,
    header_id_prefix: bool,
    front_matter_delimiter: bool,
    image_url_rewriter: bool,
    link_url_rewriter: bool,
    default_info_string: bool,
    broken_link_callback: bool,
    markdown: String,
}

fuzz_target!(|input: Input| {
    let url_rewriter: Arc<dyn options::URLRewriter> =
        Arc::new(|input: &str| format!("{input}#rewritten"));
    let broken_link_callback = |link_ref: options::BrokenLinkReference| {
        Some(ResolvedReference::new(
            link_ref.normalized,
            link_ref.original,
        ))
    };

    let mut options = input.options;
    options.extension.header_id_prefix = input.header_id_prefix.then(|| "user-content-".into());
    options.extension.front_matter_delimiter = input.front_matter_delimiter.then(|| "---".into());
    options.extension.image_url_rewriter = input.image_url_rewriter.then(|| url_rewriter.clone());
    options.extension.link_url_rewriter = input.link_url_rewriter.then(|| url_rewriter);
    options.parse.default_info_string = input.default_info_string.then(|| "rust".into());
    options.parse.broken_link_callback = input
        .broken_link_callback
        .then(|| Arc::new(broken_link_callback) as Arc<dyn options::BrokenLinkCallback>);

    let s = &input.markdown;
    let _ = markdown_to_html(s, &options);
    let _ = markdown_to_commonmark(s, &options);
    let _ = markdown_to_commonmark_xml(s, &options);
});
