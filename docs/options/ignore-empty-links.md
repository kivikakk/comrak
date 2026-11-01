---
title: Ignore empty links
---

```````````````````````````````` example
[]()
.
<p><a href=""></a></p>
````````````````````````````````

```````````````````````````````` example --ignore-empty-links
[]()
.
<p>[]()</p>
````````````````````````````````

```````````````````````````````` example
[](https://example.com/evil-link-for-seo-spam)

[    ](https://example.com/evil-link-for-seo-spam)
.
<p><a href="https://example.com/evil-link-for-seo-spam"></a></p>
<p><a href="https://example.com/evil-link-for-seo-spam">    </a></p>
````````````````````````````````

```````````````````````````````` example --ignore-empty-links
[](https://example.com/evil-link-for-seo-spam)

[    ](https://example.com/evil-link-for-seo-spam)
.
<p>[](https://example.com/evil-link-for-seo-spam)</p>
<p>[    ](https://example.com/evil-link-for-seo-spam)</p>
````````````````````````````````

```````````````````````````````` example --ignore-empty-links
![   ](https://example.com/evil-link-for-seo-spam)

[![](https://example.com/image.png)](https://example.com/)
.
<p><img src="https://example.com/evil-link-for-seo-spam" alt="   " /></p>
<p><a href="https://example.com/"><img src="https://example.com/image.png" alt="" /></a></p>
````````````````````````````````
