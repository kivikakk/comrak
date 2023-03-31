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
    assert_ast_match!(
        [extension.table],
        "123\n"
        "456\n"
        "| a | b |\n"
        "| - | - |\n"
        "| c | d |\n"
        ,
        (document (1:1-5:9) [
            (paragraph (1:1-2:3) [
                (text (1:1-1:3) "123")
                (softbreak (1:4-1:4))
                (text (2:1-2:3) "456")
            ])
            (table (3:1-5:9) [
                (table_row (3:1-3:9) [
                    (table_cell (3:2-3:4) [
                        (text (3:3-3:3) "a")
                    ])
                    (table_cell (3:6-3:8) [
                        (text (3:7-3:7) "b")
                    ])
                ])
                (table_row (5:1-5:9) [
                    (table_cell (5:2-5:4) [
                        (text (5:3-5:3) "c")
                    ])
                    (table_cell (5:6-5:8) [
                        (text (5:7-5:7) "d")
                    ])
                ])
            ])
        ])
    );
}

#[test]
fn sourcepos_with_preceding_para_offset() {
    assert_ast_match!(
        [extension.table],
        " 123\n"
        "  456\n"
        " | a | b |\n"
        " | - | - |\n"
        " | c | d |\n"
        ,
        (document (1:1-5:10) [

            // XXX This should be 1:2-2:5; see
            // crate::parser::table::try_inserting_table_header_paragraph.
            (paragraph (1:2-2:4) [

                (text (1:2-1:4) "123")
                (softbreak (1:5-1:5))
                (text (2:2-2:4) "456")
            ])
            (table (3:2-5:10) [
                (table_row (3:2-3:10) [
                    (table_cell (3:3-3:5) [
                        (text (3:4-3:4) "a")
                    ])
                    (table_cell (3:7-3:9) [
                        (text (3:8-3:8) "b")
                    ])
                ])
                (table_row (5:2-5:10) [
                    (table_cell (5:3-5:5) [
                        (text (5:4-5:4) "c")
                    ])
                    (table_cell (5:7-5:9) [
                        (text (5:8-5:8) "d")
                    ])
                ])
            ])
        ])
    );
}
