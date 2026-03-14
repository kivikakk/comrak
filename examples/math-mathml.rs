//! This example shows how to use the bundled pulldown-latex MathML plugin
//! to render math content as MathML.

use comrak::plugins::pulldown_latex::PulldownLatexAdapter;
use comrak::{Options, markdown_to_html_with_plugins, options};

fn main() {
    let adapter = PulldownLatexAdapter::default();

    let mut options = Options::default();
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    let mut plugins = options::Plugins::default();
    plugins.render.math_renderer = Some(&adapter);

    let examples = [
        ("Inline math", "The equation $E=mc^2$ is famous."),
        ("Display math", "$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$"),
        (
            "Code block math",
            "```math\n\\int_0^\\infty e^{-x} dx = 1\n```",
        ),
    ];

    for (label, input) in examples {
        println!("=== {} ===", label);
        println!("Input:  {}", input);
        let output = markdown_to_html_with_plugins(input, &options, &plugins);
        println!("Output: {}", output);
        println!();
    }
}
