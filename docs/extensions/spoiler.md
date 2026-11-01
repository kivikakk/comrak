---
title: Spoilers
---

```````````````````````````````` example -e spoiler
The ||dog dies at the end of Marley and Me||.
.
<p>The <span class="spoiler">dog dies at the end of Marley and Me</span>.</p>
````````````````````````````````

```````````````````````````````` example -e spoiler
|should not be spoiler|
||should be spoiler||
|||should be spoiler surrounded by pipes|||
.
<p>|should not be spoiler|
<span class="spoiler">should be spoiler</span>
|<span class="spoiler">should be spoiler surrounded by pipes</span>|</p>
````````````````````````````````

```````````````````````````````` example -e spoiler -e table
Text | Result
--- | ---
`||some clever text||` | ||some clever text||
.
<table>
<thead>
<tr>
<th>Text</th>
<th>Result</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>||some clever text||</code></td>
<td><span class="spoiler">some clever text</span></td>
</tr>
</tbody>
</table>
````````````````````````````````
