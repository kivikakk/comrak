---
title: Tasklists in tables
---

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
<td>
<input type="checkbox" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td>
<input type="checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e table -e tasklist --tasklist-in-table --relaxed-tasklist-character
|         | name |
| ------- | ---- |
| [~]     | Rell |
| [x]     | Kai  |
| [x] No. | Ez   |
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
<td>
<input type="checkbox" checked="" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td>
<input type="checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
<tr>
<td>[x] No.</td>
<td>Ez</td>
</tr>
</tbody>
</table>
````````````````````````````````
