---
title: Tables
---

A table arranges content in rows and columns.  The first row is a header row, and each row after it holds data.  Use tables for information that readers compare across rows or columns.

Tables originate in [GitHub Flavored Markdown](https://github.github.com/gfm/#tables-extension-).  This document describes Comrak's tables, which can differ from GitHub's.  GitLab uses Comrak's tables, so this document is also the canonical spec for [GitLab Flavored Markdown tables](https://docs.gitlab.com/user/markdown/#tables).

To enable tables, pass `-e table` (or `--gfm`) on the command line, or set `extension.table` in the options struct.

Here's a typical table:

```````````````````````````````` example -e table
| Fruit  | Colour | Price |
| ------ | :----: | ----: |
| Apple  | Red    |  1.20 |
| Banana | Yellow |  0.50 |
.
<table>
<thead>
<tr>
<th>Fruit</th>
<th align="center">Colour</th>
<th align="right">Price</th>
</tr>
</thead>
<tbody>
<tr>
<td>Apple</td>
<td align="center">Red</td>
<td align="right">1.20</td>
</tr>
<tr>
<td>Banana</td>
<td align="center">Yellow</td>
<td align="right">0.50</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Syntax

A [table row](@) is a line divided into [cells](@) by pipes (`|`), except pipes that are part of a [backslash escape].  A leading pipe and a trailing pipe are optional.  A table row must contain at least one cell.  A cell's content is the text between its pipes, without leading or trailing [spaces] or [tabs].

A [delimiter row](@) is a [table row] whose cells each consist of one or more hyphens (`-`), optionally preceded by a colon (`:`), and optionally followed by a colon.

A [table](@) begins when the last line of a [paragraph] is a [table row] and the next line is a [delimiter row] with the same number of cells.  That last line becomes the table's [header row](@), and any earlier lines remain a paragraph.  Each later line that is a [table row] becomes a [body row](@), until the table ends.

The simplest table has one column and no body rows:

```````````````````````````````` example -e table
| a |
| - |
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
````````````````````````````````

### Delimiter rows

Colons in a delimiter row set each column's alignment.  A leading colon aligns the column left, a trailing colon aligns it right, and both colons centre it.  A cell with no colons leaves the column unaligned:

```````````````````````````````` example -e table
| a | b  | c  | d   |
| - | :- | -: | :-: |
| e | f  | g  | h   |
.
<table>
<thead>
<tr>
<th>a</th>
<th align="left">b</th>
<th align="right">c</th>
<th align="center">d</th>
</tr>
</thead>
<tbody>
<tr>
<td>e</td>
<td align="left">f</td>
<td align="right">g</td>
<td align="center">h</td>
</tr>
</tbody>
</table>
````````````````````````````````

None of the following are delimiter rows:

```````````````````````````````` example -e table
| a |
|   |

| b |
| : |

| c |
| :: |

| d |
| - - |

| e |
| -:- |
.
<p>| a |
|   |</p>
<p>| b |
| : |</p>
<p>| c |
| :: |</p>
<p>| d |
| - - |</p>
<p>| e |
| -:- |</p>
````````````````````````````````

A delimiter row may be indented up to three spaces.  With four spaces, the line is [paragraph continuation text] instead:

```````````````````````````````` example -e table
| a |
   | - |

| b |
    | - |
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<p>| b |
| - |</p>
````````````````````````````````

### Pipes

Leading and trailing pipes are optional, and rows don't need to agree on them:

```````````````````````````````` example -e table
| abc | defghi |
:-: | -----------:
bar | baz
.
<table>
<thead>
<tr>
<th align="center">abc</th>
<th align="right">defghi</th>
</tr>
</thead>
<tbody>
<tr>
<td align="center">bar</td>
<td align="right">baz</td>
</tr>
</tbody>
</table>
````````````````````````````````

A delimiter row for a single column needs a pipe or a colon:

```````````````````````````````` example -e table
a
|-

b
:-
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<table>
<thead>
<tr>
<th align="left">b</th>
</tr>
</thead>
</table>
````````````````````````````````

Without one, a line of hyphens is a [setext heading underline], even if the line above has pipes:

```````````````````````````````` example -e table
a
---

| b |
---
.
<h2>a</h2>
<h2>| b |</h2>
````````````````````````````````

A delimiter row can't begin with a hyphen and a space, because that begins a [list item]:

```````````````````````````````` example -e table
a | b
- | -
.
<p>a | b</p>
<ul>
<li>| -</li>
</ul>
````````````````````````````````

### Header rows

The header row must have the same number of cells as the delimiter row.  Otherwise, the lines don't form a table:

```````````````````````````````` example -e table
| a | b |
| - |
.
<p>| a | b |
| - |</p>
````````````````````````````````

Earlier lines of the paragraph remain a paragraph:

```````````````````````````````` example -e table
123
456
| a | b |
| ---| --- |
d | e
.
<p>123
456</p>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>d</td>
<td>e</td>
</tr>
</tbody>
</table>
````````````````````````````````

Earlier lines remain a paragraph even if they contain pipes:

```````````````````````````````` example -e table
| x | y |
| a | b |
| - | - |
.
<p>| x | y |</p>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
</table>
````````````````````````````````

The delimiter row can't be a [lazy continuation line]:

```````````````````````````````` example -e table
> | a |
| - |
.
<blockquote>
<p>| a |
| - |</p>
</blockquote>
````````````````````````````````

### Cell content

Leading and trailing [spaces] and [tabs] aren't part of a cell's content:

```````````````````````````````` example -e table
|→a→|  b  |
|-|-|
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
</table>
````````````````````````````````

Cells contain inline content only.  Text that would begin a block elsewhere is literal:

```````````````````````````````` example -e table
| *a* | `b` |
| --- | --- |
| > c | # d |
.
<table>
<thead>
<tr>
<th><em>a</em></th>
<th><code>b</code></th>
</tr>
</thead>
<tbody>
<tr>
<td>&gt; c</td>
<td># d</td>
</tr>
</tbody>
</table>
````````````````````````````````

Comrak divides a row into cells before it parses inline content, so a pipe separates cells even inside a [code span] or a link:

```````````````````````````````` example -e table
| a | b |
| - | - |
| `c|d` |
| [e|f](/u) |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>`c</td>
<td>d`</td>
</tr>
<tr>
<td>[e</td>
<td>f](/u)</td>
</tr>
</tbody>
</table>
````````````````````````````````

A backslash before a pipe makes the pipe part of the cell's content.  This also works inside a [code span]:

```````````````````````````````` example -e table
| f\|oo  |
| ------ |
| b `\|` az |
| b **\|** im |
.
<table>
<thead>
<tr>
<th>f|oo</th>
</tr>
</thead>
<tbody>
<tr>
<td>b <code>|</code> az</td>
</tr>
<tr>
<td>b <strong>|</strong> im</td>
</tr>
</tbody>
</table>
````````````````````````````````

A backslash escapes whatever character follows it.  After an escaped backslash (`\\`), a pipe still separates cells:

```````````````````````````````` example -e table
| a | b |
| - | - |
| c\\| d |
| e\\\| f |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>c\</td>
<td>d</td>
</tr>
<tr>
<td>e\| f</td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

### Body rows

A body row with fewer cells than the header row gets an empty [added cell](@) at the end for each missing cell.  A body row with more cells loses the extra ones:

```````````````````````````````` example -e table
| abc | def |
| --- | --- |
| bar |
| bar | baz | boo |
.
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td></td>
</tr>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>
````````````````````````````````

A line without pipes is a body row with one cell:

```````````````````````````````` example -e table
| a | b |
| - | - |
bar
baz
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td></td>
</tr>
<tr>
<td>baz</td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

A line of two pipes is a body row with two empty cells.  A single pipe has no cells, so it isn't a table row and ends the table:

```````````````````````````````` example -e table
| a | b |
| - | - |
||
|
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td></td>
<td></td>
</tr>
</tbody>
</table>
<p>|</p>
````````````````````````````````

A second header row and delimiter row become ordinary body rows:

```````````````````````````````` example -e table
| a |
| - |
| b | c |
| - | - |
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
<td>-</td>
</tr>
</tbody>
</table>
````````````````````````````````

A [blank line] ends the table:

```````````````````````````````` example -e table
| a |
| - |

ok
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<p>ok</p>
````````````````````````````````

So does the beginning of any other block, such as a [block quote], an [ATX heading], a [list item], a [fenced code block], a [thematic break], or an [HTML block]:

```````````````````````````````` example -e table
| abc | def |
| --- | --- |
| bar | baz |
> bar
.
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>
<blockquote>
<p>bar</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e table
| a |
| - |
# b
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<h1>b</h1>
````````````````````````````````

```````````````````````````````` example -e table
| a |
| - |
- b
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<ul>
<li>b</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e table
| a |
| - |
```
b
```
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<pre class="syntax-highlighting"><code><span class="text plain">b
</span></code></pre>
````````````````````````````````

```````````````````````````````` example -e table
| a |
| - |
***
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<hr />
````````````````````````````````

```````````````````````````````` example -e table --unsafe
| a |
| - |
<div>
b
</div>
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<div>
b
</div>
````````````````````````````````

A line of hyphens after a body row is a [thematic break], not a body row:

```````````````````````````````` example -e table
| a |
| - |
| b |
---
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
</tbody>
</table>
<hr />
````````````````````````````````

A line indented four or more spaces begins an [indented code block]:

```````````````````````````````` example -e table
| a |
| - |
| b |
    c
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
</tbody>
</table>
<pre class="syntax-highlighting"><code><span class="text plain">c
</span></code></pre>
````````````````````````````````

A line of `=` characters is a body row, because only a paragraph can have a setext heading underline:

```````````````````````````````` example -e table
| a |
| - |
===
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>===</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Tables in containers

Tables may appear in any container, such as a [list item] or a [block quote]:

```````````````````````````````` example -e table
- |a|b|
  |-|-|
  |c|d|
.
<ul>
<li>
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
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e table
> |a|b|
> |-|-|
> |c|d|
.
<blockquote>
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
</blockquote>
````````````````````````````````

A table can interrupt a paragraph in a container:

```````````````````````````````` example -e table
- foo
  | a |
  | - |
.
<ul>
<li>foo
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
</li>
</ul>
````````````````````````````````

A line indented less than its list item's content ends both the table and the list item:

```````````````````````````````` example -e table
- | a |
  | - |
| b |
.
<ul>
<li>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
</li>
</ul>
<p>| b |</p>
````````````````````````````````

Body rows can't be [lazy continuation lines]:

```````````````````````````````` example -e table
> | a |
> | - |
| b |
.
<blockquote>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
</blockquote>
<p>| b |</p>
````````````````````````````````

## Output

HTML output puts the header row in a `<thead>` element and the body rows in a `<tbody>` element.  Header cells are `<th>` elements, and body cells are `<td>` elements.  The cells of an aligned column have an `align` attribute.  [Added cells][added cell] take their column's alignment:

```````````````````````````````` example -e table
|x|b|x|x|e|
|:-|:-:|-|-:|:-:|
|f|x|x|i|
.
<table>
<thead>
<tr>
<th align="left">x</th>
<th align="center">b</th>
<th>x</th>
<th align="right">x</th>
<th align="center">e</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">f</td>
<td align="center">x</td>
<td>x</td>
<td align="right">i</td>
<td align="center"></td>
</tr>
</tbody>
</table>
````````````````````````````````

A table with no body rows has no `<tbody>`:

```````````````````````````````` example -e table
| abc | def |
| --- | --- |
.
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
</table>
````````````````````````````````

CommonMark output writes each delimiter cell as `---`, `:--`, `--:`, or `:-:`, and doesn't pad cells to line up.  It writes [added cells][added cell] out as empty cells.  `--width` doesn't wrap table rows:

```````````````````````````````` example -e table --width 10
| a | b |
|:-|-|
| one two three four |
.
<table>
<thead>
<tr>
<th align="left">a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">one two three four</td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

In XML output, a table is a `table` element.  It contains a `table_header` element for the header row and a `table_row` element for each body row.  Each cell is a `table_cell` element.  Header cells in aligned columns have an `align` attribute of `left`, `right`, or `center`.

In the AST, a table is a [`NodeValue::Table`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Table).  Its [`NodeTable`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeTable.html) holds each column's alignment, and the numbers of columns, rows, and cells parsed from the source.  The cell count excludes [added cells][added cell].  Each row is a [`NodeValue::TableRow`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.TableRow), whose `NodeTableRow`'s `header` field is `true` for the header row.  Each cell is a [`NodeValue::TableCell`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.TableCell).

The source position of an [added cell] is an empty range, just after the row's last cell in the source:

```````````````````````````````` example -e table --sourcepos
| a | b |
| - | - |
| c |
.
<table data-sourcepos="1:1-3:5">
<thead>
<tr data-sourcepos="1:1-1:9">
<th data-sourcepos="1:2-1:4">a</th>
<th data-sourcepos="1:6-1:8">b</th>
</tr>
</thead>
<tbody>
<tr data-sourcepos="3:1-3:5">
<td data-sourcepos="3:2-3:4">c</td>
<td data-sourcepos="3:5-3:4"></td>
</tr>
</tbody>
</table>
````````````````````````````````

## Interactions

`--gfm` enables tables, along with the other GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
| a |
| - |
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
````````````````````````````````

With [task lists](tasklist.html) and `--tasklist-in-table`, a cell whose only content is a task list item marker becomes a checkbox.  For details, see [Task lists in tables](tasklist.html#task-lists-in-tables).

With [spoilers](spoiler.html), the `||` delimiter doesn't separate cells:

```````````````````````````````` example -e table -e spoiler
| a | b |
| - | - |
| ||c|| | d |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td><span class="spoiler">c</span></td>
<td>d</td>
</tr>
</tbody>
</table>
````````````````````````````````

With [wikilinks](wikilinks.html), escape the pipe in a wikilink.  Otherwise, it separates cells:

```````````````````````````````` example -e table -e wikilinks-title-after-pipe
| a | b |
| - | - |
| [[c|d]] |
| [[c\|d]] |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>[[c</td>
<td>d]]</td>
</tr>
<tr>
<td><a href="c" data-wikilink="true">d</a></td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

The same applies to a pipe in [dollar math](math-dollars.html):

```````````````````````````````` example -e table -e math-dollars
| a | b |
| - | - |
| $c|d$ |
| $c\|d$ |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>$c</td>
<td>d$</td>
</tr>
<tr>
<td><span data-math-style="inline">c|d</span></td>
<td></td>
</tr>
</tbody>
</table>
````````````````````````````````

[Footnote](footnotes.html) references work in cells:

```````````````````````````````` example -e table -e footnotes
| a |
| - |
| b[^1] |

[^1]: c
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>b<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></td>
</tr>
</tbody>
</table>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>c <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

With [grid tables](grid-table.html) also enabled, both kinds of table work in the same document:

```````````````````````````````` example -e table -e grid-table
+---+
| a |
+---+

| b |
| - |
.
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
<table>
<thead>
<tr>
<th>b</th>
</tr>
</thead>
</table>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, uses the same table design as Comrak.  It doesn't support the other extensions in [Interactions](#interactions), so those examples aren't comparable.  On every other example above, the two implementations agree on HTML and XML output, except that cmark-gfm:

- treats a pipe after an escaped backslash as escaped, so `| c\\| d |` is one cell that contains `c| d` ([example 18](#example-18));
- treats a vertical tab (U+000B) or a form feed (U+000C) as whitespace around cells, including in delimiter rows;
- gives an [added cell] a source position in column 0 ([example 41](#example-41)); and,
- when a table interrupts a paragraph, gives the paragraph the source position `0:0-0:0`, places the header row on the paragraph's first line, and ignores the indentation of body rows ([example 48](#example-48), [example 49](#example-49)).

With `--width`, cmark-gfm's CommonMark output wraps cell content across lines, which breaks the table ([example 40](#example-40)).  cmark-gfm also has a `--table-prefer-style-attributes` option, which aligns cells with `style` attributes instead of `align` attributes; Comrak has no equivalent.

## Regressions

The following examples guard against regressions.

Source positions of rows indented differently from the header row ([#862](https://github.com/kivikakk/comrak/pull/862)):

```````````````````````````````` example -e table --sourcepos
p
      |a|b|
|-|-|
|c|d|
  |e|
#
.
<p data-sourcepos="1:1-1:1">p</p>
<table data-sourcepos="2:7-5:5">
<thead>
<tr data-sourcepos="2:7-2:11">
<th data-sourcepos="2:8-2:8">a</th>
<th data-sourcepos="2:10-2:10">b</th>
</tr>
</thead>
<tbody>
<tr data-sourcepos="4:1-4:5">
<td data-sourcepos="4:2-4:2">c</td>
<td data-sourcepos="4:4-4:4">d</td>
</tr>
<tr data-sourcepos="5:3-5:5">
<td data-sourcepos="5:4-5:4">e</td>
<td data-sourcepos="5:5-5:4"></td>
</tr>
</tbody>
</table>
<h1 data-sourcepos="6:1-6:1"></h1>
````````````````````````````````

Source positions of a preceding paragraph whose lines begin with spaces ([#651](https://github.com/kivikakk/comrak/pull/651)):

```````````````````````````````` example -e table --sourcepos
 123
  456
| a | b |
| - | - |
| c | d |
.
<p data-sourcepos="1:2-2:5">123
456</p>
<table data-sourcepos="3:1-5:9">
<thead>
<tr data-sourcepos="3:1-3:9">
<th data-sourcepos="3:2-3:4">a</th>
<th data-sourcepos="3:6-3:8">b</th>
</tr>
</thead>
<tbody>
<tr data-sourcepos="5:1-5:9">
<td data-sourcepos="5:2-5:4">c</td>
<td data-sourcepos="5:6-5:8">d</td>
</tr>
</tbody>
</table>
````````````````````````````````

[ATX heading]: https://spec.commonmark.org/0.31.2/#atx-heading
[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[blank line]: https://spec.commonmark.org/0.31.2/#blank-line
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[fenced code block]: https://spec.commonmark.org/0.31.2/#fenced-code-block
[HTML block]: https://spec.commonmark.org/0.31.2/#html-block
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[lazy continuation lines]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[paragraph continuation text]: https://spec.commonmark.org/0.31.2/#paragraph-continuation-text
[setext heading underline]: https://spec.commonmark.org/0.31.2/#setext-heading-underline
[spaces]: https://spec.commonmark.org/0.31.2/#space
[tabs]: https://spec.commonmark.org/0.31.2/#tab
[thematic break]: https://spec.commonmark.org/0.31.2/#thematic-breaks
