---
title: Hard line breaks
---

`--hardbreaks` (`render.hardbreaks`) writes every [soft line break] as a [hard line break].  Use it for text whose authors expect each new line to appear in the rendered output, as it does in a plain text editor.

By default, a soft line break is written as a line ending, which a browser displays as a space:

```````````````````````````````` example
Hello.
World.
.
<p>Hello.
World.</p>
````````````````````````````````

With the option, it's written as `<br />`, followed by a line ending:

```````````````````````````````` example --hardbreaks
Hello.
World.
.
<p>Hello.<br />
World.</p>
````````````````````````````````

Comrak has had `--hardbreaks` since its first release, 0.1.0.

## Output

Hard line breaks don't change.  With the option, both kinds of line break are written as `<br />`:

```````````````````````````````` example --hardbreaks
a\
b
c
.
<p>a<br />
b<br />
c</p>
````````````````````````````````

The option applies to the inline content of any block.  That includes a [setext heading]:

```````````````````````````````` example --hardbreaks
a
b
===
.
<h1>a<br />
b</h1>
````````````````````````````````

The same goes for inline content nested in other inlines, like emphasis or link text:

```````````````````````````````` example --hardbreaks
*a
b* [c
d](/u)
.
<p><em>a<br />
b</em> <a href="/u">c<br />
d</a></p>
````````````````````````````````

A [lazy continuation line] continues its paragraph, so the line ending before it is a soft line break too:

```````````````````````````````` example --hardbreaks
> Roses are red,
violets are blue.
.
<blockquote>
<p>Roses are red,<br />
violets are blue.</p>
</blockquote>
````````````````````````````````

The line ending at the end of a block isn't a soft line break, and doesn't change.

Line endings inside some inlines aren't soft line breaks either.  A [code span] turns each of its line endings into a space:

```````````````````````````````` example --hardbreaks
`a
b`
.
<p><code>a b</code></p>
````````````````````````````````

[Raw HTML] is written as it appears in the input, line endings included:

```````````````````````````````` example --hardbreaks --unsafe
a <span
class="x">b</span>
.
<p>a <span
class="x">b</span></p>
````````````````````````````````

An [image description] is written as plain text in the `alt` attribute, and a soft line break in it is written as a space:

```````````````````````````````` example --hardbreaks
![a
b](/u)
.
<p><img src="/u" alt="a b" /></p>
````````````````````````````````

CommonMark output writes every soft and hard line break as a backslash followed by a line ending.  The output then means the same whether it's read with the option or not.  It's not wrapped, even with [`--width`](width.html), because a line ending that wrapping adds would become a hard line break if the output were read with the option.

XML output doesn't change.  It writes `softbreak` and `linebreak` elements, as usual.  The AST doesn't change either: a soft line break is still a `NodeValue::SoftBreak`.

## Interactions

With [`--sourcepos`](sourcepos.html), the `<br />` written for a soft line break has a `data-sourcepos` attribute, which covers its line ending:

```````````````````````````````` example --hardbreaks --sourcepos
a
b
.
<p data-sourcepos="1:1-2:1">a<br data-sourcepos="1:2-1:2" />
b</p>
````````````````````````````````

With [`--compact`](compact.html), the `<br />` isn't followed by a line ending ([example 13 on that page](compact.html#example-13)).

With [task lists](../extensions/tasklist.html), a [task list item marker](../extensions/tasklist.html#task-list-item-marker) may be followed by a line ending.  The line ending is a soft line break, and is written as `<br />` after the checkbox:

```````````````````````````````` example --hardbreaks -e tasklist
- [ ]
  a
.
<ul>
<li><input type="checkbox" disabled="" /> <br />
a</li>
</ul>
````````````````````````````````

With [wikilinks](../extensions/wikilinks.html), a title is plain text.  A line ending in a title isn't a soft line break, and is written unchanged:

```````````````````````````````` example --hardbreaks -e wikilinks-title-after-pipe
[[a|b
c]]
.
<p><a href="a" data-wikilink="true">b
c</a></p>
````````````````````````````````

With [maths](../extensions/math.html), the content of a maths span is written unchanged, line endings included:

```````````````````````````````` example --hardbreaks -e math-dollars
$a
b$
.
<p><span data-math-style="inline">a
b</span></p>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, has the same option.  It doesn't support wikilinks or maths, so those examples aren't comparable.  On every other example above, the two implementations have the same HTML output, except in these cases:

* cmark-gfm doesn't write `data-sourcepos` on any `<br />` ([example 10](#example-10)).
* cmark-gfm doesn't recognise a task list item marker followed by a line ending ([example 11](#example-11)); see [Differences from cmark-gfm](../extensions/tasklist.html#differences-from-cmark-gfm) on the task lists page.

cmark-gfm's CommonMark output writes a soft line break as two spaces and a line ending, but a hard line break as a bare line ending.  Read without the option, the output loses every hard line break, and read with it, two consecutive hard line breaks become one.

cmark-gfm also offers `--nobreaks`, which writes every soft line break as a space.  Comrak doesn't have an equivalent.

[code span]: https://spec.commonmark.org/0.31.2/#code-spans
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-breaks
[image description]: https://spec.commonmark.org/0.31.2/#image-description
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[raw HTML]: https://spec.commonmark.org/0.31.2/#raw-html
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-heading
[soft line break]: https://spec.commonmark.org/0.31.2/#soft-line-breaks
