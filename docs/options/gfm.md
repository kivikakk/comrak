---
title: GitHub Flavored Markdown
---

`--gfm` turns on the extensions from [GitHub Flavored Markdown](https://github.github.com/gfm/), plus two options that make Comrak's HTML output more like GitHub's.  It's a command-line-only shorthand for six options:

* the [strikethrough](../extensions/strikethrough.html), [tables](../extensions/table.html), [autolinks](../extensions/autolink.html) and [task lists](../extensions/tasklist.html) extensions;
* [GFM quirks](gfm-quirks.html), which stops a strong emphasis nesting directly inside another; and,
* [GitHub-style pre lang](github-pre-lang.html), which puts a code block's language on its `<pre>` element.

Without `--gfm`, none of these apply:

```````````````````````````````` example
* [x] ~~Draft~~ Write the docs
* [ ] Publish them at www.example.com

| Option | Effect |
| ------ | ------ |
| `--gfm` | ****All of it**** |

```rust
x
```
.
<ul>
<li>[x] ~~Draft~~ Write the docs</li>
<li>[ ] Publish them at www.example.com</li>
</ul>
<p>| Option | Effect |
| ------ | ------ |
| <code>--gfm</code> | <strong><strong>All of it</strong></strong> |</p>
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust">x
</span></code></pre>
````````````````````````````````

With `--gfm`, all of them do:

```````````````````````````````` example --gfm
* [x] ~~Draft~~ Write the docs
* [ ] Publish them at www.example.com

| Option | Effect |
| ------ | ------ |
| `--gfm` | ****All of it**** |

```rust
x
```
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> <del>Draft</del> Write the docs</li>
<li><input type="checkbox" disabled="" /> Publish them at <a href="http://www.example.com">www.example.com</a></li>
</ul>
<table>
<thead>
<tr>
<th>Option</th>
<th>Effect</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>--gfm</code></td>
<td><strong>All of it</strong></td>
</tr>
</tbody>
</table>
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust">x
</span></code></pre>
````````````````````````````````

The options struct has no equivalent of `--gfm`.  Set each option it enables instead:

```rust
let mut options = comrak::Options::default();
options.extension.strikethrough = true;
options.extension.table = true;
options.extension.autolink = true;
options.extension.tasklist = true;
options.render.gfm_quirks = true;
options.render.github_pre_lang = true;
```

## Differences from the GFM spec

The GFM spec describes five extensions.  Comrak supports four of them.  The fifth, [disallowed raw HTML](https://github.github.com/gfm/#disallowed-raw-html-extension-), filters out a fixed list of tags and leaves all other raw HTML alone; Comrak removed it in 0.56.0 ([#845](https://github.com/kivikakk/comrak/pull/845)).  Comrak omits raw HTML unless `--unsafe` is given, with or without `--gfm`.

The spec's examples write a fenced code block's language as a class on its `<code>` element.  `--gfm` writes it as a `lang` attribute on the `<pre>` instead, as github.com does for languages it doesn't highlight.  [GitHub-style pre lang](github-pre-lang.html) covers this.

Each extension's page compares Comrak with [cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of the spec.

## Differences from github.com

github.com renders some features that aren't in the GFM spec.  Comrak has an extension or option for each, but `--gfm` doesn't enable them, and they can differ in exactly how they're parsed and output:

* [footnotes](../extensions/footnotes.html), with `-e footnotes`;
* [alerts](../extensions/alerts.html), with `-e alerts`;
* [maths](../extensions/math.html), with `-e math-dollars` and `-e math-code`;
* [emoji shortcodes](gemoji.html), with `--gemoji`; and,
* the rest of a code block's info string as a `data-meta` attribute, with [`--full-info-string`](full-info-string.html).

github.com also highlights code in the languages it recognises, replacing the code block's HTML with its own.  Comrak leaves highlighting to a [syntax highlighter plugin](https://docs.rs/comrak/latest/comrak/options/struct.RenderPlugins.html#structfield.codefence_syntax_highlighter).
