---
title: Autolinks
---

```````````````````````````````` example -e autolink
Hello www.github.com.
.
<p>Hello <a href="http://www.github.com">www.github.com</a>.</p>
````````````````````````````````

```````````````````````````````` example -e autolink
john@smith.com
.
<p><a href="mailto:john@smith.com">john@smith.com</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink
https://google.com/search
rdar://localhost.com/blah
.
<p><a href="https://google.com/search">https://google.com/search</a>
rdar://localhost.com/blah</p>
````````````````````````````````

```````````````````````````````` example -e autolink
see http://localhost:3000/admin now
.
<p>see <a href="http://localhost:3000/admin">http://localhost:3000/admin</a> now</p>
````````````````````````````````

```````````````````````````````` example -e autolink
http://www.pokemon.com/Pikachu_(Electric)

http://www.pokemon.com/Pikachu_((Electric)

http://www.pokemon.com/Pikachu_(Electric))

http://www.pokemon.com/Pikachu_((Electric))
.
<p><a href="http://www.pokemon.com/Pikachu_(Electric)">http://www.pokemon.com/Pikachu_(Electric)</a></p>
<p><a href="http://www.pokemon.com/Pikachu_((Electric)">http://www.pokemon.com/Pikachu_((Electric)</a></p>
<p><a href="http://www.pokemon.com/Pikachu_(Electric)">http://www.pokemon.com/Pikachu_(Electric)</a>)</p>
<p><a href="http://www.pokemon.com/Pikachu_((Electric))">http://www.pokemon.com/Pikachu_((Electric))</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink
http://example.com/[abc]]...
.
<p><a href="http://example.com/%5Babc%5D%5D">http://example.com/[abc]]</a>...</p>
````````````````````````````````

```````````````````````````````` example -e autolink
[Contact user@example.com for help](http://support.example.com)

[Visit https://www.example.com](http://example.com)

[Check out www.example.com](http://example.com)
.
<p><a href="http://support.example.com">Contact user@example.com for help</a></p>
<p><a href="http://example.com">Visit https://www.example.com</a></p>
<p><a href="http://example.com">Check out www.example.com</a></p>
````````````````````````````````

```````````````````````````````` example -e autolink
[https://foo.com]
.
<p>[https://foo.com]</p>
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

```````````````````````````````` example -e autolink --relaxed-autolinks
http://example.com/[abc]]...
.
<p><a href="http://example.com/%5Babc%5D">http://example.com/[abc]</a>]...</p>
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
