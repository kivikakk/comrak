use super::*;

#[test]
fn table() {
    html_opts!(
        [extension.table],
        concat!("| a | b |\n", "|---|:-:|\n", "| c | d |\n"),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th align=\"center\">b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td align=\"center\">d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}

#[test]
fn table_regression() {
    html_opts!(
        [extension.table],
        concat!("123\n", "456\n", "| a | b |\n", "| ---| --- |\n", "d | e\n"),
        concat!(
            "<p>123\n",
            "456</p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>d</td>\n",
            "<td>e</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}

#[test]
fn table_misparse_1() {
    html_opts!([extension.table], "a\n-b", "<p>a\n-b</p>\n");
}

#[test]
fn table_misparse_2() {
    html_opts!([extension.table], "a\n-b\n-c", "<p>a\n-b\n-c</p>\n");
}

#[test]
fn nested_tables_1() {
    html_opts!(
        [extension.table],
        concat!("- p\n", "\n", "    |a|b|\n", "    |-|-|\n", "    |c|d|\n",),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>p</p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn nested_tables_2() {
    html_opts!(
        [extension.table],
        concat!("- |a|b|\n", "  |-|-|\n", "  |c|d|\n",),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn nested_tables_3() {
    html_opts!(
        [extension.table],
        concat!("> |a|b|\n", "> |-|-|\n", "> |c|d|\n",),
        concat!(
            "<blockquote>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</blockquote>\n",
        ),
    );
}
