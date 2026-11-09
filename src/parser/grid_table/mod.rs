mod layout;

use std::ops::Range;

use crate::nodes::{Ast, Node, NodeGridTable, NodeValue, Sourcepos};
use crate::parser::fragments::Fragment;
use crate::parser::{Parser, TAB_STOP};

use layout::{Layout, Line};

pub(crate) struct Pending {
    indent: usize,
    text: String,
    lines: Vec<PendingLine>,
}

struct PendingLine {
    start: usize,
    end: usize,
    line: usize,
    byte: usize,
    column: usize,
    table_byte: usize,
    table_column: usize,
}

impl Pending {
    fn text(&self, line: &PendingLine) -> &str {
        &self.text[line.start..line.end]
    }

    fn table_text(&self, line: &PendingLine) -> &str {
        &self.text[line.start + line.table_byte - line.byte..line.end]
    }

    fn table_end(&self, line: &PendingLine) -> usize {
        line.table_byte + layout::trim_end(self.table_text(line)).len()
    }
}

impl<'a: 'd, 'o, 'c, 'd> Parser<'a, 'o, 'c, 'd>
where
    'c: 'o,
{
    pub(super) fn handle_grid_table(
        &mut self,
        container: &mut Node<'a>,
        line: &str,
        indented: bool,
        maybe_lazy: bool,
    ) -> bool {
        if !self.options.extension.grid_table
            || self.grid_table_suppressed
            || indented
            || maybe_lazy
            || !layout::is_rule(
                layout::trim_end(&line[self.first_nonspace..self.curline_end_col]),
                b'-',
            )
        {
            return false;
        }

        self.close_unmatched_blocks();
        *container = self.add_child(
            container,
            NodeValue::GridTable(Box::default()),
            self.first_nonspace + 1,
        );
        self.grid_table = Some(Pending {
            indent: self.indent,
            text: String::new(),
            lines: vec![],
        });
        true
    }

    pub(super) fn grid_table_continues(&self, line: &str) -> bool {
        !self.blank
            && self.indent == self.grid_table.as_ref().unwrap().indent
            && matches!(line.as_bytes()[self.first_nonspace], b'+' | b'|')
    }

    pub(super) fn add_grid_table_line(&mut self, line: &str) {
        let pending = self.grid_table.as_mut().unwrap();
        let start = pending.text.len();
        pending
            .text
            .push_str(&line[self.offset..self.curline_end_col]);
        pending.lines.push(PendingLine {
            start,
            end: pending.text.len(),
            line: self.line_number,
            byte: self.offset,
            column: self.column,
            table_byte: self.first_nonspace,
            table_column: self.first_nonspace_column,
        });
    }

    pub(super) fn finalize_grid_table(&mut self, table: Node<'a>, ast: &mut Ast) {
        let pending = self.grid_table.take().unwrap();
        let lines: Vec<Line<'_>> = pending
            .lines
            .iter()
            .map(|line| Line {
                text: pending.table_text(line),
                column: line.table_column,
            })
            .collect();

        match layout::layout(&lines) {
            Some(layout) => self.build_grid_table(table, ast, &pending, layout),
            None => self.replay_grid_table(table, &pending),
        }
    }

    fn build_grid_table(
        &mut self,
        table: Node<'a>,
        ast: &mut Ast,
        pending: &Pending,
        layout: Layout,
    ) {
        let first = &pending.lines[0];
        let last = pending.lines.last().unwrap();
        ast.value = NodeValue::GridTable(Box::new(NodeGridTable {
            columns: layout.columns,
        }));
        ast.sourcepos = (
            first.line,
            first.table_byte + 1,
            last.line,
            pending.table_end(last),
        )
            .into();

        for row in &layout.rows {
            let top = &pending.lines[row.top];
            let bottom = &pending.lines[row.bottom - 1];
            let row_node = self.closed_node(
                NodeValue::GridTableRow(row.section),
                (
                    top.line,
                    top.table_byte + 1,
                    bottom.line,
                    pending.table_end(bottom),
                )
                    .into(),
            );
            table.append(row_node);

            for cell in &row.cells {
                let fragments: Vec<Fragment<'_>> = cell
                    .content
                    .iter()
                    .zip(&pending.lines[row.top + 1..])
                    .map(|(range, line)| cell_fragment(pending, line, range.clone(), cell.left))
                    .collect();

                let sourcepos = match (cell.content.first(), cell.content.last()) {
                    (Some(first), Some(last)) => {
                        let first_line = &pending.lines[row.top + 1];
                        let last_line = &pending.lines[row.top + cell.content.len()];
                        (
                            first_line.line,
                            first_line.table_byte + first.start + 1,
                            last_line.line,
                            last_line.table_byte + last.end,
                        )
                            .into()
                    }
                    _ => (top.line, top.table_byte + 1, top.line, top.table_byte).into(),
                };
                let cell_node = self.arena.alloc(
                    Ast::new_with_sourcepos(NodeValue::GridTableCell(cell.value), sourcepos).into(),
                );
                self.parse_fragments(cell_node, &fragments, false);
                row_node.append(cell_node);
            }
        }
    }

    fn replay_grid_table(&mut self, table: Node<'a>, pending: &Pending) {
        let fragments: Vec<Fragment<'_>> = pending
            .lines
            .iter()
            .map(|line| Fragment {
                text: pending.text(line),
                line: line.line,
                byte: line.byte,
                column: line.column,
            })
            .collect();

        let holder = self
            .arena
            .alloc(Ast::new(NodeValue::Document, (pending.lines[0].line, 1).into()).into());
        self.parse_fragments(holder, &fragments, true);

        while let Some(child) = holder.first_child() {
            table.insert_before(child);
        }
        table.detach();
    }

    fn closed_node(&self, value: NodeValue, sourcepos: Sourcepos) -> Node<'a> {
        let mut ast = Ast::new_with_sourcepos(value, sourcepos);
        ast.open = false;
        self.arena.alloc(ast.into())
    }
}

fn cell_fragment<'t>(
    pending: &'t Pending,
    line: &PendingLine,
    range: Range<usize>,
    left: usize,
) -> Fragment<'t> {
    let text = layout::trim_end(&pending.table_text(line)[range.clone()]);
    let mut byte = line.table_byte + range.start;
    let mut column = line.table_column + left + 1;

    let text = match text.as_bytes().first() {
        Some(b' ') => {
            byte += 1;
            column += 1;
            &text[1..]
        }
        Some(b'\t') if TAB_STOP - column % TAB_STOP == 1 => {
            byte += 1;
            column += 1;
            &text[1..]
        }
        Some(b'\t') => {
            column += 1;
            text
        }
        _ => text,
    };

    Fragment {
        text,
        line: line.line,
        byte,
        column,
    }
}
