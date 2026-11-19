---
title: Attributes
extensions: [header-attributes, fenced-code-attributes, inline-code-attributes, link-attributes]
---

Attributes let you set an element's `id`, its classes, and any other HTML attribute from Markdown, by writing them in braces after the element.  Use attributes to give a heading an `id` that stays put when its text changes, to size an image, or to mark a code block for a script or stylesheet to find.

Comrak's attributes follow the [attributes extension](https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/attributes.md) of commonmark-hs, which [Pandoc](https://pandoc.org/MANUAL.html#extension-attributes) also uses.  Comrak supports attributes on four kinds of element, each with its own extension: headings, fenced code blocks, code spans, and links and images.  This document describes all four, and [Differences from commonmark-hs](#differences-from-commonmark-hs) lists where Comrak parts ways.

To enable attributes on headings, pass `-e header-attributes` on the command line, or set `extension.header_attributes` in the options struct.  The other three are `-e fenced-code-attributes` (`extension.fenced_code_attributes`), `-e inline-code-attributes` (`extension.inline_code_attributes`), and `-e link-attributes` (`extension.link_attributes`).  `--gfm` doesn't enable any of them.

The extensions only parse attributes into the AST.  HTML output includes them only when you also pass both `--attributes` and `--unsafe`, as described in [Render attributes](#render-attributes).  Any attribute can be turned against the page it's on — `onclick` runs script, and even an `id` or a class can interfere with the page's own scripts and styles — so if you render untrusted input with attributes, sanitise the result.

Here's a typical use of attributes:

```````````````````````````````` example -e header-attributes -e inline-code-attributes -e link-attributes --attributes --unsafe
# Installation {#install}

Run `cargo install comrak`{.shell}, then read [the docs](https://comrak.ee){target=_blank}.
.
<h1 id="install">Installation</h1>
<p>Run <code class="shell">cargo install comrak</code>, then read <a href="https://comrak.ee" target="_blank">the docs</a>.</p>
````````````````````````````````

## Syntax

An [attribute block](@) is `{`, followed by any number of [attributes][attribute], followed by `}`.  Whitespace — spaces, tabs, and line endings — separates attributes, and may also follow `{` and precede `}`.

An [attribute](@) is one of:

* an [id attribute](@), which is `#` followed by a [value];
* a [class attribute](@), which is `.` followed by a value; or,
* a [key–value attribute](@), which is a [key], `=`, and a value.

The examples in this section put attribute blocks on code spans; [Code spans](#code-spans) describes where an attribute block goes.  Here's an attribute block with one attribute of each kind:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`hello`{#greeting .code lang=en}
.
<p><code id="greeting" class="code" lang="en">hello</code></p>
````````````````````````````````

An attribute block may be empty, and may span lines:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{} `b`{ #x
.y
k=v }
.
<p><code>a</code> <code id="x" class="y" k="v">b</code></p>
````````````````````````````````

Text in braces that doesn't follow these rules isn't an attribute block, and remains text.  None of these are attribute blocks:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#} `b`{key} `c`{key=} `d`{key = value} `e`{.x,.y}
.
<p><code>a</code>{#} <code>b</code>{key} <code>c</code>{key=} <code>d</code>{key = value} <code>e</code>{.x,.y}</p>
````````````````````````````````

### Values

A [value](@) is a [bare value] or a [quoted value].

In an id or class attribute, a [bare value](@) is one or more ASCII letters, digits, `-`, `_`, `:`, `.`, or `%`.  In a key–value attribute, a bare value may be any characters other than whitespace, `"`, `'`, `=`, `<`, `>`, `` ` ``, and `}`:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#top .x-1 data-src=../img/cat.png}
.
<p><code id="top" class="x-1" data-src="../img/cat.png">a</code></p>
````````````````````````````````

A [quoted value](@) is `"`, followed by any characters, followed by `"`.  A quoted value may contain whitespace, including line endings, and a backslash escapes any character, including `"` and `\`:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{title="Say \"hi\"" #"two words"}
.
<p><code id="two words" title="Say &quot;hi&quot;">a</code></p>
````````````````````````````````

A quoted value must be followed by whitespace or `}`:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{k="v".c}
.
<p><code>a</code>{k=&quot;v&quot;.c}</p>
````````````````````````````````

[Entity references][entity reference] are decoded in any value, unless a backslash escapes their `&`:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{k=caf&eacute; l="&amp;" m="\&amp;"}
.
<p><code k="café" l="&amp;" m="&amp;amp;">a</code></p>
````````````````````````````````

### Keys

A [key](@) is an ASCII letter, `_`, or `:`, followed by any number of ASCII letters, digits, `-`, `_`, `:`, and `.`:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{data-x=1 _k=2 xml:lang=en} `b`{1k=v}
.
<p><code data-x="1" _k="2" xml:lang="en">a</code> <code>b</code>{1k=v}</p>
````````````````````````````````

### Conflicts

An element has at most one `id`.  If an attribute block has more than one id attribute, the last one takes precedence.  A key–value attribute with the key `id` counts as an id attribute:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#one #two id=three}
.
<p><code id="three">a</code></p>
````````````````````````````````

Class attributes accumulate, and a key–value attribute with the key `class` adds its value to them:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{.one class="two three" .four}
.
<p><code class="one two three four">a</code></p>
````````````````````````````````

For any other key, the last key–value attribute with that key takes precedence.  Keys are compared without regard to ASCII case, like HTML compares attribute names:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{k=1 l=2 K=3}
.
<p><code l="2" K="3">a</code></p>
````````````````````````````````

## Headings

With `-e header-attributes`, an attribute block at the end of an [ATX heading] or a [setext heading] applies to the heading.  Whitespace must separate the attribute block from the heading's text:

```````````````````````````````` example -e header-attributes --attributes --unsafe
# Heading {#a}

Heading {#b}
-------

# Heading{#c}
.
<h1 id="a">Heading</h1>
<h2 id="b">Heading</h2>
<h1>Heading{#c}</h1>
````````````````````````````````

Whitespace may follow the attribute block, as may an ATX heading's closing sequence.  A closing sequence before the attribute block is part of the heading's text:

```````````````````````````````` example -e header-attributes --attributes --unsafe
# Heading {#a} #

# Heading # {#b}
.
<h1 id="a">Heading</h1>
<h1 id="b">Heading #</h1>
````````````````````````````````

The attribute block must end the heading, and only the last one counts:

```````````````````````````````` example -e header-attributes --attributes --unsafe
# Heading {#a} text

# Heading {#b} {.c}
.
<h1>Heading {#a} text</h1>
<h1 class="c">Heading {#b}</h1>
````````````````````````````````

In a setext heading over more than one line, the attribute block must end the last line.  It may be the last line on its own:

```````````````````````````````` example -e header-attributes --attributes --unsafe
First {#a}
second
======

First
second
{#b}
======
.
<h1>First {#a}
second</h1>
<h1 id="b">First
second</h1>
````````````````````````````````

A heading may consist of an attribute block alone:

```````````````````````````````` example -e header-attributes --attributes --unsafe
# {#empty}
.
<h1 id="empty"></h1>
````````````````````````````````

Comrak recognises the attribute block in the heading's text as written, before inline parsing, so a [backslash escape] or an [entity reference] can't form one:

```````````````````````````````` example -e header-attributes --attributes --unsafe
# Heading \{#a}

# Heading &#123;#b}
.
<h1>Heading {#a}</h1>
<h1>Heading {#b}</h1>
````````````````````````````````

## Fenced code blocks

With `-e fenced-code-attributes`, an attribute block at the end of a [fenced code block][fenced code blocks]'s [info string] applies to the code block.  The first word of the rest of the info string gives the language, as usual:

```````````````````````````````` example -e fenced-code-attributes --attributes --unsafe
```rust {#main .numbered}
fn main() {}
```
.
<pre class="syntax-highlighting numbered" id="main"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

The attribute block may be the whole info string:

```````````````````````````````` example -e fenced-code-attributes --attributes --unsafe
```{#main}
fn main() {}
```
.
<pre class="syntax-highlighting" id="main"><code><span class="text plain">fn main() {}
</span></code></pre>
````````````````````````````````

Whitespace must come before the attribute block if anything precedes it, and only whitespace may follow it.  Otherwise, the attribute block is part of the info string:

```````````````````````````````` example -e fenced-code-attributes --attributes --unsafe
```rust{#a}
fn main() {}
```

```rust {#b} more
fn main() {}
```
.
<pre class="syntax-highlighting"><code class="language-rust{#a}"><span class="text plain">fn main() {}
</span></code></pre>
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

Comrak recognises the attribute block in the info string as written, before it interprets backslash escapes and entity references, so neither can form one:

```````````````````````````````` example -e fenced-code-attributes --attributes --unsafe
```rust \{#a}
fn main() {}
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

## Code spans

With `-e inline-code-attributes`, an attribute block directly after a [code span] applies to it.  Nothing may come between them, including spaces of any kind:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#x} and `b` {#y}
.
<p><code id="x">a</code> and <code>b</code> {#y}</p>
````````````````````````````````

An attribute block may span lines, within the code span's paragraph:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#x
.y}
.
<p><code id="x" class="y">a</code></p>
````````````````````````````````

Consecutive attribute blocks apply together, as if they were one:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#x .y}{.z #w}
.
<p><code id="w" class="y z">a</code></p>
````````````````````````````````

## Links and images

With `-e link-attributes`, an attribute block directly after a [link][links] or an [image][images] applies to it.  The attribute block goes after the link's destination, or after a reference link's last `]`:

```````````````````````````````` example -e link-attributes --attributes --unsafe
[inline](/a){#a} [full][ref]{#b} [collapsed][]{#c} [ref]{#d}

[ref]: /r
[collapsed]: /c
.
<p><a href="/a" id="a">inline</a> <a href="/r" id="b">full</a> <a href="/c" id="c">collapsed</a> <a href="/r" id="d">ref</a></p>
````````````````````````````````

```````````````````````````````` example -e link-attributes --attributes --unsafe
![A cat](cat.png){width=320 .photo}
.
<p><img src="cat.png" alt="A cat" class="photo" width="320" /></p>
````````````````````````````````

CommonMark [autolinks](https://spec.commonmark.org/0.31.2/#autolinks) don't take attributes:

```````````````````````````````` example -e link-attributes --attributes --unsafe
<https://comrak.ee>{#x}
.
<p><a href="https://comrak.ee">https://comrak.ee</a>{#x}</p>
````````````````````````````````

A link's text may contain a code span or an image with attributes of its own:

```````````````````````````````` example -e inline-code-attributes -e link-attributes --attributes --unsafe
[`comrak`{.name} ![logo](comrak.png){width=16}](https://comrak.ee){.home}
.
<p><a href="https://comrak.ee" class="home"><code class="name">comrak</code> <img src="comrak.png" alt="logo" width="16" /></a></p>
````````````````````````````````

## Output

HTML output includes attributes only with both `--attributes` and `--unsafe`.  Comrak writes the `id` first, then the classes, then the key–value attributes in order.  Values are escaped as HTML attribute values:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{k=v .x title="<b> & </b>" #y}
.
<p><code id="y" class="x" k="v" title="&lt;b&gt; &amp; &lt;/b&gt;">a</code></p>
````````````````````````````````

Where Comrak writes an attribute of its own, that attribute takes precedence, and Comrak ignores any attribute of the same name from the attribute block.  Comrak's own attributes include a link's `href` and `title`, an image's `src`, `alt`, and `title`, a heading's generated `id`, and `data-sourcepos`:

```````````````````````````````` example -e link-attributes --attributes --unsafe
[a](/u "Title"){href=/v title=Other data-x=1} [b](/u){title=Used}
.
<p><a href="/u" title="Title" data-x="1">a</a> <a href="/u" title="Used">b</a></p>
````````````````````````````````

A code block's attributes go on its `<pre>` element.  Classes from the attribute block join any class the syntax highlighter gives the `<pre>` element, as in [example 19](#example-19).

Heading adapters and code fence renderers, which a program using Comrak as a library can provide, don't receive attributes.  Syntax highlighters do.

CommonMark output writes attribute blocks back.  It writes each value bare where it can, and quoted otherwise.  In a quoted value, it puts a backslash before `"` and `\`, and writes `&` and line endings as entity references.  It also escapes a `{` that would otherwise begin an attribute block, such as one straight after a code span or anywhere in a heading.  This example's CommonMark view shows both:

```````````````````````````````` example -e header-attributes -e inline-code-attributes --attributes --unsafe
# Braces \{like this} {#braces}

`a`{k="two words"}\{.literal}
.
<h1 id="braces">Braces {like this}</h1>
<p><code k="two words">a</code>{.literal}</p>
````````````````````````````````

XML output doesn't include attributes.

In the AST, a node's attributes are in its [`Ast`](https://docs.rs/comrak/latest/comrak/nodes/struct.Ast.html)'s `attrs` field, an [`Attributes`](https://docs.rs/comrak/latest/comrak/nodes/struct.Attributes.html) holding the id (`id`), the classes (`classes`), and the key–value attributes (`pairs`).  Comrak parses attributes into the AST whenever their extension is enabled, regardless of render options, meaning custom formatters always see them.

An element's source position covers its attribute blocks:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe --sourcepos
`a`{#x
.y} b
.
<p data-sourcepos="1:1-2:5"><code data-sourcepos="1:1-2:3" id="x" class="y">a</code> b</p>
````````````````````````````````

## Options

### Render attributes

With `--attributes` (`render.attributes`) and `--unsafe` (`render.r#unsafe`), HTML output includes attributes.  Without `--unsafe`, they aren't included in HTML output, but are still parsed, so they don't appear as text either.  With `--attributes` alone:

```````````````````````````````` example -e inline-code-attributes --attributes
`a`{#x .y}
.
<p><code>a</code></p>
````````````````````````````````

With both:

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`a`{#x .y}
.
<p><code id="x" class="y">a</code></p>
````````````````````````````````

## Interactions

`--gfm` doesn't enable attributes:

```````````````````````````````` example --gfm --attributes --unsafe
# Heading {#a}
.
<h1>Heading {#a}</h1>
````````````````````````````````

With `--header-id-prefix`, an id attribute replaces a heading's generated `id`, and gets the prefix too.  Generated `id`s avoid any `id` given by an attribute, wherever it appears in the document:

```````````````````````````````` example -e header-attributes --header-id-prefix user-content- --attributes --unsafe
# Hi!
# Hi! {#hi}
# Hi! {#greeting .wave}
.
<h1 id="user-content-hi-1">Hi!<a href="#hi-1" aria-label="Link to heading 'Hi!'" data-heading-content="Hi!" class="anchor"></a></h1>
<h1 id="user-content-hi">Hi!<a href="#hi" aria-label="Link to heading 'Hi!'" data-heading-content="Hi!" class="anchor"></a></h1>
<h1 id="user-content-greeting" class="wave">Hi!<a href="#greeting" aria-label="Link to heading 'Hi!'" data-heading-content="Hi!" class="anchor"></a></h1>
````````````````````````````````

With `--github-pre-lang`, the language goes in the `<pre>` element's `lang` attribute, meaning a `lang` key–value attribute is ignored:

```````````````````````````````` example -e fenced-code-attributes --github-pre-lang --attributes --unsafe
```rust {#main lang=fr}
fn main() {}
```
.
<pre class="syntax-highlighting" id="main" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

With `--full-info-string`, the words after the language, other than the attribute block, go in the `data-meta` attribute:

```````````````````````````````` example -e fenced-code-attributes --full-info-string --attributes --unsafe
```rust ignore {#main}
fn main() {}
```
.
<pre class="syntax-highlighting" id="main"><code class="language-rust" data-meta="ignore"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

With `--default-info-string`, a code block whose info string is only an attribute block gets the default language:

```````````````````````````````` example -e fenced-code-attributes --default-info-string rust --attributes --unsafe
```{#main}
fn main() {}
```
.
<pre class="syntax-highlighting" id="main"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span></span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span>
</span></code></pre>
````````````````````````````````

With [code span-based math](math-code.html), a math span doesn't take attributes:

```````````````````````````````` example -e inline-code-attributes -e math-code --attributes --unsafe
$`x^2`${#x}
.
<p><code data-math-style="inline">x^2</code>{#x}</p>
````````````````````````````````

With [wikilinks](wikilinks.html), a wikilink doesn't take attributes:

```````````````````````````````` example -e link-attributes -e wikilinks-title-after-pipe --attributes --unsafe
[[Home]]{#x}
.
<p><a href="Home" data-wikilink="true">Home</a>{#x}</p>
````````````````````````````````

With [tables](table.html), escape `|` in a value as in any other cell content:

```````````````````````````````` example -e table -e inline-code-attributes --attributes --unsafe
| a |
| - |
| `x`{title="p\|q"} |
.
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td><code title="p|q">x</code></td>
</tr>
</tbody>
</table>
````````````````````````````````

With `--smart`, values keep their punctuation as written:

```````````````````````````````` example -e inline-code-attributes --smart --attributes --unsafe
`a`{title="it's -- \"fine\""}
.
<p><code title="it's -- &quot;fine&quot;">a</code></p>
````````````````````````````````

## Differences from commonmark-hs

[Pandoc](https://pandoc.org) uses commonmark-hs's attributes extension; the comparison here is with Pandoc 3.7.  Pandoc always renders attributes, and its other extensions and options differ from Comrak's, so the examples under [Options](#options) and [Interactions](#interactions), and those with source positions, aren't comparable.  Pandoc's HTML output also differs in ways that don't concern attributes: it puts a code block's language in the `<pre>` element's classes, and it adds `data-` to keys that aren't standard HTML attributes.

On every other example above, Pandoc and Comrak align, except that Pandoc:

* leaves an empty attribute block as text ([example 3](#example-3));
* doesn't allow quoted ids or classes, or backslash escapes in quoted values ([example 6](#example-6), [example 8](#example-8));
* keeps the first id attribute and the first key–value attribute with each key, rather than the last, and compares keys case-sensitively ([example 10](#example-10), [example 12](#example-12), [example 25](#example-25));
* applies an attribute block that follows text or a space to a span around that text or space ([example 13](#example-13), [example 15](#example-15), [example 16](#example-16), [example 23](#example-23));
* does not recognise attributes in a heading that is only an attribute block ([example 17](#example-17));
* recognises an attribute block straight after a code block's language, and one formed by backslash escapes ([example 21](#example-21), [example 22](#example-22));
* recognises attributes on autolinks ([example 28](#example-28)); and,
* writes an attribute that repeats one of its own, such as a second `href` ([example 31](#example-31)).

commonmark-hs's own specification says the last id attribute takes effect, as in Comrak, suggesting Pandoc does something after the fact.

commonmark-hs also allows attributes where Comrak doesn't: after any inline element, such as emphasis; at the end of a link reference definition, for every link that uses it; and on a line of their own before a block.

## Regressions

The following examples guard against regressions.

Source positions of attribute blocks that span lines, after an image and a code span ([#902](https://github.com/kivikakk/comrak/pull/902)):

```````````````````````````````` example -e link-attributes -e inline-code-attributes --attributes --unsafe --sourcepos
a
b ![](x){w=1
   h=2} c `d` e
.
<p data-sourcepos="1:1-3:15">a
b <img data-sourcepos="2:3-3:7" src="x" alt="" w="1" h="2" /> c <code data-sourcepos="3:11-3:13">d</code> e</p>
````````````````````````````````

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe --sourcepos
a
`x`{w=1
  h=2} b
.
<p data-sourcepos="1:1-3:8">a
<code data-sourcepos="2:1-3:6" w="1" h="2">x</code> b</p>
````````````````````````````````

Source positions of a link whose destination starts on a later line ([#908](https://github.com/kivikakk/comrak/pull/908)):

```````````````````````````````` example -e link-attributes --attributes --unsafe --sourcepos
[a](
/u){.c} x
.
<p data-sourcepos="1:1-2:9"><a data-sourcepos="1:1-2:7" href="/u" class="c">a</a> x</p>
````````````````````````````````

[ATX heading]: https://spec.commonmark.org/0.31.2/#atx-heading
[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[fenced code blocks]: https://spec.commonmark.org/0.31.2/#fenced-code-blocks
[images]: https://spec.commonmark.org/0.31.2/#images
[info string]: https://spec.commonmark.org/0.31.2/#info-string
[links]: https://spec.commonmark.org/0.31.2/#links
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-heading
