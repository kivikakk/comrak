---
title: Header attributes
---

```````````````````````````````` example -e header-attributes
# Hi! {#greeting}
.
<h1>Hi!</h1>
````````````````````````````````

```````````````````````````````` example -e header-attributes
## Catgirl {author="City Girl"}
.
<h2>Catgirl</h2>
````````````````````````````````

```````````````````````````````` example -e header-attributes
## Yeww {.ok yeww="\"true\""} ##
.
<h2>Yeww</h2>
````````````````````````````````

```````````````````````````````` example -e header-attributes
## Yeww ## {x=y x=y #a #b}
.
<h2>Yeww ##</h2>
````````````````````````````````

```````````````````````````````` example -e header-attributes
Yeww {x=y x=y #a #b}
====================
.
<h1>Yeww</h1>
````````````````````````````````

```````````````````````````````` example -e header-attributes --attributes --unsafe
# Hi! {#greeting}
.
<h1 id="greeting">Hi!</h1>
````````````````````````````````

```````````````````````````````` example -e header-attributes --attributes --unsafe
## Yeww {.ok yeww="\"true\""} ##
.
<h2 class="ok" yeww="&quot;true&quot;">Yeww</h2>
````````````````````````````````

```````````````````````````````` example -e header-attributes --attributes --unsafe
## Yeww ## {x=y x=y #a #b}
.
<h2 id="b" x="y" x="y">Yeww ##</h2>
````````````````````````````````

```````````````````````````````` example -e header-attributes --header-id-prefix user-content- --attributes --unsafe
# Hi! {#greeting .wave}
# Hi!
.
<h1 id="user-content-greeting" class="wave">Hi!<a href="#greeting" aria-label="Link to heading 'Hi!'" data-heading-content="Hi!" class="anchor"></a></h1>
<h1 id="user-content-hi">Hi!<a href="#hi" aria-label="Link to heading 'Hi!'" data-heading-content="Hi!" class="anchor"></a></h1>
````````````````````````````````
