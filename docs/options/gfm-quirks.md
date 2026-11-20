---
title: GFM quirks
---

`--gfm-quirks` (`render.gfm_quirks`) makes output follow the base spec of [GitHub Flavored Markdown](https://github.github.com/gfm/) in the one place it differs from CommonMark's: a strong emphasis directly inside another strong emphasis _isn't_ written as a second, nested `<strong>` element.

CommonMark nests the two:

```````````````````````````````` example
****abcd****
.
<p><strong><strong>abcd</strong></strong></p>
````````````````````````````````

With the option, only the outer one is written:

```````````````````````````````` example --gfm-quirks
****abcd****
.
<p><strong>abcd</strong></p>
````````````````````````````````

The option changes how Comrak writes a document, not how it parses one.  `--gfm` enables it.

## Output

HTML output leaves out the `<strong>` element for a strong emphasis whose parent is a strong emphasis, and writes its content in its place.  This applies even when the inner strong emphasis has siblings:

```````````````````````````````` example --gfm-quirks
__foo __bar__ baz__
.
<p><strong>foo bar baz</strong></p>
````````````````````````````````

Any depth of nesting becomes a single `<strong>`:

```````````````````````````````` example --gfm-quirks
******abcd******
.
<p><strong>abcd</strong></p>
````````````````````````````````

Only a strong emphasis whose immediate parent is a strong emphasis is affected.  When emphasis or a link comes between them, both are written:

```````````````````````````````` example --gfm-quirks
**a *b **c** d* e**

**a [**b**](/url) c**
.
<p><strong>a <em>b <strong>c</strong> d</em> e</strong></p>
<p><strong>a <a href="/url"><strong>b</strong></a> c</strong></p>
````````````````````````````````

Emphasis inside emphasis is unaffected:

```````````````````````````````` example --gfm-quirks
*_foo_*
.
<p><em><em>foo</em></em></p>
````````````````````````````````

With [`--sourcepos`](sourcepos.html), only the outer strong emphasis has a source position, since it's the only one written:

```````````````````````````````` example --gfm-quirks --sourcepos
****abcd****
.
<p data-sourcepos="1:1-1:12"><strong data-sourcepos="1:1-1:12">abcd</strong></p>
````````````````````````````````

CommonMark output leaves out the inner strong emphasis's delimiters in the same way, so the result reads back as one strong emphasis.  The CommonMark view of [example 3](#example-3) shows this.

XML output and the AST don't change.  Both contain every strong emphasis.

## Interactions

[`--gfm`](gfm.html) enables GFM quirks, along with the GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
****abcd****
.
<p><strong>abcd</strong></p>
````````````````````````````````

With [strikethrough](../extensions/strikethrough.html), strikethrough inside strikethrough is written as usual:

```````````````````````````````` example -e strikethrough --gfm-quirks
~~a ~~b~~ c~~
.
<p><del>a <del>b</del> c</del></p>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, always writes strong emphasis this way, in both its HTML and its CommonMark output.  It has no option for it.  The GFM spec's own examples expect the same: `****foo****` is `<strong>foo</strong>` in the GFM spec, and `<strong><strong>foo</strong></strong>` in [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/#example-464).

On every example above that uses `--gfm-quirks` or `--gfm`, the two implementations agree on HTML output, except that cmark-gfm gives strong emphasis no source position ([example 7](#example-7)).  github.com writes the same HTML for all of them.

Comrak added the option in 0.25.0 ([#428](https://github.com/kivikakk/comrak/pull/428)), so that it could support both CommonMark's base spec and GFM's.
