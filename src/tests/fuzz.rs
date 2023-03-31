use super::{html, html_opts};

#[test]
fn pointy_brace_open() {
    html("<!-", "<p>&lt;!-</p>\n");
}

#[test]
fn tasklist() {
    html_opts!(
        [extension.tasklist, parse.relaxed_tasklist_matching],
        "* [*]",
        "<ul>\n<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> </li>\n</ul>\n",
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
        "<p data-sourcepos=\"1:1-2:5\">\u{15}\u{b}<br data-sourcepos=\"1:3-1:3\" />\n[^ ]:</p>\n",
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
    html_opts!(
        [extension.autolink, parse.smart, render.sourcepos],
        "3@.l-",
        "<p data-sourcepos=\"1:1-1:5\">3@.l-</p>\n"
    );
}

#[test]
fn trailing_hyphen_matches() {
    html_opts!(
        [extension.autolink, parse.smart, render.sourcepos],
        "3@.l--",
        "<p data-sourcepos=\"1:1-1:6\"><a href=\"mailto:3@.l\">3@.l</a>–</p>\n"
    );
}
