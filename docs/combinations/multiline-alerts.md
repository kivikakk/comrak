---
title: Multi-line alerts
original_title: GitLab Flavored Markdown Spec
version: 0.1
date: '2023-12-18'
license: '[CC-BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/)'
---

## Multi-line Alerts

Simple container

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
*content*
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p><em>content</em></p>
</div>
````````````````````````````````

Other kinds of alerts:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!TIP]
Optional information to help a user be more successful.
>>>
.
<div class="markdown-alert markdown-alert-tip">
<p class="markdown-alert-title">Tip</p>
<p>Optional information to help a user be more successful.</p>
</div>
````````````````````````````````

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!IMPORTANT]
Crucial information necessary for users to succeed.
>>>
.
<div class="markdown-alert markdown-alert-important">
<p class="markdown-alert-title">Important</p>
<p>Crucial information necessary for users to succeed.</p>
</div>
````````````````````````````````

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!WARNING]
Critical content demanding immediate user attention due to potential risks.
>>>
.
<div class="markdown-alert markdown-alert-warning">
<p class="markdown-alert-title">Warning</p>
<p>Critical content demanding immediate user attention due to potential risks.</p>
</div>
````````````````````````````````

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!CAUTION]
Negative potential consequences of an action.
>>>
.
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<p>Negative potential consequences of an action.</p>
</div>
````````````````````````````````

A title can be specified to override the default title:

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE] Pay attention
Highlights information that users should take into account, even when skimming.
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Pay attention</p>
<p>Highlights information that users should take into account, even when skimming.</p>
</div>
````````````````````````````````

Can contain block elements

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
### heading

-----------
>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<h3>heading</h3>
<hr />
</div>
````````````````````````````````


Ending marker can be longer

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>>>>> [!NOTE]
  hello world
>>>>>>>>>>>
normal
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>hello world</p>
</div>
<p>normal</p>
````````````````````````````````


Nested alerts

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>>>> [!NOTE]
>>>> [!CAUTION]
foo
>>>>
>>>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<p>foo</p>
</div>
</div>
````````````````````````````````

Incorrectly nested alerts

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>>> [!NOTE]
this block is closed with 5 markers below

>>>>>

auto-closed blocks
>>>>>
>>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>this block is closed with 5 markers below</p>
</div>
<p>auto-closed blocks</p>
<blockquote>
<blockquote>
</blockquote>
</blockquote>
````````````````````````````````


Marker can be indented up to 3 spaces

```````````````````````````````` example -e alerts -e multiline-block-quotes
   >>>> [!NOTE]
   first-level blockquote
    >>> [!CAUTION]
    second-level blockquote
    >>>
   >>>>
   regular paragraph
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>first-level blockquote</p>
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<p>second-level blockquote</p>
</div>
</div>
<p>regular paragraph</p>
````````````````````````````````


Fours spaces makes it a code block

```````````````````````````````` example -e alerts -e multiline-block-quotes
    >>>
    content
    >>>
.
<pre class="syntax-highlighting"><code><span class="text plain">&gt;&gt;&gt;
content
&gt;&gt;&gt;
</span></code></pre>
````````````````````````````````


Detection of embedded 4 spaces code block starts in the
column the alert starts, not from the beginning of
the line.

```````````````````````````````` example -e alerts -e multiline-block-quotes
  >>> [!NOTE]
      code block
  >>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<pre class="syntax-highlighting"><code><span class="text plain">code block
</span></code></pre>
</div>
````````````````````````````````

```````````````````````````````` example -e alerts -e multiline-block-quotes
   >>>> [!NOTE]
   content
    >>> [!CAUTION]
        code block
    >>>
   >>>>
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>content</p>
<div class="markdown-alert markdown-alert-caution">
<p class="markdown-alert-title">Caution</p>
<pre class="syntax-highlighting"><code><span class="text plain">code block
</span></code></pre>
</div>
</div>
````````````````````````````````

Closing marker can't have text on the same line

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
foo
>>> arg=123
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>foo</p>
<blockquote>
<blockquote>
<blockquote>
<p>arg=123</p>
</blockquote>
</blockquote>
</blockquote>
</div>
````````````````````````````````


Alerts self-close at the end of the document

```````````````````````````````` example -e alerts -e multiline-block-quotes
>>> [!NOTE]
foo
.
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>foo</p>
</div>
````````````````````````````````


They should terminate paragraphs

```````````````````````````````` example -e alerts -e multiline-block-quotes
blah blah
>>> [!NOTE]
content
>>>
.
<p>blah blah</p>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>content</p>
</div>
````````````````````````````````


They can be nested in lists

```````````````````````````````` example -e alerts -e multiline-block-quotes
 -  >>> [!NOTE]
    - foo
    >>>
.
<ul>
<li>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<ul>
<li>foo</li>
</ul>
</div>
</li>
</ul>
````````````````````````````````


Or in blockquotes

```````````````````````````````` example -e alerts -e multiline-block-quotes
> >>> [!NOTE]
> foo
>> bar
> baz
> >>>
.
<blockquote>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>foo</p>
<blockquote>
<p>bar
baz</p>
</blockquote>
</div>
</blockquote>
````````````````````````````````


List indentation

```````````````````````````````` example -e alerts -e multiline-block-quotes
 -  >>> [!NOTE]
    foo
    bar
    >>>

 -  >>> [!NOTE]
    foo
    bar
    >>>
.
<ul>
<li>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>foo
bar</p>
</div>
</li>
<li>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>foo
bar</p>
</div>
</li>
</ul>
````````````````````````````````


Ignored inside code blocks:

```````````````````````````````` example -e alerts -e multiline-block-quotes
```txt
# Code
>>> [!NOTE]
# Code
>>>
# Code
```
.
<pre class="syntax-highlighting"><code class="language-txt"><span class="text plain"># Code
&gt;&gt;&gt; [!NOTE]
# Code
&gt;&gt;&gt;
# Code
</span></code></pre>
````````````````````````````````


Does not require a leading or trailing blank line

```````````````````````````````` example -e alerts -e multiline-block-quotes
Some text
>>> [!NOTE]
A quote
>>>
Some other text
.
<p>Some text</p>
<div class="markdown-alert markdown-alert-note">
<p class="markdown-alert-title">Note</p>
<p>A quote</p>
</div>
<p>Some other text</p>
````````````````````````````````
