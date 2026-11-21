---
title: Description/definition lists
with_thanks_to:
  - https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/definition_lists.md
---

A description list pairs terms with descriptions, such as words with their definitions, or names with their values.  HTML calls this structure a description list, and uses `<dl>`, `<dt>`, and `<dd>` elements to describe it.  Many Markdown implementations call it a definition list.

Comrak's description lists follow the definition lists extension of [commonmark-hs](https://github.com/jgm/commonmark-hs/tree/master/commonmark-extensions), which [Pandoc](https://pandoc.org/MANUAL.html#extension-definition_lists) uses for its `commonmark` reader.  This document describes Comrak's description lists, and [Differences from commonmark-hs](#differences-from-commonmark-hs) lists where they differ.  GitLab uses Comrak's description lists, so this document is also the canonical spec for [GitLab Flavored Markdown description lists](https://docs.gitlab.com/user/markdown/#description-lists).

To enable description lists, pass `-e description-lists` on the command line, or set `extension.description_lists` in the options struct.

Here's a typical description list:

```````````````````````````````` example -e description-lists
Apple
: A red fruit.

Orange
: An orange fruit.
: A colour.
.
<dl>
<dt>Apple</dt>
<dd>A red fruit.</dd>
<dt>Orange</dt>
<dd>An orange fruit.</dd>
<dd>A colour.</dd>
</dl>
````````````````````````````````

## Syntax

A [description marker](@) is a `:` or `~`, indented by up to three spaces, and followed by a [space], a [tab], or the end of the line.

A [description](@) begins with a description marker, and contains blocks, like a [list item].  A term's first description must follow a [paragraph], which becomes the [term](@).  Each of the term's later descriptions follows the one before it.  A [description list](@) is a sequence of terms, each followed by one or more descriptions.

### Description markers

A description marker may be a `:` or a `~`:

```````````````````````````````` example -e description-lists
apple
: red fruit

orange
~ orange fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
<dt>orange</dt>
<dd>orange fruit</dd>
</dl>
````````````````````````````````

A tab may follow the marker instead of a space:

```````````````````````````````` example -e description-lists
apple
:→red fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
````````````````````````````````

The marker must be followed by a space, a tab, or the end of the line.  Otherwise, the line continues the paragraph:

```````````````````````````````` example -e description-lists
apple
:red fruit
.
<p>apple
:red fruit</p>
````````````````````````````````

The marker may be indented by up to three spaces:

```````````````````````````````` example -e description-lists
apple
   : red fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
````````````````````````````````

With four spaces, the line continues the paragraph instead:

```````````````````````````````` example -e description-lists
apple
    : red fruit
.
<p>apple
: red fruit</p>
````````````````````````````````

Without a paragraph before it, a description marker is just text:

```````````````````````````````` example -e description-lists
: red fruit
.
<p>: red fruit</p>
````````````````````````````````

### Terms

The term is the whole paragraph before the description, however many lines it spans:

```````````````````````````````` example -e description-lists
Pink Lady
apple
: red fruit
.
<dl>
<dt>Pink Lady
apple</dt>
<dd>red fruit</dd>
</dl>
````````````````````````````````

Only the paragraph immediately before the description becomes the term.  Earlier paragraphs remain paragraphs:

```````````````````````````````` example -e description-lists
Fruit:

apple
: red fruit
.
<p>Fruit:</p>
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
````````````````````````````````

Any number of blank lines may separate the term from its description:

```````````````````````````````` example -e description-lists
apple


: red fruit
.
<dl>
<dt>apple</dt>
<dd>
<p>red fruit</p>
</dd>
</dl>
````````````````````````````````

Only a paragraph can become a term.  A description marker after a heading, a list, or any other block is just text:

```````````````````````````````` example -e description-lists
# apple
: red fruit
.
<h1>apple</h1>
<p>: red fruit</p>
````````````````````````````````

```````````````````````````````` example -e description-lists
- apple

: red fruit
.
<ul>
<li>apple</li>
</ul>
<p>: red fruit</p>
````````````````````````````````

The description marker's line ends the term's paragraph before Comrak parses the term's inline content.  An inline construct can't span the term and description:

```````````````````````````````` example -e description-lists
*apple
: red*
.
<dl>
<dt>*apple</dt>
<dd>red*</dd>
</dl>
````````````````````````````````

### Description content

A description's content begins after its marker and the spaces or tabs that follow.  Later lines belong to the description if they're indented at least as far:

```````````````````````````````` example -e description-lists
apple
:   red fruit

    crisp
.
<dl>
<dt>apple</dt>
<dd>
<p>red fruit</p>
<p>crisp</p>
</dd>
</dl>
````````````````````````````````

After a blank line, a line indented less than that ends the description list:

```````````````````````````````` example -e description-lists
apple
:   red fruit

  crisp
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
<p>crisp</p>
````````````````````````````````

Comrak measures that indentation in columns, like it does for list items, with tab stops every four columns.  Here, the tab takes the content to column 4, which two spaces don't reach:

```````````````````````````````` example -e description-lists
apple
:→red fruit

  crisp
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
<p>crisp</p>
````````````````````````````````

If five or more columns of spaces follow the marker, the content begins one column after the marker, and the rest of the line begins an [indented code block]:

```````````````````````````````` example -e description-lists
apple
:     red fruit
.
<dl>
<dt>apple</dt>
<dd>
<pre class="syntax-highlighting"><code><span class="text plain">red fruit
</span></code></pre>
</dd>
</dl>
````````````````````````````````

If the marker ends its line, the content begins on the next line, again one column after the marker:

```````````````````````````````` example -e description-lists
apple
:
  red fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
````````````````````````````````

A description may be empty.  As with list items, descriptions can begin with at most one blank line: a description marker followed by a blank line makes an empty description, and what follows isn't part of it:

```````````````````````````````` example -e description-lists
apple
:

  red fruit
.
<dl>
<dt>apple</dt>
<dd></dd>
</dl>
<p>red fruit</p>
````````````````````````````````

A description may contain any blocks:

```````````````````````````````` example -e description-lists
*apple*

:   red fruit

    contains seeds,
    crisp, pleasant to taste

*orange*

:   orange fruit

        { orange code block }

    > orange block quote
.
<dl>
<dt><em>apple</em></dt>
<dd>
<p>red fruit</p>
<p>contains seeds,
crisp, pleasant to taste</p>
</dd>
<dt><em>orange</em></dt>
<dd>
<p>orange fruit</p>
<pre class="syntax-highlighting"><code><span class="text plain">{ orange code block }
</span></code></pre>
<blockquote>
<p>orange block quote</p>
</blockquote>
</dd>
</dl>
````````````````````````````````

```````````````````````````````` example -e description-lists
term

:   1. Para one

       Para two
.
<dl>
<dt>term</dt>
<dd>
<ol>
<li>
<p>Para one</p>
<p>Para two</p>
</li>
</ol>
</dd>
</dl>
````````````````````````````````

A description may contain another description list:

```````````````````````````````` example -e description-lists
fruit
:   apple
    : red fruit
.
<dl>
<dt>fruit</dt>
<dd>
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
</dd>
</dl>
````````````````````````````````

A [lazy continuation line] continues a paragraph in a description, as usual:

```````````````````````````````` example -e description-lists
apple
:   red
fruit
.
<dl>
<dt>apple</dt>
<dd>red
fruit</dd>
</dl>
````````````````````````````````

### Several descriptions

A term may have more than one description.  Each begins with its own description marker, of either kind:

```````````````````````````````` example -e description-lists
apple
: red fruit
~ computer company
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
<dd>computer company</dd>
</dl>
````````````````````````````````

A blank line may separate a term's descriptions:

```````````````````````````````` example -e description-lists
apple
: red fruit

: computer company
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
<dd>computer company</dd>
</dl>
````````````````````````````````

A new term needs a blank line before it.  Otherwise, its line is a lazy continuation line of the description before it:

```````````````````````````````` example -e description-lists
apple
: red fruit
orange
: orange fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit
orange</dd>
<dd>orange fruit</dd>
</dl>
````````````````````````````````

```````````````````````````````` example -e description-lists
apple
: red fruit

orange
: orange fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
<dt>orange</dt>
<dd>orange fruit</dd>
</dl>
````````````````````````````````

A paragraph without a description after it ends the description list:

```````````````````````````````` example -e description-lists
apple
: red fruit

orange
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
<p>orange</p>
````````````````````````````````

### Tight and loose

A term's descriptions are [loose](@) if a blank line separates the term from its first description, or separates two blocks directly inside any of the term's descriptions.  Otherwise, they're [tight](@).  HTML output wraps the paragraphs in loose descriptions in `<p>` elements.

A blank line between the term and its first description makes the descriptions loose:

```````````````````````````````` example -e description-lists
apple

: red fruit

orange

: orange fruit
.
<dl>
<dt>apple</dt>
<dd>
<p>red fruit</p>
</dd>
<dt>orange</dt>
<dd>
<p>orange fruit</p>
</dd>
</dl>
````````````````````````````````

As does a blank line between two blocks in a description:

```````````````````````````````` example -e description-lists
apple
: red fruit

  crisp
.
<dl>
<dt>apple</dt>
<dd>
<p>red fruit</p>
<p>crisp</p>
</dd>
</dl>
````````````````````````````````

A blank line in one description makes all of the term's descriptions loose, but not other terms' descriptions:

```````````````````````````````` example -e description-lists
apple
: red fruit
: computer company

  famous for its logo

orange
: orange fruit
.
<dl>
<dt>apple</dt>
<dd>
<p>red fruit</p>
</dd>
<dd>
<p>computer company</p>
<p>famous for its logo</p>
</dd>
<dt>orange</dt>
<dd>orange fruit</dd>
</dl>
````````````````````````````````

A blank line between two descriptions, or before a new term, doesn't make descriptions loose:

```````````````````````````````` example -e description-lists
apple
: red fruit

: computer company

orange
: orange fruit
.
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
<dd>computer company</dd>
<dt>orange</dt>
<dd>orange fruit</dd>
</dl>
````````````````````````````````

Nor does a blank line inside a nested block.  Here, the nested list is loose, but the description is tight:

```````````````````````````````` example -e description-lists
apple
: - red

  - green
.
<dl>
<dt>apple</dt>
<dd>
<ul>
<li>
<p>red</p>
</li>
<li>
<p>green</p>
</li>
</ul>
</dd>
</dl>
````````````````````````````````

### Containers

A description list may appear in any container, such as a list item or a [block quote]:

```````````````````````````````` example -e description-lists
- apple
  : red fruit
.
<ul>
<li>
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e description-lists
> apple
> : red fruit
.
<blockquote>
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
</blockquote>
````````````````````````````````

A description marker can't be a lazy continuation line.  Here, the marker's line continues the paragraph in the list item or block quote instead:

```````````````````````````````` example -e description-lists
- apple
: red fruit
.
<ul>
<li>apple
: red fruit</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e description-lists
> apple
: red fruit
.
<blockquote>
<p>apple
: red fruit</p>
</blockquote>
````````````````````````````````

A description list counts as one block in a list item.  A blank line between it and the next block makes the list loose, as a blank line between any two blocks in a list item would:

```````````````````````````````` example -e description-lists
- apple
  : red fruit

  crisp
.
<ul>
<li>
<dl>
<dt>apple</dt>
<dd>red fruit</dd>
</dl>
<p>crisp</p>
</li>
</ul>
````````````````````````````````

## Output

HTML output writes a description list as a `<dl>` element.  Each term is a `<dt>` element, and each description is a `<dd>` element.  HTML output never wraps a term's paragraph in a `<p>` element, and wraps a description's paragraphs only if the description is [loose].

CommonMark output writes each term, followed by each of its descriptions, with a blank line between the term and its first description if the descriptions are loose.  It writes every description marker as `:`, followed by a space, and indents a description's later lines to line up with its content:

```````````````````````````````` example -e description-lists
apple
~   red fruit
    crisp
.
<dl>
<dt>apple</dt>
<dd>red fruit
crisp</dd>
</dl>
````````````````````````````````

It also escapes a `:` or `~` at the start of a line that would otherwise begin a description, as in the CommonMark view of [example 36](#example-36).

In XML output, a description list is a `description_list` element, containing `description_item` elements.  Each `description_item` contains a `description_term` element, unless it's a term's second or later description, and a `description_details` element.  The `tight` attribute of `description_item` is `true` if the descriptions are tight, and `false` otherwise.

In the AST, a description list is a [`NodeValue::DescriptionList`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.DescriptionList), with [`NodeValue::DescriptionItem`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.DescriptionItem) children.  An item's first child is a [`NodeValue::DescriptionTerm`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.DescriptionTerm) holding the term's paragraph, and its last is a [`NodeValue::DescriptionDetails`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.DescriptionDetails) holding the description's blocks.  A term's second and later descriptions each get an item of their own, without a `DescriptionTerm`.  Each item's [`NodeDescriptionItem`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeDescriptionItem.html) records whether its descriptions are tight (`tight`), and where its marker and content begin (`marker_offset` and `padding`).

A term's [source position](../options/sourcepos.html#source-position) covers its paragraph.  A description's source position begins at its marker:

```````````````````````````````` example -e description-lists --sourcepos
apple
Pink Lady
: red fruit
: crisp
.
<dl data-sourcepos="1:1-4:7">
<dt data-sourcepos="1:1-2:9">apple
Pink Lady</dt>
<dd data-sourcepos="3:1-3:11">red fruit</dd>
<dd data-sourcepos="4:1-4:7">crisp</dd>
</dl>
````````````````````````````````

## Interactions

`--gfm` doesn't enable description lists:

```````````````````````````````` example --gfm
apple
: red fruit
.
<p>apple
: red fruit</p>
````````````````````````````````

With [tables](table.html), a line after a table's rows continues the table, even if it begins with a description marker:

```````````````````````````````` example -e description-lists -e table
| fruit |
| ----- |
| apple |
: red fruit
.
<table>
<thead>
<tr>
<th>fruit</th>
</tr>
</thead>
<tbody>
<tr>
<td>apple</td>
</tr>
<tr>
<td>: red fruit</td>
</tr>
</tbody>
</table>
````````````````````````````````

With [strikethrough](strikethrough.html), a `~` description marker still begins a description:

```````````````````````````````` example -e description-lists -e strikethrough
apple
~ red ~~fruit~~
.
<dl>
<dt>apple</dt>
<dd>red <del>fruit</del></dd>
</dl>
````````````````````````````````

With [task lists](tasklist.html), a [task list item marker](tasklist.html#task-list-item-marker) at the start of a description is just text, since a description isn't a list item:

```````````````````````````````` example -e description-lists -e tasklist
apple
: [x] red fruit
.
<dl>
<dt>apple</dt>
<dd>[x] red fruit</dd>
</dl>
````````````````````````````````

With [OFM block IDs](ofm-block-ids.html), a block ID at the end of a term or a description labels its `<dt>` or `<dd>` element, as one in a list item labels the `<li>` element:

```````````````````````````````` example -e description-lists -e ofm-block-ids
apple ^fruit
: red fruit ^red
.
<dl>
<dt id="^fruit">apple </dt>
<dd id="^red">red fruit </dd>
</dl>
````````````````````````````````

## Differences from commonmark-hs

Pandoc's `commonmark` reader uses commonmark-hs.  This section compares Comrak with Pandoc 3.7, run as `pandoc -f commonmark+definition_lists`.  Since Pandoc's other extensions differ from Comrak's, the examples under [Interactions](#interactions) aren't comparable.  On every other example above, the two agree on HTML structure, except that Pandoc:

* keeps descriptions tight when a blank line separates two blocks in a description, and writes those blocks' paragraphs without `<p>` elements, which makes them read as one paragraph ([example 14](#example-14), [example 30](#example-30), [example 31](#example-31));
* keeps a list tight when a blank line separates a description list in a list item from the next block ([example 38](#example-38)); and,
* lets a blank line follow a description marker that ends its line, and puts what follows the blank line in the description ([example 19](#example-19)).

Pandoc's own Markdown reader has a different syntax for definition lists, and isn't compared here.

## Regressions

The following examples guard against regressions.

Source positions for a description list ([#657](https://github.com/kivikakk/comrak/pull/657)):

```````````````````````````````` example -e description-lists --sourcepos
apple

: red fruit

orange

: orange fruit
.
<dl data-sourcepos="1:1-7:14">
<dt data-sourcepos="1:1-1:5">apple</dt>
<dd data-sourcepos="3:1-3:11">
<p data-sourcepos="3:3-3:11">red fruit</p>
</dd>
<dt data-sourcepos="5:1-5:6">orange</dt>
<dd data-sourcepos="7:1-7:14">
<p data-sourcepos="7:3-7:14">orange fruit</p>
</dd>
</dl>
````````````````````````````````

A multi-line term's source position ends at the end of its last line ([#908](https://github.com/kivikakk/comrak/pull/908)):

```````````````````````````````` example -e description-lists --sourcepos
a
bc
: d
.
<dl data-sourcepos="1:1-3:3">
<dt data-sourcepos="1:1-2:2">a
bc</dt>
<dd data-sourcepos="3:1-3:3">d</dd>
</dl>
````````````````````````````````

[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-block
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[space]: https://spec.commonmark.org/0.31.2/#space
[tab]: https://spec.commonmark.org/0.31.2/#tab
