use super::*;

#[test]
fn description_lists() {
    html_opts!(
        [extension.description_lists],
        concat!(
            "Term 1\n",
            "\n",
            ": Definition 1\n",
            "\n",
            "Term 2 with *inline markup*\n",
            "\n",
            ": Definition 2\n"
        ),
        concat!(
            "<dl>",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
        ),
    );

    html_opts!(
        [extension.description_lists],
        concat!(
            "* Nested\n",
            "\n",
            "    Term 1\n\n",
            "    :   Definition 1\n\n",
            "    Term 2 with *inline markup*\n\n",
            "    :   Definition 2\n\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>Nested</p>\n",
            "<dl>",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}
