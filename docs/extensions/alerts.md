---
title: Alerts
---

An alert is a block quote that calls out its content as a note, a warning, or similar.  A marker on the alert's first line gives its type, and Comrak renders the alert with a title and class names for that type.

Alerts originate on [GitHub](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts), which documents them but doesn't specify them.  This document describes Comrak's alerts, which can differ from GitHub's.  GitLab uses Comrak's alerts, so this document is also the canonical spec for [GitLab Flavored Markdown alerts](https://docs.gitlab.com/user/markdown/#alerts).

To enable alerts, pass `-e alerts` on the command line, or set `extension.alerts` in the options struct.  `--gfm` doesn't enable them.

Here's a typical alert:

```````````````````````````````` example -e alerts
> [!NOTE]
> Highlights information that users should take into account, even when skimming.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
````````````````````````````````

## Syntax

An [alert type](@) is `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, or `CAUTION`, in any mix of upper and lower case.  An [alert marker](@) is `[!`, followed by an [alert type], followed by `]`.

An [alert](@) is a [block quote] whose first line begins with an [alert marker].  The rest of that line is the alert's [title](#alert-titles).  The following lines are the alert's content, and continue the alert as they would a block quote.

### Alert markers

There are five alert types:

```````````````````````````````` example -e alerts
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>Optional information to help a user be more successful.</p>
</div>
<div class="markdown-alert markdown-alert-important">
<p class="markdown-alert-title">Important</p>
<p>Crucial information necessary for users to succeed.</p>
</div>
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Warning</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</div>
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<p>Negative potential consequences of an action.</p>
</div>
````````````````````````````````

Case doesn't matter:

```````````````````````````````` example -e alerts
> [!note]
> Lower case.

> [!Tip]
> Mixed case.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Lower case.</p>
</div>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>Mixed case.</p>
</div>
````````````````````````````````

Nothing else is an alert marker.  None of these are alerts:

```````````````````````````````` example -e alerts
> [!INFO]
> Not a type.

> [! NOTE]
> A space after the `!`.

> [NOTE]
> No `!`.
.
<blockquote>
<p>[!INFO]
Not a type.</p>
</blockquote>
<blockquote>
<p>[! NOTE]
A space after the <code>!</code>.</p>
</blockquote>
<blockquote>
<p>[NOTE]
No <code>!</code>.</p>
</blockquote>
````````````````````````````````

The alert marker may follow the `>` directly, or after spaces or tabs, as long as it would begin a [paragraph] in the block quote:

```````````````````````````````` example -e alerts
>[!NOTE]
> No space.

>  [!TIP]
> Two spaces.

>→[!IMPORTANT]
> A tab.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>No space.</p>
</div>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>Two spaces.</p>
</div>
<div class="markdown-alert markdown-alert-important">
<p class="markdown-alert-title">Important</p>
<p>A tab.</p>
</div>
````````````````````````````````

With five spaces after the `>`, the marker begins an [indented code block] instead:

```````````````````````````````` example -e alerts
>     [!NOTE]
> Not an alert.
.
<blockquote>
<pre class="syntax-highlighting"><code><span class="text plain">[!NOTE]
</span></code></pre>
<p>Not an alert.</p>
</blockquote>
````````````````````````````````

The alert marker must begin the block quote's first line.  Later in the block quote, it's just text:

```````````````````````````````` example -e alerts
> A block quote.
> [!NOTE]
.
<blockquote>
<p>A block quote.
[!NOTE]</p>
</blockquote>
````````````````````````````````

Likewise, only the first alert marker in an alert counts:

```````````````````````````````` example -e alerts
> [!NOTE]
> [!TIP]
> Still a note.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>[!TIP]
Still a note.</p>
</div>
````````````````````````````````

The alert marker must be written literally.  A [backslash escape] or an [entity reference] can't form part of it:

```````````````````````````````` example -e alerts
> \[!NOTE]
> Escaped.

> &#91;!NOTE]
> An entity reference.
.
<blockquote>
<p>[!NOTE]
Escaped.</p>
</blockquote>
<blockquote>
<p>[!NOTE]
An entity reference.</p>
</blockquote>
````````````````````````````````

Like a block quote, an alert may interrupt a paragraph:

```````````````````````````````` example -e alerts
A paragraph.
> [!NOTE]
> An alert.
.
<p>A paragraph.</p>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>An alert.</p>
</div>
````````````````````````````````

### Alert titles

The [alert title](@) is the rest of the alert's first line, after its [alert marker], with whitespace at either end removed.  If there's nothing left, the alert takes its type's default title: `Note`, `Tip`, `Important`, `Warning`, or `Caution`.  Otherwise, the alert title replaces the default:

```````````````````````````````` example -e alerts
> [!NOTE] Pay attention
> Highlights information that users should take into account, even when skimming.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Pay attention</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
````````````````````````````````

Trailing whitespace, including the two spaces that would otherwise make a [hard line break], doesn't give the alert a title:

```````````````````````````````` example -e alerts
> [!NOTE]  
> No title.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>No title.</p>
</div>
````````````````````````````````

Comrak decodes backslash escapes and entity references in an alert title, but doesn't parse it as Markdown otherwise:

```````````````````````````````` example -e alerts
> [!TIP] **Fish** &amp; \*chips\* <b>
> Plain text.
.
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">**Fish** &amp; *chips* &lt;b&gt;</p>
<p>Plain text.</p>
</div>
````````````````````````````````

### Content

An alert may contain any blocks, as a block quote may:

```````````````````````````````` example -e alerts
> [!WARNING]
> # Before you begin
>
> - Back up your data.
> - Read the release notes.
>
> ```
> make backup
> ```
.
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Warning</p>
<h1>Before you begin</h1>
<ul>
<li>Back up your data.</li>
<li>Read the release notes.</li>
</ul>
<pre class="syntax-highlighting"><code><span class="text plain">make backup
</span></code></pre>
</div>
````````````````````````````````

A [lazy continuation line] continues a paragraph in an alert, as usual:

```````````````````````````````` example -e alerts
> [!NOTE]
> This paragraph
continues lazily.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>This paragraph
continues lazily.</p>
</div>
````````````````````````````````

The first line holds the alert marker and title, not a paragraph, and so there's no paragraph for the next line to continue.  That means a line without a `>` straight after the first line ends the alert:

```````````````````````````````` example -e alerts
> [!NOTE]
Not part of the alert.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
</div>
<p>Not part of the alert.</p>
````````````````````````````````

An alert may be empty, or have only a title:

```````````````````````````````` example -e alerts
> [!NOTE]

> [!TIP] Just a title
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
</div>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Just a title</p>
</div>
````````````````````````````````

A blank line ends an alert, as it ends a block quote:

```````````````````````````````` example -e alerts
> [!NOTE]
> One alert.

> [!TIP]
> Another alert.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>One alert.</p>
</div>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>Another alert.</p>
</div>
````````````````````````````````

### Containers

An alert may appear anywhere a block quote may.  Here, an alert is in a list item:

```````````````````````````````` example -e alerts
- Item one

  > [!NOTE]
  > Highlights information that users should take into account, even when skimming.
.
<ul>
<li>
<p>Item one</p>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
</li>
</ul>
````````````````````````````````

Here, one alert is in a block quote, and another is in an alert:

```````````````````````````````` example -e alerts
> > [!NOTE]
> > In a block quote.

> [!WARNING]
> > [!TIP]
> > In an alert.
.
<blockquote>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>In a block quote.</p>
</div>
</blockquote>
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Warning</p>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>In an alert.</p>
</div>
</div>
````````````````````````````````

Since the first `>` of `>>` is a block quote marker, `>> [!NOTE]` is likewise an alert in a block quote:

```````````````````````````````` example -e alerts
>> [!NOTE]
>> In a block quote.
.
<blockquote>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>In a block quote.</p>
</div>
</blockquote>
````````````````````````````````

## Multi-line alerts

With [multi-line block quotes](multiline-block-quotes.html), an alert may also be written as a [multi-line alert](@): a multi-line block quote whose opening fence is followed by an [alert marker].  The rules for the marker and the [alert title](#alert-titles) are the same as for any alert, and the fences work as they do for any multi-line block quote:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
Highlights information that users should take into account, even when skimming.
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
````````````````````````````````

The opening fence's line may have a title:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!WARNING] Data deletion
The following instructions will make your data unrecoverable.
>>>
.
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Data deletion</p>
<p>The following instructions will make your data unrecoverable.</p>
</div>
````````````````````````````````

The content's lines don't need a `>`, and may contain any blocks, including blank lines:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE] Things to consider
You should consider the following ramifications:

1. consideration 1
1. consideration 2
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Things to consider</p>
<p>You should consider the following ramifications:</p>
<ol>
<li>consideration 1</li>
<li>consideration 2</li>
</ol>
</div>
````````````````````````````````

A closing fence must be at least as long as the opening fence.  Fences of different lengths let multi-line alerts nest:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>>>> [!NOTE]
>>>> [!CAUTION]
Nested.
>>>>
>>>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<p>Nested.</p>
</div>
</div>
````````````````````````````````

A multi-line alert without a closing fence ends at the end of the document:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
Never closed.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Never closed.</p>
</div>
````````````````````````````````

A multi-line alert may appear in a list item or a block quote:

```````````````````````````````` example -e alerts -e multiline-block-quotes
- >>> [!NOTE]
  In a list item.
  >>>

> >>> [!TIP]
> In a block quote.
> >>>
.
<ul>
<li>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>In a list item.</p>
</div>
</li>
</ul>
<blockquote>
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>In a block quote.</p>
</div>
</blockquote>
````````````````````````````````

Without multi-line block quotes, `>>>` is three block quote markers, so `>>> [!NOTE]` is an alert in two block quotes.  The next line doesn't begin with `>`, so it ends them, and the closing fence is three empty block quotes:

```````````````````````````````` example -e alerts
>>> [!NOTE]
Body
>>>
.
<blockquote>
<blockquote>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
</div>
</blockquote>
</blockquote>
<p>Body</p>
<blockquote>
<blockquote>
<blockquote>
</blockquote>
</blockquote>
</blockquote>
````````````````````````````````

Without alerts, a multi-line block quote's first line is just text:

```````````````````````````````` example -e multiline-block-quotes
>>> [!NOTE]
Body
>>>
.
<blockquote>
<blockquote>
<blockquote>
<p>[!NOTE]
Body</p>
</blockquote>
</blockquote>
</blockquote>
````````````````````````````````

## Output

HTML output renders an alert as a `<div>` element with the classes `markdown-alert` and `markdown-alert-` followed by the alert type in lower case.  Inside it, a `<p>` element with the class `markdown-alert-title` holds the alert title, and the alert's content follows:

```````````````````````````````` example -e alerts
> [!CAUTION] Mind the gap
> Negative potential consequences of an action.
.
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Mind the gap</p>
<p>Negative potential consequences of an action.</p>
</div>
````````````````````````````````

[Alert style](#alert-style) describes an alternative HTML structure.

CommonMark output writes the alert marker with the alert type in upper case, followed by the alert title, if it's not the default.  It writes a multi-line alert as an ordinary alert:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!tip] Remember
A tip.
>>>
.
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Remember</p>
<p>A tip.</p>
</div>
````````````````````````````````

In XML output, an alert is an `alert` element.  Its `type` attribute is the alert type in lower case.  It has a `title` attribute if the alert has a title, and `multiline="true"` if it's a multi-line alert.

In the AST, an alert is a [`NodeValue::Alert`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Alert), with its content as children.  Its [`NodeAlert`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeAlert.html) holds the [`AlertType`](https://docs.rs/comrak/latest/comrak/nodes/enum.AlertType.html) and the title (`None` for the default title).  For a multi-line alert, it also has `multiline` set, and records the fence's length and indentation.

An alert's source position covers all of its lines, including a multi-line alert's closing fence:

```````````````````````````````` example -e alerts --sourcepos
> [!NOTE] Title
> Body
.
<div class="markdown-alert markdown-alert-note" data-sourcepos="1:1-2:6">
<p class="markdown-alert-title">Title</p>
<p data-sourcepos="2:3-2:6">Body</p>
</div>
````````````````````````````````

## Options

### Alert style

With `--alert-style semantic` (`render.alert_style`), HTML output renders an alert as an `<aside>` element with the classes `admonition` and the alert type in lower case.  The title's `<p>` element has the class `admonition-title`.  The default is `--alert-style specific`, as described in [Output](#output):

```````````````````````````````` example -e alerts --alert-style semantic
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!WARNING] Careful now
> Critical content demanding immediate user attention due to potential risks.
.
<aside class="admonition note">
<p class="admonition-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</aside>
<aside class="admonition warning">
<p class="admonition-title">Careful now</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</aside>
````````````````````````````````

## Interactions

`--gfm` doesn't enable alerts:

```````````````````````````````` example --gfm
> [!NOTE]
> A block quote.
.
<blockquote>
<p>[!NOTE]
A block quote.</p>
</blockquote>
````````````````````````````````

With [multi-line block quotes](multiline-block-quotes.html), an alert may also be written as a [multi-line alert](#multi-line-alerts).

With [OFM callouts](ofm-callouts.html), the callout syntax takes precedence, so an alert marker begins a callout instead:

```````````````````````````````` example -e alerts -e ofm-callouts
> [!NOTE]
> A callout.
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>A callout.</p>
</div>
</div>
````````````````````````````````

## Differences from GitHub

GitHub's implementation of alerts isn't open source, so this section compares Comrak with what GitHub's Markdown API renders, as of November 2026.  On every example above with only alerts enabled, the two agree on HTML output, except that GitHub:

* doesn't support alert titles, and renders an alert marker followed by text as a plain block quote ([example 11](#example-11), [example 13](#example-13), [example 30](#example-30));
* renders an alert with no content as a plain block quote ([example 17](#example-17));
* lets a lazy continuation line follow the alert marker, as the first line of the alert's content ([example 16](#example-16));
* recognises alerts only at the top level of a document, not in a list item, a block quote, or another alert ([example 19](#example-19), [example 20](#example-20), [example 21](#example-21), [example 28](#example-28)); and,
* recognises an alert marker written with a backslash escape or an entity reference ([example 9](#example-9)).

GitHub also adds an icon to each alert title.  It has no multi-line block quotes, so it has no multi-line alerts, and it has no equivalent of [alert style](#alert-style).

## Regressions

The following examples guard against regressions.

A list item in a multi-line alert, with an unclosed fenced code block, ends at the alert's closing fence ([#907](https://github.com/kivikakk/comrak/pull/907)):

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!note]
- ```rust
  code
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<ul>
<li>
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust">code
</span></code></pre>
</li>
</ul>
</div>
````````````````````````````````

A multi-line alert's source position ends at its closing fence ([#901](https://github.com/kivikakk/comrak/pull/901)):

```````````````````````````````` example -e alerts -e multiline-block-quotes --sourcepos
>>> [!note]
Pay attention
>>>
After.
.
<div class="markdown-alert markdown-alert-note" data-sourcepos="1:1-3:3">
<p class="markdown-alert-title">Note</p>
<p data-sourcepos="2:1-2:13">Pay attention</p>
</div>
<p data-sourcepos="4:1-4:6">After.</p>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-break
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
