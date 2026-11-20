---
title: Header ID prefix in href
---

`--header-id-prefix-in-href` (`extension.header_id_prefix_in_href`) adds the [header ID prefix](header-id-prefix.html) to the `href` of the link inside each heading, so that the link points at the heading's `id`.  The option has no effect unless `--header-id-prefix` is set.

For the full rules, see [Header ID prefix in href](header-id-prefix.html#header-id-prefix-in-href).

Without the option, the `href` doesn't include the prefix:

```````````````````````````````` example --header-id-prefix user-content-
# README
.
<h1 id="user-content-readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

With the option, it does:

```````````````````````````````` example --header-id-prefix user-content- --header-id-prefix-in-href
# README
.
<h1 id="user-content-readme">README<a href="#user-content-readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````
