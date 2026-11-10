use std::mem;

use crate::nodes::{LineColumn, Node, Sourcepos};
use crate::parser::Parser;
use crate::parser::inlines::RefMap;

pub(crate) struct Fragment<'s> {
    pub(crate) text: &'s str,
    pub(crate) eol: &'s str,
    pub(crate) line: usize,
    pub(crate) byte: usize,
    pub(crate) column: usize,
}

impl<'a: 'd, 'o, 'c, 'd> Parser<'a, 'o, 'c, 'd>
where
    'c: 'o,
{
    pub(crate) fn parse_fragments(
        &mut self,
        container: Node<'a>,
        fragments: &[Fragment<'_>],
        suppress_grid_tables: bool,
    ) {
        let sourcepos = container.data().sourcepos;
        let mut child = self.fragment_parser(container, fragments, suppress_grid_tables);
        child.finalize_blocks().ok();
        self.reclaim(child, fragments);
        container.data_mut().sourcepos = sourcepos;
        remap(container, fragments);
    }

    pub(crate) fn open_fragments(
        &mut self,
        container: Node<'a>,
        fragments: &[Fragment<'_>],
        suppress_grid_tables: bool,
    ) -> Node<'a> {
        let child = self.fragment_parser(container, fragments, suppress_grid_tables);
        let current = child.current;
        #[cfg(feature = "phoenix_heex")]
        {
            self.heex_block_depth = child.heex_block_depth;
        }
        self.reclaim(child, fragments);
        remap(container, fragments);
        current
    }

    fn fragment_parser(
        &mut self,
        container: Node<'a>,
        fragments: &[Fragment<'_>],
        suppress_grid_tables: bool,
    ) -> Parser<'a, 'o, 'c, 'd> {
        assert!(container.parent().is_none());

        let mut text =
            String::with_capacity(fragments.iter().map(|f| f.text.len() + f.eol.len()).sum());
        let mut ends = Vec::with_capacity(fragments.len());
        for fragment in fragments {
            text.push_str(fragment.text);
            text.push_str(fragment.eol);
            ends.push(text.len());
        }

        let mut child = Parser::new(
            self.arena,
            container,
            self.options,
            self.cancellation.callback,
            self.delimiter_arena,
        );
        child.refmap = mem::replace(&mut self.refmap, RefMap::new());
        child.grid_table_suppressed = suppress_grid_tables;
        child.grid_table_depth = self.grid_table_depth + 1;

        let mut start = 0;
        for (fragment, &end) in fragments.iter().zip(&ends) {
            if child.cancellation.step() {
                break;
            }
            child.process_line(&text[start..end], fragment.column);
            start = end;
        }
        child
    }

    fn reclaim(&mut self, child: Parser<'a, 'o, 'c, 'd>, fragments: &[Fragment<'_>]) {
        self.refmap = child.refmap;
        self.cell_line_ends.extend(
            child
                .cell_line_ends
                .into_iter()
                .map(|end| end.remap(fragments)),
        );
        if child.cancellation.check().is_err() {
            self.cancellation.cancelled.set(true);
        }
    }
}

fn remap(container: Node<'_>, fragments: &[Fragment<'_>]) {
    let at = |lc: LineColumn| {
        let fragment = &fragments[lc.line - 1];
        LineColumn {
            line: fragment.line,
            column: fragment.byte + lc.column,
        }
    };

    for node in container.descendants().skip(1) {
        let mut ast = node.data_mut();
        let ast = &mut *ast;
        if let Some(lines) = ast.lines.as_mut() {
            let first = ast.sourcepos.start.line - 1;
            for (offset, fragment) in lines.offsets.iter_mut().zip(&fragments[first..]) {
                *offset += fragment.byte;
            }
            for position in &mut lines.escaped_pipes {
                *position = at(*position);
            }
        }
        ast.sourcepos = Sourcepos {
            start: at(ast.sourcepos.start),
            end: at(ast.sourcepos.end),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::nodes::{Ast, NodeGridTableCell, NodeValue};
    use crate::parser::Parser;
    use crate::{Arena, Options};

    use super::*;

    fn outline(node: Node<'_>) -> Vec<String> {
        node.descendants()
            .skip(1)
            .map(|n| {
                let depth = n
                    .ancestors()
                    .skip(1)
                    .take_while(|a| !a.same_node(node))
                    .count();
                let ast = n.data();
                let detail = match ast.value {
                    NodeValue::Text(ref t) => format!(" {t:?}"),
                    NodeValue::Link(ref l) => format!(" {:?}", l.url),
                    _ => String::new(),
                };
                format!(
                    "{}{} {}{}",
                    "  ".repeat(depth),
                    ast.value.xml_node_name(),
                    ast.sourcepos,
                    detail
                )
            })
            .collect()
    }

    fn parse_cell(
        fragments: &[Fragment<'_>],
        md: &str,
        options: &Options,
    ) -> (Vec<String>, Vec<String>) {
        let arena = Arena::new();
        let root = arena.alloc(Ast::new(NodeValue::Document, (1, 1).into()).into());
        let cell = arena.alloc(
            Ast::new(
                NodeValue::GridTableCell(NodeGridTableCell {
                    column: 0,
                    colspan: 1,
                    rowspan: 1,
                }),
                (1, 1).into(),
            )
            .into(),
        );
        let delimiter_arena = typed_arena::Arena::new();
        let mut parser = Parser::new(&arena, root, options, None, &delimiter_arena);
        parser.parse_fragments(cell, fragments, false);
        root.append(cell);
        let root = parser.parse(md).unwrap();

        let rest = root
            .children()
            .skip(1)
            .flat_map(|n| {
                let mut v = vec![format!(
                    "{} {}",
                    n.data().value.xml_node_name(),
                    n.data().sourcepos
                )];
                v.extend(outline(n).into_iter().map(|s| format!("  {s}")));
                v
            })
            .collect();
        (outline(cell), rest)
    }

    fn fragments<'s>(lines: &[&'s str], first_line: usize, byte: usize) -> Vec<Fragment<'s>> {
        lines
            .iter()
            .enumerate()
            .map(|(i, &text)| Fragment {
                text,
                eol: "\n",
                line: first_line + i,
                byte,
                column: byte,
            })
            .collect()
    }

    #[track_caller]
    fn assert_matches_indented(lines: &[&str], options: &Options) {
        let (cell, _) = parse_cell(&fragments(lines, 2, 2), "", options);

        let mut md = String::from("\n");
        for line in lines {
            if !line.is_empty() {
                md.push_str("  ");
            }
            md.push_str(line);
            md.push('\n');
        }
        let arena = Arena::new();
        let doc = crate::parse_document(&arena, &md, options);

        assert_eq!(cell, outline(doc));
    }

    #[test]
    fn positions_match_an_indented_document() {
        assert_matches_indented(
            &[
                "- a", "- b *c*", "  e `f`", "", "> d", "", "```", "x", "```",
            ],
            &Options::default(),
        );
    }

    #[test]
    fn escaped_pipe_positions_match_an_indented_document() {
        let mut options = Options::default();
        options.extension.table = true;
        assert_matches_indented(&["| a | b\\|c |", "|---|---|", "| d\\| | e |"], &options);
    }

    #[test]
    fn tabs_expand_from_the_fragment_column() {
        let options = Options::default();
        let at = |column| {
            let fragment = Fragment {
                text: "\tx",
                eol: "\n",
                line: 1,
                byte: 2,
                column,
            };
            parse_cell(&[fragment], "", &options).0[0].clone()
        };
        assert_eq!(at(0), "code_block 1:4-1:4");
        assert_eq!(at(2), "paragraph 1:4-1:4");
    }

    #[test]
    fn link_reference_definitions_are_shared_in_document_order() {
        let (cell, rest) = parse_cell(
            &fragments(&["[a]: /cell", "", "[a] [b]"], 1, 2),
            "[a]: /doc\n[b]: /doc\n\n[a] [b]\n",
            &Options::default(),
        );
        assert_eq!(
            cell,
            [
                "paragraph 3:3-3:9",
                "  link 3:3-3:5 \"/cell\"",
                "    text 3:4-3:4 \"a\"",
                "  text 3:6-3:6 \" \"",
                "  link 3:7-3:9 \"/doc\"",
                "    text 3:8-3:8 \"b\"",
            ]
        );
        assert_eq!(
            rest,
            [
                "paragraph 4:1-4:7",
                "  link 4:1-4:3 \"/cell\"",
                "    text 4:2-4:2 \"a\"",
                "  text 4:4-4:4 \" \"",
                "  link 4:5-4:7 \"/doc\"",
                "    text 4:6-4:6 \"b\"",
            ]
        );
    }
}
