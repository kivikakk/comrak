use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use crate::nodes::{GridTableColumn, GridTableSection, NodeGridTableCell, TableAlignment};
use crate::parser::TAB_STOP;

pub(crate) struct Line<'s> {
    pub(crate) text: &'s str,
    pub(crate) column: usize,
}

pub(crate) struct Layout {
    pub(crate) columns: Vec<GridTableColumn>,
    pub(crate) rows: Vec<Row>,
}

pub(crate) struct Row {
    pub(crate) section: GridTableSection,
    pub(crate) top: usize,
    pub(crate) bottom: usize,
    pub(crate) cells: Vec<Cell>,
}

pub(crate) struct Cell {
    pub(crate) value: NodeGridTableCell,
    pub(crate) left: usize,
    pub(crate) content: Vec<Range<usize>>,
}

pub(crate) fn layout(lines: &[Line<'_>]) -> Option<Layout> {
    let grid = Grid::new(lines)?;
    let rects = grid.trace()?;
    Some(grid.layout(lines, &rects))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Glyph {
    Plus,
    Dash,
    Pipe,
    Other,
}

impl Glyph {
    fn vertical(self) -> bool {
        matches!(self, Glyph::Plus | Glyph::Pipe)
    }

    fn horizontal(self) -> bool {
        matches!(self, Glyph::Plus | Glyph::Dash)
    }
}

#[derive(Clone, Copy)]
struct Slot {
    glyph: Glyph,
    byte: usize,
}

#[derive(Clone, Copy)]
struct Rect {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,
}

struct Grid {
    width: usize,
    height: usize,
    slots: Vec<Slot>,
    down: Vec<usize>,
    right: Vec<usize>,
    part_separators: Vec<usize>,
}

impl Grid {
    fn new(lines: &[Line<'_>]) -> Option<Self> {
        let top = trim_end(lines.first()?.text);
        if !is_rule(top, b'-') {
            return None;
        }

        let width = top.len();
        let height = lines.len();
        let mut slots = vec![];
        let mut part_separators = vec![];

        for (r, line) in lines.iter().enumerate() {
            let text = trim_end(line.text);
            let rule = r == 0 || is_rule(text, b'=');
            if r > 0 && rule {
                part_separators.push(r);
            }

            let start = slots.len();
            let mut column = line.column;
            for (byte, ch) in text.char_indices() {
                let glyph = match ch {
                    '+' => Glyph::Plus,
                    _ if rule => Glyph::Dash,
                    '-' => Glyph::Dash,
                    '|' => Glyph::Pipe,
                    _ => Glyph::Other,
                };
                let span = if ch == '\t' {
                    TAB_STOP - column % TAB_STOP
                } else {
                    ch.width().unwrap_or(0)
                };
                if slots.len() - start + span > width {
                    return None;
                }
                slots.extend((0..span).map(|_| Slot { glyph, byte }));
                column += span;
            }
            if slots.len() - start != width {
                return None;
            }
        }

        let mut down = vec![0; slots.len()];
        for r in (0..height).rev() {
            for c in 0..width {
                let i = r * width + c;
                down[i] = if r + 1 < height
                    && slots[i].glyph.vertical()
                    && slots[i + width].glyph.vertical()
                {
                    down[i + width]
                } else {
                    r
                };
            }
        }

        let mut right = vec![0; slots.len()];
        for r in 0..height {
            for c in (0..width).rev() {
                let i = r * width + c;
                right[i] = if c + 1 < width
                    && slots[i].glyph.horizontal()
                    && slots[i + 1].glyph.horizontal()
                {
                    right[i + 1]
                } else {
                    c
                };
            }
        }

        Some(Grid {
            width,
            height,
            slots,
            down,
            right,
            part_separators,
        })
    }

    fn at(&self, r: usize, c: usize) -> usize {
        r * self.width + c
    }

    fn trace(&self) -> Option<Vec<Rect>> {
        let mut done = vec![0; self.width - 1];
        let mut corners = BinaryHeap::from([Reverse((0, 0))]);
        let mut rects = vec![];

        while let Some(Reverse((top, left))) = corners.pop() {
            if top == self.height - 1 || left == self.width - 1 || done[left] > top {
                continue;
            }

            let rect = self.find_rect(top, left)?;
            let covered = &mut done[left..rect.right];
            if covered.iter().any(|&d| d != top) {
                return None;
            }
            covered.fill(rect.bottom);

            rects.push(rect);
            corners.push(Reverse((top, rect.right)));
            corners.push(Reverse((rect.bottom, left)));
        }

        if rects.is_empty() || done.iter().any(|&d| d != self.height - 1) {
            return None;
        }
        Some(rects)
    }

    fn find_rect(&self, top: usize, left: usize) -> Option<Rect> {
        let top_end = self.right[self.at(top, left)];
        let mut corners = vec![];
        let mut reach = vec![];
        let mut explored = left;

        for r in top + 1..=self.down[self.at(top, left)] {
            if self.slots[self.at(r, left)].glyph != Glyph::Plus {
                continue;
            }

            let bottom_end = self.right[self.at(r, left)].min(top_end);
            let mut i = reach.partition_point(|&d| d < r);
            while i == corners.len() && explored < bottom_end {
                explored += 1;
                if self.slots[self.at(top, explored)].glyph == Glyph::Plus {
                    let d = self.down[self.at(top, explored)];
                    reach.push(reach.last().map_or(d, |&m: &usize| m.max(d)));
                    corners.push(explored);
                    if d < r {
                        i += 1;
                    }
                }
            }

            if i < corners.len() && corners[i] <= bottom_end {
                return Some(Rect {
                    top,
                    left,
                    bottom: r,
                    right: corners[i],
                });
            }
        }

        None
    }

    fn layout(&self, lines: &[Line<'_>], rects: &[Rect]) -> Layout {
        let col_bounds = bounds(rects.iter().flat_map(|r| [r.left, r.right]));
        let row_bounds = bounds(rects.iter().flat_map(|r| [r.top, r.bottom]));
        let index = |bounds: &[usize], at: usize| bounds.binary_search(&at).unwrap();

        let alignment_line = self.part_separators.first().copied().unwrap_or(0);
        let mut columns: Vec<GridTableColumn> = col_bounds
            .windows(2)
            .map(|w| GridTableColumn {
                alignment: TableAlignment::None,
                width: w[1] - w[0] - 1,
            })
            .collect();
        let rule = trim_end(lines[alignment_line].text).as_bytes();
        let pluses: Vec<usize> = (0..rule.len()).filter(|&c| rule[c] == b'+').collect();
        for w in pluses.windows(2) {
            if let Ok(i) = col_bounds.binary_search(&w[0]) {
                columns[i].alignment = match (rule[w[0] + 1] == b':', rule[w[1] - 1] == b':') {
                    (true, false) => TableAlignment::Left,
                    (false, true) => TableAlignment::Right,
                    (true, true) => TableAlignment::Center,
                    (false, false) => TableAlignment::None,
                };
            }
        }

        let head_end = self.part_separators.first().copied();
        let foot_start = match self.part_separators[..] {
            [.., penultimate, last] if last == self.height - 1 => Some(penultimate),
            _ => None,
        };
        let mut rows: Vec<Row> = row_bounds
            .windows(2)
            .map(|w| Row {
                section: if head_end.is_some_and(|h| w[0] < h) {
                    GridTableSection::Head
                } else if foot_start.is_some_and(|f| w[0] >= f) {
                    GridTableSection::Foot
                } else {
                    GridTableSection::Body
                },
                top: w[0],
                bottom: w[1],
                cells: vec![],
            })
            .collect();

        for rect in rects {
            let column = index(&col_bounds, rect.left);
            let row = index(&row_bounds, rect.top);
            let content = (rect.top + 1..rect.bottom)
                .map(|r| {
                    self.slots[self.at(r, rect.left)].byte + 1
                        ..self.slots[self.at(r, rect.right)].byte
                })
                .collect();
            rows[row].cells.push(Cell {
                left: rect.left,
                value: NodeGridTableCell {
                    column,
                    colspan: index(&col_bounds, rect.right) - column,
                    rowspan: index(&row_bounds, rect.bottom) - row,
                },
                content,
            });
        }

        Layout { columns, rows }
    }
}

fn bounds(it: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut v: Vec<usize> = it.collect();
    v.sort_unstable();
    v.dedup();
    v
}

pub(crate) fn trim_end(text: &str) -> &str {
    text.trim_end_matches([' ', '\t'])
}

pub(crate) fn is_rule(text: &str, fill: u8) -> bool {
    let bytes = text.as_bytes();
    let Some((&b'+', mut rest)) = bytes.split_first() else {
        return false;
    };
    while !rest.is_empty() {
        let Some(end) = rest.iter().position(|&b| b == b'+') else {
            return false;
        };
        let segment = &rest[..end];
        let segment = segment.strip_prefix(b":").unwrap_or(segment);
        let segment = segment.strip_suffix(b":").unwrap_or(segment);
        if segment.is_empty() || segment.iter().any(|&b| b != fill) {
            return false;
        }
        rest = &rest[end + 1..];
    }
    true
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;

    fn trace(md: &str) -> Option<String> {
        trace_from(md, 0)
    }

    fn trace_from(md: &str, column: usize) -> Option<String> {
        let lines: Vec<Line<'_>> = md.lines().map(|text| Line { text, column }).collect();
        let layout = layout(&lines)?;

        let mut out = String::new();
        let columns: Vec<String> = layout
            .columns
            .iter()
            .map(|c| match c.alignment {
                TableAlignment::None => format!("{}", c.width),
                TableAlignment::Left => format!(":{}", c.width),
                TableAlignment::Right => format!("{}:", c.width),
                TableAlignment::Center => format!(":{}:", c.width),
            })
            .collect();
        writeln!(out, "columns {}", columns.join(" ")).unwrap();

        for row in &layout.rows {
            let section = match row.section {
                GridTableSection::Head => "head",
                GridTableSection::Body => "body",
                GridTableSection::Foot => "foot",
            };
            write!(out, "{} {}-{}", section, row.top, row.bottom).unwrap();
            for cell in &row.cells {
                let content: Vec<&str> = cell
                    .content
                    .iter()
                    .enumerate()
                    .map(|(i, range)| &lines[row.top + 1 + i].text[range.clone()])
                    .collect();
                write!(
                    out,
                    " [{} {}x{} {:?}]",
                    cell.value.column,
                    cell.value.colspan,
                    cell.value.rowspan,
                    content.join("\n")
                )
                .unwrap();
            }
            out.push('\n');
        }
        Some(out)
    }

    #[track_caller]
    fn assert_trace(md: &str, expected: &str) {
        assert_eq!(trace(md).as_deref(), Some(expected));
    }

    #[track_caller]
    fn assert_invalid(md: &str) {
        assert_eq!(trace(md), None);
    }

    #[test]
    fn single_cell() {
        assert_trace(
            "+---+\n| a |\n+---+\n",
            "columns 3\nbody 0-2 [0 1x1 \" a \"]\n",
        );
    }

    #[test]
    fn multiline_cells() {
        assert_trace(
            concat!(
                "+-----+----+\n",
                "| a   | c  |\n",
                "| b   |    |\n",
                "+-----+----+\n",
                "| d   | e  |\n",
                "+-----+----+\n",
            ),
            concat!(
                "columns 5 4\n",
                "body 0-3 [0 1x1 \" a   \\n b   \"] [1 1x1 \" c  \\n    \"]\n",
                "body 3-5 [0 1x1 \" d   \"] [1 1x1 \" e  \"]\n",
            ),
        );
    }

    #[test]
    fn empty_interiors() {
        assert_trace(
            "+-+--+\n+-+--+\n",
            "columns 1 2\nbody 0-1 [0 1x1 \"\"] [1 1x1 \"\"]\n",
        );
    }

    // https://gitlab.com/gitlab-org/gitlab/-/work_items/460864
    #[test]
    fn spans() {
        assert_trace(
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
                "columns 3 3 3\n",
                "body 0-2 [0 2x1 \" AAAAA \"] [2 1x2 \" B \\n B \\n B \"]\n",
                "body 2-4 [0 1x2 \" D \\n D \\n D \"] [1 1x1 \" E \"]\n",
                "body 4-6 [1 2x1 \" CCCCC \"]\n",
            ),
        );
    }

    #[test]
    fn columns_introduced_below() {
        assert_trace(
            concat!(
                "+-------+\n",
                "| a     |\n",
                "+---+---+\n",
                "| b | c |\n",
                "+---+---+\n",
            ),
            concat!(
                "columns 3 3\n",
                "body 0-2 [0 2x1 \" a     \"]\n",
                "body 2-4 [0 1x1 \" b \"] [1 1x1 \" c \"]\n",
            ),
        );
    }

    #[test]
    fn unaligned_border_characters_are_content() {
        assert_trace(
            concat!(
                "+---------+\n",
                "| x|y +z  |\n",
                "| --- | + |\n",
                "+---------+\n",
            ),
            "columns 9\nbody 0-3 [0 1x1 \" x|y +z  \\n --- | + \"]\n",
        );
    }

    #[test]
    fn unreachable_inner_rectangle_is_content() {
        assert_trace(
            concat!(
                "+-------+\n",
                "| x     |\n",
                "+---+   |\n",
                "|   |   |\n",
                "+---+---+\n",
            ),
            "columns 7\nbody 0-4 [0 1x1 \" x     \\n---+   \\n   |   \"]\n",
        );
    }

    #[test]
    fn head() {
        assert_trace(
            concat!(
                "+---+---+\n",
                "| a | b |\n",
                "+---+---+\n",
                "| c | d |\n",
                "+===+===+\n",
                "| e | f |\n",
                "+---+---+\n",
            ),
            concat!(
                "columns 3 3\n",
                "head 0-2 [0 1x1 \" a \"] [1 1x1 \" b \"]\n",
                "head 2-4 [0 1x1 \" c \"] [1 1x1 \" d \"]\n",
                "body 4-6 [0 1x1 \" e \"] [1 1x1 \" f \"]\n",
            ),
        );
    }

    #[test]
    fn head_only() {
        assert_trace(
            "+---+\n| a |\n+===+\n",
            "columns 3\nhead 0-2 [0 1x1 \" a \"]\n",
        );
    }

    #[test]
    fn head_and_foot() {
        assert_trace(
            concat!(
                "+---+\n", "| a |\n", "+===+\n", "| b |\n", "+---+\n", "| c |\n", "+===+\n",
                "| d |\n", "+===+\n",
            ),
            concat!(
                "columns 3\n",
                "head 0-2 [0 1x1 \" a \"]\n",
                "body 2-4 [0 1x1 \" b \"]\n",
                "body 4-6 [0 1x1 \" c \"]\n",
                "foot 6-8 [0 1x1 \" d \"]\n",
            ),
        );
    }

    #[test]
    fn two_part_separators_make_a_foot_and_no_body() {
        assert_trace(
            "+---+\n| a |\n+===+\n| b |\n+===+\n",
            "columns 3\nhead 0-2 [0 1x1 \" a \"]\nfoot 2-4 [0 1x1 \" b \"]\n",
        );
    }

    #[test]
    fn foot_needs_bottom_part_separator() {
        assert_trace(
            "+---+\n| a |\n+===+\n| b |\n+===+\n| c |\n+---+\n",
            concat!(
                "columns 3\n",
                "head 0-2 [0 1x1 \" a \"]\n",
                "body 2-4 [0 1x1 \" b \"]\n",
                "body 4-6 [0 1x1 \" c \"]\n",
            ),
        );
    }

    #[test]
    fn alignment_from_top_line_without_head() {
        assert_trace(
            "+:--+--:+:-:+---+\n| a | b | c | d |\n+---+---+---+---+\n",
            concat!(
                "columns :3 3: :3: 3\n",
                "body 0-2 [0 1x1 \" a \"] [1 1x1 \" b \"] [2 1x1 \" c \"] [3 1x1 \" d \"]\n",
            ),
        );
    }

    #[test]
    fn alignment_from_part_separator_overrides_top_line() {
        assert_trace(
            "+:--+---+\n| a | b |\n+===+==:+\n| c | d |\n+---+---+\n",
            concat!(
                "columns 3 3:\n",
                "head 0-2 [0 1x1 \" a \"] [1 1x1 \" b \"]\n",
                "body 2-4 [0 1x1 \" c \"] [1 1x1 \" d \"]\n",
            ),
        );
    }

    #[test]
    fn alignment_of_wide_segment_goes_to_its_first_column() {
        assert_trace(
            "+:------:+\n| a      |\n+---+----+\n| b | c  |\n+---+----+\n",
            concat!(
                "columns :3: 4\n",
                "body 0-2 [0 2x1 \" a      \"]\n",
                "body 2-4 [0 1x1 \" b \"] [1 1x1 \" c  \"]\n",
            ),
        );
    }

    #[test]
    fn colons_off_the_alignment_line_are_content() {
        assert_trace(
            "+---+\n| a |\n+:--+\n| b |\n+---+\n",
            "columns 3\nbody 0-4 [0 1x1 \" a \\n:--\\n b \"]\n",
        );
        assert_trace(
            "+---+\n| a |\n+=:=+\n| b |\n+---+\n",
            "columns 3\nbody 0-4 [0 1x1 \" a \\n=:=\\n b \"]\n",
        );
    }

    #[test]
    fn equals_off_part_separators_are_content() {
        assert_trace(
            "+---+---+\n| a | b |\n+===+   +\n| c |   |\n+---+---+\n",
            "columns 3 3\nbody 0-4 [0 1x1 \" a \\n===\\n c \"] [1 1x1 \" b \\n   \\n   \"]\n",
        );
        assert_invalid("+---+\n| a |\n+=--+\n");
    }

    #[test]
    fn trailing_whitespace_is_ignored() {
        assert_trace(
            "+---+  \n| a |\t\n+---+ \n",
            "columns 3\nbody 0-2 [0 1x1 \" a \"]\n",
        );
    }

    #[test]
    fn wide_characters_take_two_columns() {
        assert_trace(
            "+----+\n| 日 |\n+----+\n",
            "columns 4\nbody 0-2 [0 1x1 \" 日 \"]\n",
        );
        assert_invalid("+---+\n| 日 |\n+---+\n");
    }

    #[test]
    fn zero_width_characters_take_no_columns() {
        assert_trace(
            "+---+\n| e\u{301} |\n+---+\n",
            "columns 3\nbody 0-2 [0 1x1 \" e\\u{301} \"]\n",
        );
    }

    #[test]
    fn tabs_expand_from_the_line_column() {
        assert_trace(
            "+-------+\n|\ta   |\n+-------+\n",
            "columns 7\nbody 0-2 [0 1x1 \"\\ta   \"]\n",
        );
        assert_invalid("+-------+\n|\ta |\n+-------+\n");
        assert_eq!(
            trace_from("+-------+\n|\ta     |\n+-------+\n", 2).as_deref(),
            Some("columns 7\nbody 0-2 [0 1x1 \"\\ta     \"]\n"),
        );
    }

    #[test]
    fn invalid_top_lines() {
        assert_invalid("");
        assert_invalid("+---\n| a |\n+---+\n");
        assert_invalid(":+---+\n| a |\n+---+\n");
        assert_invalid("++\n||\n++\n");
        assert_invalid("+===+\n| a |\n+---+\n");
        assert_invalid("+-:-+\n| a |\n+---+\n");
        assert_invalid(" +---+\n | a |\n +---+\n");
    }

    #[test]
    fn top_line_alone_is_not_a_table() {
        assert_invalid("+---+\n");
    }

    #[test]
    fn missing_bottom_border() {
        assert_invalid("+---+\n| a |\n");
        assert_invalid("+---+\n| a |\n| b |\n");
    }

    #[test]
    fn ragged_lines() {
        assert_invalid("+---+\n| a |\n+--+\n");
        assert_invalid("+---+\n| a\n+---+\n");
        assert_invalid("+---+\n| a | b\n+---+\n");
        assert_invalid("+-------+-------+\n| a     | broken\n+-------+-------+\n");
    }

    #[test]
    fn incomplete_tiling() {
        assert_invalid(concat!(
            "+---+---+\n",
            "| a | b |\n",
            "+---+   |\n",
            "| c |   |\n",
            "+---+   |\n",
        ));
        assert_invalid(concat!(
            "+---+---+\n",
            "| a | b |\n",
            "+---+---+\n",
            "| c | d  \n",
            "+---+---+\n",
        ));
    }
}
