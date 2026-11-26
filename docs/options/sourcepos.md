---
title: Source positions
---

`--sourcepos` (`render.sourcepos`) adds the position of each node's Markdown source to the output.  HTML output gets a `data-sourcepos` attribute on each element, and XML output a `sourcepos` attribute on each node.  A program can use these to map rendered output back to the Markdown it came from.

Without the option, the output doesn't include positions:

```````````````````````````````` example
Hello *world*!
.
<p>Hello <em>world</em>!</p>
````````````````````````````````

With the option, the paragraph and the emphasis each have one:

```````````````````````````````` example --sourcepos
Hello *world*!
.
<p data-sourcepos="1:1-1:14">Hello <em data-sourcepos="1:7-1:13">world</em>!</p>
````````````````````````````````

## Source positions

A [source position](@) is a range of the source text, written as its start and end, separated by a hyphen.  Each is a [line number](@) and a [column](@), separated by a colon.  Both the start and the end are inclusive: in [example 2](#example-2), `1:7-1:13` runs from the 7th to the 13th column of line 1.

Line numbers count from 1.  Each [line ending] begins a new line, whether it's a line feed, a carriage return, or a carriage return followed by a line feed.  Line numbers count the whole input, including any [front matter](front-matter-delimiter.html):

```````````````````````````````` example --sourcepos --front-matter-delimiter ---
---
title: Example
---

*a*
.
<p data-sourcepos="5:1-5:3"><em data-sourcepos="5:1-5:3">a</em></p>
````````````````````````````````

A column is a byte offset into its line, counting from 1.  A character that takes several bytes in UTF-8 spans that many columns.  Here, `好` takes three:

```````````````````````````````` example --sourcepos
好 *a*
.
<p data-sourcepos="1:1-1:7">好 <em data-sourcepos="1:5-1:7">a</em></p>
````````````````````````````````

A [tab] is one column:

```````````````````````````````` example --sourcepos
a→*b*
.
<p data-sourcepos="1:1-1:5">a→<em data-sourcepos="1:3-1:5">b</em></p>
````````````````````````````````

Columns count the bytes of the input, not of the output.  A byte order mark at the start of the input takes three columns, and a NUL character takes one, though Comrak writes it as U+FFFD.  To count columns in characters instead, use [`--sourcepos-chars`](#source-positions-in-characters).

An [empty range](@) has its end one column before its start, on the same line.  Comrak assigns empty ranges to nodes that take up no source text, such as a table's [added cell](../extensions/table.html#added-cell).

## What a position covers

A block's [source position] starts at its first character, including any marker that begins it, such as a block quote's `>` or a list item's marker.  A block inside a container starts after the container's markers:

```````````````````````````````` example --sourcepos
> - > *a*
.
<blockquote data-sourcepos="1:1-1:9">
<ul data-sourcepos="1:3-1:9">
<li data-sourcepos="1:3-1:9">
<blockquote data-sourcepos="1:5-1:9">
<p data-sourcepos="1:7-1:9"><em data-sourcepos="1:7-1:9">a</em></p>
</blockquote>
</li>
</ul>
</blockquote>
````````````````````````````````

A block ends at the last character of its last line, including any trailing whitespace on that line.  An [ATX heading]'s position includes its closing sequence:

```````````````````````````````` example --sourcepos
## Foo ##→
.
<h2 data-sourcepos="1:1-1:10">Foo</h2>
````````````````````````````````

A container block ends where its last child ends.  Blank lines after a list item belong to neither the item nor list:

```````````````````````````````` example --sourcepos
- a

  b


c
.
<ul data-sourcepos="1:1-3:3">
<li data-sourcepos="1:1-3:3">
<p data-sourcepos="1:3-1:3">a</p>
<p data-sourcepos="3:3-3:3">b</p>
</li>
</ul>
<p data-sourcepos="6:1-6:1">c</p>
````````````````````````````````

A [lazy continuation line] extends its paragraph and the paragraph's containers.  The line's text starts at column 1:

```````````````````````````````` example --sourcepos
> a
b
.
<blockquote data-sourcepos="1:1-2:1">
<p data-sourcepos="1:3-2:1">a
b</p>
</blockquote>
````````````````````````````````

A [setext heading]'s position includes its underline, and a fenced code block's includes both of its fences:

```````````````````````````````` example --sourcepos
Foo
===

```
code
```
.
<h1 data-sourcepos="1:1-2:3">Foo</h1>
<pre class="syntax-highlighting" data-sourcepos="4:1-6:3"><code><span class="text plain">code
</span></code></pre>
````````````````````````````````

A fenced code block that is never closed ends at its last line:

```````````````````````````````` example --sourcepos
```
code
.
<pre class="syntax-highlighting" data-sourcepos="1:1-2:4"><code><span class="text plain">code
</span></code></pre>
````````````````````````````````

An [indented code block] starts after its indentation, and ends at the end of its last line that isn't blank:

```````````````````````````````` example --sourcepos
    a


b
.
<pre class="syntax-highlighting" data-sourcepos="1:5-1:5"><code><span class="text plain">a
</span></code></pre>
<p data-sourcepos="4:1-4:1">b</p>
````````````````````````````````

A [link reference definition] isn't a node, and so does not have a position.  Positions after it are unaffected:

```````````````````````````````` example --sourcepos
[a]: /url
b [a]
.
<p data-sourcepos="2:1-2:5">b <a data-sourcepos="2:3-2:5" href="/url">a</a></p>
````````````````````````````````

An inline's position includes its delimiters: emphasis's `*` or `_`, a code span's backticks, a link's brackets and destination, an image's `!`, and an [autolink]'s angle brackets:

```````````````````````````````` example --sourcepos
*a* **b** `c` [d](/u) ![e](/i.png) <https://f.g>
.
<p data-sourcepos="1:1-1:48"><em data-sourcepos="1:1-1:3">a</em> <strong data-sourcepos="1:5-1:9">b</strong> <code data-sourcepos="1:11-1:13">c</code> <a data-sourcepos="1:15-1:21" href="/u">d</a> <img data-sourcepos="1:23-1:34" src="/i.png" alt="e" /> <a data-sourcepos="1:36-1:48" href="https://f.g">https://f.g</a></p>
````````````````````````````````

An inline can span lines:

```````````````````````````````` example --sourcepos
[a
b](/u
"t")
.
<p data-sourcepos="1:1-3:4"><a data-sourcepos="1:1-3:4" href="/u" title="t">a
b</a></p>
````````````````````````````````

A text node covers its characters in the source.  An [entity reference] counts at its full length, and the text of a [backslash escape] starts after the backslash.  HTML output doesn't include elements for text; their positions appear only in XML output:

```````````````````````````````` example --sourcepos
\*a\* &amp; b
.
<p data-sourcepos="1:1-1:13">*a* &amp; b</p>
````````````````````````````````

A [soft line break] covers its line ending.  A [hard line break] covers its backslash, or its last two spaces, and its line ending:

```````````````````````````````` example --sourcepos
a
b\
c
.
<p data-sourcepos="1:1-3:1">a
b<br data-sourcepos="2:2-2:3" />
c</p>
````````````````````````````````

A carriage return followed by a line feed is one line ending; a break that ends with one covers both characters.

## Source positions in characters

`--sourcepos-chars` (`parse.sourcepos_chars`) counts [columns][column] in Unicode characters instead of bytes.  A character takes one column, however many bytes it takes in UTF-8:

```````````````````````````````` example --sourcepos --sourcepos-chars
好 *a*
.
<p data-sourcepos="1:1-1:5">好 <em data-sourcepos="1:3-1:5">a</em></p>
````````````````````````````````

The same applies to characters of every length:

```````````````````````````````` example --sourcepos --sourcepos-chars
ɽ**ɽ**`𐀀`
.
<p data-sourcepos="1:1-1:9">ɽ<strong data-sourcepos="1:2-1:6">ɽ</strong><code data-sourcepos="1:7-1:9">𐀀</code></p>
````````````````````````````````

A tab is still one column, and a carriage return followed by a line feed still two.  A byte order mark is one column.

The option changes the positions in the AST itself: a program that reads the AST gets character columns without `--sourcepos`.  On its own, the option doesn't change any output.

## Output

In HTML output, each element that Comrak writes for a node has a `data-sourcepos` attribute.  These don't:

* raw HTML, which Comrak writes as it is;
* text and soft line breaks, which don't have elements of their own (with [`--hardbreaks`](hardbreaks.html), a soft line break's `<br />` does have one); and,
* elements that Comrak adds around or beside a node's own element, such as the `<code>` inside a code block's `<pre>`, a table's `<thead>` and `<tbody>`, or an alert's title.

```````````````````````````````` example --sourcepos --unsafe
```rust
x
```

<div>
y
</div>

<b>z</b>
.
<pre class="syntax-highlighting" data-sourcepos="1:1-3:3"><code class="language-rust"><span class="source rust">x
</span></code></pre>
<div>
y
</div>
<p data-sourcepos="9:1-9:8"><b>z</b></p>
````````````````````````````````

In XML output, every node has a `sourcepos` attribute, including the document, text, and line breaks.

CommonMark output doesn't include source positions.

In the AST, each node's [`Ast`](https://docs.rs/comrak/latest/comrak/nodes/struct.Ast.html) has a `sourcepos` field.  It's a [`Sourcepos`](https://docs.rs/comrak/latest/comrak/nodes/struct.Sourcepos.html), whose `start` and `end` are each a [`LineColumn`](https://docs.rs/comrak/latest/comrak/nodes/struct.LineColumn.html).  Comrak computes positions whether or not `--sourcepos` is set; the option only adds them to the output.  A `Sourcepos` formats the same way as in the output:

```rust
let arena = comrak::Arena::new();
let root = comrak::parse_document(&arena, "Hello *world*!", &comrak::Options::default());
let emph = root.first_child().unwrap().children().nth(1).unwrap();
assert_eq!(emph.data().sourcepos.to_string(), "1:7-1:13");
```

A node that a program creates from a bare [`NodeValue`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html) starts with the position `0:0-0:0`.  Comrak doesn't write a position for a node whose start line is 0, in HTML or XML output.

## Interactions

Each extension's page describes the source positions of its own nodes:

* [alerts](../extensions/alerts.html#example-32), whose position covers all of their lines;
* [attributes](../extensions/attributes.html#example-33), whose blocks are part of their element's position;
* [autolinks](../extensions/autolink.html#example-33), whose position covers their text;
* [footnotes](../extensions/footnotes.html#example-27), whose references and definitions each have a position;
* [grid tables](../extensions/grid-table.html#example-44), whose cells cover their content, inside their borders;
* [maths](../extensions/math.html#example-64), whose position includes its delimiters;
* [strikethrough](../extensions/strikethrough.html#example-19), whose position includes its tildes;
* [tables](../extensions/table.html#example-41), whose [added cells](../extensions/table.html#added-cell) have an empty range;
* [task lists](../extensions/tasklist.html#example-55), whose paragraphs start after their markers; and,
* [wikilinks](../extensions/wikilinks.html#example-29), whose position includes their brackets.

With [GFM quirks](gfm-quirks.html#example-7), Comrak doesn't write a strong emphasis nested directly inside another, and so its position isn't written either.  The inner node still has a position in the AST.

With [escaped character spans](escaped-char-spans.html), each backslash escape is a span, and its position includes the backslash:

```````````````````````````````` example --sourcepos --escaped-char-spans
\*a
.
<p data-sourcepos="1:1-1:3"><span data-escaped-char data-sourcepos="1:1-1:2">*</span>a</p>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, has the same option.  The examples that use front matter, `--sourcepos-chars` or `--escaped-char-spans` aren't comparable, since cmark-gfm doesn't support them.

In HTML output, cmark-gfm writes source positions only on block elements.

In XML output, which includes inline positions too, the two implementations agree on examples [2](#example-2), [4](#example-4), [5](#example-5), [6](#example-6), [11](#example-11), [20](#example-20) and [22](#example-22).  On every other comparable example, cmark-gfm:

* ends some blocks at column 0 of the line after their last line: a [setext heading] ([example 10](#example-10)), a list and its last item when blank lines follow them ([example 8](#example-8)), an [indented code block] when blank lines follow it ([example 12](#example-12), [example 26](#example-26)), and an HTML block that starts and ends on one line, which ends at `0:0` ([example 24](#example-24));
* ends an [ATX heading] at its content, before its closing sequence ([example 7](#example-7));
* covers only a code span's content ([example 14](#example-14)), and miscounts its columns when its lines are indented ([example 27](#example-27));
* doesn't record positions for soft and hard line breaks ([example 9](#example-9), [example 17](#example-17));
* starts the text of a [backslash escape] at the backslash ([example 16](#example-16));
* puts text on the wrong line or column when it's on a [lazy continuation line] ([example 9](#example-9)), after a hard line break ([example 17](#example-17)), or after a code span that spans lines ([example 27](#example-27));
* starts a paragraph that follows a [link reference definition] on the definition's line ([example 13](#example-13));
* misplaces a link that spans lines onto its second line ([example 15](#example-15)), and ends a link on its first line when its title is on a later one ([example 25](#example-25));
* extends an unclosed fenced code block in a block quote to the end of the following paragraph ([example 23](#example-23)); and,
* counts a NUL character as the three bytes of U+FFFD.

Where cmark-gfm's positions for an extension's nodes differ from Comrak's, that extension's page says so.

## Regressions

The following examples guard against regressions.

A list before a fenced code block ends with its item ([#712](https://github.com/kivikakk/comrak/pull/712)):

```````````````````````````````` example --sourcepos
- list
```
code
```
.
<ul data-sourcepos="1:1-1:6">
<li data-sourcepos="1:1-1:6">list</li>
</ul>
<pre class="syntax-highlighting" data-sourcepos="2:1-4:3"><code><span class="text plain">code
</span></code></pre>
````````````````````````````````

An unclosed fenced code block in a block quote ends with the block quote ([#661](https://github.com/kivikakk/comrak/pull/661)):

```````````````````````````````` example --sourcepos
> ```
paragraph
.
<blockquote data-sourcepos="1:1-1:5">
<pre class="syntax-highlighting" data-sourcepos="1:3-1:5"><code></code></pre>
</blockquote>
<p data-sourcepos="2:1-2:9">paragraph</p>
````````````````````````````````

An HTML block that starts and ends on one line covers that line, as XML output shows ([#656](https://github.com/kivikakk/comrak/pull/656)):

```````````````````````````````` example --sourcepos --unsafe
<script></script>
.
<script></script>
````````````````````````````````

A link with a line break between its destination and its title ([#652](https://github.com/kivikakk/comrak/pull/652)):

```````````````````````````````` example --sourcepos
[link](   /uri
  "title"  )
.
<p data-sourcepos="1:1-2:12"><a data-sourcepos="1:1-2:12" href="/uri" title="title">link</a></p>
````````````````````````````````

An indented code block followed by blank lines ([#863](https://github.com/kivikakk/comrak/pull/863)):

```````````````````````````````` example --sourcepos
a

     b



next
.
<p data-sourcepos="1:1-1:1">a</p>
<pre class="syntax-highlighting" data-sourcepos="3:5-3:6"><code><span class="text plain"> b
</span></code></pre>
<p data-sourcepos="7:1-7:4">next</p>
````````````````````````````````

A code span that spans lines ([#855](https://github.com/kivikakk/comrak/pull/855)):

```````````````````````````````` example --sourcepos
Lead
  `a
  b` tail
.
<p data-sourcepos="1:1-3:9">Lead
<code data-sourcepos="2:3-3:4">a b</code> tail</p>
````````````````````````````````

[ATX heading]: https://spec.commonmark.org/0.31.2/#atx-headings
[autolink]: https://spec.commonmark.org/0.31.2/#autolinks
[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-break
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[line ending]: https://spec.commonmark.org/0.31.2/#line-ending
[link reference definition]: https://spec.commonmark.org/0.31.2/#link-reference-definition
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-heading
[soft line break]: https://spec.commonmark.org/0.31.2/#soft-line-breaks
[tab]: https://spec.commonmark.org/0.31.2/#tab
