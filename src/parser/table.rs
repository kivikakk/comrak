use std::borrow::Cow;
use std::cmp::min;
use std::mem;

use crate::nodes::{Ast, Node, NodeTable, NodeValue, TableAlignment};
use crate::parser::inlines::count_newlines;
use crate::parser::Parser;
use crate::scanners;
use crate::strings::trim_cow;

// Limit to prevent a malicious input from causing a denial of service.
// See get_num_autocompleted_cells.
const MAX_AUTOCOMPLETED_CELLS: usize = 500_000;

pub fn try_opening_block<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: Node<'a>,
    line: &str,
) -> Option<(Node<'a>, bool, bool)> {
    let aligns = match &container.data().value {
        NodeValue::Paragraph => None,
        NodeValue::Table(nt) => Some(nt.alignments.clone()),
        _ => return None,
    };

    match aligns {
        None => try_opening_header(parser, container, line),
        Some(ref aligns) => try_opening_row(parser, container, aligns, line),
    }
}

fn try_opening_header<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: Node<'a>,
    line: &str,
) -> Option<(Node<'a>, bool, bool)> {
    if container.data().table_visited {
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

    let mut container_content = mem::take(&mut container.data_mut().content);
    let mut header_row = match row(&container_content, spoiler) {
        Some(header_row) => header_row,
        None => {
            mem::swap(&mut container.data_mut().content, &mut container_content);
            return Some((container, false, true));
        }
    };

    if header_row.cells.len() != delimiter_row.cells.len() {
        mem::swap(&mut container.data_mut().content, &mut container_content);
        return Some((container, false, true));
    }

    if header_row.paragraph_offset > 0 {
        try_inserting_table_header_paragraph(
            parser,
            container,
            &container_content,
            header_row.paragraph_offset,
        );
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

    let start = container.data().sourcepos.start;
    let child = Ast::new(
        NodeValue::Table(Box::new(NodeTable {
            alignments,
            num_columns: header_row.cells.len(),
            num_rows: 0,
            num_nonempty_cells: 0,
        })),
        start,
    );
    let table = parser.arena.alloc(child.into());
    container.append(table);

    let header = parser.add_child(table, NodeValue::TableRow(true), start.column);
    {
        let header_ast = &mut header.data_mut();
        header_ast.sourcepos.start.line = start.line;
        header_ast.sourcepos.end =
            start.column_add((container_content.len() - 2 - header_row.paragraph_offset) as isize);
    }

    let mut i = 0;

    while i < header_row.cells.len() {
        let cell = &mut header_row.cells[i];
        let ast_cell = parser.add_child(
            header,
            NodeValue::TableCell,
            start.column + cell.start_offset - header_row.paragraph_offset,
        );
        let ast = &mut ast_cell.data_mut();
        ast.sourcepos.start.line = start.line;
        ast.sourcepos.end =
            start.column_add((cell.end_offset - header_row.paragraph_offset) as isize);
        mem::swap(&mut ast.content, cell.content.to_mut());
        ast.line_offsets.push(
            start.column + cell.start_offset - 1 + cell.internal_offset
                - header_row.paragraph_offset,
        );

        i += 1;
    }

    mem::swap(&mut container.data_mut().content, &mut container_content);
    incr_table_row_count(container, i);

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((table, true, false))
}

fn try_opening_row<'a>(
    parser: &mut Parser<'a, '_, '_>,
    container: Node<'a>,
    alignments: &[TableAlignment],
    line: &str,
) -> Option<(Node<'a>, bool, bool)> {
    if parser.blank {
        return None;
    }

    if get_num_autocompleted_cells(container) > MAX_AUTOCOMPLETED_CELLS {
        return None;
    }

    let sourcepos = container.data().sourcepos;
    let spoiler = parser.options.extension.spoiler;
    let mut this_row = row(&line[parser.first_nonspace..], spoiler)?;

    let new_row = parser.add_child(
        container,
        NodeValue::TableRow(false),
        sourcepos.start.column,
    );
    {
        new_row.data_mut().sourcepos.end.column = sourcepos.end.column;
    }

    let mut i = 0;
    let mut last_column = sourcepos.start.column;

    while i < min(alignments.len(), this_row.cells.len()) {
        let cell = &mut this_row.cells[i];
        let cell_node = parser.add_child(
            new_row,
            NodeValue::TableCell,
            sourcepos.start.column + cell.start_offset,
        );
        let cell_ast = &mut cell_node.data_mut();
        cell_ast.sourcepos.end.column = sourcepos.start.column + cell.end_offset;
        mem::swap(&mut cell_ast.content, cell.content.to_mut());
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

struct Row<'t> {
    paragraph_offset: usize,
    cells: Vec<Cell<'t>>,
}

struct Cell<'t> {
    start_offset: usize,
    end_offset: usize,
    internal_offset: usize,
    content: Cow<'t, str>,
}

fn row(string: &str, spoiler: bool) -> Option<Row<'_>> {
    let bytes = string.as_bytes();
    let len = string.len();
    let mut cells: Vec<Cell> = vec![];

    let mut offset = scanners::table_cell_end(string).unwrap_or(0);

    let mut paragraph_offset = 0;

    while offset < len {
        let cell_matched = scanners::table_cell(&string[offset..], spoiler).unwrap_or(0);
        let pipe_matched = scanners::table_cell_end(&string[offset + cell_matched..]).unwrap_or(0);

        if cell_matched > 0 || pipe_matched > 0 {
            let mut cell = unescape_pipes(&string[offset..offset + cell_matched]);
            trim_cow(&mut cell);

            let mut start_offset = offset;
            let mut internal_offset = 0;

            while start_offset > paragraph_offset && bytes[start_offset - 1] != b'|' {
                start_offset -= 1;
                internal_offset += 1;
            }

            if cells.len() == u16::MAX as usize {
                return None;
            }

            cells.push(Cell {
                start_offset,
                end_offset: offset + cell_matched - 1,
                internal_offset,
                content: cell,
            });
        }

        offset += cell_matched + pipe_matched;

        if pipe_matched == 0 {
            let row_end_offset = scanners::table_row_end(&string[offset..]).unwrap_or(0);
            offset += row_end_offset;

            if row_end_offset == 0 || offset == len {
                break;
            }

            paragraph_offset = offset;
            cells.clear();
            offset += scanners::table_cell_end(&string[offset..]).unwrap_or(0);
        }
    }

    if offset != len || cells.is_empty() {
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
    container: Node<'a>,
    container_content: &str,
    paragraph_offset: usize,
) {
    if container
        .parent()
        .map_or(false, |p| !p.can_contain_type(&NodeValue::Paragraph))
    {
        return;
    }

    let preface = &container_content[..paragraph_offset];
    let mut paragraph_content = unescape_pipes(preface);
    let (newlines, _since_newline) = count_newlines(&paragraph_content);
    trim_cow(&mut paragraph_content);
    let paragraph_content = paragraph_content.to_string();

    let container_ast = &mut container.data_mut();
    let start = container_ast.sourcepos.start;

    let mut paragraph = Ast::new(NodeValue::Paragraph, start);
    paragraph.sourcepos.end.line = start.line + newlines - 1;

    for n in 0..newlines {
        paragraph.line_offsets.push(container_ast.line_offsets[n]);
    }

    let last_line_offset = *paragraph.line_offsets.last().unwrap_or(&0);
    paragraph.sourcepos.end.column = last_line_offset
        + preface
            .as_bytes()
            .iter()
            .rev()
            .skip(1)
            .take_while(|&&c| c != b'\n')
            .count();

    container_ast.sourcepos.start.line += newlines;

    paragraph.content = paragraph_content;
    let node = parser.arena.alloc(paragraph.into());
    container.insert_before(node);
}

fn unescape_pipes(string: &str) -> Cow<'_, str> {
    let mut v = String::new();
    let mut offset = 0;
    let mut last_was_backslash = false;

    for (i, c) in string.char_indices() {
        if last_was_backslash {
            if c == '|' {
                v.push_str(&string[offset..i - 1]);
                offset = i;
            }
            last_was_backslash = false;
        } else if c == '\\' {
            last_was_backslash = true;
        }
    }

    if offset == 0 {
        string.into()
    } else {
        v.push_str(&string[offset..]);
        v.into()
    }
}

// Increment the number of rows in the table. Also update n_nonempty_cells,
// which keeps track of the number of cells which were parsed from the
// input file. (If one of the rows is too short, then the trailing cells
// are autocompleted. Autocompleted cells are not counted in n_nonempty_cells.)
// The purpose of this is to prevent a malicious input from generating a very
// large number of autocompleted cells, which could cause a denial of service
// vulnerability.
fn incr_table_row_count<'a>(container: Node<'a>, i: usize) -> bool {
    return match container.data_mut().value {
        NodeValue::Table(ref mut node_table) => {
            node_table.num_rows += 1;
            node_table.num_nonempty_cells += i;
            true
        }
        _ => false,
    };
}

// Calculate the number of autocompleted cells.
fn get_num_autocompleted_cells<'a>(container: Node<'a>) -> usize {
    return match container.data().value {
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

pub fn matches(line: &str, spoiler: bool) -> bool {
    row(line, spoiler).is_some()
}
