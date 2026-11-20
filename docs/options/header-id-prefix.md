---
title: Header IDs
---

`--header-id-prefix PREFIX` (`extension.header_id_prefix`) adds an `id` to each heading, made from the prefix and the heading's text, and puts a link to that `id` inside the heading.  Readers can then link straight to a section of a document.

Without the option, headings don't have an `id`:

```````````````````````````````` example
# README
.
<h1>README</h1>
````````````````````````````````

With the option, the heading has an `id`, and ends with a link to it:

```````````````````````````````` example --header-id-prefix user-content-
# README
.
<h1 id="user-content-readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

The prefix keeps headings' `id`s apart from any other `id`s on the page that shows the document.  A common choice is `user-content-`.  The prefix may be empty:

```````````````````````````````` example --header-id-prefix ""
# README
.
<h1 id="readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

## Slugs

A heading's [text content](@) is the text it contains, without its markup.  The text content includes the text of code spans and links, and an image's description.  It doesn't include raw HTML:

```````````````````````````````` example --header-id-prefix "" --unsafe
# The `foo` method
# *Emphasis* and [a link](/url)
# ![logo](x.png) Title
# <span>Raw</span> HTML
.
<h1 id="the-foo-method">The <code>foo</code> method<a href="#the-foo-method" aria-label="Link to heading 'The foo method'" data-heading-content="The foo method" class="anchor"></a></h1>
<h1 id="emphasis-and-a-link"><em>Emphasis</em> and <a href="/url">a link</a><a href="#emphasis-and-a-link" aria-label="Link to heading 'Emphasis and a link'" data-heading-content="Emphasis and a link" class="anchor"></a></h1>
<h1 id="logo-title"><img src="x.png" alt="logo" /> Title<a href="#logo-title" aria-label="Link to heading 'logo Title'" data-heading-content="logo Title" class="anchor"></a></h1>
<h1 id="raw-html"><span>Raw</span> HTML<a href="#raw-html" aria-label="Link to heading 'Raw HTML'" data-heading-content="Raw HTML" class="anchor"></a></h1>
````````````````````````````````

A heading's [slug](@) is its [text content] in lowercase, keeping only letters, marks, numbers, connector punctuation such as `_`, hyphens, and spaces.  Each space then becomes a hyphen:

```````````````````````````````` example --header-id-prefix ""
# Isn't it grand?
# a.b,c!d
# snake_case and kebab-case
# C++ & C#
.
<h1 id="isnt-it-grand">Isn't it grand?<a href="#isnt-it-grand" aria-label="Link to heading 'Isn't it grand?'" data-heading-content="Isn't it grand?" class="anchor"></a></h1>
<h1 id="abcd">a.b,c!d<a href="#abcd" aria-label="Link to heading 'a.b,c!d'" data-heading-content="a.b,c!d" class="anchor"></a></h1>
<h1 id="snake_case-and-kebab-case">snake_case and kebab-case<a href="#snake_case-and-kebab-case" aria-label="Link to heading 'snake_case and kebab-case'" data-heading-content="snake_case and kebab-case" class="anchor"></a></h1>
<h1 id="c--c">C++ &amp; C#<a href="#c--c" aria-label="Link to heading 'C++ &amp; C#'" data-heading-content="C++ &amp; C#" class="anchor"></a></h1>
````````````````````````````````

Every space becomes a hyphen of its own, so a run of spaces becomes a run of hyphens.  A [tab] is removed:

```````````````````````````````` example --header-id-prefix ""
# a - b
# a  b
# a→b
.
<h1 id="a---b">a - b<a href="#a---b" aria-label="Link to heading 'a - b'" data-heading-content="a - b" class="anchor"></a></h1>
<h1 id="a--b">a  b<a href="#a--b" aria-label="Link to heading 'a  b'" data-heading-content="a  b" class="anchor"></a></h1>
<h1 id="ab">a→b<a href="#ab" aria-label="Link to heading 'a→b'" data-heading-content="a→b" class="anchor"></a></h1>
````````````````````````````````

Letters and numbers in any script are kept, including numbers that aren't digits, such as `²` and `½`.  Lowercasing follows Unicode, and can lengthen a letter: `İ` becomes `i` followed by a combining dot:

```````````````````````````````` example --header-id-prefix ""
# Ünïcödé Straße
# 日本語の見出し
# İstanbul
# x² and ½
.
<h1 id="ünïcödé-straße">Ünïcödé Straße<a href="#ünïcödé-straße" aria-label="Link to heading 'Ünïcödé Straße'" data-heading-content="Ünïcödé Straße" class="anchor"></a></h1>
<h1 id="日本語の見出し">日本語の見出し<a href="#日本語の見出し" aria-label="Link to heading '日本語の見出し'" data-heading-content="日本語の見出し" class="anchor"></a></h1>
<h1 id="i̇stanbul">İstanbul<a href="#i̇stanbul" aria-label="Link to heading 'İstanbul'" data-heading-content="İstanbul" class="anchor"></a></h1>
<h1 id="x²-and-½">x² and ½<a href="#x²-and-½" aria-label="Link to heading 'x² and ½'" data-heading-content="x² and ½" class="anchor"></a></h1>
````````````````````````````````

Symbols are removed, including emoji, and symbols that look like letters, such as `Ⓐ`.  A variation selector after an emoji is a mark, though, and remains in the slug.  Here the slug of `✔️ Done` starts with an invisible U+FE0F:

```````````````````````````````` example --header-id-prefix ""
# 🎉 Party
# Ⓐ circled
# ✔️ Done
.
<h1 id="-party">🎉 Party<a href="#-party" aria-label="Link to heading '🎉 Party'" data-heading-content="🎉 Party" class="anchor"></a></h1>
<h1 id="-circled">Ⓐ circled<a href="#-circled" aria-label="Link to heading 'Ⓐ circled'" data-heading-content="Ⓐ circled" class="anchor"></a></h1>
<h1 id="️-done">✔️ Done<a href="#️-done" aria-label="Link to heading '✔️ Done'" data-heading-content="✔️ Done" class="anchor"></a></h1>
````````````````````````````````

A line ending inside a heading becomes a space:

```````````````````````````````` example --header-id-prefix ""
Multi-line
heading
=======
.
<h1 id="multi-line-heading">Multi-line
heading<a href="#multi-line-heading" aria-label="Link to heading 'Multi-line heading'" data-heading-content="Multi-line heading" class="anchor"></a></h1>
````````````````````````````````

## Header IDs

A heading's [header ID](@) is the prefix, followed by the heading's [slug] made unique within the document.  The first heading with a given slug uses the slug unchanged.  Later ones append `-1`, `-2`, and so on, skipping any slug that's already taken:

```````````````````````````````` example --header-id-prefix ""
# a
# a
# a-1
# a
# A
.
<h1 id="a">a<a href="#a" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
<h1 id="a-1">a<a href="#a-1" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
<h1 id="a-1-1">a-1<a href="#a-1-1" aria-label="Link to heading 'a-1'" data-heading-content="a-1" class="anchor"></a></h1>
<h1 id="a-2">a<a href="#a-2" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
<h1 id="a-3">A<a href="#a-3" aria-label="Link to heading 'A'" data-heading-content="A" class="anchor"></a></h1>
````````````````````````````````

Comrak numbers headings in the order it writes them.  A heading in a [footnote](../extensions/footnotes.html) definition comes after the rest of the document's headings, since footnotes appear at the end:

```````````````````````````````` example --header-id-prefix "" -e footnotes
Hi[^1].

[^1]: # Note

# Note
.
<p>Hi<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup>.</p>
<h1 id="note">Note<a href="#note" aria-label="Link to heading 'Note'" data-heading-content="Note" class="anchor"></a></h1>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<h1 id="note-1">Note<a href="#note-1" aria-label="Link to heading 'Note'" data-heading-content="Note" class="anchor"></a></h1>
<a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a>
</li>
</ol>
</section>
````````````````````````````````

The slug in a [header ID] is never empty.  A heading whose [text content] doesn't include any of the characters a slug keeps gets `-1`, then `-2`, and so on:

```````````````````````````````` example --header-id-prefix ""
#
# !!!
.
<h1 id="-1"><a href="#-1" aria-label="Link to heading ''" data-heading-content="" class="anchor"></a></h1>
<h1 id="-2">!!!<a href="#-2" aria-label="Link to heading '!!!'" data-heading-content="!!!" class="anchor"></a></h1>
````````````````````````````````

A header ID also avoids every other `id` that Comrak writes in the document: footnotes' `id`s, [block IDs](../extensions/ofm-block-ids.html), and `id`s set by [attributes](../extensions/attributes.html).  Only an `id` that begins with the prefix can match a header ID, so with a prefix like `user-content-` this rarely matters.  With an empty prefix, a heading can match a footnote's `id`:

```````````````````````````````` example --header-id-prefix "" -e footnotes
# fn-1

Hi[^1].

[^1]: Note.
.
<h1 id="fn-1-1">fn-1<a href="#fn-1-1" aria-label="Link to heading 'fn-1'" data-heading-content="fn-1" class="anchor"></a></h1>
<p>Hi<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Note. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

## Output

In HTML output, a heading's element has its header ID as its `id`.  Before the closing tag, Comrak adds an empty `<a>` element with the class `anchor`, and these attributes:

* `href`, which links to the heading as `#` followed by the unique slug, without the prefix (but see [Header ID prefix in href](#header-id-prefix-in-href));
* `aria-label`, which is `Link to heading '…'` with the heading's [text content]; and,
* `data-heading-content`, which is the heading's text content.

```````````````````````````````` example --header-id-prefix user-content-
# Say "hi" & 'bye'
.
<h1 id="user-content-say-hi--bye">Say &quot;hi&quot; &amp; 'bye'<a href="#say-hi--bye" aria-label="Link to heading 'Say &quot;hi&quot; &amp; 'bye''" data-heading-content="Say &quot;hi&quot; &amp; 'bye'" class="anchor"></a></h1>
````````````````````````````````

CommonMark and XML output don't include [header IDs][header ID].

A [heading adapter](https://docs.rs/comrak/latest/comrak/adapters/trait.HeadingAdapter.html) writes the whole of a heading's HTML, so headings it renders don't get header IDs.

GitLab writes headings with its own renderer, which uses the same slugs.

## Header ID prefix in href

`--header-id-prefix-in-href` (`extension.header_id_prefix_in_href`) adds the prefix to the link's `href` too, so that the link points at the heading's `id`.  The option has no effect unless `--header-id-prefix` is set.

Without the option, the `href` doesn't include the prefix:

```````````````````````````````` example --header-id-prefix user-content-
# README
.
<h1 id="user-content-readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

With the option, it does:

```````````````````````````````` example --header-id-prefix user-content- --header-id-prefix-in-href
# README
.
<h1 id="user-content-readme">README<a href="#user-content-readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

Without a prefix, the option doesn't change anything:

```````````````````````````````` example --header-id-prefix-in-href
# README
.
<h1>README</h1>
````````````````````````````````

## Interactions

`--gfm` doesn't set a prefix.

With [header attributes](../extensions/attributes.html#example-37), an `id` attribute replaces a heading's [slug], and takes the prefix like any other [header ID].  Other headings' slugs avoid it.

An `id` attribute on any other element is written as it is, without the prefix.  Header IDs avoid it too:

```````````````````````````````` example --header-id-prefix "" -e inline-code-attributes --attributes --unsafe
`x`{#a}

# A
.
<p><code id="a">x</code></p>
<h1 id="a-1">A<a href="#a-1" aria-label="Link to heading 'A'" data-heading-content="A" class="anchor"></a></h1>
````````````````````````````````

With [smart punctuation](smart.html), Comrak transforms the heading's text before it makes the slug.  Without `--smart`, this heading's slug is `quotes----dash`:

```````````````````````````````` example --header-id-prefix "" --smart
# "Quotes" -- dash...
.
<h1 id="quotes--dash">“Quotes” – dash…<a href="#quotes--dash" aria-label="Link to heading '“Quotes” – dash…'" data-heading-content="“Quotes” – dash…" class="anchor"></a></h1>
````````````````````````````````

With [Gemoji shortcodes](gemoji.html), a shortcode becomes an emoji, which the slug removes:

```````````````````````````````` example --header-id-prefix "" --gemoji
# :tada: Party
.
<h1 id="-party">🎉 Party<a href="#-party" aria-label="Link to heading ' Party'" data-heading-content=" Party" class="anchor"></a></h1>
````````````````````````````````

With [maths](../extensions/math.html), the text content includes a formula's source:

```````````````````````````````` example --header-id-prefix "" -e math-dollars
# Area $\pi r^2$
.
<h1 id="area-pi-r2">Area <span data-math-style="inline">\pi r^2</span><a href="#area-pi-r2" aria-label="Link to heading 'Area \pi r^2'" data-heading-content="Area \pi r^2" class="anchor"></a></h1>
````````````````````````````````

With [block IDs](../extensions/ofm-block-ids.html), a heading takes its header ID instead of its block ID.

With [source positions](sourcepos.html#output), the heading's element has a `data-sourcepos` attribute, and the `<a>` that Comrak adds doesn't.

## Differences from github.com

github.com adds header IDs to the Markdown it renders, with the prefix `user-content-`.  Comrak's slugs and suffixes follow the same rules, except in these cases:

* github.com keeps symbols that Unicode counts as alphabetic, such as `Ⓐ`, and removes numbers that aren't decimal digits, such as `²` and `½` ([example 7](#example-7), [example 8](#example-8));
* github.com keeps the zero-width joiner and zero-width non-joiner, which Comrak removes;
* github.com leaves an image's description out of the text content, so the slug of the third heading in [example 4](#example-4) is `-title`;
* github.com removes a line ending inside a heading, rather than making it a space, so the slug in [example 9](#example-9) is `multi-lineheading`;
* github.com writes an empty `id` for the first heading in [example 12](#example-12), and `-1` for the second, and writes no `id` at all for a third heading with an empty slug; and,
* with Gemoji shortcodes, github.com makes the slug from the shortcode, not the emoji, so the slug in [example 20](#example-20) is `tada-party`.

[tab]: https://spec.commonmark.org/0.31.2/#tab
