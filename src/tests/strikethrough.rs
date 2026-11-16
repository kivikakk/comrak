use super::*;

#[test]
fn strikethrough() {
    html_opts!(
        [extension.strikethrough],
        concat!(
            "This is ~strikethrough~.\n",
            "\n",
            "As is ~~this, okay~~?\n",
            "\n",
            "This ~text~~~~ is ~~~~curious~.\n",
        ),
        concat!(
            "<p>This is <del>strikethrough</del>.</p>\n",
            "<p>As is <del>this, okay</del>?</p>\n",
            "<p>This <del>text~~~~ is ~~~~curious</del>.</p>\n",
        ),
    );
}

#[test]
fn strikethrough_unequal_pair_does_not_stop_processing() {
    html_opts!(
        [extension.strikethrough],
        concat!(
            "~~a~ *b*\n",
            "\n",
            "~a~~ and *b* and ~c~\n",
            "\n",
            "~~a ~b~~ c~~\n",
            "\n",
            "~~a~ ~b~~ ~c~\n",
        ),
        concat!(
            "<p>~~a~ <em>b</em></p>\n",
            "<p>~a~~ and <em>b</em> and <del>c</del></p>\n",
            "<p><del>a ~b~~ c</del></p>\n",
            "<p>~~a~ ~b~~ <del>c</del></p>\n",
        ),
    );
}

#[test]
fn subscript_unequal_pair_does_not_stop_processing() {
    html_opts!(
        [extension.subscript],
        "~a~~ and *b* and ~c~\n",
        "<p>~a~~ and <em>b</em> and <sub>c</sub></p>\n",
    );
}
