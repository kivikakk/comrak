---
title: Task lists
---

A task list is a [list](https://spec.commonmark.org/0.31.2/#lists) that contains task list items.  Each task list item is either complete or incomplete.  Use task lists for to-do lists, sequences of steps, or any other list whose items can be done or not done.

Task lists originate in [GitHub Flavored Markdown](https://github.github.com/gfm/#task-list-items-extension-).  This document describes Comrak's task lists, which can differ from GitHub's.  GitLab uses Comrak's task lists, so this document is also the canonical spec for [GitLab Flavored Markdown task lists](https://docs.gitlab.com/user/markdown/#task-lists).

To enable task lists, pass `-e tasklist` (or `--gfm`) on the command line, or set `extension.tasklist` in the options struct.

Here's a typical task list:

```````````````````````````````` example -e tasklist
- [ ] Write the docs
- [x] Build the runner
.
<ul>
<li><input type="checkbox" disabled="" /> Write the docs</li>
<li><input type="checkbox" checked="" disabled="" /> Build the runner</li>
</ul>
````````````````````````````````

## Syntax

A [task list item symbol](@) is a [space], `x`, or `X`.

A [task list item marker](@) is a left bracket (`[`), followed by a [task list item symbol], followed by a right bracket (`]`).

A [task list item](@) is a [list item] whose first block is a [paragraph] that begins with a [task list item marker].  The marker must be followed by a [space], a [tab], a [soft line break], a [hard line break], or the end of the paragraph.

A parser determines the document's [block structure](https://spec.commonmark.org/0.31.2/#blocks-and-inlines) and inline structure before it recognizes task list item markers.  The marker and the whitespace after it must be literal characters, and must not form part of any other inline construct.

A [task list item] is [incomplete](@) if the symbol in its marker is a [space], and [complete](@) otherwise:

```````````````````````````````` example -e tasklist
- [ ] foo
- [x] bar
- [X] baz
.
<ul>
<li><input type="checkbox" disabled="" /> foo</li>
<li><input type="checkbox" checked="" disabled="" /> bar</li>
<li><input type="checkbox" checked="" disabled="" /> baz</li>
</ul>
````````````````````````````````

No other symbol is permitted unless [relaxed task list character](#relaxed-task-list-character) is enabled.  The following are not task list items:

```````````````````````````````` example -e tasklist
- [y] a
- [] b
- [  ] c
- [xx] d
- [ x] e
- [→] f
.
<ul>
<li>[y] a</li>
<li>[] b</li>
<li>[  ] c</li>
<li>[xx] d</li>
<li>[ x] e</li>
<li>[→] f</li>
</ul>
````````````````````````````````

The marker may be followed by a [space] or a [tab]:

```````````````````````````````` example -e tasklist
- [x] foo
- [x]→bar
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo</li>
<li><input type="checkbox" checked="" disabled="" /> bar</li>
</ul>
````````````````````````````````

The marker must not be followed directly by other text:

```````````````````````````````` example -e tasklist
- [x]foo
.
<ul>
<li>[x]foo</li>
</ul>
````````````````````````````````

The same applies to any other inline content:

```````````````````````````````` example -e tasklist
- [x]*foo*
.
<ul>
<li>[x]<em>foo</em></li>
</ul>
````````````````````````````````

The marker may be followed by a [soft line break] or a [hard line break]:

```````````````````````````````` example -e tasklist
- [x]
  foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> 
foo</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- [x]\
  foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> <br />
foo</li>
</ul>
````````````````````````````````

The marker may make up the whole paragraph:

```````````````````````````````` example -e tasklist
- [x]
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> </li>
</ul>
````````````````````````````````

The marker must begin the first block of a [list item], and that block must be a [paragraph].  None of the following are task list items:

```````````````````````````````` example -e tasklist
[x] foo
.
<p>[x] foo</p>
````````````````````````````````

```````````````````````````````` example -e tasklist
- foo [x] bar
.
<ul>
<li>foo [x] bar</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- *[x]* foo
.
<ul>
<li><em>[x]</em> foo</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- foo

  [x] bar
.
<ul>
<li>
<p>foo</p>
<p>[x] bar</p>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- > [x] foo
.
<ul>
<li>
<blockquote>
<p>[x] foo</p>
</blockquote>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- # [x] foo
.
<ul>
<li>
<h1>[x] foo</h1>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
-     [x] foo
.
<ul>
<li>
<pre class="syntax-highlighting"><code><span class="text plain">[x] foo
</span></code></pre>
</li>
</ul>
````````````````````````````````

Block structure comes first.  Here, the marker begins a [setext heading] rather than a paragraph, so the item is not a task list item:

```````````````````````````````` example -e tasklist
- [x] foo
  ---
.
<ul>
<li>
<h2>[x] foo</h2>
</li>
</ul>
````````````````````````````````

The first block may begin on the line after the [list marker]:

```````````````````````````````` example -e tasklist
-
  [x] foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo</li>
</ul>
````````````````````````````````

Inline structure also comes first.  A [backslash escape] or an [entity reference] cannot form part of a marker:

```````````````````````````````` example -e tasklist
- \[x] foo
.
<ul>
<li>[x] foo</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- &#91;x] foo
.
<ul>
<li>[x] foo</li>
</ul>
````````````````````````````````

[Links](https://spec.commonmark.org/0.31.2/#links) and [link reference definitions] take precedence over the marker:

```````````````````````````````` example -e tasklist
- [x](/url) foo
.
<ul>
<li><a href="/url">x</a> foo</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- [x] foo

[x]: /url
.
<ul>
<li><a href="/url">x</a> foo</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist
- [x]: /url
.
<ul>
<li></li>
</ul>
````````````````````````````````

Only the marker at the beginning of the paragraph is recognized; anything after it is content:

```````````````````````````````` example -e tasklist
- [ ] [x] foo
.
<ul>
<li><input type="checkbox" disabled="" /> [x] foo</li>
</ul>
````````````````````````````````

## Lists of tasks

A [list] containing at least one [task list item] is a [task list](@).

Task list items may appear in [ordered lists]:

```````````````````````````````` example -e tasklist
1. [ ] Bird
2. [ ] McHale
3. [x] Parish
.
<ol>
<li><input type="checkbox" disabled="" /> Bird</li>
<li><input type="checkbox" disabled="" /> McHale</li>
<li><input type="checkbox" checked="" disabled="" /> Parish</li>
</ol>
````````````````````````````````

One list may mix task list items and other list items:

```````````````````````````````` example -e tasklist
- [ ] bullet point one
- bullet point two and some extra text
- [x] bullet point three
.
<ul>
<li><input type="checkbox" disabled="" /> bullet point one</li>
<li>bullet point two and some extra text</li>
<li><input type="checkbox" checked="" disabled="" /> bullet point three</li>
</ul>
````````````````````````````````

Task lists may be nested:

```````````````````````````````` example -e tasklist
- [x] item
  - [ ] item2
  - [x] item3
- [ ] item4
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> item
<ul>
<li><input type="checkbox" disabled="" /> item2</li>
<li><input type="checkbox" checked="" disabled="" /> item3</li>
</ul>
</li>
<li><input type="checkbox" disabled="" /> item4</li>
</ul>
````````````````````````````````

A task list item may consist of the marker alone:

```````````````````````````````` example -e tasklist
- [ ]
- [x]
.
<ul>
<li><input type="checkbox" disabled="" /> </li>
<li><input type="checkbox" checked="" disabled="" /> </li>
</ul>
````````````````````````````````

The marker may also be followed only by other blocks:

```````````````````````````````` example -e tasklist
- [x]
  - [ ] foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> 
<ul>
<li><input type="checkbox" disabled="" /> foo</li>
</ul>
</li>
</ul>
````````````````````````````````

When a task list is [loose], the checkbox precedes the item's first paragraph rather than appearing inside it:

```````````````````````````````` example -e tasklist
- [x] foo

- [ ] bar
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> 
<p>foo</p>
</li>
<li><input type="checkbox" disabled="" /> 
<p>bar</p>
</li>
</ul>
````````````````````````````````

The same applies when an item with several blocks makes the list loose:

```````````````````````````````` example -e tasklist
- [x] foo

  bar
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> 
<p>foo</p>
<p>bar</p>
</li>
</ul>
````````````````````````````````

A [lazy continuation line] continues the paragraph as usual:

```````````````````````````````` example -e tasklist
- [x] foo
bar
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo
bar</li>
</ul>
````````````````````````````````

Task lists may appear in any container, such as a [block quote]:

```````````````````````````````` example -e tasklist
> - [x] foo
.
<blockquote>
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo</li>
</ul>
</blockquote>
````````````````````````````````

## Output

HTML output replaces the marker with a disabled checkbox `<input>` element followed by a single space.  The element has a `checked` attribute if the item is [complete]:

```````````````````````````````` example -e tasklist
- [ ] List item 1
- [ ] This list item is **bold**
- [x] There is some `code` here
.
<ul>
<li><input type="checkbox" disabled="" /> List item 1</li>
<li><input type="checkbox" disabled="" /> This list item is <strong>bold</strong></li>
<li><input type="checkbox" checked="" disabled="" /> There is some <code>code</code> here</li>
</ul>
````````````````````````````````

The page's CSS determines whether the rendered page shows bullets or numbers beside the checkboxes.

The output omits the [space] or [tab] after the marker, but keeps any further whitespace as content:

```````````````````````````````` example -e tasklist
- [x]  foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" />  foo</li>
</ul>
````````````````````````````````

CommonMark output writes the marker back with its symbol, followed by a space.

In XML output, a task list item is a `taskitem` element.  Its `completed` attribute is `true` if the item is [complete], and `false` otherwise.  A task list's `list` element has `tasklist="true"`.

In the AST, a task list item is a [`NodeValue::TaskItem`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.TaskItem).  Its [`NodeTaskItem`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeTaskItem.html) holds the symbol (`None` for a [space]) and the symbol's source position.  A task list's [`NodeList`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeList.html) has `is_task_list` set.

## Options

### Relaxed task list character

With `--relaxed-tasklist-character` (`parse.relaxed_tasklist_matching`), a [task list item symbol] may be any single character other than `]` or a [line ending].

Without the option, this is not a task list item:

```````````````````````````````` example -e tasklist
* [!] Red
.
<ul>
<li>[!] Red</li>
</ul>
````````````````````````````````

With the option, it is:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
* [!] Red
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> Red</li>
</ul>
````````````````````````````````

Only a [space] makes the item [incomplete]:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [~] foo
- [-] bar
- [ ] baz
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo</li>
<li><input type="checkbox" checked="" disabled="" /> bar</li>
<li><input type="checkbox" disabled="" /> baz</li>
</ul>
````````````````````````````````

The symbol must still be exactly one character:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [ab] foo
- [] bar
.
<ul>
<li>[ab] foo</li>
<li>[] bar</li>
</ul>
````````````````````````````````

Inline structure still comes first.  In the first item, `\]` is a [backslash escape].  In the second, the backtick begins a [code span]:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [\] foo
- [`] bar `baz`
.
<ul>
<li>[] foo</li>
<li>[<code>] bar </code>baz`</li>
</ul>
````````````````````````````````

### Task list classes

With `--tasklist-classes` (`render.tasklist_classes`), HTML output adds these classes for styling:

- `contains-task-list` on each [task list]
- `task-list-item` on each [task list item]
- `task-list-item-checkbox` on each checkbox

Other lists and list items get no classes:

```````````````````````````````` example -e tasklist --tasklist-classes
- [x] foo
- bar
  1. [ ] baz

* qux
.
<ul class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" checked="" disabled="" /> foo</li>
<li>bar
<ol class="contains-task-list">
<li class="task-list-item"><input type="checkbox" class="task-list-item-checkbox" disabled="" /> baz</li>
</ol>
</li>
</ul>
<ul>
<li>qux</li>
</ul>
````````````````````````````````

### Task lists in tables

With `--tasklist-in-table` (`parse.tasklist_in_table`), a [table](table.html) cell whose entire content is a [task list item marker] becomes a checkbox.  With `--tasklist-classes`, the checkbox has the class `task-list-item-checkbox`.

Without the option, such cells stay as text:

```````````````````````````````` example -e table -e tasklist
|     | name |
| --- | ---- |
| [ ] | Rell |
| [x] | Kai  |
.
<table>
<thead>
<tr>
<th></th>
<th>name</th>
</tr>
</thead>
<tbody>
<tr>
<td>[ ]</td>
<td>Rell</td>
</tr>
<tr>
<td>[x]</td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````

With the option, they become checkboxes:

```````````````````````````````` example -e table -e tasklist --tasklist-in-table
|     | name |
| --- | ---- |
| [ ] | Rell |
| [x] | Kai  |
.
<table>
<thead>
<tr>
<th></th>
<th>name</th>
</tr>
</thead>
<tbody>
<tr>
<td><input type="checkbox" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td><input type="checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
</tbody>
</table>
````````````````````````````````

This applies to header cells too:

```````````````````````````````` example -e table -e tasklist --tasklist-in-table
| [x] |
| --- |
| foo |
.
<table>
<thead>
<tr>
<th><input type="checkbox" checked="" disabled="" /> </th>
</tr>
</thead>
<tbody>
<tr>
<td>foo</td>
</tr>
</tbody>
</table>
````````````````````````````````

The marker must be the cell's only content.  None of these cells contain a checkbox:

```````````````````````````````` example -e table -e tasklist --tasklist-in-table
| a       | b       | c     |
| ------- | ------- | ----- |
| [x] foo | foo [x] | *[x]* |
.
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
<th>c</th>
</tr>
</thead>
<tbody>
<tr>
<td>[x] foo</td>
<td>foo [x]</td>
<td><em>[x]</em></td>
</tr>
</tbody>
</table>
````````````````````````````````

[Relaxed task list characters](#relaxed-task-list-character) are recognized in cells too:

```````````````````````````````` example -e table -e tasklist --tasklist-in-table --relaxed-tasklist-character
|         | name |
| ------- | ---- |
| [~]     | Rell |
| [x]     | Kai  |
| [x] No. | Ez   |
.
<table>
<thead>
<tr>
<th></th>
<th>name</th>
</tr>
</thead>
<tbody>
<tr>
<td><input type="checkbox" checked="" disabled="" /> </td>
<td>Rell</td>
</tr>
<tr>
<td><input type="checkbox" checked="" disabled="" /> </td>
<td>Kai</td>
</tr>
<tr>
<td>[x] No.</td>
<td>Ez</td>
</tr>
</tbody>
</table>
````````````````````````````````

## Interactions

`--gfm` enables task lists, along with the other GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
- [x] foo
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> foo</li>
</ul>
````````````````````````````````

With [autolinks](autolink.html), the marker must still be followed by whitespace:

```````````````````````````````` example -e tasklist -e autolink
- [x] www.example.com
- [x]www.example.com
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> <a href="http://www.example.com">www.example.com</a></li>
<li>[x]www.example.com</li>
</ul>
````````````````````````````````

With [description lists](description-lists.html), details are not [list items], so they cannot be task list items:

```````````````````````````````` example -e tasklist -e description-lists
Term

: [x] foo
.
<dl>
<dt>Term</dt>
<dd>
<p>[x] foo</p>
</dd>
</dl>
````````````````````````````````

With `--smart`, inline parsing transforms some punctuation.  Inline parsing comes before marker recognition, so a transformed character is not literal and cannot be a [task list item symbol]:

```````````````````````````````` example -e tasklist --relaxed-tasklist-character --smart
- ["] foo
.
<ul>
<li>[”] foo</li>
</ul>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, recognizes task list items differently.  It matches the marker against the source line on which a list item's first block opens, before it parses that block's content.  The line must begin with an optional indent, a [list marker], whitespace, and a [task list item marker], followed by a [space] or [tab].  To decide whether the item is [complete], cmark-gfm searches the whole line for `[x]` or `[X]`.

Comrak recognizes the marker after it determines block and inline structure, as described in [Syntax](#syntax).  The two implementations agree on every example above except in these cases:

- cmark-gfm requires a [space] or [tab] after the marker, on the same line.  It doesn't recognize a marker followed by a line break ([example 7](#example-7), [example 8](#example-8)), or a marker that makes up the whole paragraph ([example 9](#example-9), [example 28](#example-28), [example 29](#example-29)).
- cmark-gfm requires the marker to be on the same line as the [list marker], and that line to begin with the list marker.  It doesn't recognize a first block that begins on the next line ([example 18](#example-18)), or a task list inside a [block quote] ([example 33](#example-33)).
- cmark-gfm recognizes the marker before inline parsing, so the marker takes precedence over a shortcut reference link with a matching definition ([example 22](#example-22)).
- cmark-gfm recognizes the marker before it sees a [setext heading] underline, so it produces a task list item that contains a heading ([example 17](#example-17)).
- cmark-gfm removes all whitespace after the marker ([example 35](#example-35)).
- cmark-gfm marks an item [complete] if `[x]` or `[X]` appears anywhere on its line, so `- [ ] see [x]` is complete ([example 24](#example-24)).

cmark-gfm has no equivalent of the [options](#options) described above.

## Regressions

The following examples guard against regressions.

A multi-byte symbol with `--relaxed-tasklist-character` ([#689](https://github.com/kivikakk/comrak/pull/689)):

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [あ] xy
  - [い] zw
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> xy
<ul>
<li><input type="checkbox" checked="" disabled="" /> zw</li>
</ul>
</li>
</ul>
````````````````````````````````

An [entity reference] cannot supply the whitespace after a marker (found by fuzzing):

```````````````````````````````` example -e tasklist
-→[ ]&NewLine;
.
<ul>
<li>[ ]
</li>
</ul>
````````````````````````````````

Inline content immediately following a marker is preserved ([#559](https://github.com/kivikakk/comrak/pull/559)):

```````````````````````````````` example -e tasklist
- [ ] `Blue`
.
<ul>
<li><input type="checkbox" disabled="" /> <code>Blue</code></li>
</ul>
````````````````````````````````

Source positions account for the removed marker ([#542](https://github.com/kivikakk/comrak/pull/542)):

```````````````````````````````` example -e tasklist --sourcepos
- [x] foo
  - [ ] bar
.
<ul data-sourcepos="1:1-2:11">
<li data-sourcepos="1:1-2:11"><input type="checkbox" checked="" disabled="" /> foo
<ul data-sourcepos="2:3-2:11">
<li data-sourcepos="2:3-2:11"><input type="checkbox" disabled="" /> bar</li>
</ul>
</li>
</ul>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[block quote]: https://spec.commonmark.org/0.31.2/#block-quotes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-break
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[line ending]: https://spec.commonmark.org/0.31.2/#line-ending
[link reference definitions]: https://spec.commonmark.org/0.31.2/#link-reference-definition
[list]: https://spec.commonmark.org/0.31.2/#lists
[list item]: https://spec.commonmark.org/0.31.2/#list-items
[list items]: https://spec.commonmark.org/0.31.2/#list-items
[list marker]: https://spec.commonmark.org/0.31.2/#list-marker
[loose]: https://spec.commonmark.org/0.31.2/#loose
[ordered lists]: https://spec.commonmark.org/0.31.2/#ordered-list
[paragraph]: https://spec.commonmark.org/0.31.2/#paragraph
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-heading
[soft line break]: https://spec.commonmark.org/0.31.2/#soft-line-breaks
[space]: https://spec.commonmark.org/0.31.2/#space
[tab]: https://spec.commonmark.org/0.31.2/#tab
