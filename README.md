[![Build Status](https://travis-ci.org/kivikakk/comrak.svg?branch=master)](https://travis-ci.org/kivikakk/comrak)
![Spec Status: 643/643](https://img.shields.io/badge/specs-643%2F643-brightgreen.svg)
[![crates.io version](https://img.shields.io/crates/v/comrak.svg)](https://crates.io/crates/comrak)
[![docs.rs](https://docs.rs/comrak/badge.svg)](https://docs.rs/comrak)

Rust port of [github's `cmark-gfm`](https://github.com/github/cmark).

* [Usage](#usage)
* [Extensions](#extensions)
* [Legal](#legal)

## Usage

A binary is included which does everything you typically want:

```
$ comrak --help
comrak 0.1.9
Ashe Connor <ashe@kivikakk.ee>
CommonMark parser with GitHub Flavored Markdown extensions

USAGE:
    comrak [FLAGS] [OPTIONS] [--] [<FILE>]

FLAGS:
        --github-pre-lang    Use GitHub-style <pre lang> for code blocks
        --hardbreaks         Treat newlines as hard line breaks
    -h, --help               Prints help information
    -V, --version            Prints version information

OPTIONS:
    -e, --extension <EXTENSION>...    Specify an extension name to use [values: strikethrough, tagfilter, table, autolink, superscript]
    -t, --to <FORMAT>                 Specify output format [default: html]  [values: html, commonmark]
        --width <WIDTH>               Specify wrap width (0 = nowrap) [default: 0]

ARGS:
    <FILE>...    The CommonMark file to parse; or standard input if none passed
```

And there's a Rust interface.  You can use `comrak::markdown_to_html` directly:

``` rust
use comrak::{markdown_to_html, ComrakOptions};
assert_eq!(markdown_to_html("Hello, **世界**!", &ComrakOptions::default()),
           "<p>Hello, <strong>世界</strong>!</p>\n");
```

Or you can parse the input into an AST yourself, manipulate it, and then use your desired
formatter:

``` rust
extern crate comrak;
extern crate typed_arena;
use typed_arena::Arena;
use comrak::{parse_document, format_html, ComrakOptions};
use comrak::nodes::{AstNode, NodeValue};

// The returned nodes are created in the supplied Arena, and are bound by its lifetime.
let arena = Arena::new();

let root = parse_document(
    &arena,
    "This is my input.\n\n1. Also my input.\n2. Certainly my input.\n",
    &ComrakOptions::default());

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where F : Fn(&'a AstNode<'a>) {
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

iter_nodes(root, &|node| {
    match &mut node.data.borrow_mut().value {
        &mut NodeValue::Text(ref mut text) => {
            *text = text.replace("my", "your");
        }
        _ => (),
    }
});

let html: String = format_html(root, &ComrakOptions::default());

assert_eq!(
    html,
    "<p>This is your input.</p>\n\
     <ol>\n\
     <li>Also your input.</li>\n\
     <li>Certainly your input.</li>\n\
     </ol>\n");
```

## Extensions

Comrak supports the five extensions to CommonMark defined in the
[GitHub Flavored Markdown Spec](https://github.github.com/gfm/):

* [Tables](https://github.github.com/gfm/#tables-extension-)
* [Task list items](https://github.github.com/gfm/#task-list-items-extension-)
* [Strikethrough](https://github.github.com/gfm/#strikethrough-extension-)
* [Autolinks](https://github.github.com/gfm/#autolinks-extension-)
* [Disallowed Raw HTML](https://github.github.com/gfm/#disallowed-raw-html-extension-)

as well as superscript.

By default none are enabled; they are individually enabled with each parse by
setting the appropriate values in the
[`ComrakOptions` struct](https://docs.rs/comrak/newest/comrak/struct.ComrakOptions.html).

## Legal

Copyright (c) 2017, Ashe Connor.  Licensed under the [2-Clause BSD License](https://opensource.org/licenses/BSD-2-Clause).

`cmark` itself is is copyright (c) 2014, John MacFarlane.

See [COPYING](COPYING) for all the details.

## Contributors

Thank you for PRs and issues opened!

* [ConnyOnny](https://github.com/ConnyOnny)
* [killercup](https://github.com/killercup)
* [bovarysme](https://github.com/bovarysme)
* [gjtorikian](https://github.com/gjtorikian)
* [SSJohns](https://github.com/SSJohns)
* [zeantsoi](https://github.com/zeantsoi)
* [DemiMarie](https://github.com/DemiMarie)
