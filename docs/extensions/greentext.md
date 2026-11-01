---
title: Greentext
---

```````````````````````````````` example -e greentext
>implying implications
.
<p>&gt;implying implications</p>
````````````````````````````````

```````````````````````````````` example -e greentext --hardbreaks
>implying
>>implying
.
<p>&gt;implying<br />
&gt;&gt;implying</p>
````````````````````````````````

```````````````````````````````` example -e greentext
>
.
<p>&gt;</p>
````````````````````````````````

```````````````````````````````` example -e greentext
> 1
>
> 2
.
<blockquote>
<p>1</p>
<p>2</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e greentext
> one
> > two
> three
.
<blockquote>
<p>one</p>
<blockquote>
<p>two</p>
</blockquote>
<p>three</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e greentext -e alerts
> [!Note]
> This is the first line.
>
> Second line.
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>This is the first line.</p>
<p>Second line.</p>
</div>
````````````````````````````````
