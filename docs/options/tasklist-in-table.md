---
title: Task lists in tables
---

`--tasklist-in-table` (`parse.tasklist_in_table`) turns a [table](../extensions/table.html) cell into a checkbox when the cell's entire content is a [task list item marker](../extensions/tasklist.html#task-list-item-marker).  The option has no effect unless both tables and [task lists](../extensions/tasklist.html) are enabled.

For the full rules, see [Task lists in tables](../extensions/tasklist.html#task-lists-in-tables).

Without the option, such cells stay as text:

```````````````````````````````` example -e table -e tasklist
|     | name |
| --- | ---- |
| [ ] | Rell |
| [x] | Kai  |
.
<table>
<thead>
<tr>
<th></th>
<th>name</th>
</tr>
</thead>
<tbody>
<tr>
<td>[ ]</td>
<td>Rell</td>
</tr>
<tr>
<td>[x]</td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````

With the option, they become checkboxes:

```````````````````````````````` example -e table -e tasklist --tasklist-in-table
|     | name |
| --- | ---- |
| [ ] | Rell |
| [x] | Kai  |
.
<table>
<thead>
<tr>
<th></th>
<th>name</th>
</tr>
</thead>
<tbody>
<tr>
<td><input type="checkbox" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td><input type="checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````
