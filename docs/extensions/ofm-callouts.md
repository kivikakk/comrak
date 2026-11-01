---
title: OFM callouts
---

```````````````````````````````` example -e ofm-callouts
> [!note]
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> [!warning] **Bold** and *italic*
> Body
.
<div class="callout" data-callout="warning" data-callout-fold="none">
<div class="callout-title">
<strong>Bold</strong> and <em>italic</em>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> [!My Type]
> Body
.
<div class="callout" data-callout="my-type" data-callout-fold="none">
<div class="callout-title">My type</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> [!note]+
> Body
.
<div class="callout" data-callout="note" data-callout-fold="expanded">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> [!tip|right]- **Read**
> Body
.
<div class="callout" data-callout="tip" data-callout-metadata="right" data-callout-fold="collapsed">
<div class="callout-title">
<strong>Read</strong>
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> [!note] Title
Lazy body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
Title
</div>
<div class="callout-content">
<p>Lazy body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
> > [!note]
> > Body
.
<blockquote>
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">Note</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts
>  [!note]
> Body
.
<blockquote>
<p>[!note]
Body</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e alerts -e ofm-callouts
> [!important]
> Body
.
<div class="callout" data-callout="important" data-callout-fold="none">
<div class="callout-title">Important</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````

```````````````````````````````` example -e ofm-callouts -e ofm-comments
> [!note] %%hidden%%
> Body
.
<div class="callout" data-callout="note" data-callout-fold="none">
<div class="callout-title">
</div>
<div class="callout-content">
<p>Body</p>
</div>
</div>
````````````````````````````````
