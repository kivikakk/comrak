---
title: Source positions
---

```````````````````````````````` example --sourcepos
Hello *world*!
.
<p data-sourcepos="1:1-1:14">Hello <em data-sourcepos="1:7-1:13">world</em>!</p>
````````````````````````````````

```````````````````````````````` example
Hello *world*!
.
<p>Hello <em>world</em>!</p>
````````````````````````````````

```````````````````````````````` example --sourcepos
# Hello

World *with* **emphasis**
and `code`.

> - one
> - two
.
<h1 data-sourcepos="1:1-1:7">Hello</h1>
<p data-sourcepos="3:1-4:11">World <em data-sourcepos="3:7-3:12">with</em> <strong data-sourcepos="3:14-3:25">emphasis</strong>
and <code data-sourcepos="4:5-4:10">code</code>.</p>
<blockquote data-sourcepos="6:1-7:7">
<ul data-sourcepos="6:3-7:7">
<li data-sourcepos="6:3-6:7">one</li>
<li data-sourcepos="7:3-7:7">two</li>
</ul>
</blockquote>
````````````````````````````````

```````````````````````````````` example --sourcepos
[link](https://example.com) and ![image](x.png)
.
<p data-sourcepos="1:1-1:47"><a data-sourcepos="1:1-1:27" href="https://example.com">link</a> and <img data-sourcepos="1:33-1:47" src="x.png" alt="image" /></p>
````````````````````````````````
