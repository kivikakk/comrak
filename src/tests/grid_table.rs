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
