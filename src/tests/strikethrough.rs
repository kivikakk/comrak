use super::*;

#[test]
fn strikethrough() {
    html_opts!(
        [extension.strikethrough],
        concat!(
            "This is ~strikethrough~.\n",
            "\n",
            "As is ~~this, okay~~?\n"
        ),
        concat!(
            "<p>This is <del>strikethrough</del>.</p>\n",
            "<p>As is <del>this, okay</del>?</p>\n"
        ),
    );
}
