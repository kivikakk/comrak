---
title: Grid table widths
---

`--grid-table-widths` (`render.grid_table_widths`) adds a `<colgroup>` element to each [grid table](../extensions/grid-table.html) in HTML output.  It gives each column's width from the source as a percentage of the table's total.  The option has no effect unless grid tables are enabled.

For the full rules, see [Grid table widths](../extensions/grid-table.html#grid-table-widths).

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

With the option, each column has a `<col>` element:

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
