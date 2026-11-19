---
title: Alert style
---

`--alert-style` (`render.alert_style`) chooses the HTML structure of [alerts](../extensions/alerts.html).  With `specific`, the default, an alert is a `<div>` with `markdown-alert` classes, as GitHub renders it.  With `semantic`, an alert is an `<aside>` with `admonition` classes.  The option has no effect unless alerts are enabled.

For the full rules, see [Alert style](../extensions/alerts.html#alert-style).

By default, alerts are `<div>` elements:

```````````````````````````````` example -e alerts
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!WARNING] Careful now
> Critical content demanding immediate user attention due to potential risks.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Careful now</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</div>
````````````````````````````````

With `--alert-style semantic`, they're `<aside>` elements:

```````````````````````````````` example -e alerts --alert-style semantic
> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!WARNING] Careful now
> Critical content demanding immediate user attention due to potential risks.
.
<aside class="admonition note">
<p class="admonition-title">Note</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</aside>
<aside class="admonition warning">
<p class="admonition-title">Careful now</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</aside>
````````````````````````````````
