use crate::nodes::{Ast, NodeValue};

use super::*;

#[test]
fn basic_blocks_and_inlines() {
    typst(
        "# Title\n\nParagraph with [link](https://example.com), **strong**, *emphasis*, and `code`.\n\n- one\n- two\n",
        concat!(
            "= Title\n\n",
            "Paragraph with #link(\"https://example.com\")[link], *strong*, _emphasis_, and #raw(\"code\").\n\n",
            "#list(\n",
            "  tight: true,\n",
            "  [one],\n",
            "  [two],\n",
            ")\n",
        ),
    );
}

#[test]
fn ordered_lists_preserve_start() {
    typst(
        "3. third\n4. fourth\n",
        concat!(
            "#enum(\n",
            "  tight: true,\n",
            "  start: 3,\n",
            "  [third],\n",
            "  [fourth],\n",
            ")\n",
        ),
    );
}

#[test]
fn tables_render_to_typst_table_calls() {
    typst_opts(
        "| Left | Right |\n| :--- | ----: |\n| a | b |\n",
        concat!(
            "#table(\n",
            "  columns: 2,\n",
            "  table.header(\n",
            "    table.cell(align: left)[Left],\n",
            "    table.cell(align: right)[Right],\n",
            "  ),\n",
            "  table.cell(align: left)[a],\n",
            "  table.cell(align: right)[b],\n",
            ")\n",
        ),
        |opts| {
            opts.extension.table = true;
        },
    );
}

#[test]
fn standalone_images_with_titles_respect_figure_with_caption() {
    typst(
        "![Alt](image.png \"Caption\")\n",
        "#image(\"image.png\", alt: \"Alt\")\n",
    );

    typst_opts(
        "![Alt](image.png \"Caption\")\n",
        concat!(
            "#figure(\n",
            "  image(\"image.png\", alt: \"Alt\"),\n",
            "  caption: [Caption],\n",
            ")\n",
        ),
        |opts| {
            opts.render.figure_with_caption = true;
        },
    );
}

#[test]
fn typst_escapes_text_that_would_trigger_markup_or_comments() {
    typst(
        "\\* not a list item\n\nuse // for comments, /* blocks, and ~100.\n",
        concat!(
            "\\* not a list item\n\n",
            "use /\\/ for comments, /\\* blocks, and \\~100.\n",
        ),
    );
}

#[test]
fn fenced_code_blocks_use_raw_blocks() {
    typst(
        "```rust\nfn main() {}\n```\n",
        "#raw(\"fn main() {}\\n\", block: true, lang: \"rust\")\n",
    );
}

#[test]
fn typst_code_fences_passthrough_verbatim() {
    typst(
        "```typst\n#bibliography(\"refs.bib\")\n```\n",
        "#bibliography(\"refs.bib\")\n",
    );
}

#[test]
fn footnotes_render_inline() {
    typst_opts(
        "Hello[^note]\n\n[^note]: Footnote text\n",
        "Hello#footnote[Footnote text] <footnote-1>\n",
        |opts| {
            opts.extension.footnotes = true;
        },
    );
}

#[test]
fn inline_footnotes_render_inline() {
    typst_opts(
        "Hello^[Footnote text]\n",
        "Hello#footnote[Footnote text] <footnote-1>\n",
        |opts| {
            opts.extension.footnotes = true;
            opts.extension.inline_footnotes = true;
        },
    );
}

#[test]
fn math_preserves_inline_and_display_modes() {
    typst_opts(
        "$x^2$\n\n$$\n(a+b)/2\n$$\n",
        "$x^2$\n\n$\n(a+b)/2\n$\n",
        |opts| {
            opts.extension.math_dollars = true;
        },
    );
}

#[test]
fn latex_math_translates_common_typst_equivalents() {
    typst_opts(
        "$\\frac{1}{2} + \\sqrt[3]{x} + \\alpha$\n",
        "$frac(1, 2) + root(3, x) + alpha$\n",
        |opts| {
            opts.extension.math_dollars = true;
        },
    );

    typst_opts(
        "$$\\sum_{i=0}^{n} x_{i+1} \\to \\infty \\text{ units}$$\n",
        "$\nsum_(i=0)^(n) x_(i+1) -> oo \" units\"\n$\n",
        |opts| {
            opts.extension.math_dollars = true;
        },
    );
}

#[test]
fn front_matter_is_omitted() {
    typst_opts("---\ntitle: Report\n---\n\nBody\n", "Body\n", |opts| {
        opts.extension.front_matter_delimiter = Some("---".to_owned());
    });
}

#[test]
fn text_preserves_typst_references_and_citations() {
    typst(
        "See @sec:intro, @doe2024[p. 7], and me@example.com.\n",
        "See @sec:intro, @doe2024[p. 7], and me\\@example.com.\n",
    );
}

#[cfg(feature = "shortcodes")]
#[test]
fn shortcodes_render_to_emoji() {
    typst_opts("Launch :rocket:\n", "Launch 🚀\n", |opts| {
        opts.extension.shortcodes = true;
    });
}

#[test]
fn typst_text_primitives_cover_markdown_extensions() {
    typst_opts(
        "~~gone~~ ==marked== __under__ ++added++ H~2~O 1^st^\n",
        "#strike[gone] #highlight[marked] #underline[under] #underline[added] H#sub[2]O 1#super[st]\n",
        |opts| {
            opts.extension.strikethrough = true;
            opts.extension.highlight = true;
            opts.extension.underline = true;
            opts.extension.insert = true;
            opts.extension.superscript = true;
            opts.extension.subscript = true;
        },
    );
}

#[test]
fn common_inline_html_tags_map_to_typst_primitives() {
    typst(
        "H<sub>2</sub>O <strong>bold</strong> <em>em</em> <br> <mark>mark</mark> <del>gone</del>\n",
        "H#sub[2]O *bold* _em_ \\\n #highlight[mark] #strike[gone]\n",
    );
}

#[test]
fn description_lists_render_to_terms() {
    typst_opts(
        "Ligature\n: A merged glyph.\n\nKerning\n: A spacing adjustment.\n",
        concat!(
            "#terms(\n",
            "  terms.item([Ligature], [A merged glyph.]),\n",
            "  terms.item([Kerning], [A spacing adjustment.]),\n",
            ")\n",
        ),
        |opts| {
            opts.extension.description_lists = true;
        },
    );
}

#[test]
fn multiline_block_quotes_render_as_quotes() {
    typst_opts(
        ">>>\nParagraph 1\n\nParagraph 2\n>>>\n",
        "#quote(block: true)[\n  Paragraph 1\n  \n  Paragraph 2\n]\n",
        |opts| {
            opts.extension.multiline_block_quotes = true;
        },
    );
}

#[test]
fn subtext_renders_as_typst_sub() {
    typst_opts("-# Some Subtext\n", "#sub[Some Subtext]\n", |opts| {
        opts.extension.subtext = true;
    });
}

#[test]
fn task_lists_render_as_custom_marker_lists() {
    typst_opts(
        "- [ ] draft\n- [ ] review\n- [x] ship\n",
        concat!(
            "#list(\n",
            "  marker: [☐],\n",
            "  tight: true,\n",
            "  [draft],\n",
            "  [review],\n",
            ")\n",
            "#list(\n",
            "  marker: [☒],\n",
            "  tight: true,\n",
            "  [ship],\n",
            ")\n",
        ),
        |opts| {
            opts.extension.tasklist = true;
        },
    );
}

#[test]
fn alerts_render_with_quote_attribution() {
    typst_opts(
        "> [!WARNING] Pay attention\n> Something broke.\n",
        concat!(
            "#quote(\n",
            "  block: true,\n",
            "  attribution: [Pay attention],\n",
            ")[Something broke.]\n",
        ),
        |opts| {
            opts.extension.alerts = true;
        },
    );
}

#[test]
fn wikilinks_use_their_rendered_label() {
    typst_opts(
        "[[page|Display Text]]\n",
        "#link(\"page\")[Display Text]\n",
        |opts| {
            opts.extension.wikilinks_title_after_pipe = true;
        },
    );
}

#[test]
fn heading_labels_follow_header_id_prefix() {
    typst_opts("# Intro\n", "= Intro <sec:intro>\n", |opts| {
        opts.extension.header_id_prefix = Some("sec:".to_owned());
    });
}

#[test]
fn label_autolinks_and_internal_links_render_to_typst_targets() {
    typst(
        "# Intro <sec:intro>\n\n![Alt](image.png) <fig:hero>\n\n[See intro](#sec:intro)\n",
        concat!(
            "= Intro <sec:intro>\n\n",
            "#image(\"image.png\", alt: \"Alt\") <fig:hero>\n\n",
            "#link(<sec:intro>)[See intro]\n",
        ),
    );
}

#[test]
fn raw_nodes_passthrough_verbatim() {
    let arena = Arena::new();
    let root = parse_document(&arena, "User input\n", &Options::default());
    let raw_ast = Ast::new(
        NodeValue::Raw("#bibliography(\"refs.bib\")".to_string()),
        (0, 0).into(),
    );
    let raw_node = arena.alloc(raw_ast.into());
    root.first_child().unwrap().insert_after(raw_node);

    let mut output = String::new();
    crate::typst::format_document(root, &Options::default(), &mut output).unwrap();

    compare_strs(
        &output,
        "User input\n\n#bibliography(\"refs.bib\")\n",
        "typst",
        "raw typst passthrough",
    );
}
