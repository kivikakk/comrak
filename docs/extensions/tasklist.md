---
title: Task lists
---

```````````````````````````````` example -e tasklist
* [x] Done
* [ ] Not done
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> Done</li>
<li><input type="checkbox" disabled="" /> Not done</li>
</ul>
````````````````````````````````

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

```````````````````````````````` example -e tasklist
1. [ ] Bird
2. [ ] McHale
3. [x] Parish
.
<ol>
<li><input type="checkbox" disabled="" /> Bird</li>
<li><input type="checkbox" disabled="" /> McHale</li>
<li><input type="checkbox" checked="" disabled="" /> Parish</li>
</ol>
````````````````````````````````

```````````````````````````````` example -e tasklist
- [x] item
  - [ ] item2
  - [x] item3
- [ ] item4
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> item
<ul>
<li><input type="checkbox" disabled="" /> item2</li>
<li><input type="checkbox" checked="" disabled="" /> item3</li>
</ul>
</li>
<li><input type="checkbox" disabled="" /> item4</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- [ ] bullet point one
- bullet point two and some extra text
- [x] bullet point three
.
<ul>
<li><input type="checkbox" disabled="" /> bullet point one</li>
<li>bullet point two and some extra text</li>
<li><input type="checkbox" checked="" disabled="" /> bullet point three</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
* [!] Red
.
<ul>
<li>[!] Red</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
* [!] Red
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> Red</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [あ] xy
  - [い] zw
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> xy
<ul>
<li><input type="checkbox" checked="" disabled="" /> zw</li>
</ul>
</li>
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
