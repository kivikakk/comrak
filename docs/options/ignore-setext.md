---
title: Ignore setext headings
---

```````````````````````````````` example
setext heading
---
.
<h2>setext heading</h2>
````````````````````````````````

```````````````````````````````` example --ignore-setext
setext heading
---
.
<p>setext heading</p>
<hr />
````````````````````````````````

```````````````````````````````` example
Heading
=======
.
<h1>Heading</h1>
````````````````````````````````

```````````````````````````````` example --ignore-setext
Heading
=======
.
<p>Heading
=======</p>
````````````````````````````````

```````````````````````````````` example --ignore-setext
Foo *bar
baz*
====
.
<p>Foo <em>bar
baz</em>
====</p>
````````````````````````````````
