use super::*;

#[test]
fn insert() {
    html_opts!(
        [extension.insert],
        concat!("This is an ++inserted++ text.\n"),
        concat!("<p>This is an <ins>inserted</ins> text.</p>\n"),
    );
}
