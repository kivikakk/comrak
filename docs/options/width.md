---
title: CommonMark wrap width
---

```````````````````````````````` example
hello hello hello hello hello hello
.
<p>hello hello hello hello hello hello</p>
````````````````````````````````

```````````````````````````````` example --width 20
hello hello hello hello hello hello
.
<p>hello hello hello hello hello hello</p>
````````````````````````````````

```````````````````````````````` example --width 72
this should break because it has breakable characters. break right here newline

don't break

a-long-line-that-won't-break-because-there-is-no-character-it-can-break-on
.
<p>this should break because it has breakable characters. break right here newline</p>
<p>don't break</p>
<p>a-long-line-that-won't-break-because-there-is-no-character-it-can-break-on</p>
````````````````````````````````

```````````````````````````````` example --width 80
This is a
paragraph with
short lines.
.
<p>This is a
paragraph with
short lines.</p>
````````````````````````````````

```````````````````````````````` example --width 20
- a list item that goes on and on and on for quite a while

> a block quote that also goes on and on for quite a while
.
<ul>
<li>a list item that goes on and on and on for quite a while</li>
</ul>
<blockquote>
<p>a block quote that also goes on and on for quite a while</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e table --width 80
| option | description |
| --- | --- |
| -o | Write output to FILE instead of stdout |
| --gfm | Use GFM-style quirks in output HTML, such as not nesting <strong> tags, which otherwise breaks CommonMark compatibility |
.
<table>
<thead>
<tr>
<th>option</th>
<th>description</th>
</tr>
</thead>
<tbody>
<tr>
<td>-o</td>
<td>Write output to FILE instead of stdout</td>
</tr>
<tr>
<td>--gfm</td>
<td>Use GFM-style quirks in output HTML, such as not nesting <!-- raw HTML omitted --> tags, which otherwise breaks CommonMark compatibility</td>
</tr>
</tbody>
</table>
````````````````````````````````
