# Comrak

[![Build Status](https://github.com/kivikakk/comrak/actions/workflows/rust.yml/badge.svg)](https://github.com/kivikakk/comrak/actions/workflows/rust.yml) ![Spec
Status: 671/671](https://img.shields.io/badge/specs-671%2F671-brightgreen.svg) [![Financial Contributors on Open
Collective](https://opencollective.com/comrak/all/badge.svg?label=financial+contributors)](https://opencollective.com/comrak)
[![crates.io version](https://img.shields.io/crates/v/comrak.svg)](https://crates.io/crates/comrak)
[![docs.rs](https://docs.rs/comrak/badge.svg)](https://docs.rs/comrak)

Rust port of [github's `cmark-gfm`](https://github.com/github/cmark).

- [Installation](#installation)
- [Usage](#usage)
- [Security](#security)
- [Extensions](#extensions)
- [Related projects](#related-projects)
- [Contributing](#contributing)
- [Legal](#legal)

## Installation

Specify it as a requirement in `Cargo.toml`:

``` toml
[dependencies]
comrak = "0.10"
```

Comrak supports Rust stable.

### Mac & Linux Binaries

``` bash
curl https://webinstall.dev/comrak | bash
```

### Windows 10 Binaries

``` powershell
curl.exe -A "MS" https://webinstall.dev/comrak | powershell
```

## Usage

``` console
$ comrak --help
comrak 0.12.1
Ashe Connor <ashe@kivikakk.ee>
A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter

USAGE:
    comrak [FLAGS] [OPTIONS] [--] [FILE]...

FLAGS:
        --escape             Escape raw HTML instead of clobbering it
        --gfm                Enable GitHub-flavored markdown extensions strikethrough, tagfilter, table, autolink, and
                             tasklist. It also enables --github-pre-lang.
        --github-pre-lang    Use GitHub-style <pre lang> for code blocks
        --hardbreaks         Treat newlines as hard line breaks
    -h, --help               Prints help information
        --smart              Use smart punctuation
        --unsafe             Allow raw HTML and dangerous URLs
    -V, --version            Prints version information

OPTIONS:
    -c, --config-file <PATH>                    Path to config file containing command-line arguments, or `none'
                                                [default: /Users/kameliya/.config/comrak/config]
        --default-info-string <INFO>            Default value for fenced code block's info strings if none is given
    -e, --extension <EXTENSION>...              Specify an extension name to use [possible values: strikethrough,
                                                tagfilter, table, autolink, tasklist, superscript, footnotes,
                                                description-lists]
    -t, --to <FORMAT>                           Specify output format [default: html]  [possible values: html,
                                                commonmark]
        --front-matter-delimiter <DELIMITER>    Ignore front-matter that starts and ends with the given string
        --header-ids <PREFIX>                   Use the Comrak header IDs extension, with the given ID prefix
    -o, --output <FILE>                         Write output to FILE instead of stdout
        --syntax-highlighting <THEME>           Syntax highlighting for codefence blocks. Choose a theme or 'none' for
                                                disabling. [default: base16-ocean.dark]
        --width <WIDTH>                         Specify wrap width (0 = nowrap) [default: 0]

ARGS:
    <FILE>...    The CommonMark file to parse; or standard input if none passed

By default, Comrak will attempt to read command-line options from a config file specified by --config-file.  This
behaviour can be disabled by passing --config-file none.  It is not an error if the file does not exist.
```

And there's a Rust interface. You can use `comrak::markdown_to_html` directly:

``` rust
use comrak::{markdown_to_html, ComrakOptions};
assert_eq!(markdown_to_html("Hello, **世界**!", &ComrakOptions::default()),
           "<p>Hello, <strong>世界</strong>!</p>\n");
```

Or you can parse the input into an AST yourself, manipulate it, and then use your desired formatter:

``` rust
extern crate comrak;
use comrak::{parse_document, format_html, Arena, ComrakOptions};
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

As with [`cmark`](https://github.com/commonmark/cmark) and [`cmark-gfm`](https://github.com/github/cmark-gfm#security),
Comrak will scrub raw HTML and potentially dangerous links. This change was introduced in Comrak 0.4.0 in support of a
safe-by-default posture.

To allow these, use the `unsafe_` option (or `--unsafe` with the command line program). If doing so, we recommend the
use of a sanitisation library like [`ammonia`](https://github.com/notriddle/ammonia) configured specific to your needs.

## Extensions

Comrak supports the five extensions to CommonMark defined in the [GitHub Flavored Markdown
Spec](https://github.github.com/gfm/):

- [Tables](https://github.github.com/gfm/#tables-extension-)
- [Task list items](https://github.github.com/gfm/#task-list-items-extension-)
- [Strikethrough](https://github.github.com/gfm/#strikethrough-extension-)
- [Autolinks](https://github.github.com/gfm/#autolinks-extension-)
- [Disallowed Raw HTML](https://github.github.com/gfm/#disallowed-raw-html-extension-)

Comrak additionally supports its own extensions, which are yet to be specced out (PRs welcome\!):

- Superscript
- Header IDs
- Footnotes
- Description lists

By default none are enabled; they are individually enabled with each parse by setting the appropriate values in the
[`ComrakOptions` struct](https://docs.rs/comrak/newest/comrak/struct.ComrakOptions.html).

## Plugins

### Codefence syntax highlighter

At the moment syntax highlighting of codefence blocks is the only feature that can be enhanced with plugins.

Create an implementation of the `SyntaxHighlighterAdapter` trait, and then provide an instance of such adapter to
`ComrakPlugins.render.codefence_syntax_highlighter`. For formatting a markdown document with plugins, use the
`markdown_to_html_with_plugins` function, which accepts your plugin as a parameter.

See the `syntax_highlighter.rs` and `syntect.rs` examples for more details.

#### Syntect

[`syntect`](https://github.com/trishume/syntect) is a syntax highlighting library for Rust. By default, `comrak` offers
a plugin for it. In order to utilize it, create an instance of `plugins::syntect::SyntectAdapter` and use it as your
`ComrakPlugins` option.

## Related projects

Comrak's design goal is to model the upstream [`cmark-gfm`](https://github.com/github/cmark-gfm) as closely as possible
in terms of code structure. The upside of this is that a change in `cmark-gfm` has a very predictable change in Comrak.
Likewise, any bug in `cmark-gfm` is likely to be reproduced in Comrak. This could be considered a pro or a con,
depending on your use case.

The downside, of course, is that the code is not what I'd call idiomatic Rust (*so many `RefCell`s*), and while
contributors and I have made it as fast as possible, it simply won't be as fast as some other CommonMark parsers
depending on your use-case. Here are some other projects to consider:

- [Raph Levien](https://github.com/raphlinus)'s [`pulldown-cmark`](https://github.com/google/pulldown-cmark). It's
  very fast, uses a novel parsing algorithm, and doesn't construct an AST (but you can use it to make one if you
  want). `cargo doc` uses this, as do many other projects in the ecosystem.
- [Ben Navetta](https://github.com/bnavetta)'s [`rcmark`](https://github.com/bnavetta/rcmark) is a set of bindings to
  `libcmark`. It hasn't been updated in a while, though there's an [open pull
  request](https://github.com/bnavetta/rcmark/pull/2).
- Know of another library? Please open a PR to add it\!

As far as I know, Comrak is the only library to implement all of the [GitHub Flavored Markdown
extensions](https://github.github.com/gfm) to the spec, but this tends to only be important if you want to reproduce
GitHub's Markdown rendering exactly, e.g. in a GitHub client app.

## Contributing

Contributions are highly encouraged; where possible I practice [Optimistic Merging](http://hintjens.com/blog:106) as
described by Peter Hintjens. Please keep the [code of conduct](CODE_OF_CONDUCT.md) in mind when interacting with this
project.

Thank you to comrak's many contributors for PRs and issues opened\!

### Code Contributors

<a href="https://github.com/kivikakk/comrak/graphs/contributors"><img src="https://opencollective.com/comrak/contributors.svg?width=890&button=false" /></a>

### Financial Contributors

Become a financial contributor and help us sustain our community.
\[[Contribute](https://opencollective.com/comrak/contribute)\]

#### Individuals

<a href="https://opencollective.com/comrak"><img src="https://opencollective.com/comrak/individuals.svg?width=890"></a>

#### Organizations

Support this project with your organization. Your logo will show up here with a link to your website.
\[[Contribute](https://opencollective.com/comrak/contribute)\]

<a href="https://opencollective.com/comrak/organization/0/website"><img src="https://opencollective.com/comrak/organization/0/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/1/website"><img src="https://opencollective.com/comrak/organization/1/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/2/website"><img src="https://opencollective.com/comrak/organization/2/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/3/website"><img src="https://opencollective.com/comrak/organization/3/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/4/website"><img src="https://opencollective.com/comrak/organization/4/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/5/website"><img src="https://opencollective.com/comrak/organization/5/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/6/website"><img src="https://opencollective.com/comrak/organization/6/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/7/website"><img src="https://opencollective.com/comrak/organization/7/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/8/website"><img src="https://opencollective.com/comrak/organization/8/avatar.svg"></a>
<a href="https://opencollective.com/comrak/organization/9/website"><img src="https://opencollective.com/comrak/organization/9/avatar.svg"></a>

## Contact

Ashe Connor \<ashe kivikakk ee\>

## Legal

Copyright (c) 2017–2021, Ashe Connor. Licensed under the [2-Clause BSD
License](https://opensource.org/licenses/BSD-2-Clause).

`cmark` itself is is copyright (c) 2014, John MacFarlane.

See [COPYING](COPYING) for all the details.
