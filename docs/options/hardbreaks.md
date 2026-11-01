---
title: Hard line breaks
---

```````````````````````````````` example --hardbreaks
Hello.
World.
.
<p>Hello.<br />
World.</p>
````````````````````````````````

```````````````````````````````` example
Hello.
World.
.
<p>Hello.
World.</p>
````````````````````````````````

```````````````````````````````` example --hardbreaks
> Roses are red,
> violets are blue.
.
<blockquote>
<p>Roses are red,<br />
violets are blue.</p>
</blockquote>
````````````````````````````````

```````````````````````````````` example --hardbreaks
Hard  
Soft
Backslash\
end
.
<p>Hard<br />
Soft<br />
Backslash<br />
end</p>
````````````````````````````````
