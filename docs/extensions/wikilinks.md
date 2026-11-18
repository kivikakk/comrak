---
title: Wikilinks
extensions: [wikilinks-title-after-pipe, wikilinks-title-before-pipe]
---

A wikilink is a link written the way wikis write them: a page name between double brackets, with an optional title to show in its place.  Use wikilinks to link between the pages of a wiki or a collection of notes, where a page's name is enough to find it.

Comrak's wikilinks follow the wikilinks extensions of [commonmark-hs](https://github.com/jgm/commonmark-hs/tree/master/commonmark-extensions), which [Pandoc](https://pandoc.org/MANUAL.html#wikilinks) also uses.  Those extensions come in two flavours, which differ in whether a wikilink's title comes after its page name or before it.  This document describes both, and [Differences from commonmark-hs](#differences-from-commonmark-hs) lists where Comrak parts ways.  GitLab uses Comrak's wikilinks, with the title before the page name, so this document is also the canonical spec for [GitLab Flavored Markdown wikilinks](https://docs.gitlab.com/user/markdown/#gitlab-specific-references).

To enable wikilinks with the title after the page name, pass `-e wikilinks-title-after-pipe` on the command line, or set `extension.wikilinks_title_after_pipe` in the options struct.  For the title before the page name, pass `-e wikilinks-title-before-pipe`, or set `extension.wikilinks_title_before_pipe`.  `--gfm` doesn't enable either.

Here's a typical wikilink, in each flavour:

```````````````````````````````` example -e wikilinks-title-after-pipe
See [[Home]], or [[how-to-use-comrak|How to use Comrak]].
.
<p>See <a href="Home" data-wikilink="true">Home</a>, or <a href="how-to-use-comrak" data-wikilink="true">How to use Comrak</a>.</p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-before-pipe
See [[Home]], or [[How to use Comrak|how-to-use-comrak]].
.
<p>See <a href="Home" data-wikilink="true">Home</a>, or <a href="how-to-use-comrak" data-wikilink="true">How to use Comrak</a>.</p>
````````````````````````````````

## Syntax

A [wikilink](@) is `[[`, followed by one or two [wikilink parts][wikilink part], followed by `]]`.  Two parts are separated by `|`.

A wikilink has a [destination](@), which is the page name or URL it links to, and a [title](@), which is the text it shows.  A wikilink with one part uses that part as both:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home]] and [[https://example.org]]
.
<p><a href="Home" data-wikilink="true">Home</a> and <a href="https://example.org" data-wikilink="true">https://example.org</a></p>
````````````````````````````````

### Title order

A wikilink with two parts uses one as its destination and the other as its title.  With `wikilinks-title-after-pipe`, the destination comes first:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home|Start here]]
.
<p><a href="Home" data-wikilink="true">Start here</a></p>
````````````````````````````````

With `wikilinks-title-before-pipe`, the title comes first:

```````````````````````````````` example -e wikilinks-title-before-pipe
[[Start here|Home]]
.
<p><a href="Home" data-wikilink="true">Start here</a></p>
````````````````````````````````

If both extensions are enabled, `wikilinks-title-after-pipe` takes precedence:

```````````````````````````````` example -e wikilinks-title-after-pipe -e wikilinks-title-before-pipe
[[Home|Start here]]
.
<p><a href="Home" data-wikilink="true">Start here</a></p>
````````````````````````````````

The rest of this document uses `wikilinks-title-after-pipe`, except where the order matters.

### Wikilink parts

A [wikilink part](@) is a sequence of characters other than `[`, `]`, and `|`, at most 1,000 bytes long.  A [backslash escape] may include any of those three characters in a part.  Spaces and tabs at either end of a part are ignored:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[ Home ]] and [[ Home | Start here ]]
.
<p><a href="Home" data-wikilink="true">Home</a> and <a href="Home" data-wikilink="true">Start here</a></p>
````````````````````````````````

A part may be empty:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[]] and [[Home|]] and [[|Start here]]
.
<p><a href="" data-wikilink="true"></a> and <a href="Home" data-wikilink="true"></a> and <a href="" data-wikilink="true">Start here</a></p>
````````````````````````````````

A wikilink can't have more than two parts.  To use `|` in a part, escape it:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home|Start|here]] and [[Home|Start\|here]]
.
<p>[[Home|Start|here]] and <a href="Home" data-wikilink="true">Start|here</a></p>
````````````````````````````````

Brackets must be escaped too.  None of these are wikilinks, except the last:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Ho[me]] and [[Ho]me]] and [[Ho\[me\]]]
.
<p>[[Ho[me]] and [[Ho]me]] and <a href="Ho%5Bme%5D" data-wikilink="true">Ho[me]</a></p>
````````````````````````````````

A wikilink may span lines, but not [paragraphs][paragraph]:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home|Start
here]]
.
<p><a href="Home" data-wikilink="true">Start
here</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home

page]]
.
<p>[[Home</p>
<p>page]]</p>
````````````````````````````````

### Destinations and titles

A wikilink's destination and title are both plain text.  [Backslash escapes][backslash escape] and [entity references][entity reference] are interpreted in both, but no other inline syntax is:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Gesch&uuml;tztes Leerzeichen|&#xDC;ber &amp;nbsp;]]
.
<p><a href="Gesch%C3%BCtztes%20Leerzeichen" data-wikilink="true">Über &amp;nbsp;</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Name _of_ page|*Start* `here`]]
.
<p><a href="Name%20_of_%20page" data-wikilink="true">*Start* `here`</a></p>
````````````````````````````````

A [line ending] in a destination becomes a space.  A line ending in a title stays as it is:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home
page]]
.
<p><a href="Home%20page" data-wikilink="true">Home
page</a></p>
````````````````````````````````

### Precedence

Comrak recognises a wikilink when inline parsing reaches its first `[`, so a [code span] that begins earlier takes precedence:

```````````````````````````````` example -e wikilinks-title-after-pipe
`[[Home]]`
.
<p><code>[[Home]]</code></p>
````````````````````````````````

A code span that would begin inside a wikilink doesn't:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Ho`me]] and`
.
<p><a href="Ho%60me" data-wikilink="true">Ho`me</a> and`</p>
````````````````````````````````

A wikilink takes precedence over a [link][links] beginning at the same bracket, and over a [link reference definition] with the same label:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home]](/url)
.
<p><a href="Home" data-wikilink="true">Home</a>(/url)</p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home]]

[Home]: /url
.
<p><a href="Home" data-wikilink="true">Home</a></p>
````````````````````````````````

A backslash escape before the first `[` prevents a wikilink:

```````````````````````````````` example -e wikilinks-title-after-pipe
\[[Home]]
.
<p>[[Home]]</p>
````````````````````````````````

A wikilink may follow brackets that aren't part of a link:

```````````````````````````````` example -e wikilinks-title-after-pipe
[see [[Home]] and [[[Home]]]
.
<p>[see <a href="Home" data-wikilink="true">Home</a> and [<a href="Home" data-wikilink="true">Home</a>]</p>
````````````````````````````````

Links can't contain other links.  A wikilink is a link, so once Comrak recognises one, the brackets before it can't form a link:

```````````````````````````````` example -e wikilinks-title-after-pipe
[see [[Home]] here](/url)
.
<p>[see <a href="Home" data-wikilink="true">Home</a> here](/url)</p>
````````````````````````````````

An [image description][images] may contain a wikilink, as it may contain a link:

```````````````````````````````` example -e wikilinks-title-after-pipe
![see [[Home]] here](/url)
.
<p><img src="/url" alt="see Home here" /></p>
````````````````````````````````

### Embeds

A wikilink directly after `!` is an [embed](@).  An embed is meant to show the content it links to, such as an image or another page, in place.  What can be embedded, and how, is up to whatever consumes Comrak's output, so HTML output renders an embed as an ordinary wikilink:

```````````````````````````````` example -e wikilinks-title-after-pipe
![[photo.png]] and ![[photo.png|A photo]]
.
<p><a href="photo.png" data-wikilink="true">photo.png</a> and <a href="photo.png" data-wikilink="true">A photo</a></p>
````````````````````````````````

A backslash escape before the `!` prevents an embed, but not the wikilink:

```````````````````````````````` example -e wikilinks-title-after-pipe
\![[photo.png]]
.
<p>!<a href="photo.png" data-wikilink="true">photo.png</a></p>
````````````````````````````````

With `extension.wikilinks_ignore_embeds`, Comrak doesn't recognise embeds.  `![` begins an [image][images] as it does in CommonMark, and the brackets after it can't form a wikilink.  The option has no command-line flag yet.

## Output

HTML output renders a wikilink as an `<a>` element with the attribute `data-wikilink="true"`.  Its `href` attribute is the destination, percent-encoded, and its content is the title.

As with links, Comrak removes destinations that could run script, such as those beginning with `javascript:`, unless `--unsafe` is given:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[javascript:alert(1)|Click me]]
.
<p><a href="" data-wikilink="true">Click me</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-after-pipe --unsafe
[[javascript:alert(1)|Click me]]
.
<p><a href="javascript:alert(1)" data-wikilink="true">Click me</a></p>
````````````````````````````````

CommonMark output writes a wikilink with one part if its title is the same as its destination, and with two parts in the enabled order otherwise.  It escapes `\`, `[`, `]`, and `|` in each part with a backslash, and writes a `&` that could begin an entity reference as `&amp;`:

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home page]] and [[Ho\|me|Start\]here]] and [[R&amp;D]]
.
<p><a href="Home%20page" data-wikilink="true">Home page</a> and <a href="Ho%7Cme" data-wikilink="true">Start]here</a> and <a href="R&amp;D" data-wikilink="true">R&amp;D</a></p>
````````````````````````````````

In XML output, a wikilink is a `wikilink` element with a `destination` attribute, and an `embed` attribute that's `true` for an embed and `false` otherwise.

In the AST, a wikilink is a [`NodeValue::WikiLink`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.WikiLink).  Its [`NodeWikiLink`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeWikiLink.html) holds the destination (`url`), and whether it's an embed (`embed`), with the title as its children.

A wikilink's source position covers its brackets, including an embed's `!`:

```````````````````````````````` example -e wikilinks-title-after-pipe --sourcepos
[[Home|Start here]] and ![[photo.png]]
.
<p data-sourcepos="1:1-1:38"><a data-sourcepos="1:1-1:19" href="Home" data-wikilink="true">Start here</a> and <a data-sourcepos="1:25-1:38" href="photo.png" data-wikilink="true">photo.png</a></p>
````````````````````````````````

## Interactions

`--gfm` doesn't enable wikilinks:

```````````````````````````````` example --gfm
[[Home]]
.
<p>[[Home]]</p>
````````````````````````````````

With [tables](table.html), a `|` in a table row separates cells, even inside a wikilink.  Escape it to use two parts:

```````````````````````````````` example -e wikilinks-title-after-pipe -e table
| Page |
| ---- |
| [[Home\|Start here]] |
| [[Home|Start here]] |
.
<table>
<thead>
<tr>
<th>Page</th>
</tr>
</thead>
<tbody>
<tr>
<td><a href="Home" data-wikilink="true">Start here</a></td>
</tr>
<tr>
<td>[[Home</td>
</tr>
</tbody>
</table>
````````````````````````````````

With [autolinks](autolink.html), a wikilink takes precedence over an autolink inside it:

```````````````````````````````` example -e wikilinks-title-after-pipe -e autolink
[[https://example.org]] and [[Home|www.example.org]]
.
<p><a href="https://example.org" data-wikilink="true">https://example.org</a> and <a href="Home" data-wikilink="true">www.example.org</a></p>
````````````````````````````````

With [footnotes](footnotes.html), a footnote reference has one `[`, so `[[^1]]` is a wikilink:

```````````````````````````````` example -e wikilinks-title-after-pipe -e footnotes
[[^1]] and [^1]

[^1]: A footnote.
.
<p><a href="%5E1" data-wikilink="true">^1</a> and <sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>A footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

With [task lists](tasklist.html), a wikilink at the start of a list item isn't a task list item marker:

```````````````````````````````` example -e wikilinks-title-after-pipe -e tasklist
- [[x]] Home
- [x] [[Home]]
.
<ul>
<li><a href="x" data-wikilink="true">x</a> Home</li>
<li><input type="checkbox" checked="" disabled="" /> <a href="Home" data-wikilink="true">Home</a></li>
</ul>
````````````````````````````````

With `--escaped-char-spans`, backslash escapes in a title are rendered as they are elsewhere:

```````````````````````````````` example -e wikilinks-title-after-pipe --escaped-char-spans
[[Home|Start\|here]]
.
<p><a href="Home" data-wikilink="true">Start<span data-escaped-char>|</span>here</a></p>
````````````````````````````````

With `--smart`, a title's punctuation stays as it is, since a title is plain text:

```````````````````````````````` example -e wikilinks-title-after-pipe --smart
[[Home|"Start" -- here]]
.
<p><a href="Home" data-wikilink="true">&quot;Start&quot; -- here</a></p>
````````````````````````````````

## Differences from commonmark-hs

[Pandoc](https://pandoc.org) uses commonmark-hs's wikilinks extensions, so the comparison here is with Pandoc 3.7.  Its other extensions and options differ from Comrak's, so the examples under [Interactions](#interactions), and those with source positions, aren't comparable.  On every other example above, Pandoc and Comrak agree on HTML output, except that Pandoc:

* splits a wikilink at its first `|`, so a title may contain `|` ([example 9](#example-9));
* allows `[` in a part ([example 10](#example-10));
* treats a backslash in a wikilink as literal text, so a backslash escape can't include `|`, `[`, or `]` in a part ([example 9](#example-9), [example 10](#example-10), [example 28](#example-28), [example 37](#example-37));
* keeps a line ending in a destination as it is ([example 15](#example-15));
* keeps a link that contains a wikilink, and renders the wikilink as a `<span>` element instead ([example 22](#example-22));
* has no embeds, so a `!` before a wikilink is literal text ([example 24](#example-24)); and,
* doesn't remove destinations that could run script ([example 26](#example-26)).

Pandoc also has no limit on a part's length.  Its HTML output gives a wikilink the class `wikilink`, rather than a `data-wikilink` attribute, and doesn't percent-encode the destination.

## Regressions

The following examples guard against regressions.

A title with backslash escapes is a single text node ([#575](https://github.com/kivikakk/comrak/pull/575)):

```````````````````````````````` example -e wikilinks-title-after-pipe
[[Home|Start\*here\*]]
.
<p><a href="Home" data-wikilink="true">Start*here*</a></p>
````````````````````````````````

Source positions of a title with entity references that spans lines ([#908](https://github.com/kivikakk/comrak/pull/908)):

```````````````````````````````` example -e wikilinks-title-after-pipe --sourcepos
[[Home|St&auml;rt
&amp; here]]
.
<p data-sourcepos="1:1-2:12"><a data-sourcepos="1:1-2:12" href="Home" data-wikilink="true">Stärt
&amp; here</a></p>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[images]: https://spec.commonmark.org/0.31.2/#images
[line ending]: https://spec.commonmark.org/0.31.2/#line-ending
[link reference definition]: https://spec.commonmark.org/0.31.2/#link-reference-definition
[links]: https://spec.commonmark.org/0.31.2/#links
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
