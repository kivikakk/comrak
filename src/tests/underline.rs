use super::*;

#[test]
fn underline() {
    html_opts!(
        [extension.underline],
        concat!("__underlined text__\n"),
        concat!("<p><u>underlined text</u></p>\n"),
    );
}
