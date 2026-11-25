---
title: Default info string
---

`--default-info-string` (`parse.default_info_string`) supplies an [info string] for every fenced code block that lacks one.  The info string is usually the code's language, so the option provides a fallback language for the syntax highlighter when an author leaves the language out.

By default, a fenced code block without an info string doesn't have a language:

```````````````````````````````` example
```
fn hello();
```
.
<pre class="syntax-highlighting"><code><span class="text plain">fn hello();
</span></code></pre>
````````````````````````````````

With `--default-info-string rust`, it's Rust:

```````````````````````````````` example --default-info-string rust
```
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

The default is used, as if it was written in the document.  Comrak added `--default-info-string` in 0.2.8.

## Which code blocks

The default applies to a [fenced code block] whose info string is empty.  An info string of only spaces or tabs is empty:

```````````````````````````````` example --default-info-string rust
```→
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

A fence of tildes works the same way:

```````````````````````````````` example --default-info-string rust
~~~
fn hello();
~~~
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

A code block with an info string is unchanged:

```````````````````````````````` example --default-info-string rust
```python
print(1)
```
.
<pre class="syntax-highlighting"><code class="language-python"><span class="source python"><span class="meta function-call python"><span class="meta qualified-name python"><span class="support function builtin python">print</span></span><span class="punctuation section arguments begin python">(</span><span class="meta function-call arguments python"><span class="constant numeric integer decimal python">1</span></span><span class="punctuation section arguments end python">)</span></span>
</span></code></pre>
````````````````````````````````

An [indented code block] can't have an info string, and the default doesn't apply to it:

```````````````````````````````` example --default-info-string rust
    fn hello();
.
<pre class="syntax-highlighting"><code><span class="text plain">fn hello();
</span></code></pre>
````````````````````````````````

With [fenced code attributes](../extensions/attributes.html#fenced-code-blocks), an info string that contains only an attribute block is empty too: the default is applied.

Comrak trims whitespace from both ends of the default, as it does from a written info string.  Otherwise, it uses the default as given, without resolving backslash escapes or entity references:

```````````````````````````````` example --default-info-string "a&amp;amp;b"
```
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-a&amp;amp;b"><span class="text plain">fn hello();
</span></code></pre>
````````````````````````````````

A default that's empty after trimming doesn't have any effect.

## Output

HTML output takes the language from the first word of the info string, as usual.  Syntax highlighting and [codefence renderers](https://docs.rs/comrak/latest/comrak/options/struct.RenderPlugins.html#structfield.codefence_renderers) see the default language too.

CommonMark output writes the default after the opening fence ([example 2](#example-2)).  Indented blocks not changed to fenced ([example 6](#example-6)).  With `render.prefer_fenced`, Comrak all code blocks with fences, and the default _is_ applied to all empty fences.

In XML output and in the AST ([`NodeCodeBlock::info`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeCodeBlock.html#structfield.info)), the default is copied into elements without an info string.  Nothing indicates that the info string came from the default.

## Interactions

With [`--github-pre-lang`](github-pre-lang.html), the default language goes on the `<pre>`:

```````````````````````````````` example --default-info-string rust --github-pre-lang
```
fn hello();
```
.
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

With [`--full-info-string`](full-info-string.html), the rest of the default goes in `data-meta`:

```````````````````````````````` example --default-info-string "rust ignore" --full-info-string
```
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust" data-meta="ignore"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

With [code maths](../extensions/math.html#maths-code-blocks), a default of `math` makes every fenced code block without an info string a maths code block:

```````````````````````````````` example --default-info-string math -e math-code
```
x^2
```
.
<pre><code class="language-math" data-math-style="display">x^2
</code></pre>
````````````````````````````````

[fenced code block]: https://spec.commonmark.org/0.31.2/#fenced-code-blocks
[indented code block]: https://spec.commonmark.org/0.31.2/#indented-code-blocks
[info string]: https://spec.commonmark.org/0.31.2/#info-string
