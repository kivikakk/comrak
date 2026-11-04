---
title: OFM block IDs
---

Obsidian block IDs let a whole block be linked to from elsewhere. Append `^identifier` to the end of a block's last line, where the identifier is one or more ASCII letters, digits, or hyphens preceded by whitespace; the marker is removed from the rendered text, and the block gets an `id` attribute that includes the caret. See [Link to a block in a note](https://obsidian.md/help/links#Link+to+a+block+in+a+note).

```````````````````````````````` example -e ofm-block-ids
Some paragraph text ^37066d
.
<p id="^37066d">Some paragraph text </p>
````````````````````````````````

```````````````````````````````` example -e ofm-block-ids
# Title ^hdr
.
<h1 id="^hdr">Title </h1>
````````````````````````````````

```````````````````````````````` example -e ofm-block-ids
> A blockquote ^inside
.
<blockquote id="^inside">
<p>A blockquote </p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e ofm-block-ids
- a ^item
- b
.
<ul>
<li id="^item">a </li>
<li>b</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e ofm-block-ids -e table
| A | B |
| - | - |
| x | y ^cell |
.
<table id="^cell">
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>x</td>
<td>y </td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e ofm-block-ids
one ^first
two ^second
.
<p id="^second">one ^first
two </p>
````````````````````````````````
