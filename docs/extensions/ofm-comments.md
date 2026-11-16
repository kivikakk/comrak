---
title: OFM comments
---

An OFM comment is hidden text marked with two consecutive percent signs (`%%`).  Use comments for notes to yourself or other authors, drafts, and other text that shouldn't appear in the rendered document.

OFM comments originate in [Obsidian Flavored Markdown](https://help.obsidian.md/syntax#Comments).  This document describes Comrak's OFM comments.

To enable OFM comments, pass `-e ofm-comments` on the command line, or set `extension.ofm_comments` in the options struct.

Here's a paragraph with an inline comment, followed by a block comment:

```````````````````````````````` example -e ofm-comments
This is an %%inline%% comment.

%%
This is a block comment.

Block comments can span multiple lines.
%%
.
<p>This is an  comment.</p>
````````````````````````````````

## Syntax

A [comment marker](@) is two consecutive percent signs (`%%`).  A single percent sign is ordinary text.  Comrak recognises two kinds of comments: [inline comments](#inline-comments) and [block comments](#block-comments).

The contents of a comment are literal text.  Comrak doesn't parse Markdown inside a comment, and HTML output omits the comment entirely.

### Inline comments

An [inline comment](@) begins with a [comment marker] in inline content and ends at the next comment marker on the same line.  Its opening and closing markers can appear anywhere in the inline content:

```````````````````````````````` example -e ofm-comments
# A %%hidden%% heading

- An %%hidden%% item

**Bold %%hidden%% text**
.
<h1>A  heading</h1>
<ul>
<li>An  item</li>
</ul>
<p><strong>Bold  text</strong></p>
````````````````````````````````

An inline comment may be empty.  Its contents may include a single percent sign, and any Markdown inside it stays literal:

```````````````````````````````` example -e ofm-comments
a %%%% b

a %%100% *not emphasis*%% b
.
<p>a  b</p>
<p>a  b</p>
````````````````````````````````

Comment markers don't nest.  The next marker always closes the comment, so this example contains two comments with visible text between them:

```````````````````````````````` example -e ofm-comments
%%first%% visible %%second%%
.
<p> visible </p>
````````````````````````````````

An inline comment can't span a line ending.  If no closing marker appears on the same line, the opening marker is literal text:

```````````````````````````````` example -e ofm-comments
x %%aa
bb%% tail
.
<p>x %%aa
bb%% tail</p>
````````````````````````````````

A marker on a later line can close a different inline comment, but it can't close one left unclosed on an earlier line:

```````````````````````````````` example -e ofm-comments
x %%unclosed
y %%closed%% z
.
<p>x %%unclosed
y  z</p>
````````````````````````````````

### Block comments

A [block comment](@) begins with a [comment marker] at the start of a line, after up to three spaces, when the rest of that line contains no percent sign.  Any text after the opening marker is part of the comment.  Unlike an inline comment, a block comment can contain line endings, blank lines, and Markdown block structure:

```````````````````````````````` example -e ofm-comments
%% Notes for the next revision
# A hidden heading

- A hidden item
- Another hidden item
%%
.
````````````````````````````````

A block comment can interrupt a paragraph:

```````````````````````````````` example -e ofm-comments
before
%%
hidden
%%
after
.
<p>before</p>
<p>after</p>
````````````````````````````````

The block comment ends at the next comment marker, wherever it appears on a later line.  Text after the closing marker remains visible, and Comrak parses its Markdown as usual:

```````````````````````````````` example -e ofm-comments
%%
# A hidden heading

- A hidden item
%%tail

%%
hidden
%%# A visible heading
.
<p>tail</p>
<h1>A visible heading</h1>
````````````````````````````````

If the opening line contains another percent sign, it doesn't begin a block comment.  A complete pair of markers on that line forms an inline comment instead:

```````````````````````````````` example -e ofm-comments
%%inline%% visible
.
<p> visible</p>
````````````````````````````````

If that line has no complete pair, its markers remain literal; an inline comment can't continue onto the next line:

```````````````````````````````` example -e ofm-comments
%%a%b
cc%%tail
.
<p>%%a%b
cc%%tail</p>
````````````````````````````````

A block comment without a closing marker continues to the end of the document:

```````````````````````````````` example -e ofm-comments
%%
aa

bb
.
````````````````````````````````

Block comments may appear in containers, such as [block quotes] and [list items].  The usual container markers and indentation apply to the opening and closing lines:

```````````````````````````````` example -e ofm-comments
> %%
> hidden
> %%visible

- %%
  hidden
  %%visible
.
<blockquote>
<p>visible</p>
</blockquote>
<ul>
<li>visible</li>
</ul>
````````````````````````````````

### Precedence and escaping

[Code spans] and [fenced code blocks] take precedence over comments.  Comment markers inside them are literal text:

```````````````````````````````` example -e ofm-comments
`%%inline%%`

```
%%
# text
%%
```
.
<p><code>%%inline%%</code></p>
<pre class="syntax-highlighting"><code><span class="text plain">%%
# text
%%
</span></code></pre>
````````````````````````````````

A [backslash escape] or an [entity reference] produces a literal percent sign, not part of a comment marker:

```````````````````````````````` example -e ofm-comments
\%%not a comment\%%

&#37;&#37;also not a comment&#37;&#37;
.
<p>%%not a comment%%</p>
<p>%%also not a comment%%</p>
````````````````````````````````

## Output

HTML output omits each comment, including its markers and contents.  It doesn't replace the comment with an HTML comment or any other element.  Whitespace around an inline comment remains, which can leave two adjacent spaces:

```````````````````````````````` example -e ofm-comments
before %%hidden%% after
.
<p>before  after</p>
````````````````````````````````

CommonMark output writes comments back with `%%` markers.  It preserves their literal contents, and leaves an unclosed block comment unclosed.

In XML output, both kinds of comment are a `comment` element with `xml:space="preserve"`.  The element's text is the comment's literal contents, without its markers.

In the AST, an inline comment is a [`NodeValue::CommentInline`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.CommentInline).  Its [`NodeComment`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeComment.html) holds the literal contents.  A block comment is a [`NodeValue::MultilineComment`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.MultilineComment).  Its [`NodeCommentBlock`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeCommentBlock.html) holds the literal contents and whether the comment was explicitly closed.  Neither kind of comment has children.

## Interactions

Inline comments work in [table](table.html) cells just as they do in other inline content:

```````````````````````````````` example -e ofm-comments -e table
a %%hidden%% | b
--- | ---
x %%hidden%% y | z
.
<table>
<thead>
<tr>
<th>a </th>
<th>b</th>
</tr>
</thead>
<tbody>
<tr>
<td>x  y</td>
<td>z</td>
</tr>
</tbody>
</table>
````````````````````````````````

With [OFM callouts](ofm-callouts.html), an inline comment can hide part or all of a callout title:

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

A block comment between lines of a [list item] doesn't by itself make the containing [list] loose:

```````````````````````````````` example -e ofm-comments
- before
  %%
  hidden%%
  after
- next
.
<ul>
<li>before
after</li>
<li>next</li>
</ul>
````````````````````````````````

A blank line inside such a comment still affects the list's [block structure], so it can make the list loose:

```````````````````````````````` example -e ofm-comments
- %%
  first hidden paragraph

  second hidden paragraph%%after
- next
.
<ul>
<li>
<p>after</p>
</li>
<li>
<p>next</p>
</li>
</ul>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[block quotes]: https://spec.commonmark.org/0.31.2/#block-quotes
[block structure]: https://spec.commonmark.org/0.31.2/#blocks-and-inlines
[code spans]: https://spec.commonmark.org/0.31.2/#code-span
[fenced code blocks]: https://spec.commonmark.org/0.31.2/#fenced-code-block
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[list]: https://spec.commonmark.org/0.31.2/#lists
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[list items]: https://spec.commonmark.org/0.31.2/#list-items
