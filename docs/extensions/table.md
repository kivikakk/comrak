---
title: Tables
---

```````````````````````````````` example -e table
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

```````````````````````````````` example -e table
| a | b |
|---|:-:|
| c | d |
.
<table>
<thead>
<tr>
<th>a</th>
<th align="center">b</th>
</tr>
</thead>
<tbody>
<tr>
<td>c</td>
<td align="center">d</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e table
|x|b|x|x|e|
|:-|:-:|-|-:|:-:|
|f|x|x|i|
.
<table>
<thead>
<tr>
<th align="left">x</th>
<th align="center">b</th>
<th>x</th>
<th align="right">x</th>
<th align="center">e</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">f</td>
<td align="center">x</td>
<td>x</td>
<td align="right">i</td>
<td align="center"></td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e table
|a|b|
|-|-|
|c|d
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

```````````````````````````````` example -e table
| a |
| - |

ok
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<p>ok</p>
````````````````````````````````

```````````````````````````````` example -e table
123
456
| a | b |
| ---| --- |
d | e
.
<p>123
456</p>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>d</td>
<td>e</td>
</tr>
</tbody>
</table>
````````````````````````````````

```````````````````````````````` example -e table
- |a|b|
  |-|-|
  |c|d|
.
<ul>
<li>
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
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e table
> |a|b|
> |-|-|
> |c|d|
.
<blockquote>
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
</blockquote>
````````````````````````````````
