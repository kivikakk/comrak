---
title: Maths
extensions: [math-dollars, math-dollars-block, math-latex, math-code]
with_thanks_to:
  - https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/math.md
  - https://docs.gitlab.com/user/markdown/#math-equations
---

Maths extensions mark up mathematical notation, written in TeX, for a renderer such as [KaTeX](https://katex.org/) or [MathJax](https://www.mathjax.org/) to typeset in the browser.  Comrak doesn't typeset maths itself.  It finds the maths in a document, keeps its content as literal text, and marks it in the HTML output with a `data-math-style` attribute.

Comrak supports four maths syntaxes, each with its own extension:

* `math-dollars`: `$…$` and `$$…$$`, following [Pandoc](https://pandoc.org/MANUAL.html#extension-tex_math_dollars);
* `math-dollars-block`: `$$` on lines of their own, for display maths that contains blank lines or lines that look like Markdown;
* `math-latex`: LaTeX's `\(…\)` and `\[…\]`; and,
* `math-code`: `` $`…`$ `` and code blocks marked `math`, following GitLab and GitHub.

This document describes all four.  GitLab uses Comrak's dollar maths and code maths, so this document is also the canonical spec for [GitLab Flavored Markdown maths](https://docs.gitlab.com/user/markdown/#math-equations).

To enable a maths syntax, pass `-e math-dollars`, `-e math-dollars-block`, `-e math-latex` or `-e math-code` on the command line, or set `extension.math_dollars`, `extension.math_dollars_block`, `extension.math_latex` or `extension.math_code` in the options struct.  Block maths also needs `math-dollars`.

Here's some typical dollar maths:

```````````````````````````````` example -e math-dollars
Let $x$ and $y$ be integers such that
$$x = y + 2$$
.
<p>Let <span data-math-style="inline">x</span> and <span data-math-style="inline">y</span> be integers such that
<span data-math-style="display">x = y + 2</span></p>
````````````````````````````````

Each syntax produces [inline maths](@), which sits within a line of text, or [display maths](@), which a renderer sets apart on a line of its own.  In this document, [whitespace](@) means a space, a tab, a line ending, a vertical tab or a form feed.

## Dollar maths

With `math-dollars`, a single dollar sign (`$`) delimits [inline maths], and two (`$$`) delimit [display maths].  A run of three or more dollar signs can't begin maths.

### Inline dollar maths

Inline dollar maths begins with a `$` that isn't followed by [whitespace].  Its content runs to the next unescaped `$`, which closes it:

```````````````````````````````` example -e math-dollars
$a^2 + b^2 = c^2$
.
<p><span data-math-style="inline">a^2 + b^2 = c^2</span></p>
````````````````````````````````

A `$` followed by whitespace can't begin inline maths:

```````````````````````````````` example -e math-dollars
$ x$
.
<p>$ x$</p>
````````````````````````````````

The `$` that would close inline maths must not be preceded by whitespace, or followed by an ASCII digit (`0`–`9`).  Otherwise, the opening `$` is literal text:

```````````````````````````````` example -e math-dollars
$x $
.
<p>$x $</p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
$a$1
.
<p>$a$1</p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
It costs $20,000 or $30,000.
.
<p>It costs $20,000 or $30,000.</p>
````````````````````````````````

Only the next unescaped `$` can close inline maths.  If it can't, a later `$` doesn't either.  Here, the first `$` is literal, and the second begins inline maths:

```````````````````````````````` example -e math-dollars
$a $b$
.
<p>$a <span data-math-style="inline">b</span></p>
````````````````````````````````

Inline maths may begin and end inside a word:

```````````````````````````````` example -e math-dollars
a$x$b
.
<p>a<span data-math-style="inline">x</span>b</p>
````````````````````````````````

The first `$` of a `$$` run can close inline maths.  The second is literal text:

```````````````````````````````` example -e math-dollars
$x$$
.
<p><span data-math-style="inline">x</span>$</p>
````````````````````````````````

Inline maths may span lines, and keeps its line endings, but can't span paragraphs:

```````````````````````````````` example -e math-dollars
$a
b$
.
<p><span data-math-style="inline">a
b</span></p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
$a

b$
.
<p>$a</p>
<p>b$</p>
````````````````````````````````

### Display dollar maths

Display dollar maths begins with `$$` and ends at the next unescaped `$$`:

```````````````````````````````` example -e math-dollars
$$x = y + 2$$
.
<p><span data-math-style="display">x = y + 2</span></p>
````````````````````````````````

Its content may begin or end with whitespace, and may contain a single `$`:

```````````````````````````````` example -e math-dollars
$$ x = \$1 $$
.
<p><span data-math-style="display"> x = \$1 </span></p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
$$a$b$$
.
<p><span data-math-style="display">a$b</span></p>
````````````````````````````````

Display maths may span lines, but can't span paragraphs.  Without [block maths](#block-maths), it remains inside its paragraph:

```````````````````````````````` example -e math-dollars
Hence
$$
x = y
$$
.
<p>Hence
<span data-math-style="display">
x = y
</span></p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
$$
x

y
$$
.
<p>$$
x</p>
<p>y
$$</p>
````````````````````````````````

A run of three or more dollar signs can't begin maths:

```````````````````````````````` example -e math-dollars
$$$x$$$
.
<p>$$$x$$$</p>
````````````````````````````````

A `$$` run begins only display maths.  If no `$$` closes it, it's literal text, instead of a dollar sign followed by inline maths:

```````````````````````````````` example -e math-dollars
$$x$
.
<p>$$x$</p>
````````````````````````````````

### Content

The content of dollar maths is literal text.  Comrak doesn't parse Markdown, raw HTML or entity references in it, and keeps any backslashes as-is:

```````````````````````````````` example -e math-dollars
$*a* <b> &amp; \alpha$
.
<p><span data-math-style="inline">*a* &lt;b&gt; &amp;amp; \alpha</span></p>
````````````````````````````````

A backslash before a dollar sign stops that dollar sign closing the maths.  The backslash stays in the content, for the renderer to interpret:

```````````````````````````````` example -e math-dollars
$\text{\$}5$
.
<p><span data-math-style="inline">\text{\$}5</span></p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
$$\$$$
.
<p><span data-math-style="display">\$</span></p>
````````````````````````````````

A backslash escapes a dollar sign only if the backslash isn't itself escaped.  `\\` is a line break in TeX; `\\$` closes maths:

```````````````````````````````` example -e math-dollars
$a\\$ b$
.
<p><span data-math-style="inline">a\\</span> b$</p>
````````````````````````````````

### Precedence

Dollar maths and [code spans][code span] take precedence by position: whichever begins first wins:

```````````````````````````````` example -e math-dollars
$x `y$ z`
.
<p><span data-math-style="inline">x `y</span> z`</p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
`a $b` c$
.
<p><code>a $b</code> c$</p>
````````````````````````````````

Dollar maths takes precedence over emphasis and links that begin before it but end inside it:

```````````````````````````````` example -e math-dollars
*$x*$*
.
<p><em><span data-math-style="inline">x*</span></em></p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
[$x](/u)$
.
<p>[<span data-math-style="inline">x](/u)</span></p>
````````````````````````````````

Raw HTML that begins first takes precedence over dollar maths:

```````````````````````````````` example -e math-dollars --unsafe
<a title="$x">$</a>
.
<p><a title="$x">$</a></p>
````````````````````````````````

A [backslash escape] or an [entity reference] is a literal dollar sign, which can't begin or end maths:

```````````````````````````````` example -e math-dollars
\$x$
.
<p>$x$</p>
````````````````````````````````

```````````````````````````````` example -e math-dollars
&#36;x&#36;
.
<p>$x$</p>
````````````````````````````````

## Block maths

With `math-dollars-block`, a line holding only `$$` begins and ends display maths that may contain blank lines, and lines that would otherwise be Markdown.  Block maths needs `math-dollars`.

Block maths came from [#880](https://github.com/kivikakk/comrak/issues/880).  Display dollar maths lives inside a paragraph, so a line of `=` in it underlines a [setext heading]:

```````````````````````````````` example -e math-dollars
$$
1
=
2
$$
.
<h1>$$
1</h1>
<p>2
$$</p>
````````````````````````````````

With block maths, the same lines are display maths:

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$
1
=
2
$$
.
<div data-math-style="display">
1
=
2
</div>
````````````````````````````````

A [maths fence](@) is a line holding `$$`, after up to three spaces of indentation, followed by nothing but spaces and tabs.  Block maths begins with a [maths fence], and ends with the next maths fence, or at the end of its container.  Its content is literal:

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$
# x

- y &amp; z
$$
.
<div data-math-style="display">
# x

- y &amp;amp; z
</div>
````````````````````````````````

Comrak removes as much indentation from each content line as the opening fence had, up to the amount the line has:

```````````````````````````````` example -e math-dollars -e math-dollars-block
  $$
  x
    y
z
  $$
.
<div data-math-style="display">
x
  y
z
</div>
````````````````````````````````

Four spaces of indentation begin an [indented code block] instead:

```````````````````````````````` example -e math-dollars -e math-dollars-block
    $$
    x
    $$
.
<pre class="syntax-highlighting"><code><span class="text plain">$$
x
$$
</span></code></pre>
````````````````````````````````

A line with anything else after `$$` isn't a maths fence.  This includes a run of three or more dollar signs:

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$
x
$$ y
$$
.
<div data-math-style="display">
x
$$ y
</div>
````````````````````````````````

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$$
x
$$$
.
<p>$$$
x
$$$</p>
````````````````````````````````

A maths fence may interrupt a paragraph:

```````````````````````````````` example -e math-dollars -e math-dollars-block
a
$$
x
$$
b
.
<p>a</p>
<div data-math-style="display">
x
</div>
<p>b</p>
````````````````````````````````

A maths fence always begins block maths, even where it would otherwise close display dollar maths.  Here, the last line begins block maths, so `$$ x` stays literal text, and the block maths runs to the end of the document:

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$ x
y
$$
.
<p>$$ x
y</p>
<div data-math-style="display">
</div>
````````````````````````````````

Without a closing fence, block maths ends with its container.  Unlike a paragraph, it has no [lazy continuation lines][lazy continuation line]:

```````````````````````````````` example -e math-dollars -e math-dollars-block
> $$
> x
y
.
<blockquote>
<div data-math-style="display">
x
</div>
</blockquote>
<p>y</p>
````````````````````````````````

A line holding display dollar maths and nothing else isn't a maths fence, so the display maths stays in its paragraph:

```````````````````````````````` example -e math-dollars -e math-dollars-block
$$x$$
.
<p><span data-math-style="display">x</span></p>
````````````````````````````````

## LaTeX delimiters

With `math-latex`, `\(` and `\)` delimit [inline maths], and `\[` and `\]` delimit [display maths], as in LaTeX.  The extension came from [#469](https://github.com/kivikakk/comrak/issues/469).

LaTeX maths begins with `\(` or `\[`, and ends at the next unescaped `\)` or `\]` respectively.  Its content is literal text, and may begin or end with whitespace:

```````````````````````````````` example -e math-latex
\(x\) and \[ y = 2 \]
.
<p><span data-math-style="inline">x</span> and <span data-math-style="display"> y = 2 </span></p>
````````````````````````````````

Without `math-latex`, `\(` and `\[` are [backslash escapes][backslash escape].  With it, text that escapes brackets may become maths:

```````````````````````````````` example -e math-latex
\[x\](/u)
.
<p><span data-math-style="display">x</span>(/u)</p>
````````````````````````````````

When LaTeX maths can't form, its opening `\(` or `\[` is a backslash escape after all.  This happens when the content would be empty, or when there's no closing delimiter of the same kind:

```````````````````````````````` example -e math-latex
\(\) and \[\]
.
<p>() and []</p>
````````````````````````````````

```````````````````````````````` example -e math-latex
\(x\]
.
<p>(x]</p>
````````````````````````````````

A backslash escapes a closing delimiter only if the backslash isn't itself escaped.  `\\` is a line break in TeX, and so doesn't close maths:

```````````````````````````````` example -e math-latex
\(a\\)b\)
.
<p><span data-math-style="inline">a\\)b</span></p>
````````````````````````````````

LaTeX maths doesn't nest.  The first closing delimiter closes it:

```````````````````````````````` example -e math-latex
\(a\(b\)c\)
.
<p><span data-math-style="inline">a\(b</span>c)</p>
````````````````````````````````

LaTeX maths may span lines, but can't span paragraphs:

```````````````````````````````` example -e math-latex
\[
a
\]
.
<p><span data-math-style="display">
a
</span></p>
````````````````````````````````

```````````````````````````````` example -e math-latex
\(a

b\)
.
<p>(a</p>
<p>b)</p>
````````````````````````````````

LaTeX maths and [code spans][code span] take precedence by position, as dollar maths does:

```````````````````````````````` example -e math-latex
\(a `b\) c`
.
<p><span data-math-style="inline">a `b</span> c`</p>
````````````````````````````````

## Code maths

With `math-code`, a [code span] or a [fenced code block] can hold maths.  GitLab and GitHub both support this syntax.

### Inline code maths

Inline code maths is a [code span] with a dollar sign immediately before its opening backtick string and immediately after its closing backtick string:

```````````````````````````````` example -e math-code
$`a^2 + b^2 = c^2`$
.
<p><code data-math-style="inline">a^2 + b^2 = c^2</code></p>
````````````````````````````````

Its content follows the rules for code spans.  Comrak turns line endings into spaces, and strips one space from each end if both ends have one:

```````````````````````````````` example -e math-code
$` x `$ and $`a
b`$
.
<p><code data-math-style="inline">x</code> and <code data-math-style="inline">a b</code></p>
````````````````````````````````

Longer backtick strings let the content contain backticks:

```````````````````````````````` example -e math-code
$``a`b``$
.
<p><code data-math-style="inline">a`b</code></p>
````````````````````````````````

Without a dollar sign on both sides, a code span isn't code maths, and its dollar signs are literal text.  Here, the code span ends at the first backtick, so no dollar sign follows it:

```````````````````````````````` example -e math-code
$`a`b`$
.
<p>$<code>a</code>b`$</p>
````````````````````````````````

```````````````````````````````` example -e math-code
$`x` and `y`$
.
<p>$<code>x</code> and <code>y</code>$</p>
````````````````````````````````

```````````````````````````````` example -e math-code
$ `x`$
.
<p>$ <code>x</code>$</p>
````````````````````````````````

### Maths code blocks

A [fenced code block] whose [info string] begins with the word `math` is display maths.  The rest of the info string doesn't affect whether the block is maths:

```````````````````````````````` example -e math-code
```math
a^2 + b^2 = c^2
```
.
<pre><code class="language-math" data-math-style="display">a^2 + b^2 = c^2
</code></pre>
````````````````````````````````

```````````````````````````````` example -e math-code
~~~math foo
x
~~~
.
<pre><code class="language-math" data-math-style="display">x
</code></pre>
````````````````````````````````

The word is case-sensitive:

```````````````````````````````` example -e math-code
```Math
x
```
.
<pre class="syntax-highlighting"><code class="language-Math"><span class="text plain">x
</span></code></pre>
````````````````````````````````

Without `math-code`, a code block marked `math` is an ordinary code block:

```````````````````````````````` example -e math-dollars
```math
x
```
.
<pre class="syntax-highlighting"><code class="language-math"><span class="text plain">x
</span></code></pre>
````````````````````````````````

## Output

HTML output marks each kind of maths with a `data-math-style` attribute, whose value is `inline` or `display`:

* dollar maths and LaTeX maths use a `<span>`;
* inline code maths uses a `<code>`;
* block maths uses a `<div>`, with its content starting on the line after the opening tag; and,
* a maths code block uses `<pre><code class="language-math">`, with the attribute on the `<code>`.

In each case, Comrak escapes the content as HTML text.  The rendered examples on this page use KaTeX to typeset it.

With `--github-pre-lang`, a maths code block puts both `lang` and `data-math-style` on the `<pre>`:

```````````````````````````````` example -e math-code --github-pre-lang
```math
x
```
.
<pre data-math-style="display" lang="math"><code>x
</code></pre>
````````````````````````````````

Otherwise, a maths code block takes the same attributes as any other code block.  With `--full-info-string`, the rest of the info string becomes a `data-meta` attribute beside `data-math-style`, and [fenced code attributes](attributes.html#fenced-code-blocks) go on the `<pre>`:

```````````````````````````````` example -e math-code -e fenced-code-attributes --full-info-string --attributes --unsafe
```math numbered {#pythagoras}
a^2 + b^2 = c^2
```
.
<pre id="pythagoras"><code class="language-math" data-math-style="display" data-meta="numbered">a^2 + b^2 = c^2
</code></pre>
````````````````````````````````

With `math-code`, Comrak renders maths code blocks itself, ignoring any [codefence renderer](https://docs.rs/comrak/latest/comrak/options/struct.RenderPlugins.html#structfield.codefence_renderers) for `math`.  Without it, a codefence renderer for `math` handles those blocks as it would any other.

CommonMark output writes dollar maths with dollar signs, block maths with maths fences, and code maths with enough backticks to hold its content.  The AST doesn't record whether maths came from dollar signs or LaTeX delimiters.  When `math-latex` is enabled, CommonMark output uses LaTeX delimiters if `math-dollars` isn't enabled, or if dollar signs wouldn't read back as the same maths, such as inline maths that begins with a space:

```````````````````````````````` example -e math-dollars -e math-latex
\(x\) and \( y\)
.
<p><span data-math-style="inline">x</span> and <span data-math-style="inline"> y</span></p>
````````````````````````````````

CommonMark output also escapes dollar signs in text, to stop them reading back as maths:

```````````````````````````````` example -e math-dollars
$x$ costs \$5.
.
<p><span data-math-style="inline">x</span> costs $5.</p>
````````````````````````````````

In XML output, dollar maths, LaTeX maths and inline code maths are `math` elements, with a `math_style` attribute of `inline` or `display`.  Block maths is a `math_block` element.  A maths code block is a `code_block` element with `math_style="display"`.

In the AST, dollar maths, LaTeX maths and inline code maths are a [`NodeValue::Math`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Math).  Its [`NodeMath`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeMath.html) holds the content, `display_math`, and `dollar_math`, which is false for code maths and true otherwise.  Block maths is a [`NodeValue::MathBlock`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.MathBlock), and a maths code block is a [`NodeValue::CodeBlock`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.CodeBlock).

The [source position](../options/sourcepos.html#source-position) of maths covers its delimiters:

```````````````````````````````` example -e math-dollars -e math-code --sourcepos
a $x$ and $`y`$
.
<p data-sourcepos="1:1-1:15">a <span data-math-style="inline" data-sourcepos="1:3-1:5">x</span> and <code data-math-style="inline" data-sourcepos="1:11-1:15">y</code></p>
````````````````````````````````

## Interactions

The maths extensions work together:

```````````````````````````````` example -e math-dollars -e math-code -e math-latex
$x$ and $`y`$ and \(z\)
.
<p><span data-math-style="inline">x</span> and <code data-math-style="inline">y</code> and <span data-math-style="inline">z</span></p>
````````````````````````````````

With both `math-dollars` and `math-code`, a dollar sign followed by a backtick can only begin code maths:

```````````````````````````````` example -e math-dollars -e math-code
$`x` + y$
.
<p>$<code>x</code> + y$</p>
````````````````````````````````

`--gfm` doesn't enable any maths syntax:

```````````````````````````````` example --gfm
$x$
.
<p>$x$</p>
````````````````````````````````

With [tables](table.html), a pipe in maths still ends a cell, unless it's escaped.  The tables page [covers this](table.html#interactions).

With [smart punctuation](../options/smart.html), maths content stays as it is:

```````````````````````````````` example -e math-dollars --smart
$a--b "c"$ -- "d"
.
<p><span data-math-style="inline">a--b &quot;c&quot;</span> – “d”</p>
````````````````````````````````

With [autolinks](autolink.html), dollar signs in a URL are part of it:

```````````````````````````````` example -e math-dollars -e autolink
https://example.com/$x$
.
<p><a href="https://example.com/$x$">https://example.com/$x$</a></p>
````````````````````````````````

In image descriptions, maths contributes its content to the `alt` text:

```````````````````````````````` example -e math-dollars
![$x$ squared](/x2.png)
.
<p><img src="/x2.png" alt="x squared" /></p>
````````````````````````````````

With [attributes](attributes.html), inline code maths doesn't take attributes, but a maths code block does, as [Output](#output) shows.  The attributes page [covers both](attributes.html#interactions).

## Differences from Pandoc and GitHub

[Pandoc](https://pandoc.org/) 3.7 supports dollar maths in its `commonmark` reader (`commonmark+tex_math_dollars`), LaTeX delimiters in its `markdown` reader (`markdown+tex_math_single_backslash`), and code maths in its `gfm` reader.  It has no block maths.  Setting aside how each writes line breaks and code blocks in HTML, Comrak and Pandoc produce the same HTML for every example above that Pandoc supports, except in these cases:

* A closing `$` followed by a digit doesn't close inline maths in Comrak ([example 5](#example-5)).  Pandoc's `markdown` reader has this rule, but its `commonmark` reader doesn't.
* In Comrak, a run of three or more dollar signs can't begin maths, and a `$$` run can begin only display maths ([example 17](#example-17), [example 18](#example-18)).  Pandoc reads `$$$x$$$` as a dollar sign, display maths and a dollar sign, and `$$x$` as a dollar sign followed by inline maths.
* Comrak reads `$$a$b$$` as display maths that contains `a$b` ([example 14](#example-14)).  Pandoc reads a dollar sign, inline maths that contains `a`, and the text `b$$`.
* Pandoc's code maths keeps its content as written, and ends at the first backtick followed by a dollar sign ([example 51](#example-51), [example 53](#example-53), [example 54](#example-54)).  Comrak follows the rules for code spans, as GitHub does.
* With dollar maths and code maths both enabled, a dollar sign followed by a backtick can only begin code maths in Comrak ([example 65](#example-65)).  Pandoc reads it as the start of dollar maths.

Pandoc also trims whitespace from both ends of inline LaTeX maths, so `\( x \)` holds `x`.  Comrak keeps the spaces.

GitHub supports dollar maths and code maths.  Its dollar maths differs too much from Comrak's for a useful comparison: it parses Markdown inside dollar maths, and renders `$$…$$` within a line as inline maths.  Its code maths matches Comrak's on every example above, except [example 54](#example-54), where GitHub turns the text between the two code spans into maths.

## Regressions

The following examples guard against regressions.

Inline maths may immediately follow inline maths ([#533](https://github.com/kivikakk/comrak/pull/533)):

```````````````````````````````` example -e math-dollars
$\alpha$$\beta$
.
<p><span data-math-style="inline">\alpha</span><span data-math-style="inline">\beta</span></p>
````````````````````````````````

Source positions are correct for maths that spans lines ([#855](https://github.com/kivikakk/comrak/pull/855)):

```````````````````````````````` example -e math-dollars --sourcepos
a $x
y$ b
.
<p data-sourcepos="1:1-2:4">a <span data-math-style="inline" data-sourcepos="1:3-2:2">x
y</span> b</p>
````````````````````````````````

CommonMark output escapes literal dollar signs, and literal brackets when `math-latex` is enabled ([#917](https://github.com/kivikakk/comrak/pull/917)):

```````````````````````````````` example -e math-dollars
a \$x$ b
.
<p>a $x$ b</p>
````````````````````````````````

```````````````````````````````` example -e math-latex
\[x\] [y]
.
<p><span data-math-style="display">x</span> [y]</p>
````````````````````````````````

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-spans
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[fenced code block]: https://spec.commonmark.org/0.31.2/#fenced-code-blocks
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-blocks
[info string]: https://spec.commonmark.org/0.31.2/#info-string
[lazy continuation line]: https://spec.commonmark.org/0.31.2/#lazy-continuation-line
[setext heading]: https://spec.commonmark.org/0.31.2/#setext-headings
