---
title: Unsafe HTML
---

```````````````````````````````` example --unsafe
<script>
alert('xyz');
</script>

Possibly <marquee>annoying</marquee>.

[Dangerous](javascript:alert(document.cookie)).

[Safe](http://commonmark.org).
.
<script>
alert('xyz');
</script>
<p>Possibly <marquee>annoying</marquee>.</p>
<p><a href="javascript:alert(document.cookie)">Dangerous</a>.</p>
<p><a href="http://commonmark.org">Safe</a>.</p>
````````````````````````````````

```````````````````````````````` example
<script>
alert('xyz');
</script>

Possibly <marquee>annoying</marquee>.

[Dangerous](javascript:alert(document.cookie)).

[Safe](http://commonmark.org).
.
<!-- raw HTML omitted -->
<p>Possibly <!-- raw HTML omitted -->annoying<!-- raw HTML omitted -->.</p>
<p><a href="">Dangerous</a>.</p>
<p><a href="http://commonmark.org">Safe</a>.</p>
````````````````````````````````

```````````````````````````````` example --unsafe
![png](data:image/png;base64,AAAA) ![html](data:text/html;base64,AAAA)
.
<p><img src="data:image/png;base64,AAAA" alt="png" /> <img src="data:text/html;base64,AAAA" alt="html" /></p>
````````````````````````````````

```````````````````````````````` example
![png](data:image/png;base64,AAAA) ![html](data:text/html;base64,AAAA)
.
<p><img src="data:image/png;base64,AAAA" alt="png" /> <img src="" alt="html" /></p>
````````````````````````````````
