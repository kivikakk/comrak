---
title: Front matter
---

```````````````````````````````` example
---
layout: post
title: Formatting Markdown with Comrak
---

# Shorter Title

etc.
.
<hr />
<h2>layout: post
title: Formatting Markdown with Comrak</h2>
<h1>Shorter Title</h1>
<p>etc.</p>
````````````````````````````````

```````````````````````````````` example --front-matter-delimiter ---
---
layout: post
title: Formatting Markdown with Comrak
---

# Shorter Title

etc.
.
<h1>Shorter Title</h1>
<p>etc.</p>
````````````````````````````````

```````````````````````````````` example --front-matter-delimiter +++
+++
title = "Formatting Markdown with Comrak"
+++

# Shorter Title
.
<h1>Shorter Title</h1>
````````````````````````````````

```````````````````````````````` example --front-matter-delimiter ---

---
layout: post
---
Text
.
<hr />
<h2>layout: post</h2>
<p>Text</p>
````````````````````````````````

```````````````````````````````` example --front-matter-delimiter ---
 ---
layout: post
---
Text
.
<hr />
<h2>layout: post</h2>
<p>Text</p>
````````````````````````````````
