---
title: GitHub-style pre lang
---

```````````````````````````````` example --github-pre-lang
``` rust
fn hello();
```
.
<pre class="syntax-highlighting" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

```````````````````````````````` example
``` rust
fn hello();
```
.
<pre class="syntax-highlighting"><code class="language-rust"><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````

```````````````````````````````` example --github-pre-lang --full-info-string
``` rust extra info
fn hello();
```
.
<pre class="syntax-highlighting" data-meta="extra info" lang="rust"><code><span class="source rust"><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">hello</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="punctuation terminator rust">;</span>
</span></code></pre>
````````````````````````````````
