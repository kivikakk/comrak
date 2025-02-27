use super::*;

#[test]
fn pointy_brace_open() {
    html("<!-", "<p>&lt;!-</p>\n");
}

#[test]
fn tasklist() {
    html_opts!(
        [extension.tasklist, parse.relaxed_tasklist_matching],
        "* [*]",
        "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> </li>\n</ul>\n",
    );
}

#[test]
fn tasklist_with_classes() {
    html_opts!(
        [extension.tasklist, render.tasklist_classes, parse.relaxed_tasklist_matching],
        "* [*]",
        "<ul class=\"contains-task-list\">\n<li class=\"task-list-item\"><input type=\"checkbox\" class=\"task-list-item-checkbox\" checked=\"\" disabled=\"\" /> </li>\n</ul>\n",
    );
}

#[test]
fn table_nul() {
    html_opts!(
        [extension.table],
        "\0|.\n-|-\nZ",
        r##"<table>
<thead>
<tr>
<th>�</th>
<th>.</th>
</tr>
</thead>
<tbody>
<tr>
<td>Z</td>
<td></td>
</tr>
</tbody>
</table>
"##,
    );
}

#[test]
fn footnote_def() {
    html_opts!(
        [
            extension.autolink,
            extension.footnotes,
            render.sourcepos,
            render.hardbreaks
        ],
        "\u{15}\u{b}\r[^ ]:",
        "<p data-sourcepos=\"1:1-2:5\">\u{15}\u{b}<br />\n[^ ]:</p>\n",
    );
}

#[test]
fn line_end() {
    html("\u{2}\n\\\n\t-", "<p>\u{2}\n<br />\n-</p>\n");
}

#[test]
fn bracket_match() {
    html("[;\0V\n]::g\n[;\0V\n]", "<p><a href=\":g\">;�V\n</a></p>\n");
}

#[test]
fn trailing_hyphen() {
    assert_ast_match!(
        [extension.autolink, parse.smart],
        "3@.l-",
        (document (1:1-1:5) [
            (paragraph (1:1-1:5) [
                (text (1:1-1:5) "3@.l-")
            ])
        ])
    );
}

#[ignore]
#[test]
fn trailing_hyphen_matches() {
    assert_ast_match!(
        [extension.autolink, parse.smart],
        "3@.l--",
         (document (1:1-1:6) [
             (paragraph (1:1-1:6) [
                 (link (1:1-1:4) [
                     (text (1:1-1:4) "3@.l")
                 ])
                 (text (1:5-1:6) "–")
             ])
         ])
    );
}
