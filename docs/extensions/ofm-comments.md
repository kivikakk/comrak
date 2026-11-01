---
title: OFM comments
---

```````````````````````````````` example -e ofm-comments
This is an %%inline%% comment.

%%
This is a block comment.

Block comments can span multiple lines.
%%
.
<p>This is an  comment.</p>
````````````````````````````````

```````````````````````````````` example -e ofm-comments
%%
# aa

- bb
%%tail
.
<p>tail</p>
````````````````````````````````

```````````````````````````````` example -e ofm-comments
x %%aa
bb%% tail
.
<p>x %%aa
bb%% tail</p>
````````````````````````````````

```````````````````````````````` example -e ofm-comments
%%
aa

bb
.
````````````````````````````````

```````````````````````````````` example -e ofm-comments
`%%inline%%`

```
%%
# text
%%
```
.
<p><code>%%inline%%</code></p>
<pre><code>%%
# text
%%
</code></pre>
````````````````````````````````

```````````````````````````````` example -e ofm-comments -e table
a %%hidden%% | b
--- | ---
x %%hidden%% y | z
.
<table>
<thead>
<tr>
<th>a </th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>x  y</td>
<td>z</td>
</tr>
</tbody>
</table>
````````````````````````````````
