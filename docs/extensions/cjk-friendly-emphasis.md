---
title: CJK-friendly emphasis
---

```````````````````````````````` example -e cjk-friendly-emphasis
**この文は重要です。**但这句话并不重要。
.
<p><strong>この文は重要です。</strong>但这句话并不重要。</p>
````````````````````````````````

```````````````````````````````` example -e cjk-friendly-emphasis
これは**私のやりたかったこと。**だからするの。

**[製品ほげ](./product-foo)**と**[製品ふが](./product-bar)**をお試しください

先頭の**`コード`も注意。**

**末尾の`コード`**も注意。

税込**¥10,000**で入手できます。

太郎は**「こんにちわ」**といった
.
<p>これは<strong>私のやりたかったこと。</strong>だからするの。</p>
<p><strong><a href="./product-foo">製品ほげ</a></strong>と<strong><a href="./product-bar">製品ふが</a></strong>をお試しください</p>
<p>先頭の<strong><code>コード</code>も注意。</strong></p>
<p><strong>末尾の<code>コード</code></strong>も注意。</p>
<p>税込<strong>¥10,000</strong>で入手できます。</p>
<p>太郎は<strong>「こんにちわ」</strong>といった</p>
````````````````````````````````

```````````````````````````````` example -e cjk-friendly-emphasis
**이 [링크](https://example.kr/)**만을 강조하고 싶다.

**스크립트(script)**라고

패키지를 발행하려면 **`npm publish`**를 실행하십시오.

**안녕(hello)**하세요.
.
<p><strong>이 <a href="https://example.kr/">링크</a></strong>만을 강조하고 싶다.</p>
<p><strong>스크립트(script)</strong>라고</p>
<p>패키지를 발행하려면 <strong><code>npm publish</code></strong>를 실행하십시오.</p>
<p><strong>안녕(hello)</strong>하세요.</p>
````````````````````````````````

```````````````````````````````` example -e cjk-friendly-emphasis
__注意__：注意事項

注意：__注意事項__
.
<p><strong>注意</strong>：注意事項</p>
<p>注意：<strong>注意事項</strong></p>
````````````````````````````````

```````````````````````````````` example -e cjk-friendly-emphasis -e strikethrough
a~~a()~~あ

あ~~()a~~a

a~~「a~~」

「~~a」~~a

~~a~~：~~a~~
.
<p>a<del>a()</del>あ</p>
<p>あ<del>()a</del>a</p>
<p>a<del>「a</del>」</p>
<p>「<del>a」</del>a</p>
<p><del>a</del>：<del>a</del></p>
````````````````````````````````
