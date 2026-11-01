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
