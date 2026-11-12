use crate::nodes::{
    Ast, GridTableColumn, GridTableSection, LineColumn, NodeGridTable, NodeGridTableCell,
    TableAlignment,
};

use super::*;

#[test]
fn xml() {
    let arena = Arena::new();
    let options = Options::default();
    let empty = LineColumn { line: 0, column: 0 };
    let ast = |val: NodeValue| &*arena.alloc(Ast::new(val, empty).into());
    let cell = |column, colspan, rowspan, text: &'static str| {
        let cell = ast(NodeValue::GridTableCell(NodeGridTableCell {
            column,
            colspan,
            rowspan,
        }));
        let para = ast(NodeValue::Paragraph);
        para.append(ast(NodeValue::Text(text.into())));
        cell.append(para);
        cell
    };

    let root = ast(NodeValue::Document);
    let table = ast(NodeValue::GridTable(Box::new(NodeGridTable {
        columns: vec![
            GridTableColumn {
                alignment: TableAlignment::None,
                width: 3,
            },
            GridTableColumn {
                alignment: TableAlignment::Right,
                width: 5,
            },
        ],
    })));
    root.append(table);

    let head = ast(NodeValue::GridTableRow(GridTableSection::Head));
    head.append(cell(0, 1, 2, "a"));
    head.append(cell(1, 1, 1, "b"));
    table.append(head);

    let body = ast(NodeValue::GridTableRow(GridTableSection::Body));
    body.append(cell(1, 1, 1, "c"));
    table.append(body);

    let foot = ast(NodeValue::GridTableRow(GridTableSection::Foot));
    foot.append(cell(0, 2, 1, "d"));
    table.append(foot);

    let mut output = String::new();
    crate::xml::format_document(root, &options, &mut output).unwrap();
    compare_strs(
        &output,
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <grid_table alignments=\"none right\" widths=\"3 5\">\n",
            "    <grid_table_row section=\"head\">\n",
            "      <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"2\">\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">a</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "      <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">b</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "    </grid_table_row>\n",
            "    <grid_table_row section=\"body\">\n",
            "      <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">c</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "    </grid_table_row>\n",
            "    <grid_table_row section=\"foot\">\n",
            "      <grid_table_cell column=\"0\" colspan=\"2\" rowspan=\"1\">\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">d</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "    </grid_table_row>\n",
            "  </grid_table>\n",
            "</document>\n",
        ),
        "rendered",
        "<synthetic>",
    );
}

#[track_caller]
fn grid_xml(input: &str, expected: &str) {
    grid_xml_opts(input, expected, |_| ());
}

#[track_caller]
fn grid_xml_opts<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut Options),
{
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.grid_table = true;
    opts(&mut options);

    let root = parse_document(&arena, input, &options);
    let mut output = String::new();
    crate::xml::format_document(root, &options, &mut output).unwrap();
    let body: String = output.lines().skip(3).collect::<Vec<_>>()[..]
        .split_last()
        .unwrap()
        .1
        .iter()
        .map(|l| format!("{}\n", l.strip_prefix("  ").unwrap_or(l)))
        .collect();
    compare_strs(&body, expected, "regular", input);
}

#[test]
fn spans() {
    grid_xml(
        concat!(
            "+---+---+---+\n",
            "| AAAAA | B |\n",
            "+---+---+ B +\n",
            "| D | E | B |\n",
            "+ D +---+---+\n",
            "| D | CCCCC |\n",
            "+---+---+---+\n",
        ),
        concat!(
            "<grid_table alignments=\"none none none\" widths=\"3 3 3\">\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"2\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">AAAAA</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell column=\"2\" colspan=\"1\" rowspan=\"2\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">B</text>\n",
            "        <softbreak />\n",
            "        <text xml:space=\"preserve\">B</text>\n",
            "        <softbreak />\n",
            "        <text xml:space=\"preserve\">B</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"2\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">D</text>\n",
            "        <softbreak />\n",
            "        <text xml:space=\"preserve\">D</text>\n",
            "        <softbreak />\n",
            "        <text xml:space=\"preserve\">D</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">E</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"1\" colspan=\"2\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">CCCCC</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
    );
}

#[test]
fn sections_and_blocks_in_cells() {
    grid_xml(
        concat!(
            "+-----+:---------:+\n",
            "| a   | b         |\n",
            "+=====+===========+\n",
            "| c   | - d       |\n",
            "|     | - e       |\n",
            "|     |           |\n",
            "|     |     f     |\n",
            "+=====+===========+\n",
            "| g   | h         |\n",
            "+=====+===========+\n",
        ),
        concat!(
            "<grid_table alignments=\"none none\" widths=\"5 11\">\n",
            "  <grid_table_row section=\"head\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">a</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">b</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">c</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "      <list type=\"bullet\" tight=\"false\">\n",
            "        <item>\n",
            "          <paragraph>\n",
            "            <text xml:space=\"preserve\">d</text>\n",
            "          </paragraph>\n",
            "        </item>\n",
            "        <item>\n",
            "          <paragraph>\n",
            "            <text xml:space=\"preserve\">e</text>\n",
            "          </paragraph>\n",
            "          <paragraph>\n",
            "            <text xml:space=\"preserve\">f</text>\n",
            "          </paragraph>\n",
            "        </item>\n",
            "      </list>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "  <grid_table_row section=\"foot\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">g</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">h</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
    );
}

#[test]
fn tables_in_cells() {
    grid_xml_opts(
        concat!(
            "+---------------+\n",
            "| +---+         |\n",
            "| | a |         |\n",
            "| +---+         |\n",
            "|               |\n",
            "| | b | c |     |\n",
            "| |---|---|     |\n",
            "+---------------+\n",
        ),
        concat!(
            "<grid_table alignments=\"none\" widths=\"15\">\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <grid_table alignments=\"none\" widths=\"3\">\n",
            "        <grid_table_row section=\"body\">\n",
            "          <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "            <paragraph>\n",
            "              <text xml:space=\"preserve\">a</text>\n",
            "            </paragraph>\n",
            "          </grid_table_cell>\n",
            "        </grid_table_row>\n",
            "      </grid_table>\n",
            "      <table>\n",
            "        <table_row>\n",
            "          <table_cell>\n",
            "            <text xml:space=\"preserve\">b</text>\n",
            "          </table_cell>\n",
            "          <table_cell>\n",
            "            <text xml:space=\"preserve\">c</text>\n",
            "          </table_cell>\n",
            "        </table_row>\n",
            "      </table>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
        |o| o.extension.table = true,
    );
}

#[test]
fn inside_containers() {
    grid_xml(
        concat!(
            "- x\n",
            "\n",
            "  +---+\n",
            "  | a |\n",
            "  +---+\n",
            "\n",
            "> +---+\n",
            "> | b |\n",
            "> +---+\n",
        ),
        concat!(
            "<list type=\"bullet\" tight=\"false\">\n",
            "  <item>\n",
            "    <paragraph>\n",
            "      <text xml:space=\"preserve\">x</text>\n",
            "    </paragraph>\n",
            "    <grid_table alignments=\"none\" widths=\"3\">\n",
            "      <grid_table_row section=\"body\">\n",
            "        <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "          <paragraph>\n",
            "            <text xml:space=\"preserve\">a</text>\n",
            "          </paragraph>\n",
            "        </grid_table_cell>\n",
            "      </grid_table_row>\n",
            "    </grid_table>\n",
            "  </item>\n",
            "</list>\n",
            "<block_quote>\n",
            "  <grid_table alignments=\"none\" widths=\"3\">\n",
            "    <grid_table_row section=\"body\">\n",
            "      <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">b</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "    </grid_table_row>\n",
            "  </grid_table>\n",
            "</block_quote>\n",
        ),
    );
}

#[test]
fn indentation() {
    grid_xml(
        "   +---+\n   | a |\n   +---+\n",
        concat!(
            "<grid_table alignments=\"none\" widths=\"3\">\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <text xml:space=\"preserve\">a</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
    );
    grid_xml(
        "    +---+\n    | a |\n    +---+\n",
        "<code_block xml:space=\"preserve\">+---+\n| a |\n+---+\n</code_block>\n",
    );
}

#[test]
fn does_not_interrupt_a_paragraph() {
    grid_xml(
        "x\n+---+\n| a |\n+---+\n",
        concat!(
            "<paragraph>\n",
            "  <text xml:space=\"preserve\">x</text>\n",
            "  <softbreak />\n",
            "  <text xml:space=\"preserve\">+---+</text>\n",
            "  <softbreak />\n",
            "  <text xml:space=\"preserve\">| a |</text>\n",
            "  <softbreak />\n",
            "  <text xml:space=\"preserve\">+---+</text>\n",
            "</paragraph>\n",
        ),
    );
}

#[track_caller]
fn same_as_extension_off<F>(input: &str, opts: F)
where
    F: Fn(&mut Options),
{
    let arena = Arena::new();
    let mut off = Options::default();
    off.render.sourcepos = true;
    opts(&mut off);
    let mut on = off.clone();
    on.extension.grid_table = true;

    let root = parse_document(&arena, input, &on);
    assert!(
        root.descendants()
            .all(|n| !node_matches!(n, NodeValue::GridTable(..))),
        "{input:?} has a grid table"
    );
    let mut with = String::new();
    crate::xml::format_document(root, &on, &mut with).unwrap();

    let root = parse_document(&arena, input, &off);
    let mut without = String::new();
    crate::xml::format_document(root, &off, &mut without).unwrap();

    compare_strs(&with, &without, "extension off", input);
}

#[test]
fn invalid_tables_parse_as_without_the_extension() {
    for input in [
        "+---+\n| a |\n",
        "+---+\n| a |\n+ b +\n",
        "+---+\n| a |\n+---+ x\n",
        "+---+\n| a\n===\n",
        "+---+\n| a\n---\n",
        "+---+\n| a\nb\n",
        "+---+\n| a\n\nb\n",
        "> +---+\n| a |\n+---+\n",
        "> +---+\n> | a\nb\n",
        "- +---+\n  | a\nb\n",
        "- +---+\n  | a\n- b\n",
        "  +---+\n  | a |\n   +---+\n",
        "+---+\n | a |\n+---+\n",
        "+---+\n| a |\n+ ```\n  x\n  ```\n",
        "+---+\n| a |\n+ ```\nx\n",
        "+---+\n|\ta\n+---+\n",
        "+---+\n| [x]: /u\n\n[x]\n",
        "+-----+\n| a  |\n+-----+\n",
        "+---+\n| 日 |\n+---+\n",
        "+-+\r\nb",
        "+-+\n|\r\n+",
        "+-+\r| a\rb\r",
    ] {
        same_as_extension_off(input, |_| ());
    }

    same_as_extension_off("+---+---+\n| a | b |\n|---|---|\n| c | d |\n", |o| {
        o.extension.table = true
    });
    same_as_extension_off("| a |\n|---|\n+---+\n| b |\n+---+\n", |o| {
        o.extension.table = true
    });
    same_as_extension_off(">>>\n+---+\n| a |\n>>>\nb\n", |o| {
        o.extension.multiline_block_quotes = true
    });
}

#[test]
fn link_reference_definitions() {
    grid_xml(
        concat!(
            "+-----------+\n",
            "| [a]: /a   |\n",
            "|           |\n",
            "| [b]       |\n",
            "+-----------+\n",
            "\n",
            "[a]\n",
            "\n",
            "[b]: /b\n",
        ),
        concat!(
            "<grid_table alignments=\"none\" widths=\"11\">\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph>\n",
            "        <link destination=\"/b\" title=\"\">\n",
            "          <text xml:space=\"preserve\">b</text>\n",
            "        </link>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
            "<paragraph>\n",
            "  <link destination=\"/a\" title=\"\">\n",
            "    <text xml:space=\"preserve\">a</text>\n",
            "  </link>\n",
            "</paragraph>\n",
        ),
    );
}

#[test]
fn leading_column_stripped_from_tabs() {
    grid_xml(
        "+---------+\n|\t    x |\n+---------+\n",
        concat!(
            "<grid_table alignments=\"none\" widths=\"9\">\n",
            "  <grid_table_row section=\"body\">\n",
            "    <grid_table_cell column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <code_block xml:space=\"preserve\">  x\n",
            "</code_block>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
    );
}

#[test]
fn sourcepos() {
    grid_xml_opts(
        concat!("> +-----+---+\n", "> | *a* | b |\n", "> +-----+---+\n",),
        concat!(
            "<block_quote sourcepos=\"1:1-3:13\">\n",
            "  <grid_table sourcepos=\"1:3-3:13\" alignments=\"none none\" widths=\"5 3\">\n",
            "    <grid_table_row sourcepos=\"2:3-2:13\" section=\"body\">\n",
            "      <grid_table_cell sourcepos=\"2:4-2:8\" column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "        <paragraph sourcepos=\"2:5-2:7\">\n",
            "          <emph sourcepos=\"2:5-2:7\">\n",
            "            <text sourcepos=\"2:6-2:6\" xml:space=\"preserve\">a</text>\n",
            "          </emph>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "      <grid_table_cell sourcepos=\"2:10-2:12\" column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "        <paragraph sourcepos=\"2:11-2:11\">\n",
            "          <text sourcepos=\"2:11-2:11\" xml:space=\"preserve\">b</text>\n",
            "        </paragraph>\n",
            "      </grid_table_cell>\n",
            "    </grid_table_row>\n",
            "  </grid_table>\n",
            "</block_quote>\n",
        ),
        |o| o.render.sourcepos = true,
    );
}

#[test]
fn extension_off() {
    let arena = Arena::new();
    let root = parse_document(&arena, "+---+\n| a |\n+---+\n", &Options::default());
    assert!(
        root.descendants()
            .all(|n| !node_matches!(n, NodeValue::GridTable(..)))
    );
}

#[test]
fn sourcepos_of_breaks_rows_and_placeholders() {
    grid_xml_opts(
        concat!(
            "+-------+---+\n",
            "| a     | c |\n",
            "| b\\    |   |\n",
            "| d     |   |\n",
            "+-------+---+\n",
            "+-------+---+\n",
        ),
        concat!(
            "<grid_table sourcepos=\"1:1-6:13\" alignments=\"none none\" widths=\"7 3\">\n",
            "  <grid_table_row sourcepos=\"2:1-4:13\" section=\"body\">\n",
            "    <grid_table_cell sourcepos=\"2:2-4:8\" column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph sourcepos=\"2:3-4:3\">\n",
            "        <text sourcepos=\"2:3-2:3\" xml:space=\"preserve\">a</text>\n",
            "        <softbreak sourcepos=\"2:9-2:8\" />\n",
            "        <text sourcepos=\"3:3-3:3\" xml:space=\"preserve\">b</text>\n",
            "        <linebreak sourcepos=\"3:4-3:8\" />\n",
            "        <text sourcepos=\"4:3-4:3\" xml:space=\"preserve\">d</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "    <grid_table_cell sourcepos=\"2:10-4:12\" column=\"1\" colspan=\"1\" rowspan=\"1\">\n",
            "      <paragraph sourcepos=\"2:11-2:11\">\n",
            "        <text sourcepos=\"2:11-2:11\" xml:space=\"preserve\">c</text>\n",
            "      </paragraph>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "  <grid_table_row sourcepos=\"5:1-5:0\" section=\"body\">\n",
            "    <grid_table_cell sourcepos=\"5:1-5:0\" column=\"0\" colspan=\"1\" rowspan=\"1\" />\n",
            "    <grid_table_cell sourcepos=\"5:1-5:0\" column=\"1\" colspan=\"1\" rowspan=\"1\" />\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
        |o| o.render.sourcepos = true,
    );
}

#[test]
fn sourcepos_paragraph_starting_with_a_break() {
    grid_xml_opts(
        concat!("+-------+\n", "| + [ ] |\n", "| --    |\n", "+-------+\n",),
        concat!(
            "<grid_table sourcepos=\"1:1-4:9\" alignments=\"none\" widths=\"7\">\n",
            "  <grid_table_row sourcepos=\"2:1-3:9\" section=\"body\">\n",
            "    <grid_table_cell sourcepos=\"2:2-3:8\" column=\"0\" colspan=\"1\" rowspan=\"1\">\n",
            "      <list sourcepos=\"2:3-3:4\" type=\"bullet\" tasklist=\"true\" tight=\"true\">\n",
            "        <taskitem sourcepos=\"2:3-3:4\" completed=\"false\">\n",
            "          <paragraph sourcepos=\"2:9-3:4\">\n",
            "            <softbreak sourcepos=\"2:9-2:8\" />\n",
            "            <text sourcepos=\"3:3-3:4\" xml:space=\"preserve\">--</text>\n",
            "          </paragraph>\n",
            "        </taskitem>\n",
            "      </list>\n",
            "    </grid_table_cell>\n",
            "  </grid_table_row>\n",
            "</grid_table>\n",
        ),
        |o| {
            o.extension.tasklist = true;
            o.render.sourcepos = true;
        },
    );
}

#[test]
fn html_spans() {
    html_opts!(
        [extension.grid_table],
        concat!(
            "+---+---+---+\n",
            "| AAAAA | B |\n",
            "+---+---+ B +\n",
            "| D | E | B |\n",
            "+ D +---+---+\n",
            "| D | CCCCC |\n",
            "+---+---+---+\n",
        ),
        concat!(
            "<table>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td colspan=\"2\">AAAAA</td>\n",
            "<td rowspan=\"2\">B\nB\nB</td>\n",
            "</tr>\n",
            "<tr>\n",
            "<td rowspan=\"2\">D\nD\nD</td>\n",
            "<td>E</td>\n",
            "</tr>\n",
            "<tr>\n",
            "<td colspan=\"2\">CCCCC</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn html_sections_and_alignment() {
    html_opts!(
        [extension.grid_table],
        concat!(
            "+-----+-----+-----+\n",
            "| a   | b   | c   |\n",
            "+:====+====:+:===:+\n",
            "| d   | e   | f   |\n",
            "+=====+=====+=====+\n",
            "| g   | h   | i   |\n",
            "+=====+=====+=====+\n",
        ),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th align=\"left\">a</th>\n",
            "<th align=\"right\">b</th>\n",
            "<th align=\"center\">c</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td align=\"left\">d</td>\n",
            "<td align=\"right\">e</td>\n",
            "<td align=\"center\">f</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "<tfoot>\n",
            "<tr>\n",
            "<td align=\"left\">g</td>\n",
            "<td align=\"right\">h</td>\n",
            "<td align=\"center\">i</td>\n",
            "</tr>\n",
            "</tfoot>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn html_without_body() {
    html_opts!(
        [extension.grid_table],
        "+---+\n| a |\n+===+\n",
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n",
    );
    html_opts!(
        [extension.grid_table],
        "+---+\n| a |\n+===+\n| b |\n+===+\n",
        concat!(
            "<table>\n",
            "<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n",
            "<tfoot>\n<tr>\n<td>b</td>\n</tr>\n</tfoot>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn html_blocks_in_cells() {
    html_opts!(
        [extension.grid_table],
        concat!(
            "+-------+-----+---+\n",
            "| - a   | b   |   |\n",
            "| - c   |     |   |\n",
            "|       | d   |   |\n",
            "+-------+-----+---+\n",
        ),
        concat!(
            "<table>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>\n",
            "<ul>\n",
            "<li>a</li>\n",
            "<li>c</li>\n",
            "</ul>\n",
            "</td>\n",
            "<td>\n",
            "<p>b</p>\n",
            "<p>d</p>\n",
            "</td>\n",
            "<td></td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn html_sourcepos() {
    html_opts!(
        [extension.grid_table, render.sourcepos],
        "+---+---+\n| a | b |\n+===+===+\n| c     |\n+---+---+\n",
        concat!(
            "<table data-sourcepos=\"1:1-5:9\">\n",
            "<thead>\n",
            "<tr data-sourcepos=\"2:1-2:9\">\n",
            "<th data-sourcepos=\"2:2-2:4\">a</th>\n",
            "<th data-sourcepos=\"2:6-2:8\">b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr data-sourcepos=\"4:1-4:9\">\n",
            "<td data-sourcepos=\"4:2-4:8\" colspan=\"2\">c</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn html_widths() {
    html_opts!(
        [extension.grid_table, render.grid_table_widths],
        "+---+-----+--+\n| a | b   | c|\n+---+-----+--+\n",
        concat!(
            "<table>\n",
            "<colgroup>\n",
            "<col style=\"width:30%\" />\n",
            "<col style=\"width:50%\" />\n",
            "<col style=\"width:20%\" />\n",
            "</colgroup>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>a</td>\n",
            "<td>b</td>\n",
            "<td>c</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
    html_opts!(
        [extension.grid_table, render.grid_table_widths],
        "+----+--+\n| a  | b|\n+----+--+\n",
        concat!(
            "<table>\n",
            "<colgroup>\n",
            "<col style=\"width:66.67%\" />\n",
            "<col style=\"width:33.33%\" />\n",
            "</colgroup>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>a</td>\n",
            "<td>b</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
        ),
    );
}

fn grid_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.grid_table = true;
    options
}

#[test]
fn commonmark_spans() {
    commonmark(
        concat!(
            "+---+---+---+\n",
            "| AAAAA | B |\n",
            "+---+---+ B +\n",
            "| D | E | B |\n",
            "+ D +---+---+\n",
            "| D | CCCCC |\n",
            "+---+---+---+\n",
        ),
        concat!(
            "+-------+---+\n",
            "| AAAAA | B |\n",
            "+---+---+ B |\n",
            "| D | E | B |\n",
            "| D +---+---+\n",
            "| D | CCCCC |\n",
            "+---+-------+\n",
        ),
        Some(&grid_options()),
    );
}

#[test]
fn commonmark_sections_and_alignment() {
    commonmark(
        concat!(
            "+-----+---+\n",
            "| a   | b |\n",
            "+====:+:==+\n",
            "| - c | d |\n",
            "|     | e |\n",
            "+=====+===+\n",
            "| f   |   |\n",
            "+=====+===+\n",
        ),
        concat!(
            "+-----+---+\n",
            "| a   | b |\n",
            "+====:+:==+\n",
            "| - c | d |\n",
            "|     | e |\n",
            "+=====+===+\n",
            "| f   |   |\n",
            "+=====+===+\n",
        ),
        Some(&grid_options()),
    );
}

#[test]
fn commonmark_blank_rows_keep_spans() {
    html_opts!(
        [extension.grid_table],
        "+---+---+\n|       |\n+===+===+\n| a | b |\n+---+---+\n",
        concat!(
            "<table>\n",
            "<thead>\n<tr>\n<th colspan=\"2\"></th>\n</tr>\n</thead>\n",
            "<tbody>\n<tr>\n<td>a</td>\n<td>b</td>\n</tr>\n</tbody>\n",
            "</table>\n",
        ),
    );
}

#[test]
fn commonmark_separates_tables_in_tight_lists() {
    let options = grid_options();
    commonmark(
        "- +---+\n  | a |\n  +---+\n    +---+\n    | b |\n    +---+\n- c\n",
        concat!(
            "- \n",
            "   +---+\n",
            "   | a |\n",
            "   +---+\n",
            "    +---+\n",
            "    | b |\n",
            "    +---+\n",
            "- c\n",
        ),
        Some(&options),
    );
    html_opts!(
        [extension.grid_table],
        "- > x\n  >\n  > +---+\n  > | a |\n  > +---+\n  >\n  > | b | c |\n- e\n",
        concat!(
            "<ul>\n",
            "<li>\n",
            "<blockquote>\n",
            "<p>x</p>\n",
            "<table>\n",
            "<tbody>\n<tr>\n<td>a</td>\n</tr>\n</tbody>\n",
            "</table>\n",
            "<p>| b | c |</p>\n",
            "</blockquote>\n",
            "</li>\n",
            "<li>e</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn commonmark_blank_line_after_absorbing_blocks() {
    html_opts!(
        [extension.grid_table, extension.description_lists],
        "- a\n\n  : b\n\n  +---+\n  | c |\n  +---+\n- d\n",
        concat!(
            "<ul>\n",
            "<li>\n",
            "<dl>\n",
            "<dt>a</dt>\n",
            "<dd>\n<p>b</p>\n</dd>\n",
            "</dl>\n",
            "<table>\n",
            "<tbody>\n<tr>\n<td>c</td>\n</tr>\n</tbody>\n",
            "</table>\n",
            "</li>\n",
            "<li>d</li>\n",
            "</ul>\n",
        ),
    );
}
