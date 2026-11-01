---
title: Escape raw HTML
---

```````````````````````````````` example --escape
<i>italic text</i>
.
<p>&lt;i&gt;italic text&lt;/i&gt;</p>
````````````````````````````````

```````````````````````````````` example
<i>italic text</i>
.
<p><!-- raw HTML omitted -->italic text<!-- raw HTML omitted --></p>
````````````````````````````````

```````````````````````````````` example --escape
<script>
alert('xyz');
</script>

Possibly <marquee>annoying</marquee>.

[Dangerous](javascript:alert(document.cookie)).

[Safe](http://commonmark.org).
.
&lt;script&gt;
alert('xyz');
&lt;/script&gt;
<p>Possibly &lt;marquee&gt;annoying&lt;/marquee&gt;.</p>
<p><a href="">Dangerous</a>.</p>
<p><a href="http://commonmark.org">Safe</a>.</p>
````````````````````````````````

```````````````````````````````` example --escape --unsafe
<script>
alert('xyz');
</script>

Possibly <marquee>annoying</marquee>.

[Dangerous](javascript:alert(document.cookie)).

[Safe](http://commonmark.org).
.
&lt;script&gt;
alert('xyz');
&lt;/script&gt;
<p>Possibly &lt;marquee&gt;annoying&lt;/marquee&gt;.</p>
<p><a href="javascript:alert(document.cookie)">Dangerous</a>.</p>
<p><a href="http://commonmark.org">Safe</a>.</p>
````````````````````````````````
