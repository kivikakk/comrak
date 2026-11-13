---
title: Relaxed task list character
---

`--relaxed-tasklist-character` (`parse.relaxed_tasklist_matching`) lets a [task list item symbol](../extensions/tasklist.html#task-list-item-symbol) be any single character other than `]` or a line ending.  Only a space makes the item incomplete.  The option has no effect unless [task lists](../extensions/tasklist.html) are enabled.

For the full rules, see [Relaxed task list character](../extensions/tasklist.html#relaxed-task-list-character).

Without the option, only a space, `x`, or `X` is a valid symbol:

```````````````````````````````` example -e tasklist
- [ ] To do
- [x] Done
- [~] Deferred
- [-] Cancelled
.
<ul>
<li><input type="checkbox" disabled="" /> To do</li>
<li><input type="checkbox" checked="" disabled="" /> Done</li>
<li>[~] Deferred</li>
<li>[-] Cancelled</li>
</ul>
````````````````````````````````

With the option, other characters are valid too:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [ ] To do
- [x] Done
- [~] Deferred
- [-] Cancelled
.
<ul>
<li><input type="checkbox" disabled="" /> To do</li>
<li><input type="checkbox" checked="" disabled="" /> Done</li>
<li><input type="checkbox" checked="" disabled="" /> Deferred</li>
<li><input type="checkbox" checked="" disabled="" /> Cancelled</li>
</ul>
````````````````````````````````
