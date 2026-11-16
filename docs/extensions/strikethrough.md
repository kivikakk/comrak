---
title: Strikethrough
---

Strikethrough draws a line through text, to mark it as deleted or no longer accurate.  To strike text through, wrap it in one or two tildes (`~`).

Strikethrough originates in [GitHub Flavored Markdown](https://github.github.com/gfm/#strikethrough-extension-).  This document describes Comrak's strikethrough, which can differ from GitHub's.  GitLab uses Comrak's strikethrough, so this document is also the canonical spec for [GitLab Flavored Markdown strikethrough](https://docs.gitlab.com/user/markdown/#emphasis).

To enable strikethrough, pass `-e strikethrough` (or `--gfm`) on the command line, or set `extension.strikethrough` in the options struct.

Here's a typical strikethrough:

```````````````````````````````` example -e strikethrough
Meet on ~~Tuesday~~ Wednesday.
.
<p>Meet on <del>Tuesday</del> Wednesday.</p>
````````````````````````````````

## Syntax

Strikethrough works like CommonMark's [emphasis], with tildes in place of asterisks.

A [tilde run](@) is a sequence of one or two tildes that isn't preceded or followed by another tilde.  A tilde run [can open](@) strikethrough if it's a [left-flanking delimiter run], and [can close](@) strikethrough if it's a [right-flanking delimiter run].

Strikethrough begins with a [tilde run] that [can open] it, and ends with a later tilde run of the same length that [can close] it.  Comrak pairs tilde runs with CommonMark's [process emphasis] procedure, the same procedure that pairs emphasis delimiters.

### Tilde runs

A tilde run may have one tilde or two:

```````````````````````````````` example -e strikethrough
~one~ and ~~two~~
.
<p><del>one</del> and <del>two</del></p>
````````````````````````````````

Three or more tildes are literal text:

```````````````````````````````` example -e strikethrough
a ~~~b~~~ c
.
<p>a ~~~b~~~ c</p>
````````````````````````````````

The tilde runs at each end must be the same length.  Otherwise, neither run forms strikethrough:

```````````````````````````````` example -e strikethrough
~a~~ b

~~a~ b
.
<p>~a~~ b</p>
<p>~~a~ b</p>
````````````````````````````````

An unequal pair doesn't prevent other tilde runs from pairing.  Here, the inner `~` and `~~` don't pair, so the outer runs do:

```````````````````````````````` example -e strikethrough
~~a ~b~~ c~~
.
<p><del>a ~b~~ c</del></p>
````````````````````````````````

### Flanking

A tilde run followed by whitespace can't open strikethrough, and one preceded by whitespace can't close it:

```````````````````````````````` example -e strikethrough
~~ a~~

~~a ~~
.
<p>~~ a~~</p>
<p>~~a ~~</p>
````````````````````````````````

Unlike `_` emphasis, strikethrough may begin and end inside a word:

```````````````````````````````` example -e strikethrough
a~~b~~c
.
<p>a<del>b</del>c</p>
````````````````````````````````

Punctuation affects flanking as it does for emphasis.  Strikethrough may have punctuation just inside its tilde runs.  But a tilde run after a letter and before punctuation can't open strikethrough, and one after punctuation and before a letter can't close it:

```````````````````````````````` example -e strikethrough
~~(a)~~

a~~(b)~~c
.
<p><del>(a)</del></p>
<p>a~~(b)~~c</p>
````````````````````````````````

### Nesting

Strikethrough may contain strikethrough, with either length of tilde run:

```````````````````````````````` example -e strikethrough
~~a ~b~ c~~
.
<p><del>a <del>b</del> c</del></p>
````````````````````````````````

Strikethrough and emphasis may contain each other:

```````````````````````````````` example -e strikethrough
~~*a*~~ and *~~b~~*
.
<p><del><em>a</em></del> and <em><del>b</del></em></p>
````````````````````````````````

When strikethrough and emphasis overlap, the first to close wins, and the other's delimiters are literal:

```````````````````````````````` example -e strikethrough
~~a *b~~ c*

*a ~~b* c~~
.
<p><del>a *b</del> c*</p>
<p><em>a ~~b</em> c~~</p>
````````````````````````````````

### Precedence

Block structure comes first.  Three tildes at the start of a line begin a [fenced code block]:

```````````````````````````````` example -e strikethrough
~~~a~~~
.
<pre class="syntax-highlighting"><code class="language-a~~~"></code></pre>
````````````````````````````````

[Code spans][code span] and [links] take precedence over strikethrough, so tilde runs pair only within or around them:

```````````````````````````````` example -e strikethrough
~~a `b~~` c~~
.
<p><del>a <code>b~~</code> c</del></p>
````````````````````````````````

```````````````````````````````` example -e strikethrough
~~[a~~](/u)

[~~a~~](/u)
.
<p>~~<a href="/u">a~~</a></p>
<p><a href="/u"><del>a</del></a></p>
````````````````````````````````

Strikethrough may span lines, but not [paragraphs][paragraph]:

```````````````````````````````` example -e strikethrough
~~a
b~~
.
<p><del>a
b</del></p>
````````````````````````````````

```````````````````````````````` example -e strikethrough
This ~~has a

new paragraph~~.
.
<p>This ~~has a</p>
<p>new paragraph~~.</p>
````````````````````````````````

A [backslash escape] or an [entity reference] is literal text, not a tilde run:

```````````````````````````````` example -e strikethrough
\~~a~~

&#126;&#126;a&#126;&#126;
.
<p>~~a~~</p>
<p>~~a~~</p>
````````````````````````````````

## Output

HTML output renders strikethrough as a `<del>` element.

CommonMark output writes strikethrough with two tildes on each side, whatever its length in the source.  It escapes other tildes (`\~`), to prevent reading them back as tilde runs:

```````````````````````````````` example -e strikethrough
~a~ and x~~~y
.
<p><del>a</del> and x~~~y</p>
````````````````````````````````

In XML output, strikethrough is a `strikethrough` element.

In the AST, strikethrough is a [`NodeValue::Strikethrough`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Strikethrough), with its content as children.

Strikethrough's source position covers its tilde runs:

```````````````````````````````` example -e strikethrough --sourcepos
x ~~a~~ ~b~
.
<p data-sourcepos="1:1-1:11">x <del data-sourcepos="1:3-1:7">a</del> <del data-sourcepos="1:9-1:11">b</del></p>
````````````````````````````````

## Interactions

`--gfm` enables strikethrough, along with the other GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
~~a~~
.
<p><del>a</del></p>
````````````````````````````````

With [subscript](subscript.html), a tilde run of one tilde forms subscript instead, and a tilde run of two tildes forms strikethrough:

```````````````````````````````` example -e strikethrough -e subscript
H~2~O is ~~not~~ water.
.
<p>H<sub>2</sub>O is <del>not</del> water.</p>
````````````````````````````````

```````````````````````````````` example -e strikethrough -e subscript
~~H~2~O~~
.
<p><del>H<sub>2</sub>O</del></p>
````````````````````````````````

With [CJK-friendly emphasis](cjk-friendly-emphasis.html), the flanking rules change for tilde runs next to CJK characters, as they do for emphasis.  Without it, these aren't strikethrough:

```````````````````````````````` example -e strikethrough
a~~a()~~あ

「~~a」~~a
.
<p>a~~a()~~あ</p>
<p>「~~a」~~a</p>
````````````````````````````````

With it, they are:

```````````````````````````````` example -e strikethrough -e cjk-friendly-emphasis
a~~a()~~あ

「~~a」~~a
.
<p>a<del>a()</del>あ</p>
<p>「<del>a」</del>a</p>
````````````````````````````````

With [autolinks](autolink.html), the tildes around an autolink aren't part of it:

```````````````````````````````` example -e strikethrough -e autolink
~~www.example.com~~
.
<p><del><a href="http://www.example.com">www.example.com</a></del></p>
````````````````````````````````

With [description lists](description-lists.html), a line beginning with a tilde and a space marks details, not a tilde run:

```````````````````````````````` example -e strikethrough -e description-lists
Term

~ ~~details~~
.
<dl>
<dt>Term</dt>
<dd>
<p><del>details</del></p>
</dd>
</dl>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, uses the same strikethrough design as Comrak.  It doesn't support the [subscript](subscript.html), [CJK-friendly emphasis](cjk-friendly-emphasis.html), or [description lists](description-lists.html) extensions, so those examples aren't comparable.  On every other example above, the two implementations agree on HTML output, except that cmark-gfm gives strikethrough no source position ([example 19](#example-19)).

cmark-gfm also has a `--strikethrough-double-tilde` option, with which a tilde run of one tilde is literal.  Comrak has no equivalent.

## Regressions

The following examples guard against regressions.

Tilde runs of four are literal, even between a pair that forms strikethrough ([#635](https://github.com/kivikakk/comrak/pull/635)):

```````````````````````````````` example -e strikethrough
This is ~strikethrough~.

As is ~~this, okay~~?

This ~text~~~~ is ~~~~curious~.
.
<p>This is <del>strikethrough</del>.</p>
<p>As is <del>this, okay</del>?</p>
<p>This <del>text~~~~ is ~~~~curious</del>.</p>
````````````````````````````````

An unequal pair doesn't stop later delimiters from pairing:

```````````````````````````````` example -e strikethrough
~~a~ *b*

~a~~ and *b* and ~c~
.
<p>~~a~ <em>b</em></p>
<p>~a~~ and <em>b</em> and <del>c</del></p>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[emphasis]: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[fenced code block]: https://spec.commonmark.org/0.31.2/#fenced-code-block
[left-flanking delimiter run]: https://spec.commonmark.org/0.31.2/#left-flanking-delimiter-run
[links]: https://spec.commonmark.org/0.31.2/#links
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[process emphasis]: https://spec.commonmark.org/0.31.2/#process-emphasis
[right-flanking delimiter run]: https://spec.commonmark.org/0.31.2/#right-flanking-delimiter-run
