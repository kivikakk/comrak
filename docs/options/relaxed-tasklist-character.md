---
title: Relaxed tasklist characters
---

```````````````````````````````` example -e tasklist
* [ ] Red
* [x] Green
* [ ] `Blue`
* [!] Papayawhip
.
<ul>
<li><input type="checkbox" disabled="" /> Red</li>
<li><input type="checkbox" checked="" disabled="" /> Green</li>
<li><input type="checkbox" disabled="" /> <code>Blue</code></li>
<li>[!] Papayawhip</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
* [ ] Red
* [x] Green
* [ ] `Blue`
* [!] Papayawhip
.
<ul>
<li><input type="checkbox" disabled="" /> Red</li>
<li><input type="checkbox" checked="" disabled="" /> Green</li>
<li><input type="checkbox" disabled="" /> <code>Blue</code></li>
<li><input type="checkbox" checked="" disabled="" /> Papayawhip</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e tasklist --relaxed-tasklist-character
- [あ] xy
  - [い] zw
.
<ul>
<li><input type="checkbox" checked="" disabled="" /> xy
<ul>
<li><input type="checkbox" checked="" disabled="" /> zw</li>
</ul>
</li>
</ul>
````````````````````````````````
