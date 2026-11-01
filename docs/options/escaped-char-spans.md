---
title: Escaped character spans
---

```````````````````````````````` example --escaped-char-spans
Notify user \@example
.
<p>Notify user <span data-escaped-char>@</span>example</p>
````````````````````````````````

```````````````````````````````` example
Notify user \@example
.
<p>Notify user @example</p>
````````````````````````````````

```````````````````````````````` example --escaped-char-spans
Test \`hello world\` here.
.
<p>Test <span data-escaped-char>`</span>hello world<span data-escaped-char>`</span> here.</p>
````````````````````````````````

```````````````````````````````` example --escaped-char-spans --sourcepos
Test \`hello world\` here.
.
<p data-sourcepos="1:1-1:26">Test <span data-escaped-char data-sourcepos="1:6-1:7">`</span>hello world<span data-escaped-char data-sourcepos="1:19-1:20">`</span> here.</p>
````````````````````````````````
