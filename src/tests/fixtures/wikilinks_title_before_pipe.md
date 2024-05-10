---
title: Wikilinks
based_on: https://github.com/jgm/commonmark-hs/blob/master/commonmark-extensions/test/wikilinks_title_before_pipe.md
---

# Wikilinks, title before pipe

Wikilinks can have one of the following forms:

    [[https://example.org]]
    [[title|https://example.org]]
    [[name of page]]
    [[title|name of page]]

With this version of wikilinks, the title comes before the pipe.

```````````````````````````````` example
[[https://example.org]]
.
<p><a href="https://example.org">https://example.org</a></p>
````````````````````````````````

```````````````````````````````` example
[[title|https://example.org]]
.
<p><a href="https://example.org">title</a></p>
````````````````````````````````

```````````````````````````````` example
[[Name of page]]
.
<p><a href="Name%20of%20page">Name of page</a></p>
````````````````````````````````

```````````````````````````````` example
[[Title|Name of page]]
.
<p><a href="Name%20of%20page">Title</a></p>
````````````````````````````````

Regular links should still work!

```````````````````````````````` example
[Title](Name%20of%20page)
.
<p><a href="Name%20of%20page">Title</a></p>
````````````````````````````````

HTML entities are recognized both in the name of page and in the link title.

```````````````````````````````` example
[[&#xDC;ber &amp;nbsp;|Gesch&uuml;tztes Leerzeichen]]
.
<p><a href="Gesch%C3%BCtztes%20Leerzeichen">Über &amp;nbsp;</a></p>
````````````````````````````````