use std::fmt;

use unicode_width::UnicodeWidthChar;

use crate::cm::{CommonMarkFormatter, Escaping};
use crate::node_matches;
use crate::nodes::{GridTableSection, Node, NodeValue, TableAlignment};
use crate::parser::TAB_STOP;

struct Rect {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,
    lines: Vec<String>,
}

fn widest(lines: &[String]) -> usize {
    lines
        .iter()
        .map(|line| {
            line.chars()
                .map(|c| {
                    if c == '\t' {
                        TAB_STOP
                    } else {
                        c.width().unwrap_or(0)
                    }
                })
                .sum()
        })
        .max()
        .unwrap_or(0)
}

fn offsets(sizes: &[usize]) -> Vec<usize> {
    let mut offsets = vec![0];
    for size in sizes {
        offsets.push(offsets.last().unwrap() + size + 1);
    }
    offsets
}

fn absorbs(node: Node<'_>) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        match node.data().value {
            NodeValue::Paragraph | NodeValue::Table(..) => return true,
            NodeValue::HtmlBlock(ref html) if matches!(html.block_type, 6 | 7) => return true,
            NodeValue::GridTable(..) => return false,
            _ => {}
        }
        current = node.last_child().filter(|child| child.data().value.block());
    }
    false
}

fn put(canvas: &mut [Vec<Option<String>>], y: usize, x: usize, c: char) {
    canvas[y][x] = Some(c.to_string());
}

fn is(canvas: &[Vec<Option<String>>], y: usize, x: usize, s: &str) -> bool {
    canvas[y][x].as_deref() == Some(s)
}

impl<'a> CommonMarkFormatter<'a, '_, '_, '_> {
    pub(super) fn format_grid_table(&mut self, table: Node<'a>) -> fmt::Result {
        let columns = match table.data().value {
            NodeValue::GridTable(ref table) => table.columns.clone(),
            _ => unreachable!(),
        };

        let mut sections = vec![];
        let mut cells = vec![];
        for (r, row) in table.children().enumerate() {
            let NodeValue::GridTableRow(section) = row.data().value else {
                unreachable!()
            };
            sections.push(section);
            for cell in row.children() {
                let NodeValue::GridTableCell(value) = cell.data().value else {
                    unreachable!()
                };
                let mut content = String::new();
                CommonMarkFormatter::new(cell, self.options, &mut content).format(cell)?;
                let mut lines: Vec<String> =
                    content.lines().map(|l| l.trim_end().to_string()).collect();
                while lines.last().is_some_and(|l| l.is_empty()) {
                    lines.pop();
                }
                cells.push(Rect {
                    top: r,
                    left: value.column,
                    bottom: r + value.rowspan,
                    right: value.column + value.colspan,
                    lines,
                });
            }
        }

        let mut widths: Vec<usize> = columns
            .iter()
            .map(|c| match c.alignment {
                TableAlignment::None => c.width.max(1),
                _ => c.width.max(3),
            })
            .collect();
        let mut heights = vec![1; sections.len()];
        for cell in &cells {
            if cell.right == cell.left + 1 {
                widths[cell.left] = widths[cell.left].max(widest(&cell.lines) + 1);
            }
            if cell.bottom == cell.top + 1 {
                heights[cell.top] = heights[cell.top].max(cell.lines.len());
            }
        }
        for cell in &cells {
            let need = widest(&cell.lines) + 1;
            let have =
                widths[cell.left..cell.right].iter().sum::<usize>() + cell.right - cell.left - 1;
            if have < need {
                widths[cell.right - 1] += need - have;
            }
            let have =
                heights[cell.top..cell.bottom].iter().sum::<usize>() + cell.bottom - cell.top - 1;
            if have < cell.lines.len() {
                heights[cell.bottom - 1] += cell.lines.len() - have;
            }
        }

        let xs = offsets(&widths);
        let ys = offsets(&heights);
        let width = xs.last().unwrap() + 1;
        let height = ys.last().unwrap() + 1;
        let mut canvas = vec![vec![Some(" ".to_string()); width]; height];

        for cell in &cells {
            let (x0, x1, y0, y1) = (xs[cell.left], xs[cell.right], ys[cell.top], ys[cell.bottom]);
            for x in x0 + 1..x1 {
                put(&mut canvas, y0, x, '-');
                put(&mut canvas, y1, x, '-');
            }
            for y in y0 + 1..y1 {
                put(&mut canvas, y, x0, '|');
                put(&mut canvas, y, x1, '|');
            }
        }
        for cell in &cells {
            let (x0, x1, y0, y1) = (xs[cell.left], xs[cell.right], ys[cell.top], ys[cell.bottom]);
            for (y, x) in [(y0, x0), (y0, x1), (y1, x0), (y1, x1)] {
                put(&mut canvas, y, x, '+');
            }
        }

        let mut rules = vec![];
        if let Some(last_head) = sections.iter().rposition(|&s| s == GridTableSection::Head) {
            rules.push(ys[last_head + 1]);
        }
        if let Some(first_foot) = sections.iter().position(|&s| s == GridTableSection::Foot) {
            rules.push(ys[first_foot]);
            rules.push(height - 1);
        }
        rules.sort_unstable();
        rules.dedup();
        for &y in &rules {
            for x in 0..width {
                if is(&canvas, y, x, "-") {
                    put(&mut canvas, y, x, '=');
                }
            }
        }

        let alignment_line = rules.first().copied().unwrap_or(0);
        for (c, column) in columns.iter().enumerate() {
            let x = xs[c];
            if column.alignment == TableAlignment::None || !is(&canvas, alignment_line, x, "+") {
                continue;
            }
            let next = (x + 1..width)
                .find(|&x| is(&canvas, alignment_line, x, "+"))
                .unwrap();
            if matches!(
                column.alignment,
                TableAlignment::Left | TableAlignment::Center
            ) {
                put(&mut canvas, alignment_line, x + 1, ':');
            }
            if matches!(
                column.alignment,
                TableAlignment::Right | TableAlignment::Center
            ) {
                put(&mut canvas, alignment_line, next - 1, ':');
            }
        }

        let direct_tight = self.in_tight_list_item
            && table
                .parent()
                .is_some_and(|p| node_matches!(p, NodeValue::Item(..) | NodeValue::TaskItem(..)));
        let indent = if direct_tight {
            1 + table
                .preceding_siblings()
                .skip(1)
                .take_while(|n| node_matches!(n, NodeValue::GridTable(..)))
                .count()
                % 2
        } else {
            0
        };
        let origin = self.prefix.len() + indent;
        for cell in &cells {
            for (i, line) in cell.lines.iter().enumerate() {
                let row = &mut canvas[ys[cell.top] + 1 + i];
                let mut x = xs[cell.left] + 2;
                for c in line.chars() {
                    let span = if c == '\t' {
                        TAB_STOP - (origin + x) % TAB_STOP
                    } else {
                        c.width().unwrap_or(0)
                    };
                    if span == 0 {
                        row[x - 1].get_or_insert_with(String::new).push(c);
                        continue;
                    }
                    row[x] = Some(c.to_string());
                    for slot in &mut row[x + 1..x + span] {
                        *slot = None;
                    }
                    x += span;
                }
            }
        }

        let tight = self.in_tight_list_item;
        let blank_before = table
            .previous_sibling()
            .is_some_and(|previous| !direct_tight || absorbs(previous));
        self.in_tight_list_item = !blank_before;
        self.blankline();
        for row in canvas {
            let line: String = row.into_iter().flatten().collect();
            self.output(&" ".repeat(indent), false, Escaping::Literal)?;
            self.output(&line, false, Escaping::Literal)?;
            self.in_tight_list_item = tight;
            self.cr();
        }
        self.blankline();
        if !direct_tight && table.next_sibling().is_some() {
            self.in_tight_list_item = false;
            self.output("", false, Escaping::Literal)?;
            self.in_tight_list_item = tight;
        }
        Ok(())
    }
}
