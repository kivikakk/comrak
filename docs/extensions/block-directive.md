---
title: Block directives
---

```````````````````````````````` example -e block-directive
:::warning
A paragraph.

- item one
- item two
:::
.
<div class="warning">
<p>A paragraph.</p>
<ul>
<li>item one</li>
<li>item two</li>
</ul>
</div>
````````````````````````````````

```````````````````````````````` example -e block-directive
:::foo bar
Paragraph 1

Paragraph 2
:::
.
<div class="foo bar">
<p>Paragraph 1</p>
<p>Paragraph 2</p>
</div>
````````````````````````````````

```````````````````````````````` example -e block-directive
:::foo bar
> a
:::
.
<div class="foo bar">
<blockquote>
<p>a</p>
</blockquote>
</div>
````````````````````````````````

```````````````````````````````` example -e block-directive
> :::foo bar
> a
> :::
.
<blockquote>
<div class="foo bar">
<p>a</p>
</div>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e block-directive
- item one

  :::foo bar
  Paragraph 1

  Paragraph 2
  :::
- item two
.
<ul>
<li>
<p>item one</p>
<div class="foo bar">
<p>Paragraph 1</p>
<p>Paragraph 2</p>
</div>
</li>
<li>
<p>item two</p>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e block-directive
::::outer
:::inner
Paragraph
:::
::::
.
<div class="outer">
<div class="inner">
<p>Paragraph</p>
</div>
</div>
````````````````````````````````
