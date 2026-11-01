---
title: Inline footnotes
---

```````````````````````````````` example -e footnotes -e inline-footnotes
Here is an inline footnote^[This is the footnote content].
.
<p>Here is an inline footnote<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>This is the footnote content <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
Text^[With *emphasis* and **strong**].
.
<p>Text<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>With <em>emphasis</em> and <strong>strong</strong> <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
First^[First note] and second^[Second note].
.
<p>First<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup> and second<sup class="footnote-ref"><a href="#fn-__inline_2" id="fnref-__inline_2" data-footnote-ref>2</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>First note <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-__inline_2">
<p>Second note <a href="#fnref-__inline_2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
Regular[^1] and inline^[Inline note] and another regular[^2].

[^1]: First regular footnote.
[^2]: Second regular footnote.
.
<p>Regular<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup> and inline<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>2</a></sup> and another regular<sup class="footnote-ref"><a href="#fn-2" id="fnref-2" data-footnote-ref>3</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>First regular footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-__inline_1">
<p>Inline note <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
<li id="fn-2">
<p>Second regular footnote. <a href="#fnref-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="3" aria-label="Back to reference 3">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
Here is an inline note.^[Inline notes are easier to write, since
you don't have to pick an identifier and move down to type the
note.]
.
<p>Here is an inline note.<sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>Inline notes are easier to write, since
you don't have to pick an identifier and move down to type the
note. <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
A ^[outer and ^[inner] literal].
.
<p>A <sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>outer and <sup class="footnote-ref"><a href="#fn-__inline_2" id="fnref-__inline_2" data-footnote-ref>2</a></sup> literal <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-__inline_2">
<p>inner <a href="#fnref-__inline_2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
Escaped: \^[not a note]. Real: ^[yes].
.
<p>Escaped: ^[not a note]. Real: <sup class="footnote-ref"><a href="#fn-__inline_1" id="fnref-__inline_1" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-__inline_1">
<p>yes <a href="#fnref-__inline_1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e inline-footnotes
Text^[] should not parse.
.
<p>Text^[] should not parse.</p>
````````````````````````````````
