use arena_tree::Node;
use nodes;
use nodes::{Ast, AstNode, NodeValue, TableAlignment};
use parser::Parser;
use scanners;
use std::cell::RefCell;
use std::cmp::min;
use strings::trim;

pub fn try_opening_block<'a, 'o, 'c>(
    parser: &mut Parser<'a, 'o, 'c>,
    container: &'a AstNode<'a>,
    line: &[u8],
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

fn try_opening_header<'a, 'o, 'c>(
    parser: &mut Parser<'a, 'o, 'c>,
    container: &'a AstNode<'a>,
    line: &[u8],
) -> Option<(&'a AstNode<'a>, bool)> {
    if scanners::table_start(&line[parser.first_nonspace..]).is_none() {
        return Some((container, false));
    }

    let header_row = match row(&container.data.borrow().content) {
        Some(header_row) => header_row,
        None => return Some((container, false)),
    };

    let marker_row = row(&line[parser.first_nonspace..]).unwrap();

    if header_row.cells.len() != marker_row.cells.len() {
        return Some((container, false));
    }

    if header_row.paragraph_offset > 0 {
        try_inserting_table_header_paragraph(
            parser,
            container,
            &container.data.borrow().content,
            header_row.paragraph_offset,
        );
    }

    let mut alignments = vec![];
    for cell in marker_row.cells {
        let left = !cell.is_empty() && cell[0] == b':';
        let right = !cell.is_empty() && cell[cell.len() - 1] == b':';
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

    let mut child = Ast::new(NodeValue::Table(alignments));
    child.start_line = parser.line_number;
    let table = parser.arena.alloc(Node::new(RefCell::new(child)));
    container.append(table);

    let header = parser.add_child(table, NodeValue::TableRow(true));
    for header_str in header_row.cells {
        let header_cell = parser.add_child(header, NodeValue::TableCell);
        header_cell.data.borrow_mut().content = header_str;
    }

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((table, true))
}

fn try_opening_row<'a, 'o, 'c>(
    parser: &mut Parser<'a, 'o, 'c>,
    container: &'a AstNode<'a>,
    alignments: &[TableAlignment],
    line: &[u8],
) -> Option<(&'a AstNode<'a>, bool)> {
    if parser.blank {
        return None;
    }
    let this_row = row(&line[parser.first_nonspace..]).unwrap();
    let new_row = parser.add_child(container, NodeValue::TableRow(false));

    let mut i = 0;
    while i < min(alignments.len(), this_row.cells.len()) {
        let cell = parser.add_child(new_row, NodeValue::TableCell);
        cell.data.borrow_mut().content = this_row.cells[i].clone();
        i += 1;
    }

    while i < alignments.len() {
        parser.add_child(new_row, NodeValue::TableCell);
        i += 1;
    }

    let offset = line.len() - 1 - parser.offset;
    parser.advance_offset(line, offset, false);

    Some((new_row, false))
}

struct Row {
    paragraph_offset: usize,
    cells: Vec<Vec<u8>>,
}

fn row(string: &[u8]) -> Option<Row> {
    let len = string.len();
    let mut cells = vec![];
    let mut offset = 0;

    if len > 0 && string[0] == b'|' {
        offset += 1;
    }

    let mut paragraph_offset: usize = 0;

    loop {
        let cell_matched = scanners::table_cell(&string[offset..]).unwrap_or(0);
        let mut pipe_matched =
            scanners::table_cell_end(&string[offset + cell_matched..]).unwrap_or(0);

        if cell_matched > 0 || pipe_matched > 0 {
            let cell_end_offset = offset + cell_matched - 1;

            if string[cell_end_offset] == b'\n' || string[cell_end_offset] == b'\r' {
                paragraph_offset = cell_end_offset;
                cells.clear();
            } else {
                let mut cell = unescape_pipes(&string[offset..offset + cell_matched]);
                trim(&mut cell);
                cells.push(cell);
            }
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

    if offset != len || cells.is_empty() {
        None
    } else {
        Some(Row {
            paragraph_offset: paragraph_offset,
            cells: cells,
        })
    }
}

fn try_inserting_table_header_paragraph<'a, 'o, 'c>(
    parser: &mut Parser<'a, 'o, 'c>,
    container: &'a AstNode<'a>,
    parent_string: &[u8],
    paragraph_offset: usize,
) {
    let mut paragraph_content = unescape_pipes(&parent_string[..paragraph_offset]);
    trim(&mut paragraph_content);

    if !container.parent().is_some()
        || !nodes::can_contain_type(container.parent().unwrap(), &NodeValue::Paragraph)
    {
        return;
    }

    let mut paragraph = Ast::new(NodeValue::Paragraph);
    paragraph.content = paragraph_content;
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

pub fn matches(line: &[u8]) -> bool {
    row(line).is_some()
}
