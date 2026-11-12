mod layout;

use std::collections::HashMap;
use std::ops::Range;

use crate::node_matches;
use crate::nodes::{Ast, Node, NodeGridTable, NodeValue, Sourcepos};
use crate::parser::fragments::Fragment;
use crate::parser::{Parser, TAB_STOP};

use layout::{Layout, Line};

const MAX_GRID_TABLE_DEPTH: usize = 5;

pub(crate) struct Pending<'a> {
    node: Node<'a>,
    indent: usize,
    text: String,
    lines: Vec<PendingLine>,
    layout: Option<Layout>,
}

pub(crate) struct CellLineEnd {
    line: usize,
    start: usize,
    eol: usize,
}

impl CellLineEnd {
    pub(crate) fn remap(self, fragments: &[Fragment<'_>]) -> Self {
        let fragment = &fragments[self.line - 1];
        CellLineEnd {
            line: fragment.line,
            start: fragment.byte + self.start,
            eol: fragment.byte + self.eol,
        }
    }
}

struct LineEnds {
    ends: Vec<(usize, usize)>,
    parents: Vec<Option<usize>>,
}

struct PendingLine {
    start: usize,
    end: usize,
    eol: usize,
    line: usize,
    byte: usize,
    column: usize,
    table_byte: usize,
    table_column: usize,
}

impl Pending<'_> {
    fn text(&self, line: &PendingLine) -> &str {
        &self.text[line.start..line.end]
    }

    fn layout(&self) -> Option<Layout> {
        let lines: Vec<Line<'_>> = self
            .lines
            .iter()
            .map(|line| Line {
                text: self.table_text(line),
                column: line.table_column,
            })
            .collect();
        layout::layout(&lines)
    }

    fn replay_fragments(&self) -> Vec<Fragment<'_>> {
        self.lines
            .iter()
            .map(|line| Fragment {
                text: self.text(line),
                eol: &self.text[line.end..line.eol],
                line: line.line,
                byte: line.byte,
                column: line.column,
            })
            .collect()
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
            || self.grid_table_depth >= MAX_GRID_TABLE_DEPTH
            || indented
            || maybe_lazy
            || self.blank
            || node_matches!(container, NodeValue::Table(..))
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
            node: container,
            indent: self.indent,
            text: String::new(),
            lines: vec![],
            layout: None,
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
        let end = pending.text.len();
        pending.text.push_str(&line[self.curline_end_col..]);
        pending.lines.push(PendingLine {
            start,
            end,
            eol: pending.text.len(),
            line: self.line_number,
            byte: self.offset,
            column: self.column,
            table_byte: self.first_nonspace,
            table_column: self.first_nonspace_column,
        });
    }

    pub(super) fn reject_ended_grid_table(&mut self, matched: Node<'a>) -> bool {
        let Some(pending) = self.grid_table.as_mut() else {
            return false;
        };
        if matched.same_node(pending.node) {
            return false;
        }

        pending.layout = pending.layout();
        if pending.layout.is_some() {
            return false;
        }

        let pending = self.grid_table.take().unwrap();
        let parent = pending.node.parent().unwrap();
        let holder = self.replay_holder(&pending);
        let current = self.open_fragments(holder, &pending.replay_fragments(), true);
        graft(holder, pending.node);
        self.current = if current.same_node(holder) {
            parent
        } else {
            current
        };
        true
    }

    pub(super) fn finalize_grid_table(&mut self, table: Node<'a>, ast: &mut Ast) {
        let mut pending = self.grid_table.take().unwrap();
        match pending.layout.take().or_else(|| pending.layout()) {
            Some(layout) => self.build_grid_table(table, ast, &pending, layout),
            None => {
                let holder = self.replay_holder(&pending);
                self.parse_fragments(holder, &pending.replay_fragments(), true);
                graft(holder, table);
            }
        }
    }

    fn build_grid_table(
        &mut self,
        table: Node<'a>,
        ast: &mut Ast,
        pending: &Pending<'a>,
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
            let sourcepos = if row.bottom > row.top + 1 {
                let first = &pending.lines[row.top + 1];
                let last = &pending.lines[row.bottom - 1];
                (
                    first.line,
                    first.table_byte + 1,
                    last.line,
                    pending.table_end(last),
                )
                    .into()
            } else {
                (top.line, top.table_byte + 1, top.line, top.table_byte).into()
            };
            let row_node = self.closed_node(NodeValue::GridTableRow(row.section), sourcepos);
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
                self.cell_line_ends.extend(
                    cell.content
                        .iter()
                        .zip(&pending.lines[row.top + 1..])
                        .map(|(range, line)| CellLineEnd {
                            line: line.line,
                            start: line.table_byte + range.start + 1,
                            eol: line.table_byte + range.end + 1,
                        }),
                );
                row_node.append(cell_node);
            }
        }
    }

    pub(super) fn place_cell_line_ends(&mut self) {
        if self.cell_line_ends.is_empty() {
            return;
        }

        let mut by_line: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
        for end in &self.cell_line_ends {
            by_line
                .entry(end.line)
                .or_default()
                .push((end.start, end.eol));
        }
        let by_line: HashMap<usize, LineEnds> = by_line
            .into_iter()
            .map(|(line, mut ends)| {
                ends.sort_unstable();
                let mut parents = vec![None; ends.len()];
                let mut open: Vec<usize> = vec![];
                for (i, &(start, _)) in ends.iter().enumerate() {
                    while open.last().is_some_and(|&j| ends[j].1 < start) {
                        open.pop();
                    }
                    parents[i] = open.last().copied();
                    open.push(i);
                }
                (line, LineEnds { ends, parents })
            })
            .collect();

        for node in self.root.descendants() {
            let mut ast = node.data_mut();
            let soft = match ast.value {
                NodeValue::SoftBreak => true,
                NodeValue::LineBreak => false,
                _ => continue,
            };
            let start = ast.sourcepos.start;
            let Some(LineEnds { ends, parents }) = by_line.get(&start.line) else {
                continue;
            };
            let mut candidate = ends
                .partition_point(|&(s, _)| s <= start.column)
                .checked_sub(1);
            while let Some(i) = candidate
                && ends[i].1 < start.column
            {
                candidate = parents[i];
            }
            let Some(i) = candidate else {
                continue;
            };
            let eol = ends[i].1;
            if soft {
                ast.sourcepos.start.column = eol;
            }
            ast.sourcepos.end = (start.line, eol - 1).into();
            drop(ast);
            if soft {
                let mut child = node;
                while child.previous_sibling().is_none()
                    && let Some(parent) = child.parent()
                {
                    let mut parent_ast = parent.data_mut();
                    if parent_ast.sourcepos.start != start {
                        break;
                    }
                    parent_ast.sourcepos.start.column = eol;
                    drop(parent_ast);
                    child = parent;
                }
            }
        }
    }

    fn replay_holder(&self, pending: &Pending<'a>) -> Node<'a> {
        self.arena
            .alloc(Ast::new(NodeValue::Document, (pending.lines[0].line, 1).into()).into())
    }

    fn closed_node(&self, value: NodeValue, sourcepos: Sourcepos) -> Node<'a> {
        let mut ast = Ast::new_with_sourcepos(value, sourcepos);
        ast.open = false;
        self.arena.alloc(ast.into())
    }
}

fn graft<'a>(holder: Node<'a>, table: Node<'a>) {
    while let Some(child) = holder.first_child() {
        table.insert_before(child);
    }
    table.detach();
}

fn cell_fragment<'t>(
    pending: &'t Pending<'_>,
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
        eol: "\n",
        line: line.line,
        byte,
        column,
    }
}
