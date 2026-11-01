---
title: Compact HTML
---

```````````````````````````````` example --compact
# Hello

World.
.
<h1>Hello</h1><p>World.</p>
````````````````````````````````

```````````````````````````````` example
# Hello

World.
.
<h1>Hello</h1>
<p>World.</p>
````````````````````````````````

```````````````````````````````` example --compact
- one
- two
- three
.
<ul><li>one</li><li>two</li><li>three</li></ul>
````````````````````````````````

```````````````````````````````` example --compact
> quoted
.
<blockquote><p>quoted</p></blockquote>
````````````````````````````````

```````````````````````````````` example --compact -e table
| a | b |
|---|---|
| c | d |
.
<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>c</td><td>d</td></tr></tbody></table>
````````````````````````````````
