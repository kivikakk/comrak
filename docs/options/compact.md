---
title: Compact HTML
---

`--compact` (`render.compact_html`) leaves out the newlines Comrak writes to lay out its HTML output, so that blocks no longer start on lines of their own.

By default, each block starts on a new line:

```````````````````````````````` example
# Hello

World.
.
<h1>Hello</h1>
<p>World.</p>
````````````````````````````````

With the option, the blocks run together:

```````````````````````````````` example --compact
# Hello

World.
.
<h1>Hello</h1><p>World.</p>
````````````````````````````````

The option only changes how Comrak writes HTML.  CommonMark and XML output don't change.  [cmark-gfm](https://github.com/github/cmark-gfm) has no equivalent option.  Comrak added `--compact` in 0.51.0 ([#769](https://github.com/kivikakk/comrak/pull/769)), in answer to [#768](https://github.com/kivikakk/comrak/issues/768).

## Output

A [layout newline](@) is a line ending that Comrak writes between HTML elements, or between an element's tags and its content, to put a block on a line of its own.  Compact output leaves out every layout newline, however deeply the blocks are nested:

```````````````````````````````` example --compact
> - one
>
> - two
.
<blockquote><ul><li><p>one</p></li><li><p>two</p></li></ul></blockquote>
````````````````````````````````

Tables are written the same way:

```````````````````````````````` example --compact -e table
| a | b |
| --- | --- |
| c | d |
.
<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>c</td><td>d</td></tr></tbody></table>
````````````````````````````````

A hard line break is written as `<br />`, without the layout newline that otherwise follows it:

```````````````````````````````` example --compact
Hello\
world.
.
<p>Hello<br />world.</p>
````````````````````````````````

Newlines that are part of the content remain, so compact output can still span several lines.  A soft line break is written as a newline, as usual:

```````````````````````````````` example --compact
Hello
world.
.
<p>Hello
world.</p>
````````````````````````````````

A code block's content keeps its newlines:

```````````````````````````````` example --compact
```
first
second
```

World.
.
<pre class="syntax-highlighting"><code><span class="text plain">first
second
</span></code></pre><p>World.</p>
````````````````````````````````

With `--unsafe`, raw HTML is written as it appears in the input, newlines included:

```````````````````````````````` example --compact --unsafe
<div>
Hello
</div>

World.
.
<div>
Hello
</div>
<p>World.</p>
````````````````````````````````

So is the source of a [block maths](../extensions/math.html#block-maths) element:

```````````````````````````````` example --compact -e math-dollars -e math-dollars-block
$$
x = y
$$
.
<div data-math-style="display">x = y
</div>
````````````````````````````````

Compact output doesn't end with a newline.

Comrak writes the paragraphs in a [tight] list item or a [tight description](../extensions/description-lists.html#tight) without `<p>` elements.  When an [HTML block] comes before or after such a paragraph, the newline between them is the only whitespace separating the paragraph's text from the HTML, and compact output keeps it.  Without that newline, the two paragraphs in this item would run together as "ab":

```````````````````````````````` example --compact
- a
  <!-- c -->
  b
.
<ul><li>a
<!-- raw HTML omitted -->
b</li></ul>
````````````````````````````````

The same applies to a [Phoenix HEEx](../extensions/phoenix-heex.html) block.

## Interactions

With [`--unsafe`](unsafe.html), the HTML block is written as it is.  With [`--escape`](escape.html), it's written as text.  Either way, the newlines on both sides of it remain:

```````````````````````````````` example --compact --unsafe
- a
  <!-- c -->
  b
.
<ul><li>a
<!-- c -->
b</li></ul>
````````````````````````````````

```````````````````````````````` example --compact --escape
- a
  <!-- c -->
  b
.
<ul><li>a
&lt;!-- c --&gt;
b</li></ul>
````````````````````````````````

With [`--hardbreaks`](hardbreaks.html), every soft line break is written as a hard line break: `<br />`, without a newline:

```````````````````````````````` example --compact --hardbreaks
Hello
world.

- one
- two
.
<p>Hello<br />world.</p><ul><li>one</li><li>two</li></ul>
````````````````````````````````

[HTML block]: https://spec.commonmark.org/0.31.2/#html-blocks
[tight]: https://spec.commonmark.org/0.31.2/#tight
