---
title: Grid tables
---

A grid table draws its cells with `+`, `-`, and `|` characters.  Unlike a [table](table.html), a grid table's cells can span several rows or columns, and each cell can hold any blocks — paragraphs, lists, code blocks, even other tables.  Use grid tables for tabular content that a pipe table can't express.

Grid tables come from [reStructuredText](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#grid-tables), and [Pandoc](https://pandoc.org/MANUAL.html#extension-grid_tables) supports them in Markdown.  This document describes Comrak's grid tables, which follow Pandoc's closely.  [Differences from Pandoc](#differences-from-pandoc) lists where they part ways.

To enable grid tables, pass `-e grid-table` on the command line, or set `extension.grid_table` in the options struct.  The options field and the AST nodes need the `grid_table` cargo feature, which the default features include.

Here's a typical grid table:

```````````````````````````````` example -e grid-table
+--------+------------------+
| Fruit  | Notes            |
+========+==================+
| Banana | - built-in       |
|        |   wrapper        |
|        | - bright colour  |
+--------+------------------+
| Orange | Cures scurvy.    |
+--------+------------------+
.
<table>
<thead>
<tr>
<th>Fruit</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
<td>Banana</td>
<td>
<ul>
<li>built-in
wrapper</li>
<li>bright colour</li>
</ul>
</td>
</tr>
<tr>
<td>Orange</td>
<td>Cures scurvy.</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Syntax

A [separator line](@) is a `+`, followed by one or more segments of hyphens (`-`), each followed by a `+`.  A [part separator](@) is the same, with equals signs (`=`) in place of hyphens.  In a grid table's top border and its part separators, a segment may also begin or end with a colon (`:`), to set [alignment](#alignment).

A [grid table](@) begins with a [separator line], its top border.  Each following line that begins with `+` or `|`, at the same indentation, belongs to the grid table.  The table's last line is its bottom border, which must be a [separator line] or a [part separator].

Within a grid table, `+` marks a corner, `-` and `=` draw horizontal borders, and `|` draws vertical borders.  The borders must divide the table into rectangles that cover it completely.  Each rectangle is a [cell](@).  If the borders don't divide the table this way, the lines aren't a grid table, and Comrak parses them as it would with the extension off.

### Borders

The simplest grid table has one [cell]:

```````````````````````````````` example -e grid-table
+---+
| a |
+---+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
````````````````````````````````

None of these first lines is a [separator line], so none of them begins a grid table:

```````````````````````````````` example -e grid-table
+---
| a |
+---+

+-:-+
| a |
+---+

+===+
| a |
+---+

++
||
++
.
<p>+---
| a |
+---+</p>
<p>+-:-+
| a |
+---+</p>
<p>+===+
| a |
+---+</p>
<p>++
||
++</p>
````````````````````````````````

A [separator line] on its own isn't a grid table, and neither are lines without a bottom border:

```````````````````````````````` example -e grid-table
+---+

+---+
| a |
.
<p>+---+</p>
<p>+---+
| a |</p>
````````````````````````````````

Every line must be exactly as wide as the top border.  Here, the bottom border is one column short:

```````````````````````````````` example -e grid-table
+---+
| a |
+--+
.
<p>+---+
| a |
+--+</p>
````````````````````````````````

Width is measured in display columns.  A wide character, such as most CJK characters, takes two columns:

```````````````````````````````` example -e grid-table
+----+----+
| 日 | ab |
+----+----+
.
<table>
<tbody>
<tr>
<td>日</td>
<td>ab</td>
</tr>
</tbody>
</table>
````````````````````````````````

So this row is one column too wide:

```````````````````````````````` example -e grid-table
+---+
| 日 |
+---+
.
<p>+---+
| 日 |
+---+</p>
````````````````````````````````

A tab advances to the next multiple of four columns, as it does in CommonMark's [tab stops][tabs]:

```````````````````````````````` example -e grid-table
+-------+
|→a   |
+-------+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
````````````````````````````````

Trailing spaces and tabs don't count:

```````````````````````````````` example -e grid-table
+---+→
| a |
+---+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
````````````````````````````````

Lines after the bottom border that begin with `+` or `|` still belong to the grid table.  Here, the last line leaves the table without a bottom border:

```````````````````````````````` example -e grid-table
+---+
| a |
+---+
| b |
.
<p>+---+
| a |
+---+
| b |</p>
````````````````````````````````

### Cells

A `|` or `+` is a border only where it joins up with the borders around it.  Elsewhere, it's content:

```````````````````````````````` example -e grid-table
+---------+
| x|y +z  |
| --- | + |
+---------+
.
<table>
<tbody>
<tr>
<td>x|y +z
--- | +</td>
</tr>
</tbody>
</table>
````````````````````````````````

A [cell] may span several columns or rows:

```````````````````````````````` example -e grid-table
+---+---+---+
| AAAAA | B |
+---+---+ B +
| D | E | B |
+ D +---+---+
| D | CCCCC |
+---+---+---+
.
<table>
<tbody>
<tr>
<td colspan="2">AAAAA</td>
<td rowspan="2">B
B
B</td>
</tr>
<tr>
<td rowspan="2">D
D
D</td>
<td>E</td>
</tr>
<tr>
<td colspan="2">CCCCC</td>
</tr>
</tbody>
</table>
````````````````````````````````

Rows needn't share column boundaries:

```````````````````````````````` example -e grid-table
+-------+
| a     |
+---+---+
| b | c |
+---+---+
.
<table>
<tbody>
<tr>
<td colspan="2">a</td>
</tr>
<tr>
<td>b</td>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

Here, the borders leave the region under `b` without a bottom, so the lines aren't a grid table:

```````````````````````````````` example -e grid-table
+---+---+
| a | b |
+---+   |
| c |   |
+---+   |
.
<p>+---+---+
| a | b |
+---+   |
| c |   |
+---+   |</p>
````````````````````````````````

Comrak finds cells by tracing the borders from each corner, starting at the top left.  Borders it never reaches are content:

```````````````````````````````` example -e grid-table
+-------+
| x     |
+---+   |
|   |   |
+---+---+
.
<table>
<tbody>
<tr>
<td>x
---+
|</td>
</tr>
</tbody>
</table>
````````````````````````````````

A cell may be empty, and a row may have no lines between its borders:

```````````````````````````````` example -e grid-table
+-+--+
+-+--+
.
<table>
<tbody>
<tr>
<td></td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

So two grid tables without a [blank line] between them are one grid table, with an empty row in the middle:

```````````````````````````````` example -e grid-table
+---+
| a |
+---+
+---+
| b |
+---+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
<tr>
<td></td>
</tr>
<tr>
<td>b</td>
</tr>
</tbody>
</table>
````````````````````````````````

### Cell content

Comrak removes one leading space from each of a [cell]'s lines, along with any trailing spaces and tabs, then parses what's left as blocks.  So four spaces of indentation in a cell make a paragraph, and five make an [indented code block]:

```````````````````````````````` example -e grid-table
+----------+
|    a     |
+----------+
|     b    |
+----------+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
<tr>
<td>
<pre class="syntax-highlighting"><code><span class="text plain">b
</span></code></pre>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

A cell may contain any blocks:

```````````````````````````````` example -e grid-table
+------------+
| # Heading  |
|            |
| > quote    |
|            |
| 1. one     |
| 2. two     |
|            |
| ```        |
| code       |
| ```        |
+------------+
.
<table>
<tbody>
<tr>
<td>
<h1>Heading</h1>
<blockquote>
<p>quote</p>
</blockquote>
<ol>
<li>one</li>
<li>two</li>
</ol>
<pre class="syntax-highlighting"><code><span class="text plain">code
</span></code></pre>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

Inline content may span a cell's lines:

```````````````````````````````` example -e grid-table
+--------+
| *a     |
| b*     |
+--------+
.
<table>
<tbody>
<tr>
<td><em>a
b</em></td>
</tr>
</tbody>
</table>
````````````````````````````````

Because Comrak removes trailing spaces, only a backslash makes a [hard line break] in a cell:

```````````````````````````````` example -e grid-table
+-----+
| a\  |
| b   |
+-----+
.
<table>
<tbody>
<tr>
<td>a<br />
b</td>
</tr>
</tbody>
</table>
````````````````````````````````

[Link reference definitions] in a cell apply to the whole document, as do [footnote](footnotes.html) definitions:

```````````````````````````````` example -e grid-table
+------------+
| [b]: /u    |
+------------+

[b]
.
<table>
<tbody>
<tr>
<td></td>
</tr>
</tbody>
</table>
<p><a href="/u">b</a></p>
````````````````````````````````

### Head and foot

A grid table's [head](@) is the rows above its first [part separator].  A grid table has a [foot](@) if its bottom border is a part separator and another part separator comes before it.  The foot is the rows below the second-last part separator.  The other rows are the grid table's [body](@).

A head may have several rows:

```````````````````````````````` example -e grid-table
+---+---+
| a | b |
+---+---+
| c | d |
+===+===+
| e | f |
+---+---+
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
<tr>
<th>c</th>
<th>d</th>
</tr>
</thead>
<tbody>
<tr>
<td>e</td>
<td>f</td>
</tr>
</tbody>
</table>
````````````````````````````````

If the only part separator is the bottom border, every row is in the head:

```````````````````````````````` example -e grid-table
+---+
| a |
+---+
| b |
+===+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
<tr>
<th>b</th>
</tr>
</thead>
</table>
````````````````````````````````

A grid table may have a head, a body, and a foot:

```````````````````````````````` example -e grid-table
+---+
| a |
+===+
| b |
+---+
| c |
+===+
| d |
+===+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>b</td>
</tr>
<tr>
<td>c</td>
</tr>
</tbody>
<tfoot>
<tr>
<td>d</td>
</tr>
</tfoot>
</table>
````````````````````````````````

A part separator in the body doesn't begin a foot unless the bottom border is a part separator too:

```````````````````````````````` example -e grid-table
+---+
| a |
+===+
| b |
+===+
| c |
+---+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>b</td>
</tr>
<tr>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

With two part separators, the second of which is the bottom border, a grid table has a head and a foot, but no body:

```````````````````````````````` example -e grid-table
+---+
| a |
+===+
| b |
+===+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tfoot>
<tr>
<td>b</td>
</tr>
</tfoot>
</table>
````````````````````````````````

A line that's only partly `=` isn't a part separator.  Where its `=` characters don't form a border, they're content — here, a [setext heading] underline:

```````````````````````````````` example -e grid-table
+---+---+
| a | b |
+===+   +
| c |   |
+---+---+
.
<table>
<tbody>
<tr>
<td>
<h1>a</h1>
<p>c</p>
</td>
<td>b</td>
</tr>
</tbody>
</table>
````````````````````````````````

### Alignment

The first [part separator] sets each column's alignment, as a [delimiter row](table.html#delimiter-row) does in a [table](table.html).  A colon at the start of a segment aligns its column left, a colon at the end aligns it right, and both colons centre it.  A grid table without a part separator takes its alignment from its top border:

```````````````````````````````` example -e grid-table
+:--+--:+:-:+---+
| a | b | c | d |
+---+---+---+---+
.
<table>
<tbody>
<tr>
<td align="left">a</td>
<td align="right">b</td>
<td align="center">c</td>
<td>d</td>
</tr>
</tbody>
</table>
````````````````````````````````

With a part separator, colons in the top border have no effect:

```````````````````````````````` example -e grid-table
+:--+---+
| a | b |
+===+==:+
| c | d |
+---+---+
.
<table>
<thead>
<tr>
<th>a</th>
<th align="right">b</th>
</tr>
</thead>
<tbody>
<tr>
<td>c</td>
<td align="right">d</td>
</tr>
</tbody>
</table>
````````````````````````````````

A segment that spans several columns aligns only the first of them:

```````````````````````````````` example -e grid-table
+:------:+
| a      |
+---+----+
| b | c  |
+---+----+
.
<table>
<tbody>
<tr>
<td colspan="2" align="center">a</td>
</tr>
<tr>
<td align="center">b</td>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

Colons anywhere else are content:

```````````````````````````````` example -e grid-table
+---+
| a |
+:--+
| b |
+---+
.
<table>
<tbody>
<tr>
<td>a
:--
b</td>
</tr>
</tbody>
</table>
````````````````````````````````

### Beginning and ending

A grid table can't interrupt a [paragraph]:

```````````````````````````````` example -e grid-table
Not a table:
+---+
| a |
+---+
.
<p>Not a table:
+---+
| a |
+---+</p>
````````````````````````````````

The first line that doesn't begin with `+` or `|` ends the grid table, and needn't follow a [blank line]:

```````````````````````````````` example -e grid-table
+---+
| a |
+---+
text
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
<p>text</p>
````````````````````````````````

A blank line ends a grid table early.  Here, neither part is a grid table:

```````````````````````````````` example -e grid-table
+---+
| a |

| b |
+---+
.
<p>+---+
| a |</p>
<p>| b |
+---+</p>
````````````````````````````````

A grid table may be indented up to three spaces:

```````````````````````````````` example -e grid-table
   +---+
   | a |
   +---+
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
````````````````````````````````

With four, it's an [indented code block]:

```````````````````````````````` example -e grid-table
    +---+
    | a |
    +---+
.
<pre class="syntax-highlighting"><code><span class="text plain">+---+
| a |
+---+
</span></code></pre>
````````````````````````````````

Every line must have the same indentation:

```````````````````````````````` example -e grid-table
+---+
 | a |
+---+
.
<p>+---+
| a |
+---+</p>
````````````````````````````````

## Grid tables in containers

Grid tables may appear in any container, such as a [list item] or a [block quote]:

```````````````````````````````` example -e grid-table
- Inside a list item:

  +---+
  | a |
  +---+

> +---+
> | b |
> +---+
.
<ul>
<li>
<p>Inside a list item:</p>
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
</li>
</ul>
<blockquote>
<table>
<tbody>
<tr>
<td>b</td>
</tr>
</tbody>
</table>
</blockquote>
````````````````````````````````

Rows can't be [lazy continuation lines]:

```````````````````````````````` example -e grid-table
> +---+
| a |
> +---+
.
<blockquote>
<p>+---+
| a |
+---+</p>
</blockquote>
````````````````````````````````

A cell may contain a grid table, up to five grid tables deep.  Comrak doesn't recognise a sixth level:

```````````````````````````````` example -e grid-table
+-----------------------+
| +-------------------+ |
| | +---------------+ | |
| | | +-----------+ | | |
| | | | +-------+ | | | |
| | | | | +---+ | | | | |
| | | | | | x | | | | | |
| | | | | +---+ | | | | |
| | | | +-------+ | | | |
| | | +-----------+ | | |
| | +---------------+ | |
| +-------------------+ |
+-----------------------+
.
<table>
<tbody>
<tr>
<td>
<table>
<tbody>
<tr>
<td>
<table>
<tbody>
<tr>
<td>
<table>
<tbody>
<tr>
<td>
<table>
<tbody>
<tr>
<td>+---+
| x |
+---+</td>
</tr>
</tbody>
</table>
</td>
</tr>
</tbody>
</table>
</td>
</tr>
</tbody>
</table>
</td>
</tr>
</tbody>
</table>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Output

HTML output puts the [head] in a `<thead>` element, the [body] in a `<tbody>` element, and the [foot] in a `<tfoot>` element.  It leaves out any of these that has no rows.  Head cells are `<th>` elements, and other cells are `<td>` elements.  A cell that spans several columns or rows has a `colspan` or `rowspan` attribute.  A cell in an aligned column has an `align` attribute, and a cell that spans several columns takes the alignment of its first:

```````````````````````````````` example -e grid-table
+-------+-------+
| Left  | Right |
+:======+======:+
| a     | b     |
+-------+-------+
| spans both    |
+=======+=======+
| foot  | foot  |
+=======+=======+
.
<table>
<thead>
<tr>
<th align="left">Left</th>
<th align="right">Right</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">a</td>
<td align="right">b</td>
</tr>
<tr>
<td colspan="2" align="left">spans both</td>
</tr>
</tbody>
<tfoot>
<tr>
<td align="left">foot</td>
<td align="right">foot</td>
</tr>
</tfoot>
</table>
````````````````````````````````

A cell whose only block is a paragraph has no `<p>` element.  In a cell with more blocks, each paragraph has one:

```````````````````````````````` example -e grid-table
+-----+-----+
| a   | a   |
| b   |     |
|     | b   |
+-----+-----+
.
<table>
<tbody>
<tr>
<td>a
b</td>
<td>
<p>a</p>
<p>b</p>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

CommonMark output redraws each grid table.  Each column keeps at least its width from the source, and grows to fit its widest cell.  The redrawn borders have a `+` only at the corners of cells, so they can differ from the source.  For instance, see the CommonMark view of [example 12](#example-12).

In XML output, a grid table is a `grid_table` element.  Its `alignments` and `widths` attributes list each column's alignment (`left`, `right`, `center`, or `none`) and width.  Each row is a `grid_table_row` element, with a `section` attribute of `head`, `body`, or `foot`.  Each cell is a `grid_table_cell` element, with `column`, `colspan`, and `rowspan` attributes.

In the AST, a grid table is a [`NodeValue::GridTable`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.GridTable).  Its [`NodeGridTable`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeGridTable.html) holds a [`GridTableColumn`](https://docs.rs/comrak/latest/comrak/nodes/struct.GridTableColumn.html) for each column, with the column's alignment and its width between the borders, in display columns.  Each row is a [`NodeValue::GridTableRow`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.GridTableRow), whose value is its [`GridTableSection`](https://docs.rs/comrak/latest/comrak/nodes/enum.GridTableSection.html).  Each cell is a [`NodeValue::GridTableCell`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.GridTableCell), whose [`NodeGridTableCell`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeGridTableCell.html) holds the index of its first column and its spans.  A cell's children are blocks.

A row's source position covers the lines between its borders, and a cell's covers its content, inside its borders.  A row with no lines between its borders has an empty range at the start of its top border:

```````````````````````````````` example -e grid-table --sourcepos
+-----+---+
| a   | c |
| b   |   |
+-----+---+
+-----+---+
.
<table data-sourcepos="1:1-5:11">
<tbody>
<tr data-sourcepos="2:1-3:11">
<td data-sourcepos="2:2-3:6">a
b</td>
<td data-sourcepos="2:8-3:10">c</td>
</tr>
<tr data-sourcepos="4:1-4:0">
<td data-sourcepos="4:1-4:0"></td>
<td data-sourcepos="4:1-4:0"></td>
</tr>
</tbody>
</table>
````````````````````````````````

## Options

### Grid table widths

With `--grid-table-widths` (`render.grid_table_widths`), HTML output adds a `<colgroup>` element to each grid table, with a `<col>` element for each column.  Each `<col>` has a `style` attribute giving its column's width in the source, as a percentage of all the columns' widths, rounded to two decimal places.

Without the option, there's no `<colgroup>`:

```````````````````````````````` example -e grid-table
+---+-----+--+
| a | b   | c|
+---+-----+--+
.
<table>
<tbody>
<tr>
<td>a</td>
<td>b</td>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

With the option, there is:

```````````````````````````````` example -e grid-table --grid-table-widths
+---+-----+--+
| a | b   | c|
+---+-----+--+
.
<table>
<colgroup>
<col style="width:30%" />
<col style="width:50%" />
<col style="width:20%" />
</colgroup>
<tbody>
<tr>
<td>a</td>
<td>b</td>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

Because of rounding, the percentages may not add up to exactly 100:

```````````````````````````````` example -e grid-table --grid-table-widths
+--+--+--+
|a |b |c |
+--+--+--+
.
<table>
<colgroup>
<col style="width:33.33%" />
<col style="width:33.33%" />
<col style="width:33.33%" />
</colgroup>
<tbody>
<tr>
<td>a</td>
<td>b</td>
<td>c</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Interactions

`--gfm` doesn't enable grid tables, because they aren't part of GitHub Flavored Markdown.

With [tables](table.html) also enabled, both kinds of table work in the same document:

```````````````````````````````` example -e table -e grid-table
| a |
| - |

+---+
| b |
+---+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<table>
<tbody>
<tr>
<td>b</td>
</tr>
</tbody>
</table>
````````````````````````````````

A table takes every following non-blank line as a body row, so it takes the lines of a grid table straight after it.  Leave a [blank line] between them:

```````````````````````````````` example -e table -e grid-table
| a |
| - |
+---+
| b |
+---+
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>+---+</td>
</tr>
<tr>
<td>b</td>
</tr>
<tr>
<td>+---+</td>
</tr>
</tbody>
</table>
````````````````````````````````

A cell may contain a table:

```````````````````````````````` example -e table -e grid-table
+-------------+
| a   | b     |
| --- | ---   |
| c   | d     |
+-------------+
.
<table>
<tbody>
<tr>
<td>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>c</td>
<td>d</td>
</tr>
</tbody>
</table>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

With [task lists](tasklist.html), a list in a cell may contain task list items.  [Task lists in tables](tasklist.html#task-lists-in-tables) doesn't apply to grid tables, so a cell whose only content is a task list item marker stays as text:

```````````````````````````````` example -e grid-table -e tasklist --tasklist-in-table
+---------+-----+
| - [x] a | [x] |
| - [ ] b |     |
+---------+-----+
.
<table>
<tbody>
<tr>
<td>
<ul>
<li><input type="checkbox" checked="" disabled="" /> a</li>
<li><input type="checkbox" disabled="" /> b</li>
</ul>
</td>
<td>[x]</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Differences from Pandoc

[Pandoc](https://pandoc.org) has no equivalent of the [options](#options) above, and its other extensions differ from Comrak's, so the examples under [Options](#options), [Interactions](#interactions), and [Regressions](#regressions) aren't comparable.  On every other example above, Pandoc 3.7 and Comrak agree on HTML table structure, except that Pandoc:

* doesn't recognise an indented grid table ([example 36](#example-36));
* accepts rows narrower or wider than the top border, such as a row with a wide character counted as one column ([example 7](#example-7));
* accepts [lazy continuation lines] as rows ([example 40](#example-40));
* makes a grid table from whatever cells it can trace when the borders leave a gap, and drops the rest ([example 14](#example-14));
* ends a grid table at its first bottom border, and drops any lines after it that begin with `+` or `|` ([example 10](#example-10));
* keeps row and column boundaries that no cell uses, so it gives cells `rowspan` or `colspan` attributes and adds empty rows ([example 15](#example-15), [example 28](#example-28), [example 32](#example-32));
* writes an empty `<tbody>` element for a grid table with no [body] ([example 24](#example-24), [example 27](#example-27)); and,
* nests grid tables without a limit ([example 41](#example-41)).

Pandoc also writes column widths for every grid table, relative to a 72-column page, and aligns cells with `style` attributes rather than `align` attributes.  It supports a caption before or after a grid table, in a paragraph beginning with `Table:` or `:`; Comrak doesn't.

## Regressions

The following examples guard against regressions.

Source positions of a task list item in a cell, whose paragraph begins with a soft line break ([#917](https://github.com/kivikakk/comrak/pull/917)):

```````````````````````````````` example -e grid-table -e tasklist --sourcepos
+-------+
| + [ ] |
| --    |
+-------+
.
<table data-sourcepos="1:1-4:9">
<tbody>
<tr data-sourcepos="2:1-3:9">
<td data-sourcepos="2:2-3:8">
<ul data-sourcepos="2:3-3:4">
<li data-sourcepos="2:3-3:4"><input type="checkbox" disabled="" /> 
--</li>
</ul>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

[blank line]: https://spec.commonmark.org/0.31.2/#blank-line
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-break
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[lazy continuation lines]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[link reference definitions]: https://spec.commonmark.org/0.31.2/#link-reference-definition
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-heading
[tabs]: https://spec.commonmark.org/0.31.2/#tabs
