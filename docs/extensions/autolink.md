---
title: Autolinks
---

An autolink is a bare URL or email address in text, which Comrak turns into a link without any markup around it.  CommonMark's own [autolinks] need angle brackets around the address (`<https://example.com>`); this extension recognises addresses without them.  Use autolinks when you want a bare address to be clickable.

Autolinks originate in [GitHub Flavored Markdown](https://github.github.com/gfm/#autolinks-extension-).  This document describes Comrak's autolinks, which can differ from GitHub's.  GitLab uses Comrak's autolinks, so this document is also the canonical spec for [GitLab Flavored Markdown URL auto-linking](https://docs.gitlab.com/user/markdown/#url-auto-linking).

To enable autolinks, pass `-e autolink` (or `--gfm`) on the command line, or set `extension.autolink` in the options struct.

Here's a paragraph with each kind of autolink:

```````````````````````````````` example -e autolink
Visit www.commonmark.org, read https://spec.commonmark.org/, or write to hello@example.com.
.
<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>, read <a href="https://spec.commonmark.org/">https://spec.commonmark.org/</a>, or write to <a href="mailto:hello@example.com">hello@example.com</a>.</p>
````````````````````````````````

## Syntax

Comrak recognises three kinds of autolink: [www autolinks](#www-autolinks), [URL autolinks](#url-autolinks), and [email autolinks](#email-autolinks).  It finds www and URL autolinks while it parses inline content, and email autolinks afterwards, in the text that remains.

### Valid domains

A [valid domain](@) is one or more segments separated by periods (`.`).  A segment is a sequence of characters that aren't whitespace, punctuation, or symbols, except that underscores (`_`) and hyphens (`-`) are permitted.  No underscore may appear in the last two segments:

```````````````````````````````` example -e autolink
www.a_b.c.com www.a.b_c.com www.a_b.com
.
<p><a href="http://www.a_b.c.com">www.a_b.c.com</a> www.a.b_c.com www.a_b.com</p>
````````````````````````````````

Letters and digits outside ASCII are permitted:

```````````````````````````````` example -e autolink
www.bücher.de www.例え.jp
.
<p><a href="http://www.b%C3%BCcher.de">www.bücher.de</a> <a href="http://www.%E4%BE%8B%E3%81%88.jp">www.例え.jp</a></p>
````````````````````````````````

A domain with more than ten periods is valid even with underscores in its last two segments.  This exception, inherited from cmark-gfm, avoids quadratic behaviour on long inputs:

```````````````````````````````` example -e autolink
www.a.b.c.d.e.f.g.h.i.j.k_l.m_n
.
<p><a href="http://www.a.b.c.d.e.f.g.h.i.j.k_l.m_n">www.a.b.c.d.e.f.g.h.i.j.k_l.m_n</a></p>
````````````````````````````````

### www autolinks

A [www autolink](@) is the text `www.`, followed by a [valid domain] that contains at least one period, followed by any further characters up to where [the autolink ends](#where-an-autolink-ends).

The link's destination is the autolink's text with `http://` in front:

```````````````````````````````` example -e autolink
www.commonmark.org
.
<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink
Visit www.commonmark.org/help for more information.
.
<p>Visit <a href="http://www.commonmark.org/help">www.commonmark.org/help</a> for more information.</p>
````````````````````````````````

`www.` must be lowercase, and the domain after it must contain a period.  None of the following are www autolinks:

```````````````````````````````` example -e autolink
WWW.commonmark.org www.com www.
.
<p>WWW.commonmark.org www.com www.</p>
````````````````````````````````

A www autolink must begin at the start of a line, after whitespace, or after one of `*`, `_`, `~`, or `(`:

```````````````````````````````` example -e autolink
(www.a.com) *www.b.com* _www.c.com_ ~www.d.com~
.
<p>(<a href="http://www.a.com">www.a.com</a>) <em><a href="http://www.b.com">www.b.com</a></em> <em><a href="http://www.c.com">www.c.com</a></em> ~<a href="http://www.d.com">www.d.com</a>~</p>
````````````````````````````````

None of the following are www autolinks:

```````````````````````````````` example -e autolink
awww.a.com "www.b.com" .www.c.com
.
<p>awww.a.com &quot;www.b.com&quot; .www.c.com</p>
````````````````````````````````

### URL autolinks

A [URL autolink](@) is one of the schemes `http://`, `https://`, or `ftp://`, followed by a [valid domain], followed by any further characters up to where [the autolink ends](#where-an-autolink-ends).  The scheme is case-insensitive.  The domain needn't contain a period, but its first character must not be punctuation.

The link's destination is the autolink's text:

```````````````````````````````` example -e autolink
http://commonmark.org

(Visit https://encrypted.google.com/search?q=Markup+(business))

Anonymous FTP is available at ftp://foo.bar.baz.
.
<p><a href="http://commonmark.org">http://commonmark.org</a></p>
<p>(Visit <a href="https://encrypted.google.com/search?q=Markup+(business)">https://encrypted.google.com/search?q=Markup+(business)</a>)</p>
<p>Anonymous FTP is available at <a href="ftp://foo.bar.baz">ftp://foo.bar.baz</a>.</p>
````````````````````````````````

```````````````````````````````` example -e autolink
HTTPS://example.com http://localhost:3000/admin
.
<p><a href="HTTPS://example.com">HTTPS://example.com</a> <a href="http://localhost:3000/admin">http://localhost:3000/admin</a></p>
````````````````````````````````

Other schemes aren't recognised (but see [Relaxed autolinks](#relaxed-autolinks)).  None of the following are URL autolinks:

```````````````````````````````` example -e autolink
smb://a.com irc://a.com http:// http://-a.com
.
<p>smb://a.com irc://a.com http:// http://-a.com</p>
````````````````````````````````

A URL autolink may begin anywhere, unless an ASCII letter comes directly before the scheme.  That letter would be read as part of the scheme:

```````````````````````````````` example -e autolink
1http://a.com xhttp://a.com
.
<p>1<a href="http://a.com">http://a.com</a> xhttp://a.com</p>
````````````````````````````````

### Email autolinks

An [email autolink](@) consists of:

* one or more ASCII letters, digits, periods (`.`), hyphens (`-`), underscores (`_`), or plus signs (`+`);
* an at sign (`@`); and,
* one or more ASCII letters, digits, hyphens, underscores, and periods.

The part after the `@` must contain at least one period, each period must be followed by an ASCII letter or digit, and the last character must be an ASCII letter.

The link's destination is the address with `mailto:` in front:

```````````````````````````````` example -e autolink
foo@bar.baz
.
<p><a href="mailto:foo@bar.baz">foo@bar.baz</a></p>
````````````````````````````````

`+` may appear before the `@`, but not after it:

```````````````````````````````` example -e autolink
hello@mail+xyz.example isn't valid, but hello+xyz@mail.example is.
.
<p>hello@mail+xyz.example isn't valid, but <a href="mailto:hello+xyz@mail.example">hello+xyz@mail.example</a> is.</p>
````````````````````````````````

`.`, `-`, and `_` may appear on both sides of the `@`.  A final `.` isn't part of the address, and a final `-` or `_` means there's no email autolink:

```````````````````````````````` example -e autolink
a.b-c_d@a.b

a.b-c_d@a.b.

a.b-c_d@a.b-

a.b-c_d@a.b_
.
<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a></p>
<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a>.</p>
<p>a.b-c_d@a.b-</p>
<p>a.b-c_d@a.b_</p>
````````````````````````````````

The address must end with a letter, and a second `@` can't follow the first:

```````````````````````````````` example -e autolink
a@b.c1 a@b@c.com
.
<p>a@b.c1 a@<a href="mailto:b@c.com">b@c.com</a></p>
````````````````````````````````

An email autolink may begin anywhere.  The address starts just after the last character that can't be part of it:

```````````````````````````````` example -e autolink
☃foo@bar.com x:foo@bar.com
.
<p>☃<a href="mailto:foo@bar.com">foo@bar.com</a> x:<a href="mailto:foo@bar.com">foo@bar.com</a></p>
````````````````````````````````

With `mailto:` or `xmpp:` directly before the address, the prefix is part of the autolink, and no `mailto:` is added.  An `xmpp:` address may also have a resource after a slash (`/`):

```````````````````````````````` example -e autolink
mailto:foo@bar.com xmpp:foo@bar.com/home foo@bar.com/home
.
<p><a href="mailto:foo@bar.com">mailto:foo@bar.com</a> <a href="xmpp:foo@bar.com/home">xmpp:foo@bar.com/home</a> <a href="mailto:foo@bar.com">foo@bar.com</a>/home</p>
````````````````````````````````

The address must consist of literal characters.  A [backslash escape] or an [entity reference] can't form part of it:

```````````````````````````````` example -e autolink
foo\@bar.com foo&#64;bar.com
.
<p>foo@bar.com foo@bar.com</p>
````````````````````````````````

### Where an autolink ends

A www or URL autolink extends up to the next whitespace character or `<`.  Comrak then removes characters from its end, one at a time, until the autolink no longer ends with any of these:

* one of `?`, `!`, `.`, `,`, `:`, `*`, `_`, `~`, `'`, or `"`;
* a `)`, if the autolink contains more `)` than `(`;
* a `;`, together with any `&` and ASCII letters directly before it; and,
* U+2069 POP DIRECTIONAL ISOLATE, after which Comrak removes nothing further.

Trailing punctuation isn't part of the autolink, but the same characters may appear inside it:

```````````````````````````````` example -e autolink
Visit www.commonmark.org.

Visit www.commonmark.org/a.b.
.
<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>.</p>
<p>Visit <a href="http://www.commonmark.org/a.b">www.commonmark.org/a.b</a>.</p>
````````````````````````````````

```````````````````````````````` example -e autolink
"https://a.com/x", 'https://b.com/y'!
.
<p>&quot;<a href="https://a.com/x">https://a.com/x</a>&quot;, '<a href="https://b.com/y">https://b.com/y</a>'!</p>
````````````````````````````````

An autolink keeps a `)` that matches a `(` within it, so it can sit inside parentheses:

```````````````````````````````` example -e autolink
www.google.com/search?q=Markup+(business)

www.google.com/search?q=Markup+(business)))

(www.google.com/search?q=Markup+(business))

(www.google.com/search?q=Markup+(business)
.
<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>))</p>
<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>)</p>
<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
````````````````````````````````

Only a final `)` is checked, so unbalanced parentheses inside the autolink are kept:

```````````````````````````````` example -e autolink
www.google.com/search?q=(business))+ok
.
<p><a href="http://www.google.com/search?q=(business))+ok">www.google.com/search?q=(business))+ok</a></p>
````````````````````````````````

A final `&`, ASCII letters, and `;` — something that looks like an [entity reference] — aren't part of the autolink.  A `;` on its own is removed too:

```````````````````````````````` example -e autolink
www.google.com/search?q=commonmark&hl=en

www.google.com/search?q=commonmark&hl;

www.google.com/search?q=commonmark&h1;
.
<p><a href="http://www.google.com/search?q=commonmark&amp;hl=en">www.google.com/search?q=commonmark&amp;hl=en</a></p>
<p><a href="http://www.google.com/search?q=commonmark">www.google.com/search?q=commonmark</a>&amp;hl;</p>
<p><a href="http://www.google.com/search?q=commonmark&amp;h1">www.google.com/search?q=commonmark&amp;h1</a>;</p>
````````````````````````````````

`<` ends an autolink immediately:

```````````````````````````````` example -e autolink
www.commonmark.org/he<lp
.
<p><a href="http://www.commonmark.org/he">www.commonmark.org/he</a>&lt;lp</p>
````````````````````````````````

Brackets get no special treatment (but see [Relaxed autolinks](#relaxed-autolinks)):

```````````````````````````````` example -e autolink
http://example.com/[abc]]...
.
<p><a href="http://example.com/%5Babc%5D%5D">http://example.com/[abc]]</a>...</p>
````````````````````````````````

Comrak checks the domain after removing these characters, so a trailing underscore doesn't make the domain invalid:

```````````````````````````````` example -e autolink
www.a.com_ __http://b.com__
.
<p><a href="http://www.a.com">www.a.com</a>_ <strong><a href="http://b.com">http://b.com</a></strong></p>
````````````````````````````````

The autolink's text is the source text as written.  Entity references and backslash escapes in it aren't interpreted:

```````````````````````````````` example -e autolink
www.a.com/?a=1&amp;b=2 www.a.com/a\_b
.
<p><a href="http://www.a.com/?a=1&amp;amp;b=2">www.a.com/?a=1&amp;amp;b=2</a> <a href="http://www.a.com/a%5C_b">www.a.com/a\_b</a></p>
````````````````````````````````

### Autolinks and other syntax

Autolinks aren't recognised in [link text], an [image description], or a [code span]:

```````````````````````````````` example -e autolink
[www.a.com](/x) ![foo@bar.com](/y.png) `http://a.com`
.
<p><a href="/x">www.a.com</a> <img src="/y.png" alt="foo@bar.com" /> <code>http://a.com</code></p>
````````````````````````````````

Nor are they recognised inside brackets that don't form a link (but see [Relaxed autolinks](#relaxed-autolinks)):

```````````````````````````````` example -e autolink
[see www.a.com] [http://b.com] [foo@bar.com]
.
<p>[see www.a.com] [http://b.com] [foo@bar.com]</p>
````````````````````````````````

Emphasis delimiters around an autolink aren't part of it:

```````````````````````````````` example -e autolink
*www.a.com* **http://b.com** _foo@bar.com_
.
<p><em><a href="http://www.a.com">www.a.com</a></em> <strong><a href="http://b.com">http://b.com</a></strong> <em><a href="mailto:foo@bar.com">foo@bar.com</a></em></p>
````````````````````````````````

## Output

HTML output renders an autolink as an `<a>` element, just as it renders an inline link.  The `href` is the destination described above, with the usual percent-encoding.  The element's content is the autolink's text.

CommonMark output writes any link whose text equals its destination, ignoring a `mailto:` prefix, as a CommonMark [autolink][autolinks].  URL and email autolinks therefore come out as `<https://a.com>` and `<foo@bar.com>`, but a www autolink comes out as an inline link, `[www.a.com](http://www.a.com)`.  With the extension enabled, CommonMark output escapes `@` in other text (`a\@b`), to prevent reading the text back as an email autolink.

In XML output, an autolink is a `link` element with the destination in its `destination` attribute and an empty `title`.

In the AST, an autolink is a [`NodeValue::Link`](https://docs.rs/comrak/latest/comrak/nodes/enum.NodeValue.html#variant.Link), like an inline link.  Its [`NodeLink`](https://docs.rs/comrak/latest/comrak/nodes/struct.NodeLink.html) holds the destination and an empty title, and its only child is a text node.  Nothing in the AST distinguishes an autolink from an inline link with the same text and destination.

An autolink's source position covers its text:

```````````````````````````````` example -e autolink --sourcepos
x www.a.com y http://b.com z foo@bar.com
.
<p data-sourcepos="1:1-1:40">x <a data-sourcepos="1:3-1:11" href="http://www.a.com">www.a.com</a> y <a data-sourcepos="1:15-1:26" href="http://b.com">http://b.com</a> z <a data-sourcepos="1:30-1:40" href="mailto:foo@bar.com">foo@bar.com</a></p>
````````````````````````````````

## Options

### Relaxed autolinks

With `--relaxed-autolinks` (`parse.relaxed_autolinks`), Comrak recognises autolinks in more places.

Autolinks are recognised inside brackets that don't form a link:

```````````````````````````````` example -e autolink --relaxed-autolinks
[www.a.com] [http://b.com] [foo@bar.com]
.
<p>[<a href="http://www.a.com">www.a.com</a>] [<a href="http://b.com">http://b.com</a>] [<a href="mailto:foo@bar.com">foo@bar.com</a>]</p>
````````````````````````````````

A URL autolink may use any scheme of ASCII letters, or none at all:

```````````````````````````````` example -e autolink --relaxed-autolinks
smb:///Volumes/shared/foo.pdf irc://irc.freenode.net/git rdar://localhost.com/blah ://a.com
.
<p><a href="smb:///Volumes/shared/foo.pdf">smb:///Volumes/shared/foo.pdf</a> <a href="irc://irc.freenode.net/git">irc://irc.freenode.net/git</a> <a href="rdar://localhost.com/blah">rdar://localhost.com/blah</a> <a href="://a.com">://a.com</a></p>
````````````````````````````````

A www autolink's domain needn't contain a period, and a URL autolink's domain may begin with punctuation, as in an IPv6 address:

```````````````````````````````` example -e autolink --relaxed-autolinks
www.a http://[::1]:80/
.
<p><a href="http://www.a">www.a</a> <a href="http://[::1]:80/">http://[::1]:80/</a></p>
````````````````````````````````

The rule for a final `)` also applies to `]` and `}`.  Comrak removes a final `]` if the autolink contains more `]` than `[`, and likewise for `}`:

```````````````````````````````` example -e autolink --relaxed-autolinks
{https://foo.com} http://example.com/[abc]]...
.
<p>{<a href="https://foo.com">https://foo.com</a>} <a href="http://example.com/%5Babc%5D">http://example.com/[abc]</a>]...</p>
````````````````````````````````

Comrak takes a www or URL autolink that runs into `](` to be the text of a link, and doesn't recognise it:

```````````````````````````````` example -e autolink --relaxed-autolinks
[http://foo.com/](url) [www.foo.com/](url)
.
<p><a href="url">http://foo.com/</a> <a href="url">www.foo.com/</a></p>
````````````````````````````````

## Interactions

`--gfm` enables autolinks, along with the other GitHub Flavored Markdown extensions:

```````````````````````````````` example --gfm
www.commonmark.org
.
<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>
````````````````````````````````

Autolinks work in [table](table.html) cells, and a cell's pipes aren't part of them:

```````````````````````````````` example -e autolink -e table
| site | contact |
| --- | --- |
|www.a.com|foo@bar.com|
.
<table>
<thead>
<tr>
<th>site</th>
<th>contact</th>
</tr>
</thead>
<tbody>
<tr>
<td><a href="http://www.a.com">www.a.com</a></td>
<td><a href="mailto:foo@bar.com">foo@bar.com</a></td>
</tr>
</tbody>
</table>
````````````````````````````````

With [strikethrough](strikethrough.html), the tildes aren't part of the autolink:

```````````````````````````````` example -e autolink -e strikethrough
~~www.a.com~~ ~https://b.com~
.
<p><del><a href="http://www.a.com">www.a.com</a></del> <del><a href="https://b.com">https://b.com</a></del></p>
````````````````````````````````

With [task lists](tasklist.html), an autolink straight after a task list item marker means the item isn't a task list item:

```````````````````````````````` example -e autolink -e tasklist
- [ ] www.a.com
- [x]https://b.com
.
<ul>
<li><input type="checkbox" disabled="" /> <a href="http://www.a.com">www.a.com</a></li>
<li>[x]<a href="https://b.com">https://b.com</a></li>
</ul>
````````````````````````````````

With `--smart`, quotation marks around an autolink become curly, and aren't part of it:

```````````````````````````````` example -e autolink --smart
'foo@bar.com' "https://a.com"
.
<p>‘<a href="mailto:foo@bar.com">foo@bar.com</a>’ “<a href="https://a.com">https://a.com</a>”</p>
````````````````````````````````

With [wikilinks](wikilinks.html), a wikilink takes precedence over an autolink inside it:

```````````````````````````````` example -e autolink -e wikilinks-title-after-pipe
[[www.a.com]] [[Docs|https://b.com]]
.
<p><a href="www.a.com" data-wikilink="true">www.a.com</a> <a href="Docs" data-wikilink="true">https://b.com</a></p>
````````````````````````````````

## Differences from cmark-gfm

[cmark-gfm](https://github.com/github/cmark-gfm), the reference implementation of GitHub Flavored Markdown, uses the same autolink design as Comrak.  It has no equivalent of [Relaxed autolinks](#relaxed-autolinks), and doesn't support the [wikilinks](wikilinks.html) or [footnotes](footnotes.html) extensions, so those examples aren't comparable.  On every other example above, the two implementations agree on HTML output, except that cmark-gfm:

* accepts any [valid domain] after `www.`, with or without a period, so `www.com` is a www autolink ([example 7](#example-7)), and `www.foo@bar.com` is a www autolink rather than an email autolink;
* checks the domain before removing trailing characters, so a trailing underscore makes a www or URL autolink invalid, even when the underscore closes emphasis ([example 8](#example-8), [example 28](#example-28));
* recognises email autolinks formed by backslash escapes or entity references ([example 20](#example-20));
* recognises email autolinks inside brackets that don't form a link ([example 31](#example-31));
* doesn't remove a trailing U+2069 POP DIRECTIONAL ISOLATE ([example 45](#example-45)); and,
* gives autolinks no source position ([example 33](#example-33), [example 47](#example-47), [example 48](#example-48)).

## Regressions

The following examples guard against regressions.

Unicode isolates around a URL aren't part of the autolink ([#654](https://github.com/kivikakk/comrak/pull/654)).  This example has an invisible U+2068 before the URL and an invisible U+2069 after it:

```````````````````````````````` example -e autolink
⁨https://www.example.com/⁩
.
<p>⁨<a href="https://www.example.com/">https://www.example.com/</a>⁩</p>
````````````````````````````````

A relaxed email autolink in what looks like a footnote definition ([#677](https://github.com/kivikakk/comrak/pull/677)):

```````````````````````````````` example -e autolink -e footnotes --relaxed-autolinks
[^a@b.c
A]:
.
<p>[^<a href="mailto:a@b.c">a@b.c</a>
A]:</p>
````````````````````````````````

Source positions of email autolinks in a table cell, around escaped pipes:

```````````````````````````````` example -e autolink -e table --sourcepos
| a@b.cd\|e@f.gh\|i |
| - |
.
<table data-sourcepos="1:1-2:5">
<thead>
<tr data-sourcepos="1:1-1:21">
<th data-sourcepos="1:2-1:20"><a data-sourcepos="1:3-1:8" href="mailto:a@b.cd">a@b.cd</a>|<a data-sourcepos="1:11-1:16" href="mailto:e@f.gh">e@f.gh</a>|i</th>
</tr>
</thead>
</table>
````````````````````````````````

Source positions of an email autolink after smart punctuation:

```````````````````````````````` example -e autolink --smart --sourcepos
a--foo@bar.com
.
<p data-sourcepos="1:1-1:14">a–<a data-sourcepos="1:4-1:14" href="mailto:foo@bar.com">foo@bar.com</a></p>
````````````````````````````````

[autolinks]: https://spec.commonmark.org/0.31.2/#autolinks
[backslash escape]: https://spec.commonmark.org/0.31.2/#backslash-escapes
[code span]: https://spec.commonmark.org/0.31.2/#code-span
[entity reference]: https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references
[image description]: https://spec.commonmark.org/0.31.2/#image-description
[link text]: https://spec.commonmark.org/0.31.2/#link-text
