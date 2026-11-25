---
title: GitHub-style pre lang
---

`--github-pre-lang` (`render.github_pre_lang`) puts a fenced code block's language on its `<pre>` element, as a `lang` attribute, instead of on its `<code>` element as a `language-` class.  github.com does the same for code in languages it doesn't highlight.

By default, the language is a class on the `<code>`:

```````````````````````````````` example
```rust
x
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust">x
</span></code></pre>
````````````````````````````````

With the option, it's the `<pre>`'s `lang`:

```````````````````````````````` example --github-pre-lang
```rust
x
```
.
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

`--gfm` enables the option.

## Output

The language is the first word of the [info string], after Comrak resolves backslash escapes and entity references:

```````````````````````````````` example --github-pre-lang
``` f&#35; extra
x
```
.
<pre class="syntax-highlighting" lang="f#"><code><span class="text plain">x
</span></code></pre>
````````````````````````````````

A code block without an info string has no `lang` attribute, whether it's fenced or indented:

```````````````````````````````` example --github-pre-lang
```
x
```

    y
.
<pre class="syntax-highlighting"><code><span class="text plain">x
</span></code></pre>
<pre class="syntax-highlighting"><code><span class="text plain">y
</span></code></pre>
````````````````````````````````

With [`--default-info-string`](default-info-string.html), a fenced code block without an info string takes the default language:

```````````````````````````````` example --github-pre-lang --default-info-string rust
```
x
```
.
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

With `--full-info-string`, the rest of the info string moves to the `<pre>` too, as a `data-meta` attribute:

```````````````````````````````` example --github-pre-lang --full-info-string
```rust ignore
x
```
.
<pre class="syntax-highlighting" data-meta="ignore" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

CommonMark and XML output don't change.

A [codefence renderer](https://docs.rs/comrak/latest/comrak/options/struct.RenderPlugins.html#structfield.codefence_renderers) writes the whole of its code block's HTML, so the option doesn't affect blocks in that renderer's language.

## Interactions

[`--gfm`](gfm.html) enables GitHub-style pre lang, along with the GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
```rust
x
```
.
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

With [code maths](../extensions/math.html#maths-code-blocks), a maths code block puts `data-math-style` on the `<pre>`, beside `lang`:

```````````````````````````````` example -e math-code --github-pre-lang
```math
x^2
```
.
<pre data-math-style="display" lang="math"><code>x^2
</code></pre>
````````````````````````````````

With [fenced code attributes](../extensions/attributes.html#fenced-code-blocks), attributes go on the `<pre>` as usual.  Comrak's own `lang` takes precedence over a `lang` attribute:

```````````````````````````````` example -e fenced-code-attributes --github-pre-lang --attributes --unsafe
``` rust {#main lang=ruby}
x
```
.
<pre class="syntax-highlighting" id="main" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, has the same option.  It doesn't support `--default-info-string`, code maths or attributes, so those examples aren't comparable.  On every other example above, the two implementations agree on HTML output, except in attribute order.  Comrak writes a code block's attributes in alphabetical order, so `data-meta` comes before `lang` ([example 6](#example-6)), where cmark-gfm writes `lang` first.

github.com writes `<pre lang>` for code in languages it doesn't highlight, with the rest of the info string as `data-meta`.  For languages it does highlight, it replaces the code block's HTML with its own.

[info string]: https://spec.commonmark.org/0.31.2/#info-string
