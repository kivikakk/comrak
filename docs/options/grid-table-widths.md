---
title: Grid table widths
---

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
