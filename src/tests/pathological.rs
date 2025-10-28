use super::*;
use ntest::timeout;

// input: python3 -c 'n = 50000; print("*a_ " * n)'
#[test]
#[timeout(4000)]
fn pathological_emphases() {
    let n = 50_000;
    let input = "*a_ ".repeat(n).to_string();
    let mut exp = format!("<p>{}", input);
    // Right-most space is trimmed in output.
    exp.pop();
    exp += "</p>\n";

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

// input: python3 -c 'n = 10000; print("[^1]:" * n + "\n" * n)'
#[test]
#[timeout(4000)]
fn pathological_footnotes() {
    let n = 10_000;
    let input = format!("{}{}", "[^1]:".repeat(n), "\n".repeat(n));
    let exp = "";

    html_opts!([extension.footnotes], &input, &exp);
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
            "<li id=\"fn-__inline_1\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_2\" id=\"fnref-__inline_2\" data-footnote-ref>5</a></sup> b <a href=\"#fnref-__inline_1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_5\">\n<p>a ",
        ),
        "^[a ".repeat(4995),
        " b]".repeat(4995),
        concat!(
            " b <a href=\"#fnref-__inline_5\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"2\" aria-label=\"Back to reference 2\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_4\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_5\" id=\"fnref-__inline_5\" data-footnote-ref>2</a></sup> b <a href=\"#fnref-__inline_4\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"3\" aria-label=\"Back to reference 3\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_3\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_4\" id=\"fnref-__inline_4\" data-footnote-ref>3</a></sup> b <a href=\"#fnref-__inline_3\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"4\" aria-label=\"Back to reference 4\">↩</a></p>\n</li>\n",
            "<li id=\"fn-__inline_2\">\n<p>a <sup class=\"footnote-ref\"><a href=\"#fn-__inline_3\" id=\"fnref-__inline_3\" data-footnote-ref>4</a></sup> b <a href=\"#fnref-__inline_2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"5\" aria-label=\"Back to reference 5\">↩</a></p>\n</li>\n",
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
