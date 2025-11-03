#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

use comrak::{
    markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html, options, Options,
    ResolvedReference,
};

#[derive(Arbitrary, Debug)]
struct FuzzOptions {
    extension: FuzzExtensionOptions,
    parse: FuzzParseOptions,
    render: FuzzRenderOptions,
}

impl FuzzOptions {
    fn to_options<'c>(
        &self,
        url_rewriter: Arc<dyn options::URLRewriter + 'c>,
        broken_link_callback: Arc<dyn options::BrokenLinkCallback + 'c>,
    ) -> Options<'c> {
        Options {
            extension: self.extension.to_options(url_rewriter),
            parse: self.parse.to_options(broken_link_callback),
            render: self.render.to_options(),
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzExtensionOptions {
    strikethrough: bool,
    tagfilter: bool,
    table: bool,
    autolink: bool,
    tasklist: bool,
    superscript: bool,
    footnotes: bool,
    description_lists: bool,
    multiline_block_quotes: bool,
    math_dollars: bool,
    math_code: bool,
    shortcodes: bool,
    wikilinks_title_after_pipe: bool,
    wikilinks_title_before_pipe: bool,
    underline: bool,
    spoiler: bool,
    greentext: bool,
    alerts: bool,
    inline_footnotes: bool,
    subscript: bool,
    subtext: bool,
    cjk_friendly_emphasis: bool,
    highlight: bool,
    // non-bool below
    header_ids: bool,
    front_matter_delimiter: bool,
    image_url_rewriter: bool,
    link_url_rewriter: bool,
}

impl FuzzExtensionOptions {
    fn to_options<'c>(
        &self,
        url_rewriter: Arc<dyn options::URLRewriter + 'c>,
    ) -> options::Extension<'c> {
        options::Extension {
            strikethrough: self.strikethrough,
            tagfilter: self.tagfilter,
            table: self.table,
            autolink: self.autolink,
            tasklist: self.tasklist,
            superscript: self.superscript,
            footnotes: self.footnotes,
            inline_footnotes: self.inline_footnotes,
            description_lists: self.description_lists,
            multiline_block_quotes: self.multiline_block_quotes,
            math_dollars: self.math_dollars,
            math_code: self.math_code,
            shortcodes: self.shortcodes,
            wikilinks_title_after_pipe: self.wikilinks_title_after_pipe,
            wikilinks_title_before_pipe: self.wikilinks_title_before_pipe,
            underline: self.underline,
            spoiler: self.spoiler,
            greentext: self.greentext,
            alerts: self.alerts,
            subscript: self.subscript,
            subtext: self.subtext,
            cjk_friendly_emphasis: self.cjk_friendly_emphasis,
            highlight: self.highlight,
            // non-bool below
            header_ids: if self.header_ids {
                Some("user-content-".into())
            } else {
                None
            },
            front_matter_delimiter: if self.front_matter_delimiter {
                Some("---".into())
            } else {
                None
            },
            image_url_rewriter: if self.image_url_rewriter {
                Some(url_rewriter.clone())
            } else {
                None
            },
            link_url_rewriter: if self.link_url_rewriter {
                Some(url_rewriter)
            } else {
                None
            },
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzParseOptions {
    smart: bool,
    relaxed_tasklist_matching: bool,
    relaxed_autolinks: bool,
    ignore_setext: bool,
    tasklist_in_table: bool,
    leave_footnote_definitions: bool,
    default_info_string: bool,
    broken_link_callback: bool,
    escaped_char_spans: bool,
}

impl FuzzParseOptions {
    fn to_options<'c>(
        &self,
        broken_link_callback: Arc<dyn options::BrokenLinkCallback + 'c>,
    ) -> options::Parse<'c> {
        options::Parse {
            smart: self.smart,
            relaxed_tasklist_matching: self.relaxed_tasklist_matching,
            relaxed_autolinks: self.relaxed_autolinks,
            ignore_setext: self.ignore_setext,
            tasklist_in_table: self.tasklist_in_table,
            leave_footnote_definitions: self.leave_footnote_definitions,
            default_info_string: if self.default_info_string {
                Some("rust".into())
            } else {
                None
            },
            broken_link_callback: if self.broken_link_callback {
                Some(broken_link_callback)
            } else {
                None
            },
            escaped_char_spans: self.escaped_char_spans,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzRenderOptions {
    hardbreaks: bool,
    github_pre_lang: bool,
    full_info_string: bool,
    width: usize,
    r#unsafe: bool,
    escape: bool,
    list_style: options::ListStyleType,
    sourcepos: bool,
    escaped_char_spans: bool,
    ignore_empty_links: bool,
    gfm_quirks: bool,
    prefer_fenced: bool,
    figure_with_caption: bool,
    tasklist_classes: bool,
    ol_width: usize,
    experimental_minimize_commonmark: bool,
}

impl FuzzRenderOptions {
    fn to_options(&self) -> options::Render {
        options::Render {
            hardbreaks: self.hardbreaks,
            github_pre_lang: self.github_pre_lang,
            full_info_string: self.full_info_string,
            width: self.width,
            r#unsafe: self.r#unsafe,
            escape: self.escape,
            list_style: self.list_style,
            sourcepos: self.sourcepos,
            escaped_char_spans: self.escaped_char_spans,
            ignore_empty_links: self.ignore_empty_links,
            gfm_quirks: self.gfm_quirks,
            prefer_fenced: self.prefer_fenced,
            figure_with_caption: self.figure_with_caption,
            tasklist_classes: self.tasklist_classes,
            ol_width: self.ol_width,
            experimental_minimize_commonmark: self.experimental_minimize_commonmark,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct Input {
    options: FuzzOptions,
    markdown: String,
}

fuzz_target!(|input: Input| {
    let url_rewriter = |input: &str| format!("{input}#rewritten");
    let broken_link_callback = |link_ref: options::BrokenLinkReference| {
        Some(ResolvedReference {
            url: link_ref.normalized.to_string(),
            title: link_ref.original.to_string(),
        })
    };

    let s = &input.markdown;
    let options = input
        .options
        .to_options(Arc::new(url_rewriter), Arc::new(broken_link_callback));

    let _ = markdown_to_html(s, &options);
    let _ = markdown_to_commonmark(s, &options);
    let _ = markdown_to_commonmark_xml(s, &options);
});
