---
title: Superscript and subscript
extensions: [superscript, subscript]
with_thanks_to:
  - https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/superscript.md
  - https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/subscript.md
---

Superscript raises text above the baseline, as in an exponent, and subscript lowers it below the baseline, as in a chemical formula.  To write superscript, wrap text in carets (`^`).  To write subscript, wrap it in tildes (`~`).

Comrak's superscript and subscript follow the superscript and subscript extensions of [commonmark-hs](https://github.com/jgm/commonmark-hs/tree/master/commonmark-extensions), which [Pandoc](https://pandoc.org/MANUAL.html#extension-superscript-subscript) also uses.  This document describes both extensions, and [Differences from commonmark-hs](#differences-from-commonmark-hs) lists where they differ.  GitLab Flavored Markdown [doesn't support](https://docs.gitlab.com/user/markdown/#superscripts-and-subscripts) either syntax.

To enable superscript, pass `-e superscript` on the command line, or set `extension.superscript` in the options struct.  For subscript, pass `-e subscript`, or set `extension.subscript`.

Here's typical superscript and subscript:

```````````````````````````````` example -e superscript -e subscript
E = mc^2^, and water is H~2~O.
.
<p>E = mc<sup>2</sup>, and water is H<sub>2</sub>O.</p>
````````````````````````````````

## Syntax

Superscript and subscript work like CommonMark's [emphasis], with carets or tildes in place of asterisks, except that punctuation doesn't affect where they begin and end.

A [caret run](@) is a sequence of carets that isn't preceded or followed by another caret.  A [tilde run](@) is a sequence of one or two tildes that isn't preceded or followed by another tilde.  A caret run, or a tilde run of one tilde, [can open](@) superscript or subscript if it isn't followed by whitespace, and [can close](@) it if it isn't preceded by whitespace.  The beginning and end of a line count as whitespace.

Superscript begins with a caret that [can open] it, and ends with a later caret that [can close] it.  Subscript begins with a tilde run of one tilde that [can open] it, and ends with a later tilde run of one tilde that [can close] it.  Comrak pairs runs with CommonMark's [process emphasis] procedure, the same procedure that pairs emphasis delimiters.

### Caret runs

Each caret in a caret run pairs separately, so a run of two carets can begin two superscripts:

```````````````````````````````` example -e superscript -e subscript
^^a^^ and ^^^b^^^
.
<p><sup><sup>a</sup></sup> and <sup><sup><sup>b</sup></sup></sup></p>
````````````````````````````````

The caret runs at each end needn't be the same length.  Carets left over are literal text:

```````````````````````````````` example -e superscript -e subscript
^a^^ and ^^b^
.
<p><sup>a</sup>^ and ^<sup>b</sup></p>
````````````````````````````````

As with emphasis, if either of two caret runs can both open and close superscript, the two don't pair when their lengths add up to a multiple of three, unless both lengths are multiples of three.  A caret run inside a word can both open and close:

```````````````````````````````` example -e superscript -e subscript
x^^y^z
.
<p>x^^y^z</p>
````````````````````````````````

### Tilde runs

Tilde runs follow [strikethrough](strikethrough.html)'s rules, whether or not strikethrough is enabled.  A tilde run of one tilde forms subscript.  A tilde run of two tildes forms strikethrough if strikethrough is enabled, and is literal text if it isn't.  Three or more tildes are always literal text, as are tilde runs of different lengths:

```````````````````````````````` example -e subscript
~~a~~ and ~~~b~~~ and ~c~~
.
<p>~~a~~ and ~~~b~~~ and ~c~~</p>
````````````````````````````````

### Flanking

Punctuation doesn't affect whether a caret run, or a tilde run of one tilde, can open or close.  Superscript and subscript may begin before punctuation and end after it, even inside a word:

```````````````````````````````` example -e superscript -e subscript
i^-2^ and x~(1)~y and a^b.^c
.
<p>i<sup>-2</sup> and x<sub>(1)</sub>y and a<sup>b.</sup>c</p>
````````````````````````````````

Whitespace does affect whether a run can open or close.  A run followed by whitespace can't open, and one preceded by whitespace can't close:

```````````````````````````````` example -e superscript -e subscript
a^ b^ and a^b ^ and ^ ^

a~ b~ and a~b ~ and ~ ~
.
<p>a^ b^ and a^b ^ and ^ ^</p>
<p>a~ b~ and a~b ~ and ~ ~</p>
````````````````````````````````

Superscript and subscript may begin and end inside a word, and may contain spaces and other inline content:

```````````````````````````````` example -e superscript -e subscript
a^b^c and a~b~c

E = mc^2 or *3*^
.
<p>a<sup>b</sup>c and a<sub>b</sub>c</p>
<p>E = mc<sup>2 or <em>3</em></sup></p>
````````````````````````````````

### Nesting

Superscript and subscript may contain each other, or themselves:

```````````````````````````````` example -e superscript -e subscript
^a ^b^ c^ and ~a ^b^ c~
.
<p><sup>a <sup>b</sup> c</sup> and <sub>a <sup>b</sup> c</sub></p>
````````````````````````````````

Superscript, subscript and emphasis may contain each other:

```````````````````````````````` example -e superscript -e subscript
^*a*^ and *~a~*
.
<p><sup><em>a</em></sup> and <em><sub>a</sub></em></p>
````````````````````````````````

When superscript or subscript overlaps emphasis, the first to close wins, and the other's delimiters are literal text:

```````````````````````````````` example -e superscript -e subscript
*a ^b* c^ and ^a *b^ c*
.
<p><em>a ^b</em> c^ and <sup>a *b</sup> c*</p>
````````````````````````````````

### Precedence

[Code spans][code span] and [links] take precedence over superscript and subscript, so runs pair only within or around them:

```````````````````````````````` example -e superscript -e subscript
^a `b^` c^
.
<p><sup>a <code>b^</code> c</sup></p>
````````````````````````````````

```````````````````````````````` example -e superscript -e subscript
^a [b^ c](/u)

[a^2^](/u) and ~[b](/u)~
.
<p>^a <a href="/u">b^ c</a></p>
<p><a href="/u">a<sup>2</sup></a> and <sub><a href="/u">b</a></sub></p>
````````````````````````````````

Brackets that don't form a link don't affect pairing:

```````````````````````````````` example -e superscript -e subscript
[x^2^] and [ y~2~
.
<p>[x<sup>2</sup>] and [ y<sub>2</sub></p>
````````````````````````````````

Superscript and subscript may span lines, but not [paragraphs][paragraph]:

```````````````````````````````` example -e superscript -e subscript
^a
b^ and ~c
d~
.
<p><sup>a
b</sup> and <sub>c
d</sub></p>
````````````````````````````````

```````````````````````````````` example -e superscript -e subscript
^a

b^
.
<p>^a</p>
<p>b^</p>
````````````````````````````````

A [backslash escape] or an [entity reference] is literal text, not a run:

```````````````````````````````` example -e superscript -e subscript
\^a^ and \~b~

&#94;a&#94; and &#126;b&#126;
.
<p>^a^ and ~b~</p>
<p>^a^ and ~b~</p>
````````````````````````````````

## Output

HTML output renders superscript as a `<sup>` element, and subscript as a `<sub>` element.  In an image description, which becomes the image's `alt` attribute, superscript and subscript contribute only their text:

```````````````````````````````` example -e superscript -e subscript
![E = mc^2^](/e.png)
.
<p><img src="/e.png" alt="E = mc2" /></p>
````````````````````````````````

CommonMark output writes superscript with a caret on each side, and subscript with a tilde on each side.  It escapes other tildes (`\~`) and, with superscript enabled, other carets (`\^`), to prevent reading them back as runs:

```````````````````````````````` example -e superscript -e subscript
^^a^^ and \^b^ and c~d
.
<p><sup><sup>a</sup></sup> and ^b^ and c~d</p>
````````````````````````````````

In XML output, superscript is a `superscript` element, and subscript is a `subscript` element.  Without strikethrough, a pair of two-tilde runs is an `escaped_tag` element, with the tildes in its `tag` attribute.

In the AST, superscript is a [`NodeValue::Superscript`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Superscript), and subscript is a [`NodeValue::Subscript`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Subscript), each with its content as children.  Without strikethrough, a pair of two-tilde runs is a [`NodeValue::EscapedTag`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.EscapedTag) holding `"~~"`.

The [source position](../options/sourcepos.html#source-position) of superscript or subscript covers its runs:

```````````````````````````````` example -e superscript -e subscript --sourcepos
x ^a^ ~b~
.
<p data-sourcepos="1:1-1:9">x <sup data-sourcepos="1:3-1:5">a</sup> <sub data-sourcepos="1:7-1:9">b</sub></p>
````````````````````````````````

## Interactions

With [strikethrough](strikethrough.html), a tilde run of one tilde forms subscript, and a tilde run of two tildes forms strikethrough:

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

Strikethrough keeps its own flanking rules, in which punctuation matters.  Here, the two-tilde run after `a` and before `(` can't open strikethrough, but the one-tilde run can open subscript:

```````````````````````````````` example -e strikethrough -e subscript
a~~(b)~~ c and a~(b)~ c
.
<p>a~~(b)~~ c and a<sub>(b)</sub> c</p>
````````````````````````````````

With [footnotes](footnotes.html), a footnote reference's caret doesn't begin superscript.  A `[^` that doesn't form a reference isn't special: its bracket is literal text, and its caret pairs like any other:

```````````````````````````````` example -e superscript -e footnotes
a[^1] b^2^ [^x] c^d^

[^1]: A note.
.
<p>a<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup> b<sup>2</sup> [<sup>x] c</sup>d^</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>A note. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

With [inline footnotes](inline-footnotes.html), `^[` begins an inline footnote, not superscript:

```````````````````````````````` example -e superscript -e footnotes -e inline-footnotes
a^[A note.] and b^2^
.
<p>a<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup> and b<sup>2</sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>A note. <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

With [autolinks](autolink.html), a [www autolink](autolink.html#www-autolink) doesn't form straight after a caret, because a www autolink must begin at the start of a line, after whitespace, or after one of `*`, `_`, `~`, or `(`.  A [URL autolink](autolink.html#url-autolink) may end in a caret: a closing caret straight after a URL belongs to the URL:

```````````````````````````````` example -e superscript -e subscript -e autolink
^www.example.com^ and ^https://example.com^ and ~www.example.com~
.
<p><sup>www.example.com</sup> and ^<a href="https://example.com%5E">https://example.com^</a> and <sub><a href="http://www.example.com">www.example.com</a></sub></p>
````````````````````````````````

With [maths](math.html), the content of maths is literal text, so carets and tildes in it don't form superscript or subscript:

```````````````````````````````` example -e superscript -e subscript -e math-dollars
$x^2^$ and $y~1~$
.
<p><span data-math-style="inline">x^2^</span> and <span data-math-style="inline">y~1~</span></p>
````````````````````````````````

[CJK-friendly emphasis](cjk-friendly-emphasis.html) doesn't change superscript or subscript, since punctuation doesn't affect their flanking:

```````````````````````````````` example -e superscript -e subscript -e cjk-friendly-emphasis
「^a」^b and あ~い~う
.
<p>「<sup>a」</sup>b and あ<sub>い</sub>う</p>
````````````````````````````````

With [OFM block IDs](ofm-block-ids.html), a caret on a block's last line begins a block ID, unless a later caret closes it as superscript:

```````````````````````````````` example -e superscript -e ofm-block-ids
a ^b^ c ^abc

a ^abc^
.
<p id="^abc">a <sup>b</sup> c </p>
<p>a <sup>abc</sup></p>
````````````````````````````````

With [description lists](description-lists.html), a line beginning with a tilde and a space marks a description, not a tilde run:

```````````````````````````````` example -e subscript -e description-lists
Term

~ a~2~
.
<dl>
<dt>Term</dt>
<dd>
<p>a<sub>2</sub></p>
</dd>
</dl>
````````````````````````````````

## Differences from commonmark-hs

Pandoc's `commonmark` reader uses commonmark-hs.  This section compares Comrak with Pandoc 3.7, run as `pandoc -f commonmark+superscript+subscript`.  Since Pandoc's other extensions differ from Comrak's, the examples under [Interactions](#interactions) aren't comparable, and nor is [example 20](#example-20), which shows source positions.  On every other example above, the two agree on superscript and subscript, except for tilde runs.  Without strikethrough, Pandoc pairs each tilde separately, as Comrak pairs carets.  Two or three tildes form nested subscripts, and tilde runs of different lengths pair ([example 5](#example-5)).

Pandoc's own Markdown reader doesn't allow spaces in superscript or subscript, and isn't compared here.

## Regressions

The following examples guard against regressions.

Superscript and subscript may begin before punctuation ([#592](https://github.com/kivikakk/comrak/issues/592)):

```````````````````````````````` example -e superscript -e subscript
i^-2^ = -1.

f~i~ = f~-2 + i~ + f~-1 + i~.
.
<p>i<sup>-2</sup> = -1.</p>
<p>f<sub>i</sub> = f<sub>-2 + i</sub> + f<sub>-1 + i</sub>.</p>
````````````````````````````````

A footnote reference doesn't stop superscript elsewhere in the document ([#274](https://github.com/kivikakk/comrak/pull/274)):

```````````````````````````````` example -e superscript -e footnotes
Here is a footnote reference.[^1]

e = mc^2^.

[^1]: Here is the footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<p>e = mc<sup>2</sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Here is the footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[emphasis]: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[links]: https://spec.commonmark.org/0.31.2/#links
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[process emphasis]: https://spec.commonmark.org/0.31.2/#process-emphasis
