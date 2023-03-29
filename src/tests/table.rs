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

#[test]
fn sourcepos_with_preceding_para() {
    html_opts!(
        [extension.table, render.sourcepos],
        concat!(
            "123\n",
            "456\n",
            "| a | b |\n",
            "| - | - |\n",
            "| c | d |\n"
        ),
        concat!(
            "<p data-sourcepos=\"1:1-2:3\">123\n",
            "456</p>\n",
            "<table data-sourcepos=\"3:1-5:9\">\n",
            "<thead>\n",
            "<tr data-sourcepos=\"3:1-3:9\">\n",
            "<th data-sourcepos=\"3:2-3:4\">a</th>\n",
            "<th data-sourcepos=\"3:6-3:8\">b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr data-sourcepos=\"5:1-5:9\">\n",
            "<td data-sourcepos=\"5:2-5:4\">c</td>\n",
            "<td data-sourcepos=\"5:6-5:8\">d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}
#[test]
fn sourcepos_with_preceding_para_offset() {
    html_opts!(
        [extension.table, render.sourcepos],
        concat!(
            " 123\n",
            "  456\n",
            " | a | b |\n",
            " | - | - |\n",
            " | c | d |\n"
        ),
        concat!(
            // XXX This should be 1:2-2:5; see
            // crate::parser::table::try_inserting_table_header_paragraph.
            "<p data-sourcepos=\"1:2-2:4\">123\n",
            "456</p>\n",
            "<table data-sourcepos=\"3:2-5:10\">\n",
            "<thead>\n",
            "<tr data-sourcepos=\"3:2-3:10\">\n",
            "<th data-sourcepos=\"3:3-3:5\">a</th>\n",
            "<th data-sourcepos=\"3:7-3:9\">b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr data-sourcepos=\"5:2-5:10\">\n",
            "<td data-sourcepos=\"5:3-5:5\">c</td>\n",
            "<td data-sourcepos=\"5:7-5:9\">d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}
