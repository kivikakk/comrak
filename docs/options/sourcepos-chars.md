---
title: Source positions in characters
---

```````````````````````````````` example --sourcepos --sourcepos-chars
aɽ好𐀀
.
<p data-sourcepos="1:1-1:4">aɽ好𐀀</p>
````````````````````````````````

```````````````````````````````` example --sourcepos
aɽ好𐀀
.
<p data-sourcepos="1:1-1:10">aɽ好𐀀</p>
````````````````````````````````

```````````````````````````````` example --sourcepos --sourcepos-chars
ɽ**ɽ**ɽ
.
<p data-sourcepos="1:1-1:7">ɽ<strong data-sourcepos="1:2-1:6">ɽ</strong>ɽ</p>
````````````````````````````````

```````````````````````````````` example --sourcepos
ɽ**ɽ**ɽ
.
<p data-sourcepos="1:1-1:10">ɽ<strong data-sourcepos="1:3-1:8">ɽ</strong>ɽ</p>
````````````````````````````````

```````````````````````````````` example --sourcepos --sourcepos-chars
`𐀀`
.
<p data-sourcepos="1:1-1:3"><code data-sourcepos="1:1-1:3">𐀀</code></p>
````````````````````````````````

```````````````````````````````` example --sourcepos
`𐀀`
.
<p data-sourcepos="1:1-1:6"><code data-sourcepos="1:1-1:6">𐀀</code></p>
````````````````````````````````
