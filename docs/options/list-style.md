---
title: CommonMark list style
---

```````````````````````````````` example
- one
- two
- three
.
<ul>
<li>one</li>
<li>two</li>
<li>three</li>
</ul>
````````````````````````````````

```````````````````````````````` example --list-style plus
- one
- two
- three
.
<ul>
<li>one</li>
<li>two</li>
<li>three</li>
</ul>
````````````````````````````````

```````````````````````````````` example --list-style star
- one
- two
- three
.
<ul>
<li>one</li>
<li>two</li>
<li>three</li>
</ul>
````````````````````````````````

```````````````````````````````` example --list-style dash
+ a
.
<ul>
<li>a</li>
</ul>
````````````````````````````````

```````````````````````````````` example --list-style plus
* one
  * two
    1. three
    2. four
.
<ul>
<li>one
<ul>
<li>two
<ol>
<li>three</li>
<li>four</li>
</ol>
</li>
</ul>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example --list-style plus
* a

+ b

- c
.
<ul>
<li>a</li>
</ul>
<ul>
<li>b</li>
</ul>
<ul>
<li>c</li>
</ul>
````````````````````````````````
