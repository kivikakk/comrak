---
title: Header IDs
---

```````````````````````````````` example
# README
.
<h1>README</h1>
````````````````````````````````

```````````````````````````````` example --header-id-prefix user-content-
# README
.
<h1 id="user-content-readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

```````````````````````````````` example --header-id-prefix ""
# README
.
<h1 id="readme">README<a href="#readme" aria-label="Link to heading 'README'" data-heading-content="README" class="anchor"></a></h1>
````````````````````````````````

```````````````````````````````` example --header-id-prefix user-content-
# Hi.
## Hi 1.
### Hi.
#### Hello.
##### Hi.
###### Hello.
# Isn't it grand?
.
<h1 id="user-content-hi">Hi.<a href="#hi" aria-label="Link to heading 'Hi.'" data-heading-content="Hi." class="anchor"></a></h1>
<h2 id="user-content-hi-1">Hi 1.<a href="#hi-1" aria-label="Link to heading 'Hi 1.'" data-heading-content="Hi 1." class="anchor"></a></h2>
<h3 id="user-content-hi-2">Hi.<a href="#hi-2" aria-label="Link to heading 'Hi.'" data-heading-content="Hi." class="anchor"></a></h3>
<h4 id="user-content-hello">Hello.<a href="#hello" aria-label="Link to heading 'Hello.'" data-heading-content="Hello." class="anchor"></a></h4>
<h5 id="user-content-hi-3">Hi.<a href="#hi-3" aria-label="Link to heading 'Hi.'" data-heading-content="Hi." class="anchor"></a></h5>
<h6 id="user-content-hello-1">Hello.<a href="#hello-1" aria-label="Link to heading 'Hello.'" data-heading-content="Hello." class="anchor"></a></h6>
<h1 id="user-content-isnt-it-grand">Isn't it grand?<a href="#isnt-it-grand" aria-label="Link to heading 'Isn't it grand?'" data-heading-content="Isn't it grand?" class="anchor"></a></h1>
````````````````````````````````

```````````````````````````````` example --header-id-prefix user-content-
# a
# a
# a-1
# a
.
<h1 id="user-content-a">a<a href="#a" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
<h1 id="user-content-a-1">a<a href="#a-1" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
<h1 id="user-content-a-1-1">a-1<a href="#a-1-1" aria-label="Link to heading 'a-1'" data-heading-content="a-1" class="anchor"></a></h1>
<h1 id="user-content-a-2">a<a href="#a-2" aria-label="Link to heading 'a'" data-heading-content="a" class="anchor"></a></h1>
````````````````````````````````
