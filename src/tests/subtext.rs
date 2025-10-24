use super::*;

#[test]
fn subtext() {
    html_opts!(
        [extension.subtext],
        concat!("-# Some Subtext\n"),
        concat!("<p><sub>Some Subtext</sub></p>\n"),
    );
}
