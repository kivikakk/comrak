---
title: Minimise CommonMark escapes
---

```````````````````````````````` example
__hi
.
<p>__hi</p>
````````````````````````````````

```````````````````````````````` example --experimental-minimize-commonmark
__hi
.
<p>__hi</p>
````````````````````````````````

```````````````````````````````` example
This is some text.

It contains [brackets] which could be important.

It contains #hashes# and !exclamation marks! and * asterisks *, _ underscores _,
the < works >.

Let's include some *important\* _ones\_ too.
.
<p>This is some text.</p>
<p>It contains [brackets] which could be important.</p>
<p>It contains #hashes# and !exclamation marks! and * asterisks *, _ underscores _,
the &lt; works &gt;.</p>
<p>Let's include some *important* _ones_ too.</p>
````````````````````````````````

```````````````````````````````` example --experimental-minimize-commonmark
This is some text.

It contains [brackets] which could be important.

It contains #hashes# and !exclamation marks! and * asterisks *, _ underscores _,
the < works >.

Let's include some *important\* _ones\_ too.
.
<p>This is some text.</p>
<p>It contains [brackets] which could be important.</p>
<p>It contains #hashes# and !exclamation marks! and * asterisks *, _ underscores _,
the &lt; works &gt;.</p>
<p>Let's include some *important* _ones_ too.</p>
````````````````````````````````

```````````````````````````````` example --experimental-minimize-commonmark
1\. not a list

\# not a heading

snake_case_word and 2 * 3 * 4
.
<p>1. not a list</p>
<p># not a heading</p>
<p>snake_case_word and 2 * 3 * 4</p>
````````````````````````````````

```````````````````````````````` example -e autolink --experimental-minimize-commonmark
a_b@c.d_
.
<p>a_b@c.d_</p>
````````````````````````````````
