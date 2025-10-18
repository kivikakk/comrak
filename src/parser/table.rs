use crate::arena_tree::Node;
use crate::nodes::{Ast, AstNode, NodeTable, NodeValue, TableAlignment};
use crate::parser::Parser;
use crate::scanners;
use crate::strings::trim;
use std::cell::RefCell;
use std::cmp::min;

use super::inlines::count_newlines;

// Limit to prevent a malicious input from causing a denial of service.
const MAX_AUTOCOMPLETED_CELLS: usize = 500_000;

pub fn try_opening_block<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: &'a AstNode<'a>,
    line: &[u8],
) -> Option<(&'a AstNode<'a>, bool, bool)> {
    let aligns = match container.data.borrow().value {
        NodeValue::Paragraph => None,
        NodeValue::Table(NodeTable { ref alignments, .. }) => Some(alignments.clone()),
        _ => return None,
    };

    match aligns {
        None => try_opening_header(parser, container, line),
        Some(ref aligns) => try_opening_row(parser, container, aligns, line),
    }
}

fn try_opening_header<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: &'a AstNode<'a>,
    line: &[u8],
) -> Option<(&'a AstNode<'a>, bool, bool)> {
    if container.data.borrow().table_visited {
        return Some((container, false, false));
    }

    if scanners::table_start(&line[parser.first_nonspace..]).is_none() {
        return Some((container, false, false));
    }

    let spoiler = parser.options.extension.spoiler;

    let delimiter_row = match row(&line[parser.first_nonspace..], spoiler) {
        Some(delimiter_row) => delimiter_row,
        None => return Some((container, false, true)),
    };

    let header_row = match row(container.data.borrow().content.as_bytes(), spoiler) {
        Some(header_row) => header_row,
        None => return Some((container, false, true)),
    };

    if header_row.cells.len() != delimiter_row.cells.len() {
        return Some((container, false, true));
    }

    if header_row.paragraph_offset > 0 {
        try_inserting_table_header_paragraph(parser, container, header_row.paragraph_offset);
    }

    let mut alignments = vec![];
    for cell in delimiter_row.cells {
        let cell_content = cell.content.as_bytes();
        let left = !cell_content.is_empty() && cell_content[0] == b':';
        let right = !cell_content.is_empty() && cell_content[cell_content.len() - 1] == b':';
        alignments.push(if left && right {
            TableAlignment::Center
        } else if left {
            TableAlignment::Left
        } else if right {
            TableAlignment::Right
        } else {
            TableAlignment::None
        });
    }

    let start = container.data.borrow().sourcepos.start;
    let child = Ast::new(
        NodeValue::Table(NodeTable {
            alignments,
            num_columns: header_row.cells.len(),
            num_rows: 0,
            num_nonempty_cells: 0,
        }),
        start,
    );
    let table = parser.arena.alloc(Node::new(RefCell::new(child)));
    container.append(table);

    let header = parser.add_child(table, NodeValue::TableRow(true), start.column);
    {
        let header_ast = &mut header.data.borrow_mut();
        header_ast.sourcepos.start.line = start.line;
        header_ast.sourcepos.end = start.column_add(
            (container.data.borrow().content.len() - 2 - header_row.paragraph_offset) as isize,
        );
    }

    let mut i = 0;

    while i < header_row.cells.len() {
        let cell = &header_row.cells[i];
        let ast_cell = parser.add_child(
            header,
            NodeValue::TableCell,
            start.column + cell.start_offset - header_row.paragraph_offset,
        );
        let ast = &mut ast_cell.data.borrow_mut();
        ast.sourcepos.start.line = start.line;
        ast.sourcepos.end =
            start.column_add((cell.end_offset - header_row.paragraph_offset) as isize);
        ast.internal_offset = cell.internal_offset;
        ast.content.clone_from(&cell.content);
        ast.line_offsets.push(
            start.column + cell.start_offset - 1 + cell.internal_offset
                - header_row.paragraph_offset,
        );

        i += 1;
    }

    incr_table_row_count(container, i);

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((table, true, false))
}

fn try_opening_row<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: &'a AstNode<'a>,
    alignments: &[TableAlignment],
    line: &[u8],
) -> Option<(&'a AstNode<'a>, bool, bool)> {
    if parser.blank {
        return None;
    }

    if get_num_autocompleted_cells(container) > MAX_AUTOCOMPLETED_CELLS {
        return None;
    }

    let sourcepos = container.data.borrow().sourcepos;
    let spoiler = parser.options.extension.spoiler;
    let this_row = row(&line[parser.first_nonspace..], spoiler)?;

    let new_row = parser.add_child(
        container,
        NodeValue::TableRow(false),
        sourcepos.start.column,
    );
    {
        new_row.data.borrow_mut().sourcepos.end.column = sourcepos.end.column;
    }

    let mut i = 0;
    let mut last_column = sourcepos.start.column;

    while i < min(alignments.len(), this_row.cells.len()) {
        let cell = &this_row.cells[i];
        let cell_node = parser.add_child(
            new_row,
            NodeValue::TableCell,
            sourcepos.start.column + cell.start_offset,
        );
        let cell_ast = &mut cell_node.data.borrow_mut();
        cell_ast.internal_offset = cell.internal_offset;
        cell_ast.sourcepos.end.column = sourcepos.start.column + cell.end_offset;
        cell_ast.content.clone_from(&cell.content);
        cell_ast
            .line_offsets
            .push(sourcepos.start.column + cell.start_offset - 1 + cell.internal_offset);

        last_column = cell_ast.sourcepos.end.column;

        i += 1;
    }

    incr_table_row_count(container, i);

    while i < alignments.len() {
        parser.add_child(new_row, NodeValue::TableCell, last_column);
        i += 1;
    }

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((new_row, false, false))
}

struct Row {
    paragraph_offset: usize,
    cells: Vec<Cell>,
}

struct Cell {
    start_offset: usize,
    end_offset: usize,
    internal_offset: usize,
    content: String,
}

fn row(string: &[u8], spoiler: bool) -> Option<Row> {
    let len = string.len();
    let mut cells: Vec<Cell> = vec![];

    let mut offset = scanners::table_cell_end(string).unwrap_or(0);

    let mut paragraph_offset = 0;
    let mut expect_more_cells = true;
    let mut max_columns_abort = false;

    while offset < len && expect_more_cells {
        let cell_matched = scanners::table_cell(&string[offset..], spoiler).unwrap_or(0);
        let pipe_matched = scanners::table_cell_end(&string[offset + cell_matched..]).unwrap_or(0);

        if cell_matched > 0 || pipe_matched > 0 {
            let mut cell = unescape_pipes(&string[offset..offset + cell_matched]);
            trim(&mut cell);

            let mut start_offset = offset;
            let mut internal_offset = 0;

            while start_offset > paragraph_offset && string[start_offset - 1] != b'|' {
                start_offset -= 1;
                internal_offset += 1;
            }

            // set an upper limit on the number of columns
            if cells.len() == <u16 as Into<usize>>::into(u16::MAX) {
                max_columns_abort = true;
                break;
            }

            cells.push(Cell {
                start_offset,
                end_offset: offset + cell_matched - 1,
                internal_offset,
                content: String::from_utf8(cell).unwrap(),
            });
        }

        offset += cell_matched + pipe_matched;

        if pipe_matched > 0 {
            expect_more_cells = true;
        } else {
            let row_end_offset = scanners::table_row_end(&string[offset..]).unwrap_or(0);
            offset += row_end_offset;

            if row_end_offset > 0 && offset != len {
                paragraph_offset = offset;
                cells.clear();
                offset += scanners::table_cell_end(&string[offset..]).unwrap_or(0);
                expect_more_cells = true;
            } else {
                expect_more_cells = false;
            }
        }
    }

    if offset != len || cells.is_empty() || max_columns_abort {
        None
    } else {
        Some(Row {
            paragraph_offset,
            cells,
        })
    }
}

fn try_inserting_table_header_paragraph<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: &'a AstNode<'a>,
    paragraph_offset: usize,
) {
    let container_ast = &mut container.data.borrow_mut();

    let preface = &container_ast.content.as_bytes()[..paragraph_offset];
    let mut paragraph_content = unescape_pipes(preface);
    let (newlines, _since_newline) = count_newlines(&paragraph_content);
    trim(&mut paragraph_content);

    if container.parent().is_none()
        || !container
            .parent()
            .unwrap()
            .can_contain_type(&NodeValue::Paragraph)
    {
        return;
    }

    let start = container_ast.sourcepos.start;

    let mut paragraph = Ast::new(NodeValue::Paragraph, start);
    paragraph.sourcepos.end.line = start.line + newlines - 1;

    // copy over the line offsets related to the paragraph
    for n in 0..newlines {
        paragraph.line_offsets.push(container_ast.line_offsets[n]);
    }

    let last_line_offset = *paragraph.line_offsets.last().unwrap_or(&0);
    paragraph.sourcepos.end.column = last_line_offset
        + preface
            .iter()
            .rev()
            .skip(1)
            .take_while(|&&c| c != b'\n')
            .count();

    container_ast.sourcepos.start.line += newlines;

    paragraph.content = String::from_utf8(paragraph_content).unwrap();
    let node = parser.arena.alloc(Node::new(RefCell::new(paragraph)));
    container.insert_before(node);
}

fn unescape_pipes(string: &[u8]) -> Vec<u8> {
    let len = string.len();
    let mut v = Vec::with_capacity(len);

    for (i, &c) in string.iter().enumerate() {
        if c == b'\\' && i + 1 < len && string[i + 1] == b'|' {
            continue;
        } else {
            v.push(c);
        }
    }

    v
}

// Increment the number of rows in the table. Also update n_nonempty_cells,
// which keeps track of the number of cells which were parsed from the
// input file. (If one of the rows is too short, then the trailing cells
// are autocompleted. Autocompleted cells are not counted in n_nonempty_cells.)
// The purpose of this is to prevent a malicious input from generating a very
// large number of autocompleted cells, which could cause a denial of service
// vulnerability.
fn incr_table_row_count<'a>(container: &'a AstNode<'a>, i: usize) -> bool {
    return match container.data.borrow_mut().value {
        NodeValue::Table(ref mut node_table) => {
            node_table.num_rows += 1;
            node_table.num_nonempty_cells += i;
            true
        }
        _ => false,
    };
}

// Calculate the number of autocompleted cells.
fn get_num_autocompleted_cells<'a>(container: &'a AstNode<'a>) -> usize {
    return match container.data.borrow().value {
        NodeValue::Table(ref node_table) => {
            let num_cells = node_table.num_columns * node_table.num_rows;

            if num_cells < node_table.num_nonempty_cells {
                0
            } else {
                (node_table.num_columns * node_table.num_rows) - node_table.num_nonempty_cells
            }
        }
        _ => 0,
    };
}

pub fn matches(line: &[u8], spoiler: bool) -> bool {
    row(line, spoiler).is_some()
}
