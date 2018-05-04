[![Build Status](https://travis-ci.org/kivikakk/comrak.svg?branch=master)](https://travis-ci.org/kivikakk/comrak)
![Spec Status: 647/647](https://img.shields.io/badge/specs-647%2F647-brightgreen.svg)
[![crates.io version](https://img.shields.io/crates/v/comrak.svg)](https://crates.io/crates/comrak)
[![docs.rs not building comrak right now!](https://docs.rs/comrak/badge.svg)](https://kivikakk.github.io/comrak/comrak/)

Rust port of [github's `cmark-gfm`](https://github.com/github/cmark).

* [Installation](#installation)
* [Usage](#usage)
* [Security](#security)
* [Extensions](#extensions)
* [Related projects](#related-projects)
* [Contributors](#contributors)
* [Legal](#legal)

## Installation

Specify it as a requirement in `Cargo.toml`:

```toml
[dependencies]
comrak = "0.2"
```

## Usage

A binary is included which does everything you typically want:

```
$ comrak --help
comrak 0.2.10
Ashe Connor <kivikakk@github.com>
A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter

USAGE:
    comrak [FLAGS] [OPTIONS] [--] [FILE]...

FLAGS:
        --footnotes          Parse footnotes
        --github-pre-lang    Use GitHub-style <pre lang> for code blocks
        --hardbreaks         Treat newlines as hard line breaks
    -h, --help               Prints help information
        --smart              Use smart punctuation
    -V, --version            Prints version information

OPTIONS:
        --default-info-string <INFO>    Default value for fenced code block's info strings if none is given
    -e, --extension <EXTENSION>...      Specify an extension name to use [values: strikethrough, tagfilter, table,
                                        autolink, tasklist, superscript, footnotes]
    -t, --to <FORMAT>                   Specify output format [default: html]  [values: html, commonmark]
        --header-ids <PREFIX>           Use the Comrak header IDs extension, with the given ID prefix
        --width <WIDTH>                 Specify wrap width (0 = nowrap) [default: 0]

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
            let orig = std::mem::replace(text, vec![]);
            *text = String::from_utf8(orig).unwrap().replace("my", "your").as_bytes().to_vec();
        }
        _ => (),
    }
});

let mut html = vec![];
format_html(root, &ComrakOptions::default(), &mut html).unwrap();

assert_eq!(
    String::from_utf8(html).unwrap(),
    "<p>This is your input.</p>\n\
     <ol>\n\
     <li>Also your input.</li>\n\
     <li>Certainly your input.</li>\n\
     </ol>\n");
```

## Security

As with [`cmark-gfm`](https://github.com/github/cmark#security), Comrak will pass through inline HTML, dangerous links, anything you can imagine — it only performs Markdown to HTML conversion per the CommonMark/GFM spec.  We recommend the use of a sanitisation library like [`ammonia`](https://github.com/notriddle/ammonia) configured specific to your needs.

## Extensions

Comrak supports the five extensions to CommonMark defined in the
[GitHub Flavored Markdown Spec](https://github.github.com/gfm/):

* [Tables](https://github.github.com/gfm/#tables-extension-)
* [Task list items](https://github.github.com/gfm/#task-list-items-extension-)
* [Strikethrough](https://github.github.com/gfm/#strikethrough-extension-)
* [Autolinks](https://github.github.com/gfm/#autolinks-extension-)
* [Disallowed Raw HTML](https://github.github.com/gfm/#disallowed-raw-html-extension-)

as well as superscript and footnotes.

By default none are enabled; they are individually enabled with each parse by
setting the appropriate values in the
[`ComrakOptions` struct](https://docs.rs/comrak/newest/comrak/struct.ComrakOptions.html).

## Related projects

Comrak's design goal is to model the upstream [`cmark-gfm`](https://github.com/github/cmark) as closely as possible in terms of code structure. The upside of this is that a change in `cmark-gfm` has a very predictable change in Comrak. It helps that I maintain both, and tend to update Comrak in lock-step with `cmark-gfm`. Likewise, any bug in `cmark-gfm` is likely to be reproduced in Comrak. This could be considered a pro or a con, depending on your use case.

The downside, of course, is that the code is not what I'd call idiomatic Rust (_so many `RefCell`s_), and while contributors and I have made it as fast as possible, it simply won't be as fast as some other CommonMark parsers depending on your use-case. Here are some other projects to consider:

* [Raph Levien](https://github.com/raphlinus)'s [`pulldown-cmark`](https://github.com/google/pulldown-cmark). It's very fast, uses a novel parsing algorithm, and doesn't construct an AST (but you can use it to make one if you want). Recent `cargo doc` uses this, as do many other projects in the ecosystem. It's not quite at 100% spec compatibility yet.
* [Ben Navetta](https://github.com/bnavetta)'s [`rcmark`](https://github.com/bnavetta/rcmark) is a set of bindings to `libcmark`. It hasn't been updated in a while, though there's an [open pull request](https://github.com/bnavetta/rcmark/pull/2).
* Know of another library? Please add it!

As far as I know, Comrak is the only library to implement all of the [GitHub Flavored Markdown extensions](https://github.github.com/gfm) to the spec, but this tends to only be important if you want to reproduce GitHub's Markdown rendering exactly, e.g. in a GitHub client app.

## Contributors

Thank you for PRs and issues opened!

* [ConnyOnny](https://github.com/ConnyOnny)
* [killercup](https://github.com/killercup)
* [bovarysme](https://github.com/bovarysme)
* [gjtorikian](https://github.com/gjtorikian)
* [SSJohns](https://github.com/SSJohns)
* [zeantsoi](https://github.com/zeantsoi)
* [DemiMarie](https://github.com/DemiMarie)
* [ivanceras](https://github.com/ivanceras)
* [anthonynguyen](https://github.com/anthonynguyen)
* [JordanMilne](https://github.com/JordanMilne)
* [carols10cents](https://github.com/carols10cents)
* [treiff](https://github.com/treiff)
* [steveklabnik](https://github.com/steveklabnik)
* [brson](https://github.com/brson)

## Legal

Copyright (c) 2017–2018, Ashe Connor.  Licensed under the [2-Clause BSD License](https://opensource.org/licenses/BSD-2-Clause).

`cmark` itself is is copyright (c) 2014, John MacFarlane.

See [COPYING](COPYING) for all the details.
