use comrak::Options;

type Toggle = (&'static str, fn(&mut Options<'static>));

pub const TOGGLES: &[Toggle] = &[
    ("strikethrough", |o| o.extension.strikethrough = true),
    ("table", |o| o.extension.table = true),
    ("autolink", |o| o.extension.autolink = true),
    ("tasklist", |o| o.extension.tasklist = true),
    ("superscript", |o| o.extension.superscript = true),
    ("footnotes", |o| o.extension.footnotes = true),
    ("inline_footnotes", |o| {
        o.extension.footnotes = true;
        o.extension.inline_footnotes = true;
    }),
    ("description_lists", |o| {
        o.extension.description_lists = true
    }),
    ("multiline_block_quotes", |o| {
        o.extension.multiline_block_quotes = true
    }),
    ("alerts", |o| o.extension.alerts = true),
    ("ofm_callouts", |o| o.extension.ofm_callouts = true),
    ("math_dollars", |o| o.extension.math_dollars = true),
    ("math_code", |o| o.extension.math_code = true),
    ("math_latex", |o| o.extension.math_latex = true),
    ("shortcodes", |o| o.extension.shortcodes = true),
    ("wikilinks_after", |o| {
        o.extension.wikilinks_title_after_pipe = true
    }),
    ("wikilinks_before", |o| {
        o.extension.wikilinks_title_before_pipe = true
    }),
    ("underline", |o| o.extension.underline = true),
    ("subscript", |o| o.extension.subscript = true),
    ("spoiler", |o| o.extension.spoiler = true),
    ("greentext", |o| o.extension.greentext = true),
    ("ofm_comments", |o| o.extension.ofm_comments = true),
    ("ofm_block_ids", |o| o.extension.ofm_block_ids = true),
    ("cjk_friendly_emphasis", |o| {
        o.extension.cjk_friendly_emphasis = true
    }),
    ("subtext", |o| o.extension.subtext = true),
    ("highlight", |o| o.extension.highlight = true),
    ("insert", |o| o.extension.insert = true),
    ("phoenix_heex", |o| o.extension.phoenix_heex = true),
    ("block_directive", |o| o.extension.block_directive = true),
    ("header_attributes", |o| {
        o.extension.header_attributes = true
    }),
    ("fenced_code_attributes", |o| {
        o.extension.fenced_code_attributes = true
    }),
    ("inline_code_attributes", |o| {
        o.extension.inline_code_attributes = true
    }),
    ("link_attributes", |o| o.extension.link_attributes = true),
    ("smart", |o| o.parse.smart = true),
    ("relaxed_autolinks", |o| {
        o.extension.autolink = true;
        o.parse.relaxed_autolinks = true;
    }),
    ("relaxed_tasklist", |o| {
        o.extension.tasklist = true;
        o.parse.relaxed_tasklist_matching = true;
    }),
    ("tasklist_in_table", |o| {
        o.extension.tasklist = true;
        o.extension.table = true;
        o.parse.tasklist_in_table = true;
    }),
    ("ignore_setext", |o| o.parse.ignore_setext = true),
    ("leave_footnote_definitions", |o| {
        o.extension.footnotes = true;
        o.parse.leave_footnote_definitions = true;
    }),
    ("escaped_char_spans", |o| o.parse.escaped_char_spans = true),
    ("front_matter", |o| {
        o.extension.front_matter_delimiter = Some("---".to_string())
    }),
];

pub const COMPOSITES: &[(&str, &[&str])] = &[
    ("cm", &[]),
    (
        "gfm",
        &[
            "strikethrough",
            "table",
            "autolink",
            "tasklist",
            "footnotes",
            "leave_footnote_definitions",
        ],
    ),
    (
        "all",
        &[
            "strikethrough",
            "table",
            "autolink",
            "tasklist",
            "superscript",
            "footnotes",
            "inline_footnotes",
            "description_lists",
            "multiline_block_quotes",
            "alerts",
            "math_dollars",
            "math_code",
            "math_latex",
            "shortcodes",
            "wikilinks_after",
            "underline",
            "spoiler",
            "ofm_comments",
            "subtext",
            "highlight",
            "insert",
            "block_directive",
            "smart",
            "leave_footnote_definitions",
            "escaped_char_spans",
        ],
    ),
    (
        "attrs",
        &[
            "header_attributes",
            "fenced_code_attributes",
            "inline_code_attributes",
            "link_attributes",
            "table",
            "autolink",
        ],
    ),
    (
        "ofm",
        &[
            "ofm_callouts",
            "ofm_comments",
            "ofm_block_ids",
            "wikilinks_before",
            "highlight",
            "footnotes",
            "inline_footnotes",
            "table",
            "tasklist",
            "strikethrough",
        ],
    ),
    (
        "containers",
        &[
            "multiline_block_quotes",
            "block_directive",
            "ofm_callouts",
            "description_lists",
            "footnotes",
            "leave_footnote_definitions",
            "table",
            "tasklist",
            "ofm_comments",
            "subtext",
        ],
    ),
];

pub fn named(name: &str) -> Options<'static> {
    let toggles = COMPOSITES
        .iter()
        .find(|(composite, _)| *composite == name)
        .map_or(vec![name], |(_, toggles)| toggles.to_vec());
    let mut options = Options::default();
    for toggle in toggles {
        let (_, apply) = TOGGLES
            .iter()
            .find(|(candidate, _)| *candidate == toggle)
            .unwrap();
        apply(&mut options);
    }
    options
}

pub fn composite_names() -> Vec<&'static str> {
    COMPOSITES.iter().map(|(name, _)| *name).collect()
}

pub fn all_names() -> Vec<&'static str> {
    let mut names = composite_names();
    names.extend(TOGGLES.iter().map(|(name, _)| *name));
    names
}
