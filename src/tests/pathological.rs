#![cfg(not(target_arch = "riscv64"))]

use super::*;
use ntest::timeout;
use pretty_assertions::assert_eq;

// input: python3 -c 'n = 50000; print("*a_ " * n)'
#[test]
#[timeout(4000)]
fn pathological_emphases() {
    let n = 50_000;
    let input = "*a_ ".repeat(n);
    let mut exp = format!("<p>{}", input);
    // Right-most space is trimmed in output.
    exp.pop();
    exp += "</p>\n";

    html(&input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_emphasis_mod_three_1() {
    let n = 40_000;
    let input = "x**x".to_owned() + &" _x".repeat(n) + &"x* ".repeat(n);
    let exp = format!("<p>{}</p>\n", input.trim_end());

    html(&input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_emphasis_mod_three_2() {
    let n = 20_000;
    let input = "*x ".repeat(n) + &".**".repeat(n);
    let exp = format!(
        "<p>{}{}</p>\n",
        "*x ".repeat(n),
        ".<strong>.</strong>".repeat(n / 2)
    );

    html(&input, &exp);
}

// input: python3 -c 'n = 10000; print("|" + "x|" * n + "\n|" + "-|" * n)'
#[test]
#[timeout(4000)]
fn pathological_table_columns_1() {
    let n = 100_000;
    let input = format!("{}{}{}{}", "|", "x|".repeat(n), "\n|", "-|".repeat(n));
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.table], &input, &exp);
}

// input: python3 -c 'n = 70000; print("|" + "x|" * n + "\n|" + "-|" * n + "\n" + "a\n" * n)'
#[test]
#[timeout(4000)]
fn pathological_table_columns_2() {
    let n = 100_000;
    let input = format!(
        "{}{}{}{}{}{}",
        "|",
        "x|".repeat(n),
        "\n|",
        "-|".repeat(n),
        "\n",
        "a\n".repeat(n)
    );

    let extension = parser::options::Extension {
        table: true,
        ..Default::default()
    };

    // Not interested in the actual html, just that we don't timeout
    markdown_to_html(
        &input,
        &Options {
            extension,
            ..Default::default()
        },
    );
}

fn escaped_pipes_table(cell: &str) -> String {
    format!(
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>{cell}</td>\n<td>x</td>\n</tr>\n</tbody>\n</table>\n"
    )
}

#[test]
#[timeout(4000)]
fn pathological_table_escaped_pipes_cell() {
    let n = 20_000;
    let input = format!("a|b\n-|-\n{}|x\n", "\\|ä".repeat(n));
    let exp = escaped_pipes_table(&"|ä".repeat(n));

    html_opts!([extension.table], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_table_escaped_pipes_inlines() {
    let n = 10_000;
    let input = format!("a|b\n-|-\n{}|x\n", "*e*\\|".repeat(n));
    let exp = escaped_pipes_table(&"<em>e</em>|".repeat(n));

    html_opts!([extension.table], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_table_escaped_pipes_paragraph() {
    let n = 10_000;
    let input = format!("{}a|b\n-|-\n", "x\\|y\n".repeat(n));
    let exp = format!(
        "<p>{}</p>\n<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n</table>\n",
        "x|y\n".repeat(n).trim_end()
    );

    html_opts!([extension.table], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_table_cell_alignment_html() {
    let (arena, options) = (Arena::new(), wide_table_options());
    let root = parse_wide_table(&arena, &options);

    let mut output = String::new();
    html::format_document(root, &options, &mut output).unwrap();
}

#[test]
#[timeout(4000)]
fn pathological_table_cell_alignment_xml() {
    let (arena, options) = (Arena::new(), wide_table_options());
    let root = parse_wide_table(&arena, &options);

    let mut output = String::new();
    crate::xml::format_document(root, &options, &mut output).unwrap();
}

fn wide_table_options() -> Options<'static> {
    Options {
        extension: parser::options::Extension {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn parse_wide_table<'a>(arena: &'a Arena<'a>, options: &Options) -> Node<'a> {
    let n = u16::MAX as usize - 1;
    let r = 7;
    let input = format!(
        "{}|\n{}|\n{}",
        "|a".repeat(n),
        "|-".repeat(n),
        "a\n".repeat(r)
    );

    let root = parse_document(arena, &input, options);
    let table = root.first_child().unwrap();
    assert!(matches!(table.data().value, NodeValue::Table(..)));
    assert_eq!(table.children().count(), 1 + r);
    for row in table.children() {
        assert_eq!(row.children().count(), n);
    }
    root
}

#[test]
#[timeout(4000)]
fn pathological_table_autocompleted_cells() {
    let c = 2_000;
    let r = 2_000;
    let input = format!(
        "{}\n{}\n{}",
        "|".repeat(c + 1),
        "|-".repeat(c) + "|",
        "|a\n".repeat(r)
    );

    let options = Options {
        extension: parser::options::Extension {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let arena = typed_arena::Arena::new();
    let root = parse_document(&arena, &input, &options);
    let table = root
        .descendants()
        .find(|n| matches!(n.data().value, NodeValue::Table(..)))
        .expect("table node missing");
    let num_rows = table
        .children()
        .filter(|n| matches!(n.data().value, NodeValue::TableRow(..)))
        .count();
    assert_eq!(num_rows, 252);
    assert!(matches!(
        table.next_sibling(),
        Some(n) if matches!(n.data().value, NodeValue::Paragraph)
    ));
}

// input: python3 -c 'n = 10000; print("[^1]:" * n + "\n" * n)'
#[test]
fn pathological_table_autocompleted_cells_across_tables() {
    let n = 708;
    let table = format!(
        "{}a\n{}\n{}\n",
        "|".repeat(n),
        "-|".repeat(n),
        "a\n".repeat(n)
    );
    let cells = |tables: usize| {
        let mut options = Options::default();
        options.extension.table = true;
        markdown_to_html(&table.repeat(tables), &options)
            .matches("<td")
            .count()
    };

    assert_eq!(cells(4), cells(1));
}

#[test]
#[timeout(4000)]
fn pathological_footnotes() {
    let n = 10_000;
    let input = format!("{}{}", "[^1]:".repeat(n), "\n".repeat(n));
    let exp = "";

    html_opts!([extension.footnotes], &input, exp);
}

#[test]
fn pathological_recursion() {
    let n = 5_000;
    let input = format!("{}{}", "*a **a ".repeat(n), " a** a*".repeat(n));
    let exp = format!(
        "<p>{}{}</p>\n",
        "<em>a <strong>a ".repeat(n),
        " a</strong> a</em>".repeat(n)
    );

    // The footnote code happened to be the cause of the pathological recursion, but it didn't
    // actually depend on any footnotes being present; it was the search that caused it.
    html_opts!([extension.footnotes], &input, &exp);
}

#[test]
fn pathological_recursion_inline_footnotes() {
    let n = 5_000;
    let input = format!("{}{}", "^[a ".repeat(n), " b]".repeat(n));
    let exp = format!(
        "{}{}{}{}",
        concat!(
            "<p><sup class=\"footnote-ref\"><a href=\"#fn-__inline_1\" id=\"fnref-__inline_1\" data-footnote-ref>1</a></sup></p>\n",
            "<section class=\"footnotes\" data-footnotes>\n<ol>\n",
            "<li id=\"fn-__inline_1\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_2\" id=\"fnref-__inline_2\" data-footnote-ref>2</a></sup> b <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_2\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_3\" id=\"fnref-__inline_3\" data-footnote-ref>3</a></sup> b <a href=\"#fnref-__inline_2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_3\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_4\" id=\"fnref-__inline_4\" data-footnote-ref>4</a></sup> b <a href=\"#fnref-__inline_3\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"3\" aria-label=\"Back to reference 3\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_4\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_5\" id=\"fnref-__inline_5\" data-footnote-ref>5</a></sup> b <a href=\"#fnref-__inline_4\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"4\" aria-label=\"Back to reference 4\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_5\">\n<p>a ",
        ),
        "^[a ".repeat(4995),
        " b]".repeat(4995),
        concat!(
            " b <a href=\"#fnref-__inline_5\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"5\" aria-label=\"Back to reference 5\">↩</a></p>\n</li>\n",
            "</ol>\n</section>\n"
        )
    );

    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        &input,
        &exp,
        no_roundtrip
    );
}

#[test]
#[timeout(4000)]
fn pathological_footnote_label_reconstruction() {
    let n = 20_000;
    let input = "[^".repeat(n) + &"]".repeat(n);
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.footnotes], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_description_lists() {
    let n = 20_000;
    let input = "t\n: d\n\n".repeat(n);
    let exp = format!("<dl>\n{}</dl>\n", "<dt>t</dt>\n<dd>d</dd>\n".repeat(n));

    html_opts!([extension.description_lists], &input, &exp, no_roundtrip);
}

#[test]
#[timeout(4000)]
fn pathological_relaxed_autolink_url_dead_zone() {
    let n = 40_000;
    let input = "://a".repeat(n) + "](";
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.autolink, parse.relaxed_autolinks], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_relaxed_autolink_www_dead_zone() {
    let n = 40_000;
    let input = "~www.a".repeat(n) + "](";
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.autolink, parse.relaxed_autolinks], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_autolink_www_failing_domains() {
    let n = 20_000;
    let input = "*www.".repeat(n);
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.autolink], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_autolink_url_failing_domains() {
    let n = 20_000;
    let input = "http://a_".repeat(n);
    let exp = format!(
        "<p>{}<a href=\"http://a\">http://a</a>_</p>\n",
        "http://a_".repeat(n - 1)
    );

    html_opts!([extension.autolink], &input, &exp, no_roundtrip);
}

#[test]
#[timeout(4000)]
fn pathological_autolink_www_trailing_punctuation_unreached() {
    let n = 20_000;
    let input = "*www.a.b_*/".repeat(n);
    let exp = format!("<p>{}</p>\n", "<em>www.a.b_</em>/".repeat(n));

    html_opts!([extension.autolink], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_autolink_url_lt_separated() {
    let n = 20_000;
    let input = "http://a.com<".repeat(n);
    let exp = format!(
        "<p>{}</p>\n",
        "<a href=\"http://a.com\">http://a.com</a>&lt;".repeat(n)
    );

    html_opts!([extension.autolink], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_relaxed_autolink_failing_domains() {
    let n = 20_000;
    let input = "a://_".repeat(n);
    let exp = format!(
        "<p>{}a://<em><a href=\"a://\">a://</a></em></p>\n",
        "a://_".repeat(n - 2)
    );

    html_opts!([extension.autolink, parse.relaxed_autolinks], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_autolink_email_non_literal() {
    let n = 40_000;
    let input = "a&#64;b.c ".repeat(n);
    let exp = format!("<p>{}</p>\n", "a@b.c ".repeat(n).trim_end());

    html_opts!([extension.autolink], &input, &exp, no_roundtrip);
}

#[test]
#[timeout(4000)]
fn pathological_block_ids_long_name_nested() {
    let n = 40_000;
    let name = "a".repeat(n);
    let input = format!("{} x ^{name}\n", ">".repeat(n));

    let mut options = Options::default();
    options.extension.ofm_block_ids = true;
    let html = markdown_to_html(&input, &options);
    assert_eq!(html.matches(&format!("id=\"^{name}\"")).count(), 1);
}

#[test]
#[timeout(4000)]
fn pathological_block_ids_subtree_renders() {
    let n = 10_000;
    let input = format!("{}{} x ^a\n", "> p\n>\n".repeat(n), "> ".repeat(n));

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.ofm_block_ids = true;
    let root = parse_document(&arena, &input, &options);
    let mut ids = 0;
    for child in root.first_child().unwrap().children() {
        let mut html = String::new();
        html::format_document(child, &options, &mut html).unwrap();
        ids += html.matches("id=\"^a\"").count();
    }
    assert_eq!(ids, 0);
}

#[test]
#[timeout(4000)]
fn pathological_block_ids_nested() {
    let n = 20_000;
    let input = format!("{} x ^a\n", ">".repeat(n));
    let exp = format!(
        "<blockquote id=\"^a\">\n{}<p>x </p>\n{}",
        "<blockquote>\n".repeat(n - 1),
        "</blockquote>\n".repeat(n)
    );

    html_opts!(
        [
            extension.ofm_block_ids = true,
            extension.header_id_prefix = Some("".to_owned())
        ],
        &input,
        &exp
    );
}

#[test]
#[timeout(4000)]
fn pathological_wikilinks_nested() {
    let n = 32_000;
    let input = format!("{} {}\n", ">".repeat(n), "[[a]]".repeat(n));
    let exp = format!(
        "{}<p>{}</p>\n{}",
        "<blockquote>\n".repeat(n),
        "<a href=\"a\" data-wikilink=\"true\">a</a>".repeat(n),
        "</blockquote>\n".repeat(n)
    );

    html_opts!([extension.wikilinks_title_after_pipe], &input, &exp);
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_attribute_pairs() {
    let n = 30_000;
    let pairs: Vec<String> = (0..n).map(|i| format!("k{i}=v")).collect();
    let input = format!("# h {{{}}}\n", pairs.join(" "));

    html_opts!([extension.header_attributes], &input, "<h1>h</h1>\n");
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_attribute_chain() {
    let n = 30_000;
    let chain: String = (0..n).map(|i| format!("{{k{i}=v}}")).collect();
    let input = format!("[a](b){chain}\n");

    html_opts!(
        [extension.link_attributes],
        &input,
        "<p><a href=\"b\">a</a></p>\n"
    );
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_link_attributes_unclosed() {
    let n = 30_000;
    let input = "[a](b){.c k=".repeat(n);
    let exp = format!("<p>{}</p>\n", "<a href=\"b\">a</a>{.c k=".repeat(n));

    html_opts!([extension.link_attributes], &input, &exp);
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_link_attributes_rejected() {
    let n = 30_000;
    let input = format!("{}<", "[a](b){.c k=".repeat(n));
    let exp = format!("<p>{}&lt;</p>\n", "<a href=\"b\">a</a>{.c k=".repeat(n));

    html_opts!([extension.link_attributes], &input, &exp);
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_header_attributes_unparsable_openers() {
    let n = 30_000;
    let content = format!("h {} !}}", "{.a k=x".repeat(n));
    let input = format!("# {content}\n");
    let exp = format!("<h1>{content}</h1>\n");

    html_opts!([extension.header_attributes], &input, &exp);
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_header_attributes_spaced_unparsable_openers() {
    let n = 30_000;
    let content = format!("h{} !}}", " {.a k=x".repeat(n));
    let input = format!("# {content}\n");
    let exp = format!("<h1>{content}</h1>\n");

    html_opts!([extension.header_attributes], &input, &exp);
}

#[cfg(feature = "attributes")]
#[test]
#[timeout(4000)]
fn pathological_fenced_code_attributes_unparsable_openers() {
    let n = 30_000;
    let input = format!("``` h {} !}}\nx\n```\n", "{.a k=x".repeat(n));

    html_opts!(
        [extension.fenced_code_attributes],
        &input,
        "<pre><code class=\"language-h\">x\n</code></pre>\n"
    );
}

#[test]
#[timeout(4000)]
fn pathological_sourcepos_chars_long_line() {
    let n = 10_000;
    let input = "é *a* ".repeat(n);
    let exp = format!("<p>{}</p>\n", "é <em>a</em> ".repeat(n).trim_end());

    html_opts!([parse.sourcepos_chars], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_block_directive_nesting() {
    let n = 50_000;
    let input = "::: x\n".repeat(n);

    let mut options = Options::default();
    options.extension.block_directive = true;
    let html = markdown_to_html(&input, &options);
    assert_eq!(html.matches("<div class=\"x\">").count(), 100);
}

#[test]
#[timeout(4000)]
fn pathological_multiline_alert_nesting() {
    let n = 50_000;
    let input = ">>> [!note]\n".repeat(n);

    let mut options = Options::default();
    options.extension.alerts = true;
    options.extension.multiline_block_quotes = true;
    let html = markdown_to_html(&input, &options);
    assert_eq!(
        html.matches("<div class=\"markdown-alert markdown-alert-note\">")
            .count(),
        101
    );
}

#[test]
#[timeout(4000)]
fn pathological_alert_marker_run() {
    let n = 50_000;
    let input = format!("{}\n", ">".repeat(n));
    let exp = format!(
        "{}{}",
        "<blockquote>\n".repeat(n),
        "</blockquote>\n".repeat(n)
    );

    html_opts!([extension.alerts], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_inline_footnotes_unclosed() {
    let n = 60_000;
    let input = "^[".repeat(n);
    let exp = format!("<p>{}</p>\n", input);

    html_opts!(
        [extension.footnotes, extension.inline_footnotes],
        &input,
        &exp
    );
}

#[test]
#[timeout(4000)]
fn pathological_latex_math_unclosed() {
    let n = 60_000;
    let input = "\\(".repeat(n);
    let exp = format!("<p>{}</p>\n", "(".repeat(n));

    html_opts!([extension.math_latex], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_skip_char_flanking() {
    let n = 40_000;
    let input = "~=".repeat(n);
    let exp = format!("<p>{}</p>\n", input);

    html_opts!([extension.strikethrough, extension.highlight], &input, &exp);
}

#[cfg(feature = "phoenix_heex")]
#[test]
#[timeout(4000)]
fn pathological_heex_unclosed_directive_paragraph() {
    let n = 40_000;
    let input = format!("a <%\n{}", "a\n".repeat(n));
    let exp = format!("<p>a &lt;%\n{}</p>\n", "a\n".repeat(n).trim_end());

    html_opts!([extension.phoenix_heex], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_minimize_commonmark() {
    let n = 20_000;
    let input = "a[".repeat(n);

    let arena = Arena::new();
    let mut options = Options::default();
    let root = parse_document(&arena, &input, &options);
    let mut unminimized = String::new();
    cm::format_document(root, &options, &mut unminimized).unwrap();

    options.render.experimental_minimize_commonmark = true;
    let mut minimized = String::new();
    cm::format_document(root, &options, &mut minimized).unwrap();
    assert_eq!(minimized, unminimized);
}

#[test]
#[timeout(4000)]
fn pathological_multiline_block_quote_marker_run() {
    let n = 40_000;
    let input = format!("{}x\n", ">".repeat(n));
    let exp = format!(
        "{}<p>x</p>\n{}",
        "<blockquote>\n".repeat(n),
        "</blockquote>\n".repeat(n)
    );

    html_opts!([extension.multiline_block_quotes], &input, &exp);
}

#[test]
#[timeout(4000)]
fn pathological_skip_char_flanking_variation_selector() {
    let n = 40_000;
    let input = format!("a\u{fe00}{}!", "~=".repeat(n));
    let exp = format!("<p>{}</p>\n", input);

    html_opts!(
        [
            extension.strikethrough,
            extension.highlight,
            extension.cjk_friendly_emphasis
        ],
        &input,
        &exp
    );
}

#[test]
#[timeout(4000)]
fn pathological_list_nesting_blank_lines() {
    let k = 20;
    let input = (0..k)
        .map(|i| format!("{}{}a\n", " ".repeat(200 * i), "- ".repeat(100)))
        .collect::<String>()
        + &"\n".repeat(100 * k * k);

    let html = markdown_to_html(&input, &Options::default());
    assert_eq!(html.matches("<li>").count(), 50);
}

#[test]
#[timeout(4000)]
fn pathological_footnote_nesting_blank_lines() {
    let k = 20;
    let input = "x[^a]\n\n".to_string()
        + &(0..k)
            .map(|i| format!("{}{}x\n", " ".repeat(396 * i), "[^a]: ".repeat(99)))
            .collect::<String>()
        + &"\n".repeat(400 * k * k);

    let mut options = Options::default();
    options.extension.footnotes = true;
    let html = markdown_to_html(&input, &options);
    assert_eq!(html.matches("<li id=\"fn-a\">").count(), 1);
}

#[test]
#[timeout(4000)]
fn pathological_commonmark_lazy_continuation() {
    let n = 5_000;
    let input = format!("{} a\n{}", ">".repeat(n), "b\n".repeat(n));

    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, &input, &options);
    let mut output = String::new();
    cm::format_document(root, &options, &mut output).unwrap();
    assert!(output.len() < 4 * input.len());
    assert_eq!(
        markdown_to_html(&output, &options),
        markdown_to_html(&input, &options)
    );
}

#[test]
#[timeout(4000)]
fn pathological_header_ids() {
    let n = 20_000;
    let input = "# a\n".repeat(n);
    let mut exp = String::new();
    for i in 0..n {
        let id = if i == 0 {
            "a".to_string()
        } else {
            format!("a-{}", i)
        };
        exp.push_str(&format!(
            "<h1 id=\"{id}\">a<a href=\"#{id}\" aria-label=\"Link to heading 'a'\" data-heading-content=\"a\" class=\"anchor\"></a></h1>\n"
        ));
    }

    html_opts!(
        [extension.header_id_prefix = Some("".to_owned())],
        &input,
        &exp,
        no_roundtrip
    );
}

#[cfg(feature = "grid_table")]
fn grid_table_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.grid_table = true;
    options
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_cells() {
    let n = 200;
    let separator = format!("+{}\n", "-+".repeat(n));
    let row = format!("|{}\n", "a|".repeat(n));
    let input = format!("{separator}{}", format!("{row}{separator}").repeat(n));

    let html = markdown_to_html(&input, &grid_table_options());
    assert_eq!(html.matches("<td>a</td>").count(), n * n);
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_commonmark_reference_area() {
    let n = 8_000;
    let input = format!(
        "+-----+\n| [r] |\n+-----+\n\n[r]: /{} \"{}\"\n",
        "u".repeat(n),
        "x\n".repeat(n)
    );

    let output = grid_table_commonmark(&input, &grid_table_options()).unwrap();
    assert!(output.len() < 2 * input.len());
    assert!(!output.contains("+-"));
}

#[cfg(feature = "grid_table")]
fn grid_table_commonmark(input: &str, options: &Options) -> Result<String, std::fmt::Error> {
    let arena = Arena::new();
    let root = parse_document(&arena, input, options);
    let mut output = String::new();
    cm::format_document(root, options, &mut output).map(|()| output)
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_commonmark_sibling_area() {
    let tables = 2_000;
    let input = format!(
        "{}[r]: /{}\n[s]: /u \"{}\"\n",
        "+-----+-----+\n| [r] | [s] |\n+-----+-----+\n\n".repeat(tables),
        "u".repeat(360),
        "x\n".repeat(180)
    );

    let output = grid_table_commonmark(&input, &grid_table_options()).unwrap();
    assert!(output.len() < 20 * input.len() + (1 << 16));
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_commonmark_nested_area() {
    let box2 = |a: &str, b: &str| {
        let sep = format!("+{}+{}+", "-".repeat(a.len() + 2), "-".repeat(b.len() + 2));
        vec![sep.clone(), format!("| {a} | {b} |"), sep]
    };
    let mut lines = box2("[u0]", "[t0]");
    let mut defs = vec![(40, 20)];
    let (mut w, mut h) = (50, 22);
    for level in 1..4 {
        let (u, t) = (format!("[u{level}]"), format!("[t{level}]"));
        let inner = lines[0].len();
        let sep = format!("+{}+{}+", "-".repeat(inner + 2), "-".repeat(u.len() + 2));
        let mut next = vec![sep.clone()];
        for (i, line) in lines.iter().enumerate() {
            let right = if i == 0 {
                u.clone()
            } else {
                " ".repeat(u.len())
            };
            next.push(format!("| {line} | {right} |"));
        }
        next.push(sep.clone());
        next.push(format!("| {t:inner$} | {} |", " ".repeat(u.len())));
        next.push(sep);
        lines = next;
        let (x, y) = (w * 5 / 2, h * 5 / 2);
        defs.push((x, y));
        (w, h) = (w + x + 10, h + y + 2);
    }
    let mut input = lines.join("\n") + "\n\n";
    for (level, (x, y)) in defs.into_iter().enumerate() {
        input += &format!(
            "[u{level}]: /{}\n[t{level}]: /t \"{}\"\n",
            "u".repeat(x),
            "x\n".repeat(y)
        );
    }

    let output = grid_table_commonmark(&input, &grid_table_options()).unwrap();
    assert!(output.len() < 20 * input.len() + (1 << 16));
}

#[cfg(feature = "grid_table")]
#[test]
fn grid_table_commonmark_cells_ignore_width() {
    let cell = "a ".repeat(200);
    let rule = format!(
        "+{}+{}+\n",
        "-".repeat(cell.len() + 2),
        "-".repeat(cell.len() + 2)
    );
    let input = format!(
        "{rule}{}",
        format!("| {cell} | {cell} |\n{rule}").repeat(20)
    );

    let mut options = grid_table_options();
    options.render.width = 20;
    let output = grid_table_commonmark(&input, &options).unwrap();
    assert_eq!(output.lines().count(), 41);

    let multiline = "+---+\n| a |\n| b |\n+---+\n";
    assert_eq!(
        grid_table_commonmark(multiline, &options).unwrap(),
        multiline
    );
}

#[cfg(feature = "grid_table")]
#[test]
fn grid_table_commonmark_sparse_within_area() {
    let n = 2_000;
    let header = "a".repeat(100);
    let rule = format!("+{}+\n", "-".repeat(102));
    let input = format!(
        "{rule}| {header} |\n{}",
        format!("{rule}| {:100} |\n", "b").repeat(n) + &rule
    );

    assert_eq!(
        grid_table_commonmark(&input, &grid_table_options()).unwrap(),
        input
    );
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_tracing() {
    let n = 200;
    let separator = format!("+{}\n", "-+".repeat(n));
    let row = format!("|{}\n", "a|".repeat(n));
    let bars = format!("|{}\n", " |".repeat(n));
    let blank = format!("|{}|\n", " ".repeat(2 * n - 1));
    let band = format!("{row}{separator}{}{blank}{separator}", bars.repeat(4));
    let input = format!("{separator}{}", band.repeat(n / 4));

    let html = markdown_to_html(&input, &grid_table_options());
    assert_eq!(html.matches("<tr>").count(), n / 2);
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_nesting() {
    let mut lines = vec!["x".to_string()];
    for _ in 0..200 {
        let width = lines.iter().map(String::len).max().unwrap();
        let border = format!("+{}+", "-".repeat(width + 2));
        let mut nested = vec![border.clone()];
        nested.extend(lines.iter().map(|l| format!("| {l:width$} |")));
        nested.push(border);
        lines = nested;
    }
    let input = lines.join("\n");

    let html = markdown_to_html(&input, &grid_table_options());
    assert_eq!(html.matches("<table>").count(), 5);
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_rejections() {
    let n = 50_000;
    let input = format!("{}{}", "+-+\n|\n\n".repeat(n), "+-+\n|\n".repeat(n));

    let html = markdown_to_html(&input, &grid_table_options());
    assert!(!html.contains("<table>"));
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_ragged_lines() {
    let n = 100_000;
    let input = format!("+{}+\n{}", "-".repeat(n), "|\n".repeat(n));

    let html = markdown_to_html(&input, &grid_table_options());
    assert!(!html.contains("<table>"));
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_breaks_on_one_line() {
    let n = 40_000;
    let separator = format!("+{}\n", "-+".repeat(n));
    let row = format!("|{}\n", "a|".repeat(n));
    let input = format!("{separator}{row}{row}{separator}");

    let html = markdown_to_html(&input, &grid_table_options());
    assert_eq!(html.matches("<td>a\na</td>").count(), n);
}

#[cfg(feature = "grid_table")]
#[test]
#[timeout(4000)]
fn pathological_grid_table_consecutive_in_tight_list() {
    let n = 20_000;
    let mut input = String::from("- +-+\n  |a|\n  +-+\n");
    for i in 1..n {
        let indent = if i % 2 == 1 { "   " } else { "  " };
        input.push_str(&format!("{indent}+-+\n{indent}|a|\n{indent}+-+\n"));
    }
    input.push_str("- b\n");

    let options = grid_table_options();
    let commonmark = markdown_to_commonmark(&input, &options);
    let html = markdown_to_html(&commonmark, &options);
    assert_eq!(html.matches("<table>").count(), n);
}
