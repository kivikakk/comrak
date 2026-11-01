---
title: Phoenix HEEx
---

```````````````````````````````` example -e phoenix-heex
Click <.button>here</.button> to continue
.
<p>Click <.button>here</.button> to continue</p>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
User {user.name} has {user.age} items
.
<p>User {user.name} has {user.age} items</p>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
Value: {user.name} or <%= @user.name %>
.
<p>Value: {user.name} or <%= @user.name %></p>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
I am some text!
<%# Look a comment! %>
I am more some text!
.
<p>I am some text!
<%# Look a comment! %>
I am more some text!</p>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
# Contact Form

<.form
  for={@form}
  phx-change="change_name"
>
  <.input field={@form[:email]} />
</.form>

## Footer

Socials
.
<h1>Contact Form</h1>
<.form
  for={@form}
  phx-change="change_name"
>
  <.input field={@form[:email]} />
</.form>
<h2>Footer</h2>
<p>Socials</p>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
<MyComponent>

  Content here

</MyComponent>
.
<MyComponent>

  Content here

</MyComponent>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
<.modal>
  <:header>
    This is the top of the modal.
  </:header>
  This is the content of the modal.
</.modal>
.
<.modal>
  <:header>
    This is the top of the modal.
  </:header>
  This is the content of the modal.
</.modal>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
<.table rows={@users}>
  <:col :let={user} label="Name">
    {user.name}
  </:col>
  <:col :let={user} label="Email">
    {user.email}
  </:col>
</.table>
.
<.table rows={@users}>
  <:col :let={user} label="Name">
    {user.name}
  </:col>
  <:col :let={user} label="Email">
    {user.email}
  </:col>
</.table>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
<%= if true do %>
  <%= @x %>
<% end %>
.
<%= if true do %>
  <%= @x %>
<% end %>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex --unsafe
<div :for={item <- @items}>
  <%= item %>
</div>
.
<div :for={item <- @items}>
  <%= item %>
</div>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex -e table
<.wrapper>

| a | b |
|---|---|
| c | d |

</.wrapper>
.
<.wrapper>

| a | b |
|---|---|
| c | d |

</.wrapper>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
- <.button>
    text
  </.button>
.
<ul>
<li>
<.button>
  text
</.button>
</li>
</ul>
````````````````````````````````

```````````````````````````````` example -e phoenix-heex
Header: <:title>My Title</:title>
.
<p>Header: &lt;:title&gt;My Title&lt;/:title&gt;</p>
````````````````````````````````
