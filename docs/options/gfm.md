---
title: GitHub Flavored Markdown
---

```````````````````````````````` example
Hello ~world~ there.

Hello www.github.com.

* [x] Done
* [ ] Not done

****abcd****
.
<p>Hello ~world~ there.</p>
<p>Hello www.github.com.</p>
<ul>
<li>[x] Done</li>
<li>[ ] Not done</li>
</ul>
<p><strong><strong>abcd</strong></strong></p>
````````````````````````````````

```````````````````````````````` example --gfm
Hello ~world~ there.

Hello www.github.com.

* [x] Done
* [ ] Not done

****abcd****
.
<p>Hello <del>world</del> there.</p>
<p>Hello <a href="http://www.github.com">www.github.com</a>.</p>
<ul>
<li><input type="checkbox" checked="" disabled="" /> Done</li>
<li><input type="checkbox" disabled="" /> Not done</li>
</ul>
<p><strong>abcd</strong></p>
````````````````````````````````

```````````````````````````````` example
| a | b |
|---|---|
| c | d |
.
<p>| a | b |
|---|---|
| c | d |</p>
````````````````````````````````

```````````````````````````````` example --gfm
| a | b |
|---|---|
| c | d |
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
<td>c</td>
<td>d</td>
</tr>
</tbody>
</table>
````````````````````````````````
