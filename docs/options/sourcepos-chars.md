---
title: Source positions in characters
---

`--sourcepos-chars` (`parse.sourcepos_chars`) counts the columns of [source positions](sourcepos.html#source-position) in Unicode characters instead of bytes.  It changes the positions in the AST, and doesn't change any output unless [`--sourcepos`](sourcepos.html) is set.

For the full rules, see [Source positions in characters](sourcepos.html#source-positions-in-characters).

Without the option, each byte of a character is a column:

```````````````````````````````` example --sourcepos
aɽ好𐀀
.
<p data-sourcepos="1:1-1:10">aɽ好𐀀</p>
````````````````````````````````

With the option, each character is a column:

```````````````````````````````` example --sourcepos --sourcepos-chars
aɽ好𐀀
.
<p data-sourcepos="1:1-1:4">aɽ好𐀀</p>
````````````````````````````````
