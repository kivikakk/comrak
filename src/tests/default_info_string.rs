use ntest::test_case;

use super::*;

#[test]
fn default_info_string_is_trimmed() {
    html_opts!(
        [parse.default_info_string = Some(" \trust ignore \n".into())],
        "```\nx\n```\n",
        "<pre><code class=\"language-rust\">x\n</code></pre>\n",
    );
}

#[test]
fn default_info_string_only_whitespace() {
    html_opts!(
        [parse.default_info_string = Some("  ".into())],
        "```\nx\n```\n",
        "<pre><code>x\n</code></pre>\n",
    );
}

#[test]
fn default_info_string_is_verbatim() {
    html_opts!(
        [
            parse.default_info_string = Some("a&amp;\\*`\nb".into()),
            render.full_info_string = true
        ],
        "```\nx\n```\n",
        "<pre><code class=\"language-a&amp;amp;\\*`\" data-meta=\"b\">x\n</code></pre>\n",
    );
}

#[test_case("    x\n", "    x\n", "<pre><code>x\n</code></pre>\n")]
#[test_case("     x\n", "     x\n", "<pre><code> x\n</code></pre>\n")]
#[test_case(
    "-     x\n      y\n",
    "-     x\n      y\n",
    "<ul>\n<li>\n<pre><code>x\ny\n</code></pre>\n</li>\n</ul>\n"
)]
#[test_case(
    "```\nx\n```\n\n    y\n",
    "```rust\nx\n```\n\n    y\n",
    "<pre><code class=\"language-rust\">x\n</code></pre>\n<pre><code>y\n</code></pre>\n"
)]
fn default_info_string_keeps_indented_code_indented(input: &str, cm: &str, expected: &str) {
    let mut options = Options::default();
    options.parse.default_info_string = Some("rust".into());
    commonmark(input, cm, Some(&options));
    html_opts_w(input, expected, true, &options);
}
