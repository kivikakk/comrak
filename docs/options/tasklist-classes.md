---
title: Tasklist classes
---

```````````````````````````````` example -e tasklist
- [ ] List item 1
- [ ] This list item is **bold**
- [x] There is some `code` here
.
<ul>
<li><input type="checkbox" disabled="" /> List item 1</li>
<li><input type="checkbox" disabled="" /> This list item is <strong>bold</strong></li>
<li><input type="checkbox" checked="" disabled="" /> There is some <code>code</code> here</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --tasklist-classes
- [ ] List item 1
- [ ] This list item is **bold**
- [x] There is some `code` here
.
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> List item 1</li>
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> This list item is <strong>bold</strong></li>
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> There is some <code>code</code> here</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --tasklist-classes
- [ ] bullet point one
- bullet point two and some extra text
- [x] bullet point three
.
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> bullet point one</li>
<li>bullet point two and some extra text</li>
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> bullet point three</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --tasklist-classes
* [ ] Red
  * [x] Green
    * [ ] Blue
.
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> Red
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> Green
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> Blue</li>
</ul>
</li>
</ul>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e table -e tasklist --tasklist-in-table --tasklist-classes
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
<td><input type="checkbox" class="task-list-item-checkbox" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````
