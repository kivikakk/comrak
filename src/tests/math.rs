use super::*;
use ntest::test_case;

#[test_case("$2+2$", "<p><math>2+2</math></p>\n")]
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
        .replace("<math>", "<code data-math-style=\"inline\">")
        .replace("</math>", "</code>");

    html_opts!([extension.math_dollars], markdown, &result);
}

#[test_case("$$2+2$$", "<p><math>2+2</math></p>\n")]
#[test_case("$$   2+2  $$", "<p><math>  2+2 </math></p>\n")]
#[test_case("$22 and $$2+2$$", "<p>$22 and <math>2+2</math></p>\n")]
#[test_case("$$a!$$", "<p><math>a!</math></p>\n")]
#[test_case("$$x$$", "<p><math>x</math></p>\n")]
#[test_case("$$20,000 and $$30,000", "<p><math>20,000 and </math>30,000</p>\n")]
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
        .replace("<math>", "<code data-math-style=\"display\">")
        .replace("</math>", "</code>");

    html_opts!([extension.math_dollars], markdown, &result);
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

#[test_case("`2+2`", "<p><code>2+2</code></p>\n")]
// #[test_case("test $`2+2` test", "<p>test $<code>2+2</code> test</p>\n")]
#[test_case("test `2+2`$ test", "<p>test <code>2+2</code>$ test</p>\n")]
#[test_case("$20,000 and $30,000", "<p>$20,000 and $30,000</p>\n")]
#[test_case("$20,000 in $USD", "<p>$20,000 in $USD</p>\n")]
#[test_case("$ a^2 $", "<p>$ a^2 $</p>\n")]
// #[test_case("test $$\n2+2\n$$", "<p>test $$\n2+2\n$$</p>\n")]
#[test_case("$\n$", "<p>$\n$</p>\n")]
#[test_case("$$$", "<p>$$$</p>\n")]
#[test_case("`$1+2$`", "<p><code>$1+2$</code></p>\n")]
#[test_case("`$$1+2$$`", "<p><code>$$1+2$$</code></p>\n")]
#[test_case("`$\\$1+2$$`", "<p><code>$\\$1+2$$</code></p>\n")]
fn math_unrecognized_syntax(markdown: &str, html: &str) {
    html_opts!(
        [extension.math_dollars, extension.math_code],
        markdown,
        html
    );
}

// #[test]
// fn sourcepos() {
//     assert_ast_match!(
//         [extension.multiline_block_quotes],
//         "- item one\n"
//         "\n"
//         "  >>>\n"
//         "  Paragraph 1\n"
//         "  >>>\n"
//         "- item two\n",
//         (document (1:1-6:10) [
//             (list (1:1-6:10) [
//                 (item (1:1-5:5) [      // (description_item (1:1-3:4) [
//                     (paragraph (1:3-1:10) [
//                         (text (1:3-1:10) "item one")
//                     ])
//                     (multiline_block_quote (3:3-5:5) [
//                         (paragraph (4:3-4:13) [
//                             (text (4:3-4:13) "Paragraph 1")
//                         ])
//                     ])
//                 ])
//                 (item (6:1-6:10) [      // (description_item (5:1-7:6) [
//                     (paragraph (6:3-6:10) [
//                         (text (6:3-6:10) "item two")
//                     ])
//                 ])
//             ])
//         ])
//     );
// }
