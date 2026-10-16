use super::*;

#[test]
fn comments() {
    for (input, expected) in [
        ("x %%aa%% tail", "<p>x  tail</p>\n"),
        ("%%%%tail", "<p>tail</p>\n"),
        ("%%aa%%# heading", "<p># heading</p>\n"),
        ("%%a%b%%tail", "<p>tail</p>\n"),
        ("%%\n# aa\n\n- bb\n%%tail", "<p>tail</p>\n"),
        ("%%aa\nbb%%_tail_\nnext", "<p><em>tail</em>\nnext</p>\n"),
        (
            "before\n%%\naa%%# heading",
            "<p>before</p>\n<h1>heading</h1>\n",
        ),
        ("%%\naa\n\nbb", ""),
        ("x %%aa\nbb%% tail", "<p>x %%aa\nbb%% tail</p>\n"),
        ("%%a%b\ncc%%tail", "<p>%%a%b\ncc%%tail</p>\n"),
        ("- %%\n  aa%%bb", "<ul>\n<li>bb</li>\n</ul>\n"),
        ("   %%\r\naa%%bb", "<p>bb</p>\n"),
    ] {
        html_opts!([extension.ofm_comments], input, expected);
    }
}

#[test]
fn commonmark_comments() {
    let mut options = Options::default();
    options.extension.ofm_comments = true;

    commonmark(
        concat!(
            "100% %%inline%% \\%\\%text\\%\\% \\%%%hidden%%\n",
            "\n",
            "%%\n",
            "block\n",
            "%%tail\n",
        ),
        concat!(
            "100% %%inline%% \\%%text\\%% \\%%%hidden%%\n",
            "\n",
            "%%\n",
            "block\n",
            "%%tail\n",
        ),
        Some(&options),
    );
}

#[test]
fn comments_in_table() {
    html_opts!(
        [extension.ofm_comments, extension.table],
        "a %%hidden%% | b\n--- | ---\nx %%hidden%% y | z\n",
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a </th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>x  y</td>\n",
            "<td>z</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn comments_in_code() {
    html_opts!(
        [extension.ofm_comments],
        "`%%inline%%`\n\n```\n%%\n# text\n%%\n```\n",
        concat!(
            "<p><code>%%inline%%</code></p>\n",
            "<pre><code>%%\n# text\n%%\n</code></pre>\n",
        ),
    );
}

#[test]
fn comments_in_blockquote() {
    html_opts!(
        [extension.ofm_comments],
        "> %%\n> aa%%bb",
        "<blockquote>\n<p>bb</p>\n</blockquote>\n",
    );
}

#[test]
fn closing_comments_keep_list_tight() {
    html_opts!(
        [extension.ofm_comments],
        "- before\n  %%\n  hidden%%\n  after\n- %%\n  hidden%%\n- next",
        "<ul>\n<li>before\nafter</li>\n<li></li>\n<li>next</li>\n</ul>\n",
    );
}

#[test]
fn blank_line_in_comment_loosens_list() {
    html_opts!(
        [extension.ofm_comments],
        "- %%\n  a\n\n  b%%after\n- next",
        "<ul>\n<li>\n<p>after</p>\n</li>\n<li>\n<p>next</p>\n</li>\n</ul>\n",
    );
}

#[test]
fn tight_list_separator_survives_compact_html() {
    html_opts!(
        [extension.ofm_comments, render.compact_html],
        "- before\n  %%\n  hidden%%after\n- next",
        "<ul><li>before\nafter</li><li>next</li></ul>",
    );
}
