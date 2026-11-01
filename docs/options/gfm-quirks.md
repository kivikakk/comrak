---
title: GFM quirks
---

```````````````````````````````` example
****abcd**** *_foo_*
.
<p><strong><strong>abcd</strong></strong> <em><em>foo</em></em></p>
````````````````````````````````

```````````````````````````````` example --gfm-quirks
****abcd**** *_foo_*
.
<p><strong>abcd</strong> <em><em>foo</em></em></p>
````````````````````````````````

```````````````````````````````` example
__foo __bar__ baz__
.
<p><strong>foo <strong>bar</strong> baz</strong></p>
````````````````````````````````

```````````````````````````````` example --gfm-quirks
__foo __bar__ baz__
.
<p><strong>foo bar baz</strong></p>
````````````````````````````````

```````````````````````````````` example --gfm-quirks
____foo__ bar__
.
<p><strong>foo bar</strong></p>
````````````````````````````````
