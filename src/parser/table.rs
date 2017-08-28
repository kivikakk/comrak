use arena_tree::Node;

use nodes::{NodeValue, TableAlignment, AstNode, make_block};
use parser::Parser;
use scanners;
use std::cell::RefCell;
use std::cmp::min;
use strings::trim;

pub fn try_opening_block<'a, 'o>(
    parser: &mut Parser<'a, 'o>,
    container: &'a AstNode<'a>,
    line: &str,
) -> Option<(&'a AstNode<'a>, bool)> {
    let aligns = match container.data.borrow().value {
        NodeValue::Paragraph => None,
        NodeValue::Table(ref aligns) => Some(aligns.clone()),
        _ => return None,
    };

    match aligns {
        None => try_opening_header(parser, container, line),
        Some(ref aligns) => try_opening_row(parser, container, aligns, line),
    }
}

fn try_opening_header<'a, 'o>(
    parser: &mut Parser<'a, 'o>,
    container: &'a AstNode<'a>,
    line: &str,
) -> Option<(&'a AstNode<'a>, bool)> {
    if scanners::table_start(&line[parser.first_nonspace..]).is_none() {
        return Some((container, false));
    }

    let header_row = match row(&container.data.borrow().content) {
        Some(header_row) => header_row,
        None => return Some((container, false)),
    };

    let marker_row = row(&line[parser.first_nonspace..]).unwrap();

    if header_row.len() != marker_row.len() {
        return Some((container, false));
    }

    let mut alignments = vec![];
    for cell in marker_row {
        let left = !cell.is_empty() && cell.as_bytes()[0] == b':';
        let right = !cell.is_empty() && cell.as_bytes()[cell.len() - 1] == b':';
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

    let start_column = container.data.borrow().start_column;
    let child = make_block(NodeValue::Table(alignments), parser.line_number, start_column);
    let table = parser.arena.alloc(Node::new(RefCell::new(child)));
    container.append(table);

    let header = parser.add_child(table, NodeValue::TableRow(true), start_column);
    for header_str in header_row {
        let header_cell = parser.add_child(header, NodeValue::TableCell, start_column);
        header_cell.data.borrow_mut().content = header_str;
    }

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((table, true))
}


fn try_opening_row<'a, 'o>(
    parser: &mut Parser<'a, 'o>,
    container: &'a AstNode<'a>,
    alignments: &[TableAlignment],
    line: &str,
) -> Option<(&'a AstNode<'a>, bool)> {
    if parser.blank {
        return None;
    }
    let this_row = row(line).unwrap();
    let new_row = parser.add_child(
        container,
        NodeValue::TableRow(false),
        container.data.borrow().start_column,
    );

    let mut i = 0;
    while i < min(alignments.len(), this_row.len()) {
        let cell = parser.add_child(
            new_row,
            NodeValue::TableCell,
            container.data.borrow().start_column,
        );
        cell.data.borrow_mut().content = this_row[i].clone();
        i += 1;
    }

    while i < alignments.len() {
        parser.add_child(
            new_row,
            NodeValue::TableCell,
            container.data.borrow().start_column,
        );
        i += 1;
    }

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((new_row, false))
}

fn row(string: &str) -> Option<Vec<String>> {
    let len = string.len();
    let mut v = vec![];
    let mut offset = 0;

    if len > 0 && string.as_bytes()[0] == b'|' {
        offset += 1;
    }

    loop {
        let cell_matched = scanners::table_cell(&string[offset..]).unwrap_or(0);
        let mut pipe_matched = scanners::table_cell_end(&string[offset + cell_matched..])
            .unwrap_or(0);

        if cell_matched > 0 || pipe_matched > 0 {
            let mut cell = unescape_pipes(&string[offset..offset + cell_matched]);
            trim(&mut cell);
            v.push(cell);
        }

        offset += cell_matched + pipe_matched;

        if pipe_matched == 0 {
            pipe_matched = scanners::table_row_end(&string[offset..]).unwrap_or(0);
            offset += pipe_matched;
        }

        if !((cell_matched > 0 || pipe_matched > 0) && offset < len) {
            break;
        }
    }

    if offset != len || v.is_empty() {
        None
    } else {
        Some(v)
    }
}

fn unescape_pipes(string: &str) -> String {
    let mut v = String::with_capacity(string.len());
    let mut escaping = false;

    for c in string.chars() {
        if escaping {
            v.push(c);
            escaping = false;
        } else if c == '\\' {
            escaping = true;
        } else {
            v.push(c);
        }
    }

    if escaping {
        v.push('\\');
    }

    v
}

pub fn matches(line: &str) -> bool {
    row(line).is_some()
}
