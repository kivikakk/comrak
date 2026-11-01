---
title: Multi-line block quotes
original_title: GitLab Flavored Markdown Spec
version: 0.1
date: '2023-12-18'
license: '[CC-BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/)'
---

## Multi-line Blockquotes

Simple container

```````````````````````````````` example -e multiline-block-quotes
>>>
*content*
>>>
.
<blockquote>
<p><em>content</em></p>
</blockquote>
````````````````````````````````


Can contain block elements

```````````````````````````````` example -e multiline-block-quotes
>>>
### heading

-----------
>>>
.
<blockquote>
<h3>heading</h3>
<hr />
</blockquote>
````````````````````````````````


Ending marker can be longer

```````````````````````````````` example -e multiline-block-quotes
>>>>>>
  hello world
>>>>>>>>>>>
normal
.
<blockquote>
<p>hello world</p>
</blockquote>
<p>normal</p>
````````````````````````````````


Nested blockquotes

```````````````````````````````` example -e multiline-block-quotes
>>>>>
>>>>
foo
>>>>
>>>>>
.
<blockquote>
<blockquote>
<p>foo</p>
</blockquote>
</blockquote>
````````````````````````````````

Incorrectly nested blockquotes

```````````````````````````````` example -e multiline-block-quotes
>>>>
this block is closed with 5 markers below

>>>>>

auto-closed blocks
>>>>>
>>>>
.
<blockquote>
<p>this block is closed with 5 markers below</p>
</blockquote>
<p>auto-closed blocks</p>
<blockquote>
<blockquote>
</blockquote>
</blockquote>
````````````````````````````````


Marker can be indented up to 3 spaces

```````````````````````````````` example -e multiline-block-quotes
   >>>>
   first-level blockquote
    >>>
    second-level blockquote
    >>>
   >>>>
   regular paragraph
.
<blockquote>
<p>first-level blockquote</p>
<blockquote>
<p>second-level blockquote</p>
</blockquote>
</blockquote>
<p>regular paragraph</p>
````````````````````````````````


Fours spaces makes it a code block

```````````````````````````````` example -e multiline-block-quotes
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
column the blockquote starts, not from the beginning of
the line.

```````````````````````````````` example -e multiline-block-quotes
  >>>
      code block
  >>>
.
<blockquote>
<pre class="syntax-highlighting"><code><span class="text plain">code block
</span></code></pre>
</blockquote>
````````````````````````````````

```````````````````````````````` example -e multiline-block-quotes
   >>>>
   content
    >>>
        code block
    >>>
   >>>>
.
<blockquote>
<p>content</p>
<blockquote>
<pre class="syntax-highlighting"><code><span class="text plain">code block
</span></code></pre>
</blockquote>
</blockquote>
````````````````````````````````

Closing marker can't have text on the same line

```````````````````````````````` example -e multiline-block-quotes
>>>
foo
>>> arg=123
.
<blockquote>
<p>foo</p>
<blockquote>
<blockquote>
<blockquote>
<p>arg=123</p>
</blockquote>
</blockquote>
</blockquote>
</blockquote>
````````````````````````````````


Blockquotes self-close at the end of the document

```````````````````````````````` example -e multiline-block-quotes
>>>
foo
.
<blockquote>
<p>foo</p>
</blockquote>
````````````````````````````````


They should terminate paragraphs

```````````````````````````````` example -e multiline-block-quotes
blah blah
>>>
content
>>>
.
<p>blah blah</p>
<blockquote>
<p>content</p>
</blockquote>
````````````````````````````````


They can be nested in lists

```````````````````````````````` example -e multiline-block-quotes
 -  >>>
    - foo
    >>>
.
<ul>
<li>
<blockquote>
<ul>
<li>foo</li>
</ul>
</blockquote>
</li>
</ul>
````````````````````````````````


Or in blockquotes

```````````````````````````````` example -e multiline-block-quotes
> >>>
> foo
>> bar
> baz
> >>>
.
<blockquote>
<blockquote>
<p>foo</p>
<blockquote>
<p>bar
baz</p>
</blockquote>
</blockquote>
</blockquote>
````````````````````````````````


List indentation

```````````````````````````````` example -e multiline-block-quotes
 -  >>>
    foo
    bar
    >>>

 -  >>>
    foo
    bar
    >>>
.
<ul>
<li>
<blockquote>
<p>foo
bar</p>
</blockquote>
</li>
<li>
<blockquote>
<p>foo
bar</p>
</blockquote>
</li>
</ul>
````````````````````````````````


Ignored inside code blocks:

```````````````````````````````` example -e multiline-block-quotes
```txt
# Code
>>>
# Code
>>>
# Code
```
.
<pre class="syntax-highlighting"><code class="language-txt"><span class="text plain"># Code
&gt;&gt;&gt;
# Code
&gt;&gt;&gt;
# Code
</span></code></pre>
````````````````````````````````


Does not require a leading or trailing blank line

```````````````````````````````` example -e multiline-block-quotes
Some text
>>>
A quote
>>>
Some other text
.
<p>Some text</p>
<blockquote>
<p>A quote</p>
</blockquote>
<p>Some other text</p>
````````````````````````````````
