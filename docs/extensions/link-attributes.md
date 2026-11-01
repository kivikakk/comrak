---
title: Link attributes
---

```````````````````````````````` example -e link-attributes
Incredible [Jhin](game){lies=true}.
.
<p>Incredible <a href="game">Jhin</a>.</p>
````````````````````````````````

```````````````````````````````` example -e link-attributes
Incredible ![Jhin](game){lies=true}.
.
<p>Incredible <img src="game" alt="Jhin" />.</p>
````````````````````````````````

```````````````````````````````` example -e link-attributes
For instance:

![A photo of an open-hearth furnace](zaporizhstal.jfif){data-date=2012-04-03}
.
<p>For instance:</p>
<p><img src="zaporizhstal.jfif" alt="A photo of an open-hearth furnace" /></p>
````````````````````````````````
