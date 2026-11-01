---
title: Relaxed autolinks
---

```````````````````````````````` example -e autolink
[https://foo.com]

[[https://foo.com]]

{https://foo.com}
.
<p>[https://foo.com]</p>
<p>[[https://foo.com]]</p>
<p>{<a href="https://foo.com%7D">https://foo.com}</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink --relaxed-autolinks
[https://foo.com]

[[https://foo.com]]

{https://foo.com}
.
<p>[<a href="https://foo.com">https://foo.com</a>]</p>
<p>[[<a href="https://foo.com">https://foo.com</a>]]</p>
<p>{<a href="https://foo.com">https://foo.com</a>}</p>
````````````````````````````````

```````````````````````````````` example -e autolink
smb:///Volumes/shared/foo.pdf

irc://irc.freenode.net/git

rdar://localhost.com/blah
.
<p>smb:///Volumes/shared/foo.pdf</p>
<p>irc://irc.freenode.net/git</p>
<p>rdar://localhost.com/blah</p>
````````````````````````````````

```````````````````````````````` example -e autolink --relaxed-autolinks
smb:///Volumes/shared/foo.pdf

irc://irc.freenode.net/git

rdar://localhost.com/blah
.
<p><a href="smb:///Volumes/shared/foo.pdf">smb:///Volumes/shared/foo.pdf</a></p>
<p><a href="irc://irc.freenode.net/git">irc://irc.freenode.net/git</a></p>
<p><a href="rdar://localhost.com/blah">rdar://localhost.com/blah</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink
http://example.com/[abc]]...
.
<p><a href="http://example.com/%5Babc%5D%5D">http://example.com/[abc]]</a>...</p>
````````````````````````````````

```````````````````````````````` example -e autolink --relaxed-autolinks
http://example.com/[abc]]...
.
<p><a href="http://example.com/%5Babc%5D">http://example.com/[abc]</a>]...</p>
````````````````````````````````
