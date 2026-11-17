use super::*;
use ntest::test_case;

#[test_case("$2+2$", "<p><math>2+2</math></p>\n")]
#[test_case(
    "$\\alpha$$\\beta$",
    "<p><math>\\alpha</math><math>\\beta</math></p>\n"
)]
#[test_case("$2+2$$2+2$", "<p><math>2+2</math><math>2+2</math></p>\n")]
#[test_case("$22 and $2+2$", "<p>$22 and <math>2+2</math></p>\n")]
#[test_case("$a!$", "<p><math>a!</math></p>\n")]
#[test_case("$x$", "<p><math>x</math></p>\n")]
#[test_case("$1+2\\$$", "<p><math>1+2\\$</math></p>\n")]
#[test_case("$1+\\$2$", "<p><math>1+\\$2</math></p>\n")]
#[test_case("$1+\\%2$", "<p><math>1+\\%2</math></p>\n")]
#[test_case(
    "$22+1$ and $22 + a^2$",
    "<p><math>22+1</math> and <math>22 + a^2</math></p>\n"
)]
#[test_case(
    "$2+2$ $22 and dollars$22 $2+2$",
    "<p><math>2+2</math> $22 and dollars$22 <math>2+2</math></p>\n"
)]
#[test_case(
    "$1/2$ &lt;b&gt;test&lt;/b&gt;",
    "<p><math>1/2</math> &lt;b&gt;test&lt;/b&gt;</p>\n"
)]
fn math_dollars_inline(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<span data-math-style=\"inline\">")
        .replace("</math>", "</span>");

    html_opts!([extension.math_dollars], markdown, &result);
}

#[test_case("$$2+2$$", "<p><math>2+2</math></p>\n")]
#[test_case("$$   2+2  $$", "<p><math>   2+2  </math></p>\n")]
#[test_case("$22 and $$2+2$$", "<p>$22 and <math>2+2</math></p>\n")]
#[test_case("$$a!$$", "<p><math>a!</math></p>\n")]
#[test_case("$$x$$", "<p><math>x</math></p>\n")]
#[test_case("$$20,000 and $$30,000", "<p><math>20,000 and </math>30,000</p>\n")]
#[test_case("test $$\n2+2\n$$", "<p>test <math>\n2+2\n</math></p>\n")]
#[test_case(
    "$$22+1$$ and $$22 + a^2$$",
    "<p><math>22+1</math> and <math>22 + a^2</math></p>\n"
)]
#[test_case(
    "$$2+2$$ $22 and dollars$22 $$2+2$$",
    "<p><math>2+2</math> $22 and dollars$22 <math>2+2</math></p>\n"
)]
#[test_case(
    "dollars$22 and $$a^2 + b^2 = c^2$$",
    "<p>dollars$22 and <math>a^2 + b^2 = c^2</math></p>\n"
)]
fn math_dollars_inline_display(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<span data-math-style=\"display\">")
        .replace("</math>", "</span>");

    html_opts!([extension.math_dollars], markdown, &result);
}

#[test_case("$$\n2+2\n4+4\n$$", "<p><math>\n2+2\n4+4\n</math></p>\n")]
fn math_dollars_block(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<span data-math-style=\"display\">")
        .replace("</math>", "</span>");

    html_opts!([extension.math_dollars], markdown, &result);
}

#[test_case("\\(2+2\\)", "<p><math>2+2</math></p>\n")]
#[test_case(
    "Calc \\(x^2\\) and \\(y^2\\)",
    "<p>Calc <math>x^2</math> and <math>y^2</math></p>\n"
)]
#[test_case("\\(\\alpha + \\beta\\)", "<p><math>\\alpha + \\beta</math></p>\n")]
fn math_latex_inline(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<span data-math-style=\"inline\">")
        .replace("</math>", "</span>");

    html_opts!([extension.math_latex], markdown, &result);
}

#[test_case("\\[2+2\\]", "<p><math>2+2</math></p>\n")]
#[test_case("\\[\n2+2\n\\]", "<p><math>\n2+2\n</math></p>\n")]
fn math_latex_display(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<span data-math-style=\"display\">")
        .replace("</math>", "</span>");

    html_opts!([extension.math_latex], markdown, &result);
}

#[test_case("`\\(2+2\\)`", "<p><code>\\(2+2\\)</code></p>\n")]
#[test_case(
    "```text\n\\[2+2\\]\n```",
    "<pre><code class=\"language-text\">\\[2+2\\]\n</code></pre>\n"
)]
#[test_case("\\(2+2", "<p>(2+2</p>\n")]
#[test_case("\\[2+2", "<p>[2+2</p>\n")]
#[test_case("\\(x\\]", "<p>(x]</p>\n")]
#[test_case("\\[x\\)", "<p>[x)</p>\n")]
#[test_case("\\(\\)", "<p>()</p>\n")]
#[test_case("\\[\\]", "<p>[]</p>\n")]
fn math_latex_unrecognized_syntax(markdown: &str, html: &str) {
    html_opts!([extension.math_latex], markdown, html);
}

#[test_case("$`2+2`$", "<p><math>2+2</math></p>\n")]
#[test_case("$22 and $`2+2`$", "<p>$22 and <math>2+2</math></p>\n")]
#[test_case("$`1+\\$2`$", "<p><math>1+\\$2</math></p>\n")]
#[test_case(
    "$`22+1`$ and $`22 + a^2`$",
    "<p><math>22+1</math> and <math>22 + a^2</math></p>\n"
)]
#[test_case(
    "$`2+2`$ $22 and dollars$22 $`2+2`$",
    "<p><math>2+2</math> $22 and dollars$22 <math>2+2</math></p>\n"
)]
fn math_code_inline(markdown: &str, html: &str) {
    let result = html
        .replace("<math>", "<code data-math-style=\"inline\">")
        .replace("</math>", "</code>");

    html_opts!([extension.math_code], markdown, &result);
}

#[test_case("```math\n2+2\n4+4\n```", "<pre><math>2+2\n4+4\n</math></pre>\n")]
fn math_code_block(markdown: &str, html: &str) {
    let result = html
        .replace(
            "<math>",
            "<code class=\"language-math\" data-math-style=\"display\">",
        )
        .replace("</math>", "</code>");

    html_opts!([extension.math_dollars], markdown, &result);
}

#[test_case("`2+2`", "<p><code>2+2</code></p>\n")]
#[test_case("test `2+2`$ test", "<p>test <code>2+2</code>$ test</p>\n")]
#[test_case("$20,000 and $30,000", "<p>$20,000 and $30,000</p>\n")]
#[test_case("$20,000 in $USD", "<p>$20,000 in $USD</p>\n")]
#[test_case("$ a^2 $", "<p>$ a^2 $</p>\n")]
#[test_case("$\n$", "<p>$\n$</p>\n")]
#[test_case("$$$", "<p>$$$</p>\n")]
#[test_case("`$1+2$`", "<p><code>$1+2$</code></p>\n")]
#[test_case("`$$1+2$$`", "<p><code>$$1+2$$</code></p>\n")]
#[test_case("`$\\$1+2$$`", "<p><code>$\\$1+2$$</code></p>\n")]
#[test_case("test $`2+2` test", "<p>test $<code>2+2</code> test</p>\n")]
fn math_unrecognized_syntax(markdown: &str, html: &str) {
    html_opts!(
        [extension.math_dollars, extension.math_code],
        markdown,
        html
    );
}

#[test_case("$`$", "<p data-sourcepos=\"1:1-1:3\">$`$</p>\n")]
fn math_unrecognized_syntax_non_roundtrip(markdown: &str, html: &str) {
    html_opts!(
        [
            extension.math_dollars,
            extension.math_code,
            render.sourcepos
        ],
        markdown,
        html,
        no_roundtrip
    );
}

#[test]
fn sourcepos() {
    assert_ast_match!(
        [extension.math_dollars, extension.math_code],
        "$x^2$ and $$y^2$$ and $`z^2`$\n"
        "\n"
        "$$\n"
        "a^2\n"
        "$$\n"
        "\n"
        "```math\n"
        "b^2\n"
        "```\n",
        (document (1:1-9:3) [
            (paragraph (1:1-1:29) [
                (math (1:1-1:5) "x^2")
                (text (1:6-1:10) " and ")
                (math (1:11-1:17) "y^2")
                (text (1:18-1:22) " and ")
                (math (1:23-1:29) "z^2")
            ])
            (paragraph (3:1-5:2) [
                (math (3:1-5:2) "\na^2\n")
            ])
            (code_block (7:1-9:3) info:"math" "b^2\n")
        ])
    );
}

#[test]
fn math_latex_sourcepos() {
    assert_ast_match!(
        [extension.math_latex],
        "\\(x^2\\) and \\[y^2\\]\n"
        "\n"
        "\\[\n"
        "a^2\n"
        "\\]\n",
        (document (1:1-5:2) [
            (paragraph (1:1-1:19) [
                (math (1:1-1:7) "x^2")
                (text (1:8-1:12) " and ")
                (math (1:13-1:19) "y^2")
            ])
            (paragraph (3:1-5:2) [
                (math (3:1-5:2) "\na^2\n")
            ])
        ])
    );
}

#[test_case("a [x] b", "a &#91;x\\] b\n", "<p>a [x] b</p>\n")]
#[test_case("\\\\[x\\\\]", "\\\\&#91;x\\\\\\]\n", "<p>\\[x\\]</p>\n")]
#[test_case(
    "\\[x\\] [y]",
    "\\[x\\] &#91;y\\]\n",
    "<p><span data-math-style=\"display\">x</span> [y]</p>\n"
)]
fn math_latex_text_brackets_stay_text(markdown: &str, cm: &str, html: &str) {
    let mut options = Options::default();
    options.extension.math_latex = true;
    commonmark(markdown, cm, Some(&options));
    html_opts!([extension.math_latex], markdown, html);
}

#[test_case("a \\$x$ b", "a \\$x\\$ b\n", "<p>a $x$ b</p>\n")]
#[test_case(
    "$x$ and \\$5",
    "$x$ and \\$5\n",
    "<p><span data-math-style=\"inline\">x</span> and $5</p>\n"
)]
#[test_case("\\$`x`\\$", "\\$`x`\\$\n", "<p>$<code>x</code>$</p>\n")]
fn math_dollars_text_dollars_stay_text(markdown: &str, cm: &str, html: &str) {
    let mut options = Options::default();
    options.extension.math_dollars = true;
    commonmark(markdown, cm, Some(&options));
    html_opts!([extension.math_dollars], markdown, html);
}

#[test_case("a \\$x$ b", "a \\$x\\$ b\n", "<p>a $x$ b</p>\n")]
#[test_case("\\$`x`\\$", "\\$`x`\\$\n", "<p>$<code>x</code>$</p>\n")]
fn math_code_text_dollars_stay_text(markdown: &str, cm: &str, html: &str) {
    let mut options = Options::default();
    options.extension.math_code = true;
    commonmark(markdown, cm, Some(&options));
    html_opts!([extension.math_code], markdown, html);
}

#[test_case(
    "\\( x\\)",
    "\\( x\\)\n",
    "<p><span data-math-style=\"inline\"> x</span></p>\n"
)]
#[test_case(
    "\\(a$b\\)",
    "\\(a$b\\)\n",
    "<p><span data-math-style=\"inline\">a$b</span></p>\n"
)]
#[test_case(
    "\\(x\\)5",
    "\\(x\\)5\n",
    "<p><span data-math-style=\"inline\">x</span>5</p>\n"
)]
#[test_case(
    "\\[a$$b\\]",
    "\\[a$$b\\]\n",
    "<p><span data-math-style=\"display\">a$$b</span></p>\n"
)]
#[test_case(
    "\\(x\\)",
    "$x$\n",
    "<p><span data-math-style=\"inline\">x</span></p>\n"
)]
fn math_latex_keeps_form_dollars_cannot_hold(markdown: &str, cm: &str, html: &str) {
    let mut options = Options::default();
    options.extension.math_latex = true;
    options.extension.math_dollars = true;
    commonmark(markdown, cm, Some(&options));
    html_opts!(
        [extension.math_latex, extension.math_dollars],
        markdown,
        html
    );
}

#[test]
fn math_latex_keeps_form_before_math_code_opener() {
    let mut options = Options::default();
    options.extension.math_latex = true;
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    commonmark("\\(`x\\)", "\\(`x\\)\n", Some(&options));
    html_opts!(
        [
            extension.math_latex,
            extension.math_dollars,
            extension.math_code
        ],
        "\\(`x\\)",
        "<p><span data-math-style=\"inline\">`x</span></p>\n"
    );
}

#[test_case(
    "$$\n1\n=\n2\n$$\n",
    "<div data-math-style=\"display\">\n1\n=\n2\n</div>\n"
)]
#[test_case(
    "$$\n# heading\n\n- list\n$$\n",
    "<div data-math-style=\"display\">\n# heading\n\n- list\n</div>\n"
)]
#[test_case(
    "$$\n<span>&</span>\n$$\n",
    "<div data-math-style=\"display\">\n&lt;span&gt;&amp;&lt;/span&gt;\n</div>\n"
)]
#[test_case("$$\n$$\n", "<div data-math-style=\"display\">\n</div>\n")]
#[test_case(
    "before\n$$\na\n$$\nafter\n",
    "<p>before</p>\n<div data-math-style=\"display\">\na\n</div>\n<p>after</p>\n"
)]
#[test_case(
    "$$\na\n$$\n$$\nb\n$$\n",
    "<div data-math-style=\"display\">\na\n</div>\n<div data-math-style=\"display\">\nb\n</div>\n"
)]
#[test_case(
    "> $$\n> a\n> =\n> b\n> $$\n",
    "<blockquote>\n<div data-math-style=\"display\">\na\n=\nb\n</div>\n</blockquote>\n"
)]
#[test_case(
    "- $$\n  a\n  =\n  b\n  $$\n",
    "<ul>\n<li>\n<div data-math-style=\"display\">\na\n=\nb\n</div>\n</li>\n</ul>\n"
)]
#[test_case(
    "$$\na\n=\nb\n",
    "<div data-math-style=\"display\">\na\n=\nb\n</div>\n"
)]
#[test_case("$$$\na\n$$$\n", "<p>$$$\na\n$$$</p>\n")]
#[test_case("$$ trailing\na\n", "<p>$$ trailing\na</p>\n")]
#[test_case(
    "$$\na\n$$ trailing\n",
    "<div data-math-style=\"display\">\na\n$$ trailing\n</div>\n"
)]
fn math_dollars_block_standalone(markdown: &str, html: &str) {
    html_opts!(
        [extension.math_dollars, extension.math_dollars_block],
        markdown,
        html
    );
}

#[test_case(
    "$$\n1\n2\n$$\n",
    "<p><span data-math-style=\"display\">\n1\n2\n</span></p>\n"
)]
fn math_dollars_block_disabled(markdown: &str, html: &str) {
    html_opts!([extension.math_dollars], markdown, html);
}

#[test_case("   $$\n    $$\n   $$\n", " $$\n")]
#[test_case("  $$\n    $$\n  $$\n", "  $$\n")]
#[test_case(" $$\n    $$\n $$\n", "   $$\n")]
#[test_case("   $$\n\t$$\n   $$\n", " $$\n")]
fn math_block_indented_literal_fence_roundtrip(input: &str, literal: &str) {
    html_opts!(
        [extension.math_dollars, extension.math_dollars_block],
        input,
        &format!("<div data-math-style=\"display\">\n{literal}</div>\n")
    );
}

#[test_case(">    $$\n>     $$\n>    $$\n")]
#[test_case("- item\n\n     $$\n      $$\n     $$\n")]
fn math_block_indented_container_roundtrip(input: &str) {
    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.math_dollars_block = true;
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);
    assert!(
        root.descendants().any(
            |node| matches!(&node.data().value, NodeValue::MathBlock(m) if m.literal == " $$\n")
        )
    );
    let expected = markdown_to_html(input, &options);
    html_opts_w(input, &expected, true, &options);
}

#[test_case(false)]
#[test_case(true)]
fn math_block_latex_display_roundtrip(block: bool) {
    html_opts!(
        [
            extension.math_latex = true,
            extension.math_dollars = true,
            extension.math_dollars_block = block
        ],
        "\\[\nx\n\\]\n",
        "<p><span data-math-style=\"display\">\nx\n</span></p>\n"
    );
}

#[test]
fn math_block_callout_title_roundtrip() {
    let mut options = Options::default();
    options.extension.ofm_callouts = true;
    options.extension.math_dollars = true;
    options.extension.math_dollars_block = true;
    let input = "> [!note] $$\n> body\n";
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);
    let math = root
        .descendants()
        .find(|node| matches!(node.data().value, NodeValue::MathBlock(_)))
        .unwrap();
    assert!(matches!(
        math.parent().unwrap().data().value,
        NodeValue::CalloutTitle
    ));
    assert!(matches!(&math.data().value, NodeValue::MathBlock(m) if m.literal.is_empty()));
    let expected = markdown_to_html(input, &options);
    html_opts_w(input, &expected, true, &options);
}
