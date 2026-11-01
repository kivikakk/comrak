---
title: Smart punctuation
---

```````````````````````````````` example --smart
'Hello,' "world" ...
.
<p>‘Hello,’ “world” …</p>
````````````````````````````````

```````````````````````````````` example
'Hello,' "world" ...
.
<p>'Hello,' &quot;world&quot; ...</p>
````````````````````````````````

```````````````````````````````` example --smart
Why 'hello' "there". It's good.
.
<p>Why ‘hello’ “there”. It’s good.</p>
````````````````````````````````

```````````````````````````````` example --smart
Hm. Hm.. hm... yes- indeed-- quite---!
.
<p>Hm. Hm.. hm… yes- indeed– quite—!</p>
````````````````````````````````

```````````````````````````````` example --smart
<<hello>>

<< guillemets >> here

a < b > c

<https://example.com>
.
<p>«hello»</p>
<p>« guillemets » here</p>
<p>a &lt; b &gt; c</p>
<p><a href="https://example.com">https://example.com</a></p>
````````````````````````````````
