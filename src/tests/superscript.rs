use super::*;

#[test]
fn superscript() {
    html_opts!(
        [extension.superscript],
        concat!("e = mc^2^.\n"),
        concat!("<p>e = mc<sup>2</sup>.</p>\n"),
    );
}
