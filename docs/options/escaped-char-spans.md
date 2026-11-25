---
title: Escaped character spans
---

`--escaped-char-spans` (`render.escaped_char_spans`) wraps each character made literal by a [backslash escape] in a `<span data-escaped-char>` element.  The HTML then records where the author escaped a character, which a post-processing filter otherwise can't determine.  For example, a filter that turns `@user` into a link to a user's profile can leave `\@user` alone.

By default, an escaped character is written as the character alone:

```````````````````````````````` example
Thanks, \@user!  See \#123.
.
<p>Thanks, @user!  See #123.</p>
````````````````````````````````

With the option, each one is wrapped:

```````````````````````````````` example --escaped-char-spans
Thanks, \@user!  See \#123.
.
<p>Thanks, <span data-escaped-char>@</span>user!  See <span data-escaped-char>#</span>123.</p>
````````````````````````````````

The option doesn't change anything else about the HTML: removing the spans leaves the output as Comrak would write otherwise.  Comrak added `--escaped-char-spans` in 0.22.0 ([#367](https://github.com/kivikakk/comrak/pull/367)).

## Escaped characters

An [escaped character](@) is an [ASCII punctuation character] preceded by a backslash, in inline content where [backslash escapes][backslash escape] apply.  Escaped characters are wrapped wherever the inline content appears, including inside emphasis and link text:

```````````````````````````````` example --escaped-char-spans
*a\*b* and [c\]d](/url)
.
<p><em>a<span data-escaped-char>*</span>b</em> and <a href="/url">c<span data-escaped-char>]</span>d</a></p>
````````````````````````````````

A backslash before any other character is a literal backslash, and not an escape:

```````````````````````````````` example --escaped-char-spans
\a \é \1
.
<p>\a \é \1</p>
````````````````````````````````

A backslash at the end of a line is a [hard line break]:

```````````````````````````````` example --escaped-char-spans
a\
b
.
<p>a<br />
b</p>
````````````````````````````````

An escaped backslash is an escaped character too:

```````````````````````````````` example --escaped-char-spans
\\
.
<p><span data-escaped-char>\</span></p>
````````````````````````````````

Backslash escapes don't apply in code spans, code blocks, [autolinks] or raw HTML, so backslashes in them are literal:

```````````````````````````````` example --escaped-char-spans
`\*` <https://example.com/\*>
.
<p><code>\*</code> <a href="https://example.com/%5C*">https://example.com/\*</a></p>
````````````````````````````````

In a link destination, a link title and an image description, a backslash escape works as usual.  Comrak writes these as attribute values, which can't contain elements; no wrapping can occur:

```````````````````````````````` example --escaped-char-spans
![a\*b](/c\*d "e\*f")
.
<p><img src="/c*d" alt="a*b" title="e*f" /></p>
````````````````````````````````

The same goes for the `aria-label` and `data-heading-content` of a heading's anchor, with [header IDs](header-id-prefix.html), and for an [alert](../extensions/alerts.html)'s title.

## Output

HTML output writes an [escaped character] inside a `<span data-escaped-char>` element.  The character is entity-encoded for HTML as usual:

```````````````````````````````` example --escaped-char-spans
\<b\> \&amp;
.
<p><span data-escaped-char>&lt;</span>b<span data-escaped-char>&gt;</span> <span data-escaped-char>&amp;</span>amp;</p>
````````````````````````````````

With [`--sourcepos`](sourcepos.html), each span's [source position](sourcepos.html#source-position) covers both the backslash and the character:

```````````````````````````````` example --escaped-char-spans --sourcepos
Test \`hello world\` here.
.
<p data-sourcepos="1:1-1:26">Test <span data-escaped-char data-sourcepos="1:6-1:7">`</span>hello world<span data-escaped-char data-sourcepos="1:19-1:20">`</span> here.</p>
````````````````````````````````

XML output writes an escaped character as an `escaped` element that contains its `text`.

CommonMark output writes every escaped character with its backslash.  Without the option, CommonMark output decides for itself where a backslash is needed, and doesn't include author's backslashes it doesn't consider necessary, so `\@user` comes out as `@user`.  Compare the CommonMark views of [example 1](#example-1) and [example 2](#example-2).

In the AST, an escaped character is a [`NodeValue::Escaped`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Escaped) node, with a single text node as its child.

## Escaped characters in the AST

The `parse.escaped_char_spans` option keeps `Escaped` nodes in the AST without changing the HTML.  CommonMark output then keeps every backslash escape, like it does with `--escaped-char-spans`:

```rust
use comrak::{Options, markdown_to_commonmark};

let mut options = Options::default();
assert_eq!(markdown_to_commonmark("\\@user", &options), "@user\n");

options.parse.escaped_char_spans = true;
assert_eq!(markdown_to_commonmark("\\@user", &options), "\\@user\n");
```

XML output only writes `escaped` elements with `render.escaped_char_spans`.  With `parse.escaped_char_spans` alone, it writes each escaped character as a `text` element of its own.

`parse.escaped_char_spans` is only available through the API; the command line doesn't have a flag for it.  `render.escaped_char_spans` keeps `Escaped` nodes whether or not `parse.escaped_char_spans` is set.  Comrak added the option in 0.48.0 ([#679](https://github.com/kivikakk/comrak/pull/679)).

## Interactions

With [wikilinks](../extensions/wikilinks.html), escaped characters in a wikilink's title are wrapped:

```````````````````````````````` example -e wikilinks-title-after-pipe --escaped-char-spans
[[Home|Start\|here]]
.
<p><a href="Home" data-wikilink="true">Start<span data-escaped-char>|</span>here</a></p>
````````````````````````````````

With [tables](../extensions/table.html), a backslash before a pipe in a cell [escapes the pipe](../extensions/table.html#cell-content) for the table, not as inline content.  The table removes that backslash before parsing the cell, and so it doesn't get wrapped.  Other escaped characters in a cell do:

```````````````````````````````` example -e table --escaped-char-spans
| a \| b | `c\|d` | e\*f |
| - | - | - |
.
<table>
<thead>
<tr>
<th>a | b</th>
<th><code>c|d</code></th>
<th>e<span data-escaped-char>*</span>f</th>
</tr>
</thead>
</table>
````````````````````````````````

With [autolinks](../extensions/autolink.html), a URL or www autolink [takes a backslash as part of the address](../extensions/autolink.html#where-an-autolink-ends).  The backslash doesn't escape the character after it, and nothing is wrapped:

```````````````````````````````` example -e autolink --escaped-char-spans
www.example.com\_
.
<p><a href="http://www.example.com%5C">www.example.com\</a>_</p>
````````````````````````````````

With [`--smart`](smart.html), an escaped quotation mark, hyphen or full stop isn't converted to typographic punctuation, and is wrapped:

```````````````````````````````` example --smart --escaped-char-spans
\"a\" b\-\-c d\.\.\.
.
<p><span data-escaped-char>&quot;</span>a<span data-escaped-char>&quot;</span> b<span data-escaped-char>-</span><span data-escaped-char>-</span>c d<span data-escaped-char>.</span><span data-escaped-char>.</span><span data-escaped-char>.</span></p>
````````````````````````````````

[ASCII punctuation character]: https://spec.commonmark.org/0.31.2/#ascii-punctuation-character
[autolinks]: https://spec.commonmark.org/0.31.2/#autolinks
[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[hard line break]: https://spec.commonmark.org/0.31.2/#hard-line-break
