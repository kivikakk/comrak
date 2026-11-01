---
title: Strikethrough
---

```````````````````````````````` example -e strikethrough
Hello ~world~ there.
.
<p>Hello <del>world</del> there.</p>
````````````````````````````````

```````````````````````````````` example -e strikethrough
This is ~strikethrough~.

As is ~~this, okay~~?

This ~text~~~~ is ~~~~curious~.
.
<p>This is <del>strikethrough</del>.</p>
<p>As is <del>this, okay</del>?</p>
<p>This <del>text~~~~ is ~~~~curious</del>.</p>
````````````````````````````````

```````````````````````````````` example -e strikethrough,subscript
~~H~2~O~~
.
<p><del>H<sub>2</sub>O</del></p>
````````````````````````````````

```````````````````````````````` example -e strikethrough
a~~a()~~あ

「~~a」~~a
.
<p>a~~a()~~あ</p>
<p>「~~a」~~a</p>
````````````````````````````````

```````````````````````````````` example -e strikethrough,cjk-friendly-emphasis
a~~a()~~あ

「~~a」~~a
.
<p>a<del>a()</del>あ</p>
<p>「<del>a」</del>a</p>
````````````````````````````````
