---
title: Fenced code attributes
---

```````````````````````````````` example -e fenced-code-attributes
```rust {#example}
const fn dogs() -> ! { yay }
```
.
<pre><code class="language-rust">const fn dogs() -&gt; ! { yay }
</code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes
```german {#beispel}
Ähm... egal.
```
.
<pre><code class="language-german">Ähm... egal.
</code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes
```{.zig #truism}
fn cats() noreturn { yay_too }
```
.
<pre><code>fn cats() noreturn { yay_too }
</code></pre>
````````````````````````````````
