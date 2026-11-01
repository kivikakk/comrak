---
title: Code span-based math
version: 0.1
date: '2024-03-22'
license: '[CC-BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/)'
based_on: https://docs.gitlab.com/ee/user/markdown.html#math
---

# TeX Math

Inline math goes between `` $` `` characters, and display math in fenced code
blocks with the info string `math`:

```````````````````````````````` example -e math-code
Let $`x`$ and $`y`$ be integers such that

```math
x=y + 2
```
.
<p>Let <code data-math-style="inline">x</code> and <code data-math-style="inline">y</code> be integers such that</p>
<pre><code class="language-math" data-math-style="display">x=y + 2
</code></pre>
````````````````````````````````


In inline math, it behaves just like inline code:

```````````````````````````````` example -e math-code
This is math:$`2000`$.
.
<p>This is math:<code data-math-style="inline">2000</code>.</p>
````````````````````````````````


Note that math can contain embedded math:

```````````````````````````````` example -e math-code
This is display math:

```math
\text{Hello $x^2$}
```
And this is inline math:
$`\text{Hello $x$ there!}`$
.
<p>This is display math:</p>
<pre><code class="language-math" data-math-style="display">\text{Hello $x^2$}
</code></pre>
<p>And this is inline math:
<code data-math-style="inline">\text{Hello $x$ there!}</code></p>
````````````````````````````````


Dollar signs are not required to be escaped with backslashes. They may be
required to render the math properly, but it's not required for parsing:

```````````````````````````````` example -e math-code
$`\text{$}`$
.
<p><code data-math-style="inline">\text{$}</code></p>
````````````````````````````````

Everything inside the math construct is treated as math, and not given its
normal CommonMark meaning:

```````````````````````````````` example -e math-code
$`b<a>c`$
.
<p><code data-math-style="inline">b&lt;a&gt;c</code></p>
````````````````````````````````

Double dollar signs are not supported:

```````````````````````````````` example -e math-code
$$`1 + 2`$$
.
<p>$$<code>1 + 2</code>$$</p>
````````````````````````````````
