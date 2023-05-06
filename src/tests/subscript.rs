use super::*;

#[test]
fn subscript() {
    html_opts!(
        [extension.subscript],
        concat!("Water is H~2~O.\n"),
        concat!("<p>Water is H<sub>2</sub>O.</p>\n"),
    );
}
