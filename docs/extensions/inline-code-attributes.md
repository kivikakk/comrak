---
title: Inline code attributes
---

```````````````````````````````` example -e inline-code-attributes
Uhh `totally`{yea=so} hm.
.
<p>Uhh <code>totally</code> hm.</p>
````````````````````````````````

```````````````````````````````` example -e inline-code-attributes
More inline spans should be `syntax-highlighted`{.common-lisp}.
.
<p>More inline spans should be <code>syntax-highlighted</code>.</p>
````````````````````````````````

```````````````````````````````` example -e inline-code-attributes
`T1 Faker`{role=mid
  kda="0/0/0"
  cs=120}
.
<p><code>T1 Faker</code></p>
````````````````````````````````

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
More inline spans should be `syntax-highlighted`{.common-lisp}.
.
<p>More inline spans should be <code class="common-lisp">syntax-highlighted</code>.</p>
````````````````````````````````

```````````````````````````````` example -e inline-code-attributes --attributes --unsafe
`T1 Faker`{role=mid
  kda="0/0/0"
  cs=120}
.
<p><code role="mid" kda="0/0/0" cs="120">T1 Faker</code></p>
````````````````````````````````
