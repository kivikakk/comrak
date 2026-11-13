---
title: Task list classes
---

`--tasklist-classes` (`render.tasklist_classes`) adds these styling classes to the HTML output of [task lists](../extensions/tasklist.html):

- `contains-task-list` on each task list
- `task-list-item` on each task list item
- `task-list-item-checkbox` on each checkbox

Other lists and list items get no classes.  The option has no effect unless task lists are enabled.

For the full rules, see [Task list classes](../extensions/tasklist.html#task-list-classes).

Without the option, the output has no classes:

```````````````````````````````` example -e tasklist
- [ ] To do
- [x] Done
- Not a task
.
<ul>
<li><input type="checkbox" disabled="" /> To do</li>
<li><input type="checkbox" checked="" disabled="" /> Done</li>
<li>Not a task</li>
</ul>
````````````````````````````````

With the option, the task list, its task list items, and their checkboxes have classes:

```````````````````````````````` example -e tasklist --tasklist-classes
- [ ] To do
- [x] Done
- Not a task
.
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> To do</li>
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> Done</li>
<li>Not a task</li>
</ul>
````````````````````````````````
