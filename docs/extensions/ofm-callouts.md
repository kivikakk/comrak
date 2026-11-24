---
title: OFM callouts
draft: true
---

An OFM callout is a block quote that calls out its content as a note, a warning, or another kind of information.  A marker on the callout's first line gives its type, and the callout can have a custom title, metadata, and an initial fold state.

OFM callouts originate in [Obsidian Flavored Markdown](https://help.obsidian.md/callouts).  They resemble [GFM alerts](alerts.html), but allow arbitrary types, Markdown titles, metadata, fold markers, and a lazy first body line.  This document describes Comrak's OFM callouts, which can differ from Obsidian's.

To enable OFM callouts, pass `-e ofm-callouts` on the command line, or set `extension.ofm_callouts` in the options struct.

Here's a typical callout:

```````````````````````````````` example -e ofm-callouts
> [!note]
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

## Syntax

A [callout header](@) is `[!`, followed by a non-empty payload and `]`.  The payload consists of a raw type and, optionally, `|` followed by metadata.  A `+` or `-` immediately after the `]` gives the callout's initial fold state.  The rest of the line, if any, is the callout's title and must be separated from the header by whitespace.

An [OFM callout](@) is a [block quote] whose first line begins with a [callout header].  The following lines are the callout's content, and continue the callout as they would a block quote.

### Callout headers

The header may follow the `>` directly, or after one ASCII space:

```````````````````````````````` example -e ofm-callouts
>[!note]
> No space.

> [!tip]
> One space.
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>No space.</p>
</div>
</div>
<div class="callout" data-callout="tip" data-callout-fold="none">
<div class="callout-title">Tip</div>
<div class="callout-content">
<p>One space.</p>
</div>
</div>
````````````````````````````````

More than one space, or a tab, prevents the line from being recognised as a callout header.  The header is stricter than the start of a paragraph in an ordinary block quote:

```````````````````````````````` example -e ofm-callouts
>  [!note]
> Two spaces.

>→[!tip]
> A tab.
.
<blockquote>
<p>[!note]
Two spaces.</p>
</blockquote>
<blockquote>
<p>[!tip]
A tab.</p>
</blockquote>
````````````````````````````````

The header may be indented up to three spaces before the `>`.  Four spaces make an [indented code block] instead:

```````````````````````````````` example -e ofm-callouts
   > [!note]
   > Three spaces before the marker.

    > [!note]
    > Four spaces before the marker.
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Three spaces before the marker.</p>
</div>
</div>
<pre class="syntax-highlighting"><code><span class="text plain">&gt; [!note]
&gt; Four spaces before the marker.
</span></code></pre>
````````````````````````````````

The payload must contain at least one character, and the header must have a closing `]`.  After the closing bracket and optional fold marker, only whitespace and a title may follow:

```````````````````````````````` example -e ofm-callouts
> [!]
> Empty payload.

> [!note
> No closing bracket.

> [!note]++
> Text immediately after the fold marker.

> [!note]Title
> No whitespace before the title.
.
<blockquote>
<p>[!]
Empty payload.</p>
</blockquote>
<blockquote>
<p>[!note
No closing bracket.</p>
</blockquote>
<blockquote>
<p>[!note]++
Text immediately after the fold marker.</p>
</blockquote>
<blockquote>
<p>[!note]Title
No whitespace before the title.</p>
</blockquote>
````````````````````````````````

The header must begin the block quote's first line.  Later in the block quote, it's just text:

```````````````````````````````` example -e ofm-callouts
> A block quote.
> [!note]
.
<blockquote>
<p>A block quote.
[!note]</p>
</blockquote>
````````````````````````````````

Likewise, only the first header in a callout counts:

```````````````````````````````` example -e ofm-callouts
> [!note]
> [!tip]
> Still a note.
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>[!tip]
Still a note.</p>
</div>
</div>
````````````````````````````````

The header must be written literally.  A [backslash escape] or an [entity reference] can't form part of it:

```````````````````````````````` example -e ofm-callouts
> \[!note]
> Escaped.

> &#91;!note]
> An entity reference.
.
<blockquote>
<p>[!note]
Escaped.</p>
</blockquote>
<blockquote>
<p>[!note]
An entity reference.</p>
</blockquote>
````````````````````````````````

Like a block quote, a callout may interrupt a paragraph:

```````````````````````````````` example -e ofm-callouts
A paragraph.
> [!note]
> A callout.
.
<p>A paragraph.</p>
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>A callout.</p>
</div>
</div>
````````````````````````````````

### Types

Comrak accepts any raw type; it doesn't restrict callouts to a fixed set of names.  It trims whitespace from both ends of the type, converts it to lower case, and replaces each run of whitespace with one hyphen.  Other characters are preserved:

```````````````````````````````` example -e ofm-callouts
> [!My Type]
> Body.

> [!  Custom   TYPE  ]
> Body.
.
<div class="callout" data-callout="my-type" data-callout-fold="none">
<div class="callout-title">My type</div>
<div class="callout-content">
<p>Body.</p>
</div>
</div>
<div class="callout" data-callout="custom-type" data-callout-fold="none">
<div class="callout-title">Custom type</div>
<div class="callout-content">
<p>Body.</p>
</div>
</div>
````````````````````````````````

The payload must be non-empty, but the raw type may consist entirely of whitespace and therefore normalise to an empty type:

```````````````````````````````` example -e ofm-callouts
> [! ]
> Body.
.
<div class="callout" data-callout="" data-callout-fold="none">
<div class="callout-title"></div>
<div class="callout-content">
<p>Body.</p>
</div>
</div>
````````````````````````````````

The normalised type determines the default title and the value of the HTML `data-callout` attribute.  Comrak doesn't assign colours, icons, or other presentation to particular types; a renderer or stylesheet can use the type as it chooses.

### Metadata

A `|` in the payload separates the raw type from its [callout metadata](@).  Metadata is preserved literally, without trimming or normalisation.  It may be empty and may contain further `|` characters:

```````````````````````````````` example -e ofm-callouts
> [!note|]
> Empty metadata.

> [!note|right|wide]
> More than one separator.
.
<div class="callout" data-callout="note" data-callout-metadata="" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Empty metadata.</p>
</div>
</div>
<div class="callout" data-callout="note" data-callout-metadata="right|wide" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>More than one separator.</p>
</div>
</div>
````````````````````````````````

HTML output writes metadata in the `data-callout-metadata` attribute.  Comrak records the metadata but doesn't interpret it.

### Folding

A `+` immediately after the closing `]` makes the callout initially expanded:

```````````````````````````````` example -e ofm-callouts
> [!note]+
> Body
.
<div class="callout" data-callout="note" data-callout-fold="expanded">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

A `-` makes it initially collapsed.  The fold marker follows any metadata's closing bracket:

```````````````````````````````` example -e ofm-callouts
> [!tip|right]- **Read**
> Body
.
<div class="callout" data-callout="tip" data-callout-metadata="right" data-callout-fold="collapsed">
<div class="callout-title">
<strong>Read</strong>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

Without either marker, the callout isn't foldable.  HTML output represents these states as `data-callout-fold="expanded"`, `"collapsed"`, and `"none"` respectively.  Comrak doesn't implement a folding user interface.

### Titles

If the header has no title, Comrak derives one from the normalised type: it replaces hyphens with spaces and uppercases the first character.  The examples above use this default title.

Any text after the header and its separating whitespace replaces the default title.  Unlike an [alert title](alerts.html#alert-titles), a callout title is parsed as Markdown:

```````````````````````````````` example -e ofm-callouts
> [!warning] **Bold** and *italic*
> Body
.
<div class="callout" data-callout="warning" data-callout-fold="none">
<div class="callout-title">
<strong>Bold</strong> and <em>italic</em>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

The title is parsed as a full block subtree, not just as inline content.  For example, an ATX heading on the header line becomes a heading in the title:

```````````````````````````````` example -e ofm-callouts
> [!note] # Heading
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
<h1>Heading</h1>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

The title is confined to the header line.  Inline markup doesn't continue between the title and the callout's content:

```````````````````````````````` example -e ofm-callouts
> [!note] *Title
> *Body*
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
*Title
</div>
<div class="callout-content">
<p><em>Body</em></p>
</div>
</div>
````````````````````````````````

### Content

A callout may contain any blocks, as a block quote may:

```````````````````````````````` example -e ofm-callouts
> [!warning]
> # Before you begin
>
> - Back up your data.
> - Read the release notes.
>
> ```
> make backup
> ```
.
<div class="callout" data-callout="warning" data-callout-fold="none">
<div class="callout-title">Warning</div>
<div class="callout-content">
<h1>Before you begin</h1>
<ul>
<li>Back up your data.</li>
<li>Read the release notes.</li>
</ul>
<pre class="syntax-highlighting"><code><span class="text plain">make backup
</span></code></pre>
</div>
</div>
````````````````````````````````

Unlike an alert, a callout allows a [lazy continuation line] immediately after its header, even when there is no body paragraph yet.  The line becomes the first paragraph of the content:

```````````````````````````````` example -e ofm-callouts
> [!note] Title
Lazy body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
Title
</div>
<div class="callout-content">
<p>Lazy body</p>
</div>
</div>
````````````````````````````````

A callout may be empty, or have only a title:

```````````````````````````````` example -e ofm-callouts
> [!note]

> [!tip] Just a title
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
</div>
</div>
<div class="callout" data-callout="tip" data-callout-fold="none">
<div class="callout-title">
Just a title
</div>
<div class="callout-content">
</div>
</div>
````````````````````````````````

A blank line ends a callout, as it ends a block quote:

```````````````````````````````` example -e ofm-callouts
> [!note]
> One callout.

> [!tip]
> Another callout.
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>One callout.</p>
</div>
</div>
<div class="callout" data-callout="tip" data-callout-fold="none">
<div class="callout-title">Tip</div>
<div class="callout-content">
<p>Another callout.</p>
</div>
</div>
````````````````````````````````

### Containers

A callout may appear anywhere a block quote may.  Here, a callout is in a list item:

```````````````````````````````` example -e ofm-callouts
- Item one

  > [!note]
  > In a list item.
.
<ul>
<li>
<p>Item one</p>
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>In a list item.</p>
</div>
</div>
</li>
</ul>
````````````````````````````````

Here, one callout is in a block quote, and another is nested in a callout:

```````````````````````````````` example -e ofm-callouts
> > [!note]
> > In a block quote.

> [!warning]
> > [!tip]
> > In a callout.
.
<blockquote>
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>In a block quote.</p>
</div>
</div>
</blockquote>
<div class="callout" data-callout="warning" data-callout-fold="none">
<div class="callout-title">Warning</div>
<div class="callout-content">
<div class="callout" data-callout="tip" data-callout-fold="none">
<div class="callout-title">Tip</div>
<div class="callout-content">
<p>In a callout.</p>
</div>
</div>
</div>
</div>
````````````````````````````````

## Output

HTML output renders a callout as a `<div>` element with class `callout`.  Its `data-callout` attribute holds the normalised type, its optional `data-callout-metadata` attribute holds the metadata, and its `data-callout-fold` attribute holds the fold state.  Inside it, a `<div>` with class `callout-title` holds the title, and a `<div>` with class `callout-content` holds the content:

```````````````````````````````` example -e ofm-callouts
> [!tip|right]- **Read**
> Body
.
<div class="callout" data-callout="tip" data-callout-metadata="right" data-callout-fold="collapsed">
<div class="callout-title">
<strong>Read</strong>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

CommonMark output writes the normalised type, metadata, and fold marker, followed by the custom title if there is one.  It writes the content with block quote markers.

In XML output, a callout is a `callout` element with `type`, optional `metadata`, and `fold` attributes.  A custom title is represented by a `callout_title` child; a default title has no separate XML node.

In the AST, a callout is a [`NodeValue::Callout`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Callout), with its content as children.  Its [`NodeCallout`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeCallout.html) holds the normalised type, optional metadata, and [`CalloutFold`](https://docs.rs/comrak/latest/comrak/nodes/enum.CalloutFold.html).  If the source has a custom title, the first child is a [`NodeValue::CalloutTitle`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.CalloutTitle) containing the title's parsed content.

A callout's [source position](../options/sourcepos.html#source-position) covers all of its lines.  A custom title has its own source position:

```````````````````````````````` example -e ofm-callouts --sourcepos
> [!note] Title
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none" data-sourcepos="1:1-2:6">
<div class="callout-title" data-sourcepos="1:11-1:15">
Title
</div>
<div class="callout-content">
<p data-sourcepos="2:3-2:6">Body</p>
</div>
</div>
````````````````````````````````

## Differences from alerts

Although callouts and [alerts](alerts.html) have similar markers, they are distinct extensions:

* **Types.**  Alerts have five case-insensitive types: `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, and `CAUTION`.  Callouts accept arbitrary types, so `info` and `note` remain distinct even if a stylesheet presents them alike.
* **Custom titles.**  An alert title is plain text, with backslash escapes and entity references decoded.  A callout title is a Markdown AST subtree, and can contain inline markup or a block such as a heading.
* **Folding.**  Alerts have no fold state.  A callout can be initially expanded or collapsed.
* **Metadata.**  Alerts have no metadata.  A callout can preserve metadata after `|`.
* **Title separation.**  An alert accepts `[!note]Title` and gives it the title `Title`.  A callout requires whitespace before the title: `[!note] Title`.
* **Block quote separation.**  Both accept `>[!note]`.  An alert also accepts spaces or tabs after `>` when the marker would begin a paragraph; a callout accepts at most one ASCII space.
* **Lazy content after the header.**  With an alert, an unmarked line immediately after the header is outside the alert.  With a callout, it becomes the first body paragraph.

For example, this is an alert followed by a separate paragraph:

```````````````````````````````` example -e alerts
> [!note]
Lazy body
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
</div>
<p>Lazy body</p>
````````````````````````````````

The equivalent callout keeps `Lazy body` in its content, as shown in [Content](#content).

When both extensions are enabled and both can parse the header, the callout takes precedence:

```````````````````````````````` example -e alerts -e ofm-callouts
> [!important]
> Body
.
<div class="callout" data-callout="important" data-callout-fold="none">
<div class="callout-title">Important</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

If the callout parser rejects the header, the alert parser can still recognise it.  Here, the missing whitespace before `Title` prevents a callout, so the input becomes an alert:

```````````````````````````````` example -e alerts -e ofm-callouts
> [!note]Title
> Body
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Title</p>
<p>Body</p>
</div>
````````````````````````````````

## Interactions

OFM callouts are disabled by default, and `--gfm` doesn't enable them:

```````````````````````````````` example --gfm
> [!note]
> A block quote.
.
<blockquote>
<p>[!note]
A block quote.</p>
</blockquote>
````````````````````````````````

With [OFM comments](ofm-comments.html), an inline comment can hide part or all of a callout title.  A title container is still present when its source contains a title, even if nothing remains visible:

```````````````````````````````` example -e ofm-callouts -e ofm-comments
> [!note] %%hidden%%
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-blocks
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
