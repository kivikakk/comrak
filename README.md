# [Comrak](https://github.com/kivikakk/comrak)

[![Build status](https://github.com/kivikakk/comrak/actions/workflows/rust.yml/badge.svg)](https://github.com/kivikakk/comrak/actions/workflows/rust.yml)
[![CommonMark: 652/652](https://img.shields.io/badge/commonmark-652%2F652-brightgreen.svg)](https://github.com/commonmark/commonmark-spec/blob/9103e341a973013013bb1a80e13567007c5cef6f/spec.txt)
[![GFM: 670/670](https://img.shields.io/badge/gfm-670%2F670-brightgreen.svg)](https://github.com/kivikakk/cmark-gfm/blob/2f13eeedfe9906c72a1843b03552550af7bee29a/test/spec.txt)
[![crates.io version](https://img.shields.io/crates/v/comrak.svg)](https://crates.io/crates/comrak)
[![docs.rs](https://docs.rs/comrak/badge.svg)](https://docs.rs/comrak)

Rust port of [github's `cmark-gfm`](https://github.com/github/cmark-gfm).

Compliant with [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) in default mode.
GFM support synced with release `0.29.0.gfm.13`.

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
comrak = "0.24"
```

Comrak's library supports Rust <span class="msrv">1.62.1</span>+.

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
A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter

Usage: comrak [OPTIONS] [FILE]...

Arguments:
  [FILE]...
          CommonMark file(s) to parse; or standard input if none passed

Options:
  -c, --config-file <PATH>
          Path to config file containing command-line arguments, or 'none'
          
          [default: /Users/kivikakk/.config/comrak/config]

  -i, --inplace
          To perform an in-place formatting

      --hardbreaks
          Treat newlines as hard line breaks

      --smart
          Use smart punctuation

      --github-pre-lang
          Use GitHub-style <pre lang> for code blocks

      --full-info-string
          Enable full info strings for code blocks

      --gfm
          Enable GitHub-flavored markdown extensions: strikethrough, tagfilter, table, autolink, and
          tasklist. Also enables --github-pre-lang and --gfm-quirks

      --gfm-quirks
          Enables GFM-style quirks in output HTML, such as not nesting <strong> tags, which
          otherwise breaks CommonMark compatibility

      --relaxed-tasklist-character
          Enable relaxing which character is allowed in a tasklists

      --relaxed-autolinks
          Enable relaxing of autolink parsing, allow links to be recognized when in brackets and
          allow all url schemes

      --default-info-string <INFO>
          Default value for fenced code block's info strings if none is given

      --unsafe
          Allow raw HTML and dangerous URLs

      --gemojis
          Translate gemojis into UTF-8 characters

      --escape
          Escape raw HTML instead of clobbering it

      --escaped-char-spans
          Wrap escaped characters in span tags

  -e, --extension <EXTENSION>
          Specify extension name(s) to use
          
          Multiple extensions can be delimited with ",", e.g. --extension strikethrough,table
          
          [possible values: strikethrough, tagfilter, table, autolink, tasklist, superscript,
          footnotes, description-lists, multiline-block-quotes, math-dollars, math-code,
          wikilinks-title-after-pipe, wikilinks-title-before-pipe, underline, spoiler, greentext]

  -t, --to <FORMAT>
          Specify output format
          
          [default: html]
          [possible values: html, xml, commonmark]

  -o, --output <FILE>
          Write output to FILE instead of stdout

      --width <WIDTH>
          Specify wrap width (0 = nowrap)
          
          [default: 0]

      --header-ids <PREFIX>
          Use the Comrak header IDs extension, with the given ID prefix

      --front-matter-delimiter <DELIMITER>
          Ignore front-matter that starts and ends with the given string

      --syntax-highlighting <THEME>
          Syntax highlighting for codefence blocks. Choose a theme or 'none' for disabling
          
          [default: base16-ocean.dark]

      --list-style <LIST_STYLE>
          Specify bullet character for lists (-, +, *) in CommonMark output
          
          [default: dash]
          [possible values: dash, plus, star]

      --sourcepos
          Include source position attribute in HTML and XML output

      --ignore-setext
          Ignore setext headers

      --ignore-empty-links
          Ignore empty links

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

By default, Comrak will attempt to read command-line options from a config file specified by
--config-file. This behaviour can be disabled by passing --config-file none. It is not an error if
the file does not exist.
```

And there's a Rust interface. You can use `comrak::markdown_to_html` directly:

``` rust
use comrak::{markdown_to_html, Options};
assert_eq!(markdown_to_html("Hello, **世界**!", &Options::default()),
           "<p>Hello, <strong>世界</strong>!</p>\n");
```

Or you can parse the input into an AST yourself, manipulate it, and then use your desired formatter:

``` rust
extern crate comrak;
use comrak::nodes::NodeValue;
use comrak::{format_html, parse_document, Arena, Options};

fn replace_text(document: &str, orig_string: &str, replacement: &str) -> String {
    // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
    let arena = Arena::new();

    // Parse the document into a root `AstNode`
    let root = parse_document(&arena, document, &Options::default());

    // Iterate over all the descendants of root.
    for node in root.descendants() {
        if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
            // If the node is a text node, perform the string replacement.
            *text = text.replace(orig_string, replacement)
        }
    }

    let mut html = vec![];
    format_html(root, &Options::default(), &mut html).unwrap();

    String::from_utf8(html).unwrap()
}

fn main() {
    let doc = "This is my input.\n\n1. Also [my](#) input.\n2. Certainly *my* input.\n";
    let orig = "my";
    let repl = "your";
    let html = replace_text(&doc, &orig, &repl);

    println!("{}", html);
}
```

## Benchmarking

For running benchmarks, you will need to [install hyperfine](https://github.com/sharkdp/hyperfine#installation) and optionally cmake.

If you want to just run the benchmark for `comrak`, with the current state of the repo, you can simply run

``` bash
make bench-comrak
```

This will build comrak in release mode, and run benchmark on it. You will see the time measurements as reported by hyperfine in the console.

Makefile also provides a way to run benchmarks for `comrak` current state (with your changes), `comrak` main branch, [`cmark-gfm`](https://github.com/github/cmark-gfm), [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark) and [`markdown-it.rs`](https://github.com/rlidwka/markdown-it.rs). For this you will need to install `cmake`. After that make sure that you have set-up the git submodules. In case you have not installed submodules when cloning, you can do it by running

``` bash
git submodule update --init
```

After this is done, you can run

``` bash
make bench-all
```

which will run benchmarks across all, and report the time take by each as well as relative time.

Apart from this, CI is also setup for running benchmarks when a pull request is first opened. It will add a comment with the results on the pull request in a tabular format comparing the 5 versions. After that you can manually trigger this CI by commenting `/run-bench` on the PR, this will update the existing comment with new results. Note benchmarks won't be automatically run on each push.

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
- Front matter
- Shortcodes
- Math
- Multiline Blockquotes

By default none are enabled; they are individually enabled with each parse by setting the appropriate values in the
[`ComrakExtensionOptions` struct](https://docs.rs/comrak/newest/comrak/type.ComrakExtensionOptions.html).

## Plugins

### Codefence syntax highlighter

At the moment syntax highlighting of codefence blocks is the only feature that can be enhanced with plugins.

Create an implementation of the `SyntaxHighlighterAdapter` trait, and then provide an instance of such adapter to
`Plugins.render.codefence_syntax_highlighter`. For formatting a markdown document with plugins, use the
`markdown_to_html_with_plugins` function, which accepts your plugin as a parameter.

See the `syntax_highlighter.rs` and `syntect.rs` examples for more details.

#### Syntect

[`syntect`](https://github.com/trishume/syntect) is a syntax highlighting library for Rust. By default, `comrak` offers
a plugin for it. In order to utilize it, create an instance of `plugins::syntect::SyntectAdapter` and use it as your
`Plugins` option.

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
- [markdown-rs](https://github.com/wooorm/markdown-rs) (1.x) looks worth watching.
- Know of another library? Please open a PR to add it\!

As far as I know, Comrak is the only library to implement all of the [GitHub Flavored Markdown
extensions](https://github.github.com/gfm) to the spec, but this tends to only be important if you want to reproduce
GitHub's Markdown rendering exactly, e.g. in a GitHub client app.

## Contributing

Contributions are highly encouraged; where possible I practice [Optimistic Merging](http://hintjens.com/blog:106) as
described by Peter Hintjens. Please keep the [code of conduct](CODE_OF_CONDUCT.md) in mind when interacting with this
project.

Thank you to Comrak's many contributors for PRs and issues opened\!

### Code Contributors

<a href="https://github.com/kivikakk/comrak/graphs/contributors"><img src="https://opencollective.com/comrak/contributors.svg?width=890&button=false" /></a>

### Financial Contributors

Become a financial contributor and help sustain Comrak's development.  I'm
self-employed --- open-source software relies on the collective.

- [GitHub Sponsors](https://github.com/sponsors/kivikakk)

## Contact

Asherah Connor \<ashe kivikakk ee\>

## Legal

Copyright (c) 2017–2024, Asherah Connor and Comrak contributors. Licensed under
the [2-Clause BSD License](https://opensource.org/licenses/BSD-2-Clause).

`cmark` itself is is copyright (c) 2014, John MacFarlane.

See [COPYING](COPYING) for all the details.
