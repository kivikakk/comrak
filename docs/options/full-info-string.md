---
title: Full info string
---

`--full-info-string` (`render.full_info_string`) writes the rest of a fenced code block's [info string], after its first word, in the `data-meta` attribute.  The first word is usually the code's language.  The rest is free-form, and the option makes it available to scripts and stylesheets on the rendered page.

By default, HTML output only includes the language:

```````````````````````````````` example
``` rust extra info
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

With the option, the rest of the info string is put in `data-meta`:

```````````````````````````````` example --full-info-string
``` rust extra info
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust" data-meta="extra info"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

Comrak added `--full-info-string` in 0.17.0 ([#276](https://github.com/kivikakk/comrak/pull/276)).

## Output

A fenced code block's [meta](@) is the rest of its [info string] after the first word and the spaces or tabs that follow it.  The option writes the meta in a `data-meta` attribute, on the same element as the language.

Comrak skips every space or tab between the first word and the meta.  Intra-meta whitespace is kept:

```````````````````````````````` example --full-info-string
``` rust   extra   info
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust" data-meta="extra   info"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

A code block without a meta doesn't get the attribute.  That includes a code block whose info string is a single word, and one without an info string, like an [indented code block]:

```````````````````````````````` example --full-info-string
``` rust
fn hello();
```

    fn hello();
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
<pre class="syntax-highlighting"><code><span class="text plain">fn hello();
</span></code></pre>
````````````````````````````````

Comrak resolves [backslash escapes][backslash escape] and [entity references][entity reference] in the meta, as in the rest of the info string.  HTML output then escapes the result:

```````````````````````````````` example --full-info-string
``` rust title="m&ouml;de.rs" \{linenos\}
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust" data-meta="title=&quot;möde.rs&quot; {linenos}"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

CommonMark and XML output don't change.  Both write the whole info string; XML writes it in the `code_block` element's `info` attribute.

A [syntax highlighter](https://docs.rs/comrak/latest/comrak/adapters/struct.SyntaxHighlightMeta.html#structfield.meta) or a [codefence renderer](https://docs.rs/comrak/latest/comrak/adapters/struct.CodefenceMeta.html#structfield.meta) receives the meta whether or not the option is set.  The option only adds the attribute.  Comrak passes `data-meta` to a syntax highlighter with the code block's other attributes.  A codefence renderer writes all of its code block's HTML, so `data-meta` only appears if the renderer writes it.

## Interactions

With [`--github-pre-lang`](github-pre-lang.html), the language and the meta are both on the `<pre>`:

```````````````````````````````` example --full-info-string --github-pre-lang
``` rust extra info
fn hello();
```
.
<pre class="syntax-highlighting" data-meta="extra info" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

With [`--default-info-string`](default-info-string.html), a default of more than one word provides a meta too ([example 9 on that page](default-info-string.html#example-9)).

With [code maths](../extensions/math.html#maths-code-blocks), a maths code block's meta is in `data-meta`, next to `data-math-style`:

```````````````````````````````` example --full-info-string -e math-code
``` math numbered
x^2
```
.
<pre><code class="language-math" data-math-style="display" data-meta="numbered">x^2
</code></pre>
````````````````````````````````

With [fenced code attributes](../extensions/attributes.html#fenced-code-blocks), the attribute block at the end of the info string isn't part of the meta:

```````````````````````````````` example --full-info-string -e fenced-code-attributes --attributes --unsafe
``` rust extra {.x}
fn hello();
```
.
<pre class="syntax-highlighting x"><code class="language-rust" data-meta="extra"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

An author's `data-meta` attribute goes on the `<pre>`, like any fenced code attribute, and doesn't conflict with the option's `data-meta` on the `<code>`.  With `--github-pre-lang`, both are on the `<pre>`, and the option's `data-meta` takes precedence:

```````````````````````````````` example --full-info-string --github-pre-lang -e fenced-code-attributes --attributes --unsafe
``` rust extra {data-meta=y}
fn hello();
```
.
<pre class="syntax-highlighting" data-meta="extra" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, has the same option.  It doesn't support code maths or attributes, so those examples aren't comparable.  On every other example above, the two implementations have the same HTML output, except in these cases:

* cmark-gfm skips only the first space or tab after the language, and keeps the rest at the start of `data-meta` ([example 3](#example-3)).
* With `--github-pre-lang`, cmark-gfm writes `lang` before `data-meta`, whereas Comrak writes a code block's attributes in alphabetical order ([example 6](#example-6)).

[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-blocks
[info string]: https://spec.commonmark.org/0.31.2/#info-string
