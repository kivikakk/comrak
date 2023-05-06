use super::*;

#[test]
fn highlight() {
    html_opts!(
        [extension.highlight],
        concat!("This is an ==important== word.\n"),
        concat!("<p>This is an <mark>important</mark> word.</p>\n"),
    );
}
