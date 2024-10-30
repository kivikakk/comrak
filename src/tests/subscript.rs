use super::*;

#[test]
fn subscript() {
    html_opts!(
        [extension.subscript],
        concat!("H%2%O\n"),
        concat!("<p>H<sub>2</sub>O</p>\n"),
    );
}
