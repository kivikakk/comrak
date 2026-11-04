---
title: Fenced code attributes
---

```````````````````````````````` example -e fenced-code-attributes
```rust {#example}
const fn dogs() -> ! { yay }
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="storage type rust">const</span> <span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">dogs</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> <span class="meta function return-type rust"><span class="punctuation separator rust">-&gt;</span> </span></span><span class="keyword operator rust">!</span> <span class="meta block rust"><span class="punctuation section block begin rust">{</span> yay </span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span>
</span></code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes
```german {#beispel}
Ähm... egal.
```
.
<pre class="syntax-highlighting"><code class="language-german"><span class="text plain">Ähm... egal.
</span></code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes
```{.zig #truism}
fn cats() noreturn { yay_too }
```
.
<pre class="syntax-highlighting"><code><span class="text plain">fn cats() noreturn { yay_too }
</span></code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes --attributes --unsafe
```{.zig #truism}
fn cats() noreturn { yay_too }
```
.
<pre class="syntax-highlighting zig" id="truism"><code><span class="text plain">fn cats() noreturn { yay_too }
</span></code></pre>
````````````````````````````````

```````````````````````````````` example -e fenced-code-attributes --github-pre-lang --attributes --unsafe
```german {#beispel lang=fr}
Ähm... egal.
```
.
<pre class="syntax-highlighting" id="beispel" lang="german"><code><span class="text plain">Ähm... egal.
</span></code></pre>
````````````````````````````````
