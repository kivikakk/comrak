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
    let exp = format!("<p>{}{}</p>\n", "<em>a <strong>a ".repeat(n), " a</strong> a</em>".repeat(n));

    html_opts!([extension.footnotes], &input, &exp);
}
