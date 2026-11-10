use std::fmt::{self, Write};

use crate::html::{ChildRendering, Context, render_alignment, render_block_id, render_sourcepos};
use crate::nodes::{GridTableSection, Node, NodeGridTableCell, NodeValue};

pub(super) fn render_grid_table<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    context.cr()?;
    if entering {
        context.write_str("<table")?;
        render_block_id(context, node)?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</table>")?;
    }
    context.lf()?;
    Ok(ChildRendering::HTML)
}

fn section_tag(section: GridTableSection) -> &'static str {
    match section {
        GridTableSection::Head => "thead",
        GridTableSection::Body => "tbody",
        GridTableSection::Foot => "tfoot",
    }
}

fn in_section(node: Option<Node<'_>>, section: GridTableSection) -> bool {
    node.is_some_and(|n| matches!(n.data().value, NodeValue::GridTableRow(s) if s == section))
}

pub(super) fn render_grid_table_row<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    section: GridTableSection,
) -> Result<ChildRendering, fmt::Error> {
    context.cr()?;
    if entering {
        if !in_section(node.previous_sibling(), section) {
            write!(context, "<{}>", section_tag(section))?;
            context.lf()?;
        }
        context.write_str("<tr")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</tr>")?;
        if !in_section(node.next_sibling(), section) {
            context.cr()?;
            write!(context, "</{}>", section_tag(section))?;
        }
    }
    Ok(ChildRendering::HTML)
}

pub(super) fn render_grid_table_cell<T>(
    context: &mut Context<T>,
    node: Node<'_>,
    entering: bool,
    cell: NodeGridTableCell,
) -> Result<ChildRendering, fmt::Error> {
    let row = node.parent().unwrap();
    let tag = match row.data().value {
        NodeValue::GridTableRow(GridTableSection::Head) => "th",
        _ => "td",
    };

    if entering {
        let alignment = match row.parent().unwrap().data().value {
            NodeValue::GridTable(ref table) => table.columns[cell.column].alignment,
            _ => unreachable!(),
        };
        context.cr()?;
        write!(context, "<{tag}")?;
        render_sourcepos(context, node)?;
        if cell.colspan > 1 {
            write!(context, " colspan=\"{}\"", cell.colspan)?;
        }
        if cell.rowspan > 1 {
            write!(context, " rowspan=\"{}\"", cell.rowspan)?;
        }
        render_alignment(context, alignment)?;
        context.write_str(">")?;
    } else {
        write!(context, "</{tag}>")?;
    }
    Ok(ChildRendering::HTML)
}
