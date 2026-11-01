---
title: Alert style
---

```````````````````````````````` example -e alerts --alert-style specific
> [!note]
> Pay attention
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Pay attention</p>
</div>
````````````````````````````````

```````````````````````````````` example -e alerts --alert-style semantic
> [!note]
> Pay attention
.
<aside class="admonition note">
<p class="admonition-title">Note</p>
<p>Pay attention</p>
</aside>
````````````````````````````````

```````````````````````````````` example -e alerts --alert-style semantic
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.
.
<aside class="admonition note">
<p class="admonition-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</aside>
<aside class="admonition tip">
<p class="admonition-title">Tip</p>
<p>Optional information to help a user be more successful.</p>
</aside>
<aside class="admonition important">
<p class="admonition-title">Important</p>
<p>Crucial information necessary for users to succeed.</p>
</aside>
<aside class="admonition warning">
<p class="admonition-title">Warning</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</aside>
<aside class="admonition caution">
<p class="admonition-title">Caution</p>
<p>Negative potential consequences of an action.</p>
</aside>
````````````````````````````````

```````````````````````````````` example -e alerts --alert-style semantic
> [!NOTE] Pay attention
> Highlights information that users should take into account, even when skimming.
.
<aside class="admonition note">
<p class="admonition-title">Pay attention</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</aside>
````````````````````````````````
