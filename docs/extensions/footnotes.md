---
title: Footnotes
---

A footnote is a note that's set apart from the main text.  A numbered mark in the text refers to it.  Comrak collects footnotes into a numbered list at the end of the document, with links from each mark to its footnote and back again.  Use footnotes for asides, citations, or anything else that would interrupt the text around it.

The [GitHub Flavored Markdown](https://github.github.com/gfm/) spec doesn't describe footnotes, but [cmark-gfm](https://github.com/github/cmark-gfm), its reference implementation, supports them, and Comrak's footnotes follow its design.  This document describes Comrak's footnotes, which can differ from cmark-gfm's.  GitLab uses Comrak's footnotes, so this document is also the canonical spec for [GitLab Flavored Markdown footnotes](https://docs.gitlab.com/user/markdown/#footnotes).

To enable footnotes, pass `-e footnotes` on the command line, or set `extension.footnotes` in the options struct.  `--gfm` doesn't enable them.

Here's a typical footnote:

```````````````````````````````` example -e footnotes
Hi[^x].

[^x]: A greeting.
.
<p>Hi<sup class="footnote-ref"><a href="#fn-x" id="fnref-x" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-x">
<p>A greeting. <a href="#fnref-x" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

## Syntax

A footnote has two parts: a [footnote reference] in the text, and a [footnote definition] that holds the footnote's content.  A reference can come before or after its definition.

### Footnote labels

A [footnote label](@) is one or more characters other than a [space], a [tab], a [line ending], or `]`.  Labels may contain punctuation and characters outside ASCII:

```````````````````````````````` example -e footnotes
A[^a.b-c_d] and B[^über].

[^a.b-c_d]: Punctuation.
[^über]: Not ASCII.
.
<p>A<sup class="footnote-ref"><a href="#fn-a.b-c_d" id="fnref-a.b-c_d" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-%C3%BCber" id="fnref-%C3%BCber" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a.b-c_d">
<p>Punctuation. <a href="#fnref-a.b-c_d" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-%C3%BCber">
<p>Not ASCII. <a href="#fnref-%C3%BCber" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

A label can't contain whitespace.  Neither of these lines is a footnote reference or a footnote definition:

```````````````````````````````` example -e footnotes
Foo.[^one two]

[^one two]: Here is the footnote.
.
<p>Foo.[^one two]</p>
<p>[^one two]: Here is the footnote.</p>
````````````````````````````````

A label can't be empty either:

```````````````````````````````` example -e footnotes
Foo.[^]

[^]: Here is the footnote.
.
<p>Foo.[^]</p>
<p>[^]: Here is the footnote.</p>
````````````````````````````````

A footnote reference matches a footnote definition when their labels are the same, ignoring case.  The footnote takes its label's case from its definition:

```````````````````````````````` example -e footnotes
Here is a footnote reference.[^AB] and [^ab]

[^aB]: Here is the footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-aB" id="fnref-aB" data-footnote-ref>1</a></sup> and <sup class="footnote-ref"><a href="#fn-aB" id="fnref-aB-2" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-aB">
<p>Here is the footnote. <a href="#fnref-aB" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-aB-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

Whitespace at either end of the reference's label is also ignored:

```````````````````````````````` example -e footnotes
A[^a], B[^ a] and C[^a ].

[^a]: Note.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>, B<sup class="footnote-ref"><a href="#fn-a" id="fnref-a-2" data-footnote-ref>1</a></sup> and C<sup class="footnote-ref"><a href="#fn-a" id="fnref-a-3" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Note. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-a-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a> <a href="#fnref-a-3" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-3" aria-label="Back to reference 1-3">↩<sup class="footnote-ref">3</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

Otherwise, labels must match exactly as they're written in the source.  A [backslash escape] or an [entity reference] in a label isn't interpreted, and inline syntax in a label doesn't stop it matching.  Here, `[^&#97;]` doesn't match the definition labelled `a`, but `` [^`b`] `` matches the definition labelled `` `b` ``:

```````````````````````````````` example -e footnotes
A[^&#97;] and B[^`b`].

[^a]: Plain.
[^`b`]: Backticks.
.
<p>A[^a] and B<sup class="footnote-ref"><a href="#fn-%60b%60" id="fnref-%60b%60" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-%60b%60">
<p>Backticks. <a href="#fnref-%60b%60" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

### Footnote definitions

A [footnote definition](@) begins with a line that has up to three spaces of indentation, then `[^`, a [footnote label], and `]:`.  Any spaces or tabs after the colon are skipped, and the footnote's content begins with what follows.

The content is a sequence of blocks, like a [list item]'s.  Later lines belong to the definition if they're blank, or indented by four or more spaces:

```````````````````````````````` example -e footnotes
A[^a].

[^a]: First paragraph.

    Second paragraph.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>First paragraph.</p>
<p>Second paragraph. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

A line with less indentation ends the definition:

```````````````````````````````` example -e footnotes
A[^a].

[^a]: First paragraph.

  Not in the footnote.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<p>Not in the footnote.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>First paragraph. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

A [lazy continuation line] continues a paragraph as usual:

```````````````````````````````` example -e footnotes
A[^a].

[^a]: First line,
second line.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>First line,
second line. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

The content may begin on a later line, and may begin with any kind of block:

```````````````````````````````` example -e footnotes
A[^a] and B[^b].

[^a]:
    On the next line.

[^b]: > A block quote.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>On the next line. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-b">
<blockquote>
<p>A block quote.</p>
</blockquote>
<a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a>
</li>
</ol>
</section>
````````````````````````````````

A footnote definition may be empty:

```````````````````````````````` example -e footnotes
A[^a].

[^a]:
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a>
</li>
</ol>
</section>
````````````````````````````````

A footnote definition may interrupt a paragraph:

```````````````````````````````` example -e footnotes
A[^a].
[^a]: Note.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Note. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

The colon must directly follow the `]`, but the content needn't be separated from the colon:

```````````````````````````````` example -e footnotes
A[^a] and B[^b].

[^a] : Not a definition.

[^b]:Note.
.
<p>A[^a] and B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>1</a></sup>.</p>
<p>[^a] : Not a definition.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-b">
<p>Note. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

Four spaces of indentation begin an [indented code block] instead:

```````````````````````````````` example -e footnotes
A[^a].

    [^a]: Code.
.
<p>A[^a].</p>
<pre class="syntax-highlighting"><code><span class="text plain">[^a]: Code.
</span></code></pre>
````````````````````````````````

A footnote definition may appear anywhere a block may, including inside a [block quote], a [list item], or another footnote definition.  Comrak moves it to the footnotes at the end of the document:

```````````````````````````````` example -e footnotes
> A[^a] and B[^b].
>
> [^a]: Inside a block quote.
>
>     [^b]: Inside a footnote definition.
.
<blockquote>
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>2</a></sup>.</p>
</blockquote>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Inside a block quote. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-b">
<p>Inside a footnote definition. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

If more than one footnote definition has the same label, the first one in the document is used, and the rest are ignored:

```````````````````````````````` example -e footnotes
A[^a].

[^a]: First.

[^a]: Second.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>First. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

### Footnote references

A [footnote reference](@) is `[^`, followed by text that matches the label of a [footnote definition], followed by `]`.  [Footnote labels](#footnote-labels) describes how labels match.

Brackets without a matching label are parsed like any other non-footnote Markdown:

```````````````````````````````` example -e footnotes
A[^a] and B[^*b*].
.
<p>A[^a] and B[^<em>b</em>].</p>
````````````````````````````````

A footnote reference may appear anywhere inline content may, such as in a heading or in emphasis:

```````````````````````````````` example -e footnotes
# Title[^a]

*Emphasis[^a]*

[^a]: Note.
.
<h1>Title<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup></h1>
<p><em>Emphasis<sup class="footnote-ref"><a href="#fn-a" id="fnref-a-2" data-footnote-ref>1</a></sup></em></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Note. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-a-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

An [inline link] takes precedence over a footnote reference:

```````````````````````````````` example -e footnotes
A[^a](/url).

[^a]: Note.
.
<p>A<a href="/url">^a</a>.</p>
````````````````````````````````

## Numbering

Comrak numbers footnotes in [reading order](@).  It reads the document first, skipping footnote definitions, and then reads the content of each numbered footnote in turn.  Each footnote gets the next number when Comrak first reads a reference to it, and further references use the same number:

```````````````````````````````` example -e footnotes
A[^b], B[^a], and C[^b].

[^a]: Alpha.
[^b]: Beta.
.
<p>A<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>1</a></sup>, B<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>2</a></sup>, and C<sup class="footnote-ref"><a href="#fn-b" id="fnref-b-2" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-b">
<p>Beta. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-b-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
<li id="fn-a">
<p>Alpha. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

A footnote first referred to from another footnote comes after those referred to from the text:

```````````````````````````````` example -e footnotes
A[^a] and C[^c].

[^a]: See B[^b].
[^b]: Beta.
[^c]: Gamma.
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup> and C<sup class="footnote-ref"><a href="#fn-c" id="fnref-c" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>See B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>3</a></sup>. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-c">
<p>Gamma. <a href="#fnref-c" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
<li id="fn-b">
<p>Beta. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="3" aria-label="Back to reference 3">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

Where the definitions are located in the document doesn't matter:

```````````````````````````````` example -e footnotes
[^a]: See B[^b].
[^b]: Beta.

A[^a].
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>See B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>2</a></sup>. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-b">
<p>Beta. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

The footnotes at the end of the document are those with a number.  A footnote definition that isn't referred to, either from the text or from another numbered footnote, is dropped:

```````````````````````````````` example -e footnotes
A[^c].

[^a]: Not referred to.
[^b]: Only referred to from[^a].
[^c]: Gamma.
.
<p>A<sup class="footnote-ref"><a href="#fn-c" id="fnref-c" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-c">
<p>Gamma. <a href="#fnref-c" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

## Output

HTML output renders a footnote reference as a `<sup>` element with the class `footnote-ref`.  It contains a link to the footnote, whose text is the footnote's number.  The link's `id` is `fnref-` followed by the label, with `-2`, `-3`, and so on added for the second and later references to the same footnote.

The footnotes follow the rest of the document, in a `<section>` element with the class `footnotes` that contains an `<ol>` list.  Each footnote is an `<li>` element whose `id` is `fn-` followed by the label.  At the end of each footnote are backlinks, one for each reference to it.  The backlinks go inside the footnote's last paragraph if the footnote ends with a paragraph, and otherwise after its last block:

```````````````````````````````` example -e footnotes
A[^a] and B[^a].

[^a]: Ends with code.

        code
.
<p>A<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-a" id="fnref-a-2" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Ends with code.</p>
<pre class="syntax-highlighting"><code><span class="text plain">code
</span></code></pre>
<a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-a-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a>
</li>
</ol>
</section>
````````````````````````````````

Comrak percent-encodes labels in `href` and `id` attributes:

```````````````````````````````` example -e footnotes
A[^über] and B[^<b>].

[^über]: Umlaut.
[^<b>]: Angle brackets.
.
<p>A<sup class="footnote-ref"><a href="#fn-%C3%BCber" id="fnref-%C3%BCber" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-%3Cb%3E" id="fnref-%3Cb%3E" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-%C3%BCber">
<p>Umlaut. <a href="#fnref-%C3%BCber" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-%3Cb%3E">
<p>Angle brackets. <a href="#fnref-%3Cb%3E" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

CommonMark output writes each footnote reference with its definition's label, and writes the numbered footnote definitions at the end of the document, in number order.  Each definition's content goes on the lines after its label, indented by four spaces.

In XML output, a footnote reference is a `footnote_reference` element, and a footnote definition is a `footnote_definition` element.  Both have a `label` attribute.  The footnote definitions come at the end of the document, in number order.

In the AST, a footnote reference is a [`NodeValue::FootnoteReference`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.FootnoteReference).  Its [`NodeFootnoteReference`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeFootnoteReference.html) holds the label, the footnote's number (`ix`), and which reference to the footnote this is (`ref_num`).  A footnote definition is a [`NodeValue::FootnoteDefinition`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.FootnoteDefinition), whose [`NodeFootnoteDefinition`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeFootnoteDefinition.html) holds the label and the number of references to it (`total_references`), with the content as its children.

Comrak moves numbered footnote definitions to the end of the document, in number order, and removes the rest.  With `parse.leave_footnote_definitions`, every footnote definition stays where it is in the document, including those that would otherwise be dropped.  The option has no command-line flag yet.

Footnote references and definitions have source positions:

```````````````````````````````` example -e footnotes --sourcepos
A[^a].

[^a]: Note.
.
<p data-sourcepos="1:1-1:6">A<sup data-sourcepos="1:2-1:5" class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup>.</p>
<section data-sourcepos="3:1-3:11" class="footnotes" data-footnotes>
<ol>
<li data-sourcepos="3:1-3:11" id="fn-a">
<p data-sourcepos="3:7-3:11">Note. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

## Interactions

`--gfm` doesn't enable footnotes.  Without the extension, what looks like a footnote definition forms a [link reference definition] instead if its content is a single word, and what looks like a footnote reference links to it:

```````````````````````````````` example --gfm
A[^a].

[^a]: Alpha.
.
<p>A<a href="Alpha.">^a</a>.</p>
````````````````````````````````

With [inline footnotes](inline-footnotes.html), inline footnotes and other footnotes share one sequence:

```````````````````````````````` example -e footnotes -e inline-footnotes
A^[Inline.] and B[^b].

[^b]: Labelled.
.
<p>A<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup> and B<sup class="footnote-ref"><a href="#fn-b" id="fnref-b" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>Inline. <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-b">
<p>Labelled. <a href="#fnref-b" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

With [superscript](superscript.html), a footnote reference's `^` doesn't begin superscript:

```````````````````````````````` example -e footnotes -e superscript
Here is a footnote reference.[^1]

Here is a longer footnote reference.[^ref]

e = mc^2^.

[^1]: Here is the footnote.
[^ref]: Here is another footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<p>Here is a longer footnote reference.<sup class="footnote-ref"><a href="#fn-ref" id="fnref-ref" data-footnote-ref>2</a></sup></p>
<p>e = mc<sup>2</sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Here is the footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-ref">
<p>Here is another footnote. <a href="#fnref-ref" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

Footnote references work in [table](table.html) cells:

```````````````````````````````` example -e footnotes -e table
A footnote in a paragraph[^1]

| Column1   | Column2 |
| --------- | ------- |
| foot [^1] | note    |

[^1]: a footnote
.
<p>A footnote in a paragraph<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<table>
<thead>
<tr>
<th>Column1</th>
<th>Column2</th>
</tr>
</thead>
<tbody>
<tr>
<td>foot <sup class="footnote-ref"><a href="#fn-1" id="fnref-1-2" data-footnote-ref>1</a></sup></td>
<td>note</td>
</tr>
</tbody>
</table>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>a footnote <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-1-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, uses the same footnote design as Comrak.  It doesn't support the [inline footnotes](inline-footnotes.html) or [superscript](superscript.html) extensions, so those examples aren't comparable.  On every other example above, the two implementations agree on HTML output, except that cmark-gfm:

* numbers footnotes in the order their first references appear in the source, including references inside footnote definitions wherever those definitions are, such that a footnote referred to from a definition near the top of the document can come first ([example 23](#example-23));
* lists every footnote with a reference anywhere in the document, even one whose only reference is in a footnote definition that's dropped, such that its backlink leads nowhere ([example 24](#example-24));
* outputs the source text of brackets whose label matches no footnote definition, without parsing their content ([example 7](#example-7), [example 18](#example-18), [example 35](#example-35));
* ends a footnote definition at a blank line inside a container such as a block quote, or at a line containing only spaces ([example 16](#example-16)); and,
* gives footnote references and definitions no source position ([example 27](#example-27)).

cmark-gfm's XML output doesn't support footnotes.

## Regressions

The following examples guard against regressions.

A label with an emoji ([#306](https://github.com/kivikakk/comrak/issues/306)):

```````````````````````````````` example -e footnotes
Here is a footnote reference.[^😄ref]

[^😄ref]: Here is the footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-%F0%9F%98%84ref" id="fnref-%F0%9F%98%84ref" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-%F0%9F%98%84ref">
<p>Here is the footnote. <a href="#fnref-%F0%9F%98%84ref" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

A label that inline parsing splits into several text nodes ([#311](https://github.com/kivikakk/comrak/pull/311)):

```````````````````````````````` example -e footnotes
Foo.[^_ab]

[^_ab]: Here is the footnote.
.
<p>Foo.<sup class="footnote-ref"><a href="#fn-_ab" id="fnref-_ab" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-_ab">
<p>Here is the footnote. <a href="#fnref-_ab" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

An exclamation mark directly before a footnote reference ([#43](https://github.com/kivikakk/comrak/issues/43)):

```````````````````````````````` example -e footnotes
Here's my footnote![^a]

[^a]: Yep.
.
<p>Here's my footnote!<sup class="footnote-ref"><a href="#fn-a" id="fnref-a" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-a">
<p>Yep. <a href="#fnref-a" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

Footnote-like brackets with emphasis and strikethrough delimiters, and no footnote definitions (cmark-gfm [d3a819c](https://github.com/github/cmark-gfm/commit/d3a819c)):

```````````````````````````````` example -e footnotes -e table -e strikethrough -e autolink
|Tot.....[^_a_]|

[^~~is~~1]
.
<p>|Tot.....[^<em>a</em>]|</p>
<p>[^<del>is</del>1]</p>
````````````````````````````````

A document combining several of the cases above, from Comrak's original footnote tests:

```````````````````````````````` example -e footnotes
Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]

This is another note.[^note] And footnote[^longnote] is referenced again.

[^note]: Hi.

[^1]: Here is the footnote.

[^longnote]: Here's one with multiple blocks.

    Subsequent paragraphs are indented.

        code

This is regular content.

[^unused]: This is not used.
.
<p>Here is a[^nowhere] footnote reference,<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup> and another.<sup class="footnote-ref"><a href="#fn-longnote" id="fnref-longnote" data-footnote-ref>2</a></sup></p>
<p>This is another note.<sup class="footnote-ref"><a href="#fn-note" id="fnref-note" data-footnote-ref>3</a></sup> And footnote<sup class="footnote-ref"><a href="#fn-longnote" id="fnref-longnote-2" data-footnote-ref>2</a></sup> is referenced again.</p>
<p>This is regular content.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Here is the footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-longnote">
<p>Here's one with multiple blocks.</p>
<p>Subsequent paragraphs are indented.</p>
<pre class="syntax-highlighting"><code><span class="text plain">code
</span></code></pre>
<a href="#fnref-longnote" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a> <a href="#fnref-longnote-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2-2" aria-label="Back to reference 2-2">↩<sup class="footnote-ref">2</sup></a>
</li>
<li id="fn-note">
<p>Hi. <a href="#fnref-note" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="3" aria-label="Back to reference 3">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[inline link]: https://spec.commonmark.org/0.31.2/#inline-link
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[line ending]: https://spec.commonmark.org/0.31.2/#line-ending
[link reference definition]: https://spec.commonmark.org/0.31.2/#link-reference-definition
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[space]: https://spec.commonmark.org/0.31.2/#space
[tab]: https://spec.commonmark.org/0.31.2/#tab
