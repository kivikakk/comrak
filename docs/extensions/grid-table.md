---
title: Grid tables
---

```````````````````````````````` example -e grid-table
+---+---+
| a | b |
+===+===+
| c | d |
+---+---+
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

```````````````````````````````` example -e grid-table
+---+---+---+
| AAAAA | B |
+---+---+ B +
| D | E | B |
+ D +---+---+
| D | CCCCC |
+---+---+---+
.
<table>
<tbody>
<tr>
<td colspan="2">AAAAA</td>
<td rowspan="2">B
B
B</td>
</tr>
<tr>
<td rowspan="2">D
D
D</td>
<td>E</td>
</tr>
<tr>
<td colspan="2">CCCCC</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e grid-table
+-------+-------+-------+
| Left  | Right | Centre|
+:======+======:+:=====:+
| a     | b     | c     |
+=======+=======+=======+
| foot  | foot  | foot  |
+=======+=======+=======+
.
<table>
<thead>
<tr>
<th align="left">Left</th>
<th align="right">Right</th>
<th align="center">Centre</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">a</td>
<td align="right">b</td>
<td align="center">c</td>
</tr>
</tbody>
<tfoot>
<tr>
<td align="left">foot</td>
<td align="right">foot</td>
<td align="center">foot</td>
</tr>
</tfoot>
</table>
````````````````````````````````

```````````````````````````````` example -e grid-table
+:------+-------:+
| a     | b      |
+-------+--------+
.
<table>
<tbody>
<tr>
<td align="left">a</td>
<td align="right">b</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e grid-table
+----------+----------------+
| Fruit    | Advantages     |
+==========+================+
| Bananas  | - built-in     |
|          |   wrapper      |
|          | - bright color |
+----------+----------------+
| Oranges  | ```            |
|          | cures scurvy   |
|          | ```            |
|          |                |
|          | Tasty.         |
+----------+----------------+
.
<table>
<thead>
<tr>
<th>Fruit</th>
<th>Advantages</th>
</tr>
</thead>
<tbody>
<tr>
<td>Bananas</td>
<td>
<ul>
<li>built-in
wrapper</li>
<li>bright color</li>
</ul>
</td>
</tr>
<tr>
<td>Oranges</td>
<td>
<pre class="syntax-highlighting"><code><span class="text plain">cures scurvy
</span></code></pre>
<p>Tasty.</p>
</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e grid-table
- Inside a list item:

  +---+
  | a |
  +---+

> +---+
> | b |
> +---+
.
<ul>
<li>
<p>Inside a list item:</p>
<table>
<tbody>
<tr>
<td>a</td>
</tr>
</tbody>
</table>
</li>
</ul>
<blockquote>
<table>
<tbody>
<tr>
<td>b</td>
</tr>
</tbody>
</table>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e grid-table
+---+
| a |
+ b +
.
<p>+---+
| a |</p>
<ul>
<li>b +</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e grid-table
Not a table:
+---+
| a |
+---+
.
<p>Not a table:
+---+
| a |
+---+</p>
````````````````````````````````
