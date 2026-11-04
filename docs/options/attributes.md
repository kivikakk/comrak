---
title: Attributes
---

```````````````````````````````` example -e link-attributes --attributes --unsafe
![Comrak](comrak.png){width=100 .logo}
.
<p><img src="comrak.png" alt="Comrak" class="logo" width="100" /></p>
````````````````````````````````

```````````````````````````````` example -e link-attributes --attributes
![Comrak](comrak.png){width=100 .logo}
.
<p><img src="comrak.png" alt="Comrak" /></p>
````````````````````````````````

```````````````````````````````` example -e link-attributes --unsafe
![Comrak](comrak.png){width=100 .logo}
.
<p><img src="comrak.png" alt="Comrak" /></p>
````````````````````````````````

```````````````````````````````` example -e link-attributes --attributes --unsafe
[Comrak](https://comrak.ee "Home"){#home href="https://example.com"}
.
<p><a href="https://comrak.ee" title="Home" id="home" href="https://example.com">Comrak</a></p>
````````````````````````````````
