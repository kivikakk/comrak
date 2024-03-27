use super::*;
use ntest::test_case;

#[test]
fn commonmark_removes_redundant_strong() {
    let options = Options::default();

    let input = "This is **something **even** better**";
    let output = "This is **something even better**\n";

    commonmark(input, output, Some(&options));
}

#[test_case("$$x^2$$ and $1 + 2$ and $`y^2`$", "$$x^2$$ and $1 + 2$ and $`y^2`$\n")]
#[test_case("$$\nx^2\n$$", "$$\nx^2\n$$\n")]
#[test_case("```math\nx^2\n```", "``` math\nx^2\n```\n")]
fn math(markdown: &str, cm: &str) {
    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    commonmark(markdown, cm, Some(&options));
}
