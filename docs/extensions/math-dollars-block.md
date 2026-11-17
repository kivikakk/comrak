---
title: Block-level dollar math
---

The `math-dollars-block` extension parses standalone `$$` lines as a
block-level math span.  It requires [dollar math](math-dollars.html)
(`math-dollars`) to also be enabled.

Everything between the delimiters is literal math, including blank lines and
Markdown syntax, so a line containing only `=` is not mistaken for a setext
heading underline.

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$
1
=
2
$$
.
<div data-math-style="display">
1
=
2
</div>
````````````````````````````````

The opening `$$` may be indented by up to three spaces relative to its
container, and may interrupt a paragraph.  The opening and closing lines must
contain only `$$` and optional surrounding whitespace; a longer dollar run or
a delimiter followed by text is not a block delimiter.

```````````````````````````````` example -e math-dollars -e math-dollars-block
before
$$
# not a heading

- not a list
$$
after
.
<p>before</p>
<div data-math-style="display">
# not a heading

- not a list
</div>
<p>after</p>
````````````````````````````````

An unmatched opening fence continues until its enclosing container ends or the
document ends.  A closing fence cannot be continued lazily out of a block
quote or list item.

```````````````````````````````` example -e math-dollars -e math-dollars-block
> $$
> 1
> =
> 2
.
<blockquote>
<div data-math-style="display">
1
=
2
</div>
</blockquote>
````````````````````````````````
