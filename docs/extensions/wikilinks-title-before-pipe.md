---
title: Wikilinks (title before pipe)
based_on: https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/wikilinks_title_before_pipe.md
---

# Wikilinks, title before pipe

Wikilinks can have one of the following forms:

    [[https://example.org]]
    [[title|https://example.org]]
    [[name of page]]
    [[title|name of page]]

With this version of wikilinks, the title comes before the pipe.

```````````````````````````````` example -e wikilinks-title-before-pipe
[[https://example.org]]
.
<p><a href="https://example.org" data-wikilink="true">https://example.org</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-before-pipe
[[title|https://example.org]]
.
<p><a href="https://example.org" data-wikilink="true">title</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-before-pipe
[[Name of page]]
.
<p><a href="Name%20of%20page" data-wikilink="true">Name of page</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-before-pipe
[[Title|Name of page]]
.
<p><a href="Name%20of%20page" data-wikilink="true">Title</a></p>
````````````````````````````````

Regular links should still work!

```````````````````````````````` example -e wikilinks-title-before-pipe
[Title](Name%20of%20page)
.
<p><a href="Name%20of%20page">Title</a></p>
````````````````````````````````

HTML entities are recognized both in the name of page and in the link title.

```````````````````````````````` example -e wikilinks-title-before-pipe
[[&#xDC;ber &amp;nbsp;|Gesch&uuml;tztes Leerzeichen]]
.
<p><a href="Gesch%C3%BCtztes%20Leerzeichen" data-wikilink="true">Über &amp;nbsp;</a></p>
````````````````````````````````

Escaping characters is supported

```````````````````````````````` example -e wikilinks-title-before-pipe
[[foo\[\]bar|https://example.org]]
.
<p><a href="https://example.org" data-wikilink="true">foo[]bar</a></p>
````````````````````````````````

```````````````````````````````` example -e wikilinks-title-before-pipe
[[Name \[of\] page]]
.
<p><a href="Name%20%5Bof%5D%20page" data-wikilink="true">Name [of] page</a></p>
````````````````````````````````

Emphasis or other inline markdown is not supported

```````````````````````````````` example -e wikilinks-title-before-pipe
[[Name _of_ page]]
.
<p><a href="Name%20_of_%20page" data-wikilink="true">Name _of_ page</a></p>
````````````````````````````````
