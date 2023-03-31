use crate::*;

#[cfg(not(target_arch = "wasm32"))]
use propfuzz::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[propfuzz]
fn propfuzz_doesnt_crash(md: String) {
    let options = ComrakOptions {
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
            front_matter_delimiter: None,
            #[cfg(feature = "shortcodes")]
            shortcodes: true,
        },
        parse: ComrakParseOptions {
            smart: true,
            default_info_string: Some("Rust".to_string()),
            relaxed_tasklist_matching: true,
        },
        render: ComrakRenderOptions {
            hardbreaks: true,
            github_pre_lang: true,
            full_info_string: true,
            width: 80,
            unsafe_: true,
            escape: false,
            list_style: ListStyleType::Dash,
            sourcepos: true,
        },
    };

    parse_document(&Arena::new(), &md, &options);
}
