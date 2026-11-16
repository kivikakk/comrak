---
title: Relaxed autolinks
---

`--relaxed-autolinks` (`parse.relaxed_autolinks`) makes [autolinks](../extensions/autolink.html) more permissive.  Most importantly:

* autolinks are recognised inside brackets;
* a URL autolink may use any scheme; and,
* a www autolink's domain needn't contain a period.

The option has no effect unless autolinks are enabled.

For the full rules, see [Relaxed autolinks](../extensions/autolink.html#relaxed-autolinks).

Without the option, none of these are autolinks:

```````````````````````````````` example -e autolink
[https://foo.com]

irc://irc.freenode.net/git

www.example
.
<p>[https://foo.com]</p>
<p>irc://irc.freenode.net/git</p>
<p>www.example</p>
````````````````````````````````

With the option, they all are:

```````````````````````````````` example -e autolink --relaxed-autolinks
[https://foo.com]

irc://irc.freenode.net/git

www.example
.
<p>[<a href="https://foo.com">https://foo.com</a>]</p>
<p><a href="irc://irc.freenode.net/git">irc://irc.freenode.net/git</a></p>
<p><a href="http://www.example">www.example</a></p>
````````````````````````````````
