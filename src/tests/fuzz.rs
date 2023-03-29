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
