use super::*;

#[test]
fn superscript() {
    html_opts!(
        [extension.superscript],
        concat!("e = mc^2^.\n"),
        concat!("<p>e = mc<sup>2</sup>.</p>\n"),
    );
}

#[test]
fn superscript_negative_exponents_are_real() {
    html_opts!(
        [extension.superscript],
        concat!("i^-2^ = -1.\n"),
        concat!("<p>i<sup>-2</sup> = -1.</p>\n"),
    );
}
