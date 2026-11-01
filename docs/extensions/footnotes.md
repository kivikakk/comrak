---
title: Footnotes
---

```````````````````````````````` example -e footnotes
Hi[^x].

[^x]: A greeting.
.
<p>Hi<sup class="footnote-ref"><a href="#fn-x" id="fnref-x" data-footnote-ref>1</a></sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-x">
<p>A greeting. <a href="#fnref-x" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes
Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]

This is another note.[^note] And footnote[^longnote] is referenced again.

[^note]: Hi.

[^1]: Here is the footnote.

[^longnote]: Here's one with multiple blocks.

    Subsequent paragraphs are indented.

        code

This is regular content.

[^unused]: This is not used.
.
<p>Here is a[^nowhere] footnote reference,<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup> and another.<sup class="footnote-ref"><a href="#fn-longnote" id="fnref-longnote" data-footnote-ref>2</a></sup></p>
<p>This is another note.<sup class="footnote-ref"><a href="#fn-note" id="fnref-note" data-footnote-ref>3</a></sup> And footnote<sup class="footnote-ref"><a href="#fn-longnote" id="fnref-longnote-2" data-footnote-ref>2</a></sup> is referenced again.</p>
<p>This is regular content.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Here is the footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-longnote">
<p>Here's one with multiple blocks.</p>
<p>Subsequent paragraphs are indented.</p>
<pre><code>code
</code></pre>
<a href="#fnref-longnote" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a> <a href="#fnref-longnote-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2-2" aria-label="Back to reference 2-2">↩<sup class="footnote-ref">2</sup></a>
</li>
<li id="fn-note">
<p>Hi. <a href="#fnref-note" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="3" aria-label="Back to reference 3">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes
Here is a footnote reference.[^AB] and [^ab]

[^aB]: Here is the footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-aB" id="fnref-aB" data-footnote-ref>1</a></sup> and <sup class="footnote-ref"><a href="#fn-aB" id="fnref-aB-2" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-aB">
<p>Here is the footnote. <a href="#fnref-aB" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-aB-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes
Here is a footnote reference.[^😄ref]

[^😄ref]: Here is the footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-%F0%9F%98%84ref" id="fnref-%F0%9F%98%84ref" data-footnote-ref>1</a></sup></p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-%F0%9F%98%84ref">
<p>Here is the footnote. <a href="#fnref-%F0%9F%98%84ref" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes
Foo.[^one two]

[^one two]: Here is the footnote.
.
<p>Foo.[^one two]</p>
<p>[^one two]: Here is the footnote.</p>
````````````````````````````````

```````````````````````````````` example -e footnotes -e table
A footnote in a paragraph[^1]

| Column1   | Column2 |
| --------- | ------- |
| foot [^1] | note    |

[^1]: a footnote
.
<p>A footnote in a paragraph<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<table>
<thead>
<tr>
<th>Column1</th>
<th>Column2</th>
</tr>
</thead>
<tbody>
<tr>
<td>foot <sup class="footnote-ref"><a href="#fn-1" id="fnref-1-2" data-footnote-ref>1</a></sup></td>
<td>note</td>
</tr>
</tbody>
</table>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>a footnote <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a> <a href="#fnref-1-2" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1-2" aria-label="Back to reference 1-2">↩<sup class="footnote-ref">2</sup></a></p>
</li>
</ol>
</section>
````````````````````````````````

```````````````````````````````` example -e footnotes -e superscript
Here is a footnote reference.[^1]

Here is a longer footnote reference.[^ref]

e = mc^2^.

[^1]: Here is the footnote.
[^ref]: Here is another footnote.
.
<p>Here is a footnote reference.<sup class="footnote-ref"><a href="#fn-1" id="fnref-1" data-footnote-ref>1</a></sup></p>
<p>Here is a longer footnote reference.<sup class="footnote-ref"><a href="#fn-ref" id="fnref-ref" data-footnote-ref>2</a></sup></p>
<p>e = mc<sup>2</sup>.</p>
<section class="footnotes" data-footnotes>
<ol>
<li id="fn-1">
<p>Here is the footnote. <a href="#fnref-1" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="1" aria-label="Back to reference 1">↩</a></p>
</li>
<li id="fn-ref">
<p>Here is another footnote. <a href="#fnref-ref" class="footnote-backref" data-footnote-backref data-footnote-backref-idx="2" aria-label="Back to reference 2">↩</a></p>
</li>
</ol>
</section>
````````````````````````````````
