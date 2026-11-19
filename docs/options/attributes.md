---
title: Render attributes
---

`--attributes` (`render.attributes`) includes parsed [attributes](../extensions/attributes.html) in HTML output.  It has no effect without `--unsafe` too, since any attribute can be turned against the page it's on, and none unless at least one attributes extension is enabled.

For the full rules, see [Render attributes](../extensions/attributes.html#render-attributes).

Without the option, Comrak parses attribute blocks but writes no attributes:

```````````````````````````````` example -e link-attributes --unsafe
![Comrak](comrak.png){width=100 .logo}
.
<p><img src="comrak.png" alt="Comrak" /></p>
````````````````````````````````

With the option, it writes them:

```````````````````````````````` example -e link-attributes --attributes --unsafe
![Comrak](comrak.png){width=100 .logo}
.
<p><img src="comrak.png" alt="Comrak" class="logo" width="100" /></p>
````````````````````````````````
