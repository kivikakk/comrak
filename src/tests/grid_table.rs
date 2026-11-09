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
