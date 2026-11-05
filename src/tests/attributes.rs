#![cfg(feature = "attributes")]

use crate::tests::{assert_ast_match, html_opts, html_opts_i};

#[test]
fn heading_with_attrs() {
    assert_ast_match!(
        [extension.header_attributes],
        "# Hi! {#greeting}\n",
        (document (1:1-1:17) [
            (heading (1:1-1:17) {#greeting} [
                (text (1:3-1:5) "Hi!")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "## Yeww {.ok yeww=\"\\\"true\\\"\"} ##\n",
        (document (1:1-1:32) [
            (heading (1:1-1:32) {.ok yeww="\"true\""} [
                (text (1:4-1:7) "Yeww")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "## Yeww ## {x=y x=y #a #b}\n",
        (document (1:1-1:26) [
            (heading (1:1-1:26) {#b x="y" x="y"} [
                (text (1:4-1:10) "Yeww ##")
            ])
        ])
    );

    assert_ast_match!(
        [extension.header_attributes],
        "Yeww {x=y x=y #a #b}\n"
        "====================\n",
        (document (1:1-2:20) [
            (heading (1:1-2:20) {#b x="y" x="y"} [
                (text (1:1-1:4) "Yeww")
            ])
        ])
    );
}

#[test]
fn fenced_code_with_attrs() {
    assert_ast_match!(
        [extension.fenced_code_attributes],
        "```rust {#example}\n"
        "const fn dogs() -> ! { yay }\n"
        "```\n",
        (document (1:1-3:3) [
            (code_block (1:1-3:3) info:"rust" {#example} "const fn dogs() -> ! { yay }\n")
        ])
    );

    // Pandoc does some weird language/first-class interpretative dance,
    // but I'm not yet convinced I want to do that. Let the formatter decide.
    assert_ast_match!(
        [extension.fenced_code_attributes],
        "```{.zig #truism}\n"
        "fn cats() noreturn { yay_too }\n"
        "```\n",
        (document (1:1-3:3) [
            (code_block (1:1-3:3) info:"" {#truism .zig} "fn cats() noreturn { yay_too }\n")
        ])
    );
}

#[test]
fn inline_code_with_attrs() {
    assert_ast_match!(
        [extension.inline_code_attributes],
        "Uhh `totally`{yea=so} hm.",
        (document (1:1-1:25) [
            (paragraph (1:1-1:25) [
                (text (1:1-1:4) "Uhh ")
                // There's probably someone out there depending on sourcepos not being this way.
                // But then again, if they enable attributes, it's on them!
                (code (1:5-1:21) {yea="so"} "totally")
                (text (1:22-1:25) " hm.")
            ])
        ])
    );

    // "Bart, can we stop for newlines?"
    // "I don't see why not."
    assert_ast_match!(
        [extension.inline_code_attributes],
        "`T1 Faker`{role=mid\n"
        "  kda=\"0/0/0\"\n"
        "  cs=120}\n",
        (document (1:1-3:9) [
            (paragraph (1:1-3:9) [
                (code (1:1-3:9) {role="mid" kda="0/0/0" cs="120"} "T1 Faker")
            ])
        ])
    );
}

#[test]
fn links_and_images_with_attrs() {
    assert_ast_match!(
        [extension.link_attributes],
        "Incredible [Jhin](game){lies=true}.\n",
        (document (1:1-1:35) [
            (paragraph (1:1-1:35) [
                (text (1:1-1:11) "Incredible ")
                (link (1:12-1:34) {lies="true"} "game" [
                    (text (1:13-1:16) "Jhin")
                ])
                (text (1:35-1:35) ".")
            ])
        ])
    );

    assert_ast_match!(
        [extension.link_attributes],
        "Incredible ![Jhin](game){lies=true}.\n",
        (document (1:1-1:36) [
            (paragraph (1:1-1:36) [
                (text (1:1-1:11) "Incredible ")
                (image (1:12-1:35) {lies="true"} "game" [
                    (text (1:14-1:17) "Jhin")
                ])
                (text (1:36-1:36) ".")
            ])
        ])
    );
}

#[test]
fn multiline_link_attrs_sourcepos() {
    assert_ast_match!(
        [extension.link_attributes, extension.inline_code_attributes],
        "a\n"
        "b ![](x){w=1\n"
        "   h=2} c `d` e\n"
        "f *g*\n",
        (document (1:1-4:5) [
            (paragraph (1:1-4:5) [
                (text (1:1-1:1) "a")
                (softbreak (1:2-1:2))
                (text (2:1-2:2) "b ")
                (image (2:3-3:7) {w="1" h="2"} "x")
                (text (3:8-3:10) " c ")
                (code (3:11-3:13) "d")
                (text (3:14-3:15) " e")
                (softbreak (3:16-3:16))
                (text (4:1-4:2) "f ")
                (emph (4:3-4:5) [
                    (text (4:4-4:4) "g")
                ])
            ])
        ])
    );
}

#[test]
fn multiline_inline_code_attrs_sourcepos() {
    assert_ast_match!(
        [extension.inline_code_attributes],
        "a\n"
        "`x`{w=1\n"
        "  h=2} b\n",
        (document (1:1-3:8) [
            (paragraph (1:1-3:8) [
                (text (1:1-1:1) "a")
                (softbreak (1:2-1:2))
                (code (2:1-3:6) {w="1" h="2"} "x")
                (text (3:7-3:8) " b")
            ])
        ])
    );
}

#[test]
fn render_inline_attrs() {
    html_opts!(
        [
            extension.link_attributes,
            extension.inline_code_attributes,
            render.attributes,
            render.r#unsafe
        ],
        "[a](u \"t\"){#i .c .d k=v} ![b](s \"t\"){.c k=\"v w\"} `x`{#y .c}\n",
        "<p><a href=\"u\" title=\"t\" id=\"i\" class=\"c d\" k=\"v\">a</a> <img src=\"s\" alt=\"b\" title=\"t\" class=\"c\" k=\"v w\" /> <code id=\"y\" class=\"c\">x</code></p>\n",
        no_roundtrip,
    );
}

#[test]
fn render_attrs_after_own() {
    html_opts!(
        [
            extension.link_attributes,
            render.attributes,
            render.r#unsafe
        ],
        "![b](s){src=\"javascript:alert(1)\"} [a](u){href=v}\n",
        "<p><img src=\"s\" alt=\"b\" src=\"javascript:alert(1)\" /> <a href=\"u\" href=\"v\">a</a></p>\n",
        no_roundtrip,
    );
}

#[test]
fn render_attrs_escaped() {
    html_opts!(
        [
            extension.link_attributes,
            render.attributes,
            render.r#unsafe
        ],
        "[a](u){title=\"x\\\"<y>&\" .\"p&q\"}\n",
        "<p><a href=\"u\" class=\"p&amp;q\" title=\"x&quot;&lt;y&gt;&amp;\">a</a></p>\n",
        no_roundtrip,
    );
}

#[test]
fn render_attrs_needs_unsafe() {
    html_opts!(
        [
            extension.link_attributes,
            extension.inline_code_attributes,
            extension.header_attributes,
            extension.fenced_code_attributes,
            render.attributes
        ],
        "# H {#h .c}\n\n[a](u){.c k=v} `x`{.c}\n\n```rust {.c}\nx\n```\n",
        "<h1>H</h1>\n<p><a href=\"u\">a</a> <code>x</code></p>\n<pre><code class=\"language-rust\">x\n</code></pre>\n",
        no_roundtrip,
    );
}

#[test]
fn render_attrs_needs_render_option() {
    html_opts!(
        [
            extension.link_attributes,
            extension.inline_code_attributes,
            render.r#unsafe
        ],
        "[a](u){.c k=v} `x`{.c}\n",
        "<p><a href=\"u\">a</a> <code>x</code></p>\n",
        no_roundtrip,
    );
}

#[test]
fn render_heading_attrs() {
    html_opts!(
        [
            extension.header_attributes,
            render.attributes,
            render.r#unsafe
        ],
        "# Hi {#greeting .c k=v}\n",
        "<h1 id=\"greeting\" class=\"c\" k=\"v\">Hi</h1>\n",
        no_roundtrip,
    );
}

#[test]
fn render_heading_attrs_with_id_prefix() {
    html_opts_i(
        "# Hi {#greeting .c}\n\n# Hi\n\n# Hey {#\"a\\\"b\"}\n",
        concat!(
            "<h1 id=\"user-content-greeting\" class=\"c\">Hi<a href=\"#greeting\" aria-label=\"Link to heading 'Hi'\" data-heading-content=\"Hi\" class=\"anchor\"></a></h1>\n",
            "<h1 id=\"user-content-hi\">Hi<a href=\"#hi\" aria-label=\"Link to heading 'Hi'\" data-heading-content=\"Hi\" class=\"anchor\"></a></h1>\n",
            "<h1 id=\"user-content-a&quot;b\">Hey<a href=\"#a&quot;b\" aria-label=\"Link to heading 'Hey'\" data-heading-content=\"Hey\" class=\"anchor\"></a></h1>\n",
        ),
        false,
        |opts| {
            opts.extension.header_attributes = true;
            opts.extension.header_id_prefix = Some("user-content-".to_owned());
            opts.render.attributes = true;
            opts.render.r#unsafe = true;
        },
    );
}

#[test]
fn render_code_block_attrs() {
    html_opts!(
        [
            extension.fenced_code_attributes,
            render.attributes,
            render.r#unsafe
        ],
        "```rust {#ex .c .d k=v}\nx\n```\n",
        "<pre class=\"c d\" id=\"ex\" k=\"v\"><code class=\"language-rust\">x\n</code></pre>\n",
        no_roundtrip,
    );
}

#[test]
fn render_code_block_attrs_own_win() {
    html_opts!(
        [
            extension.fenced_code_attributes,
            render.github_pre_lang,
            render.attributes,
            render.r#unsafe
        ],
        "```rust {lang=evil k=v}\nx\n```\n",
        "<pre k=\"v\" lang=\"rust\"><code>x\n</code></pre>\n",
        no_roundtrip,
    );
}

#[test]
#[cfg(feature = "syntect")]
fn render_code_block_attrs_syntect() {
    let adapter = crate::plugins::syntect::SyntectAdapter::new(None);
    let mut plugins = crate::options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let mut options = crate::Options::default();
    options.extension.fenced_code_attributes = true;
    options.render.attributes = true;
    options.render.r#unsafe = true;

    let arena = crate::Arena::new();
    let root = crate::parse_document(&arena, "```rust {#ex .c}\nx\n```\n", &options);
    let mut output = String::new();
    crate::format_html_with_plugins(root, &options, &mut output, &plugins).unwrap();

    assert!(
        output.starts_with(
            "<pre class=\"syntax-highlighting c\" id=\"ex\"><code class=\"language-rust\">"
        ),
        "{output}"
    );
}

#[test]
fn explicit_heading_ids_are_reserved() {
    html_opts_i(
        "# Hi!\n\n# Hi! {#hi}\n\n# Hi!\n",
        concat!(
            "<h1 id=\"user-content-hi-1\">Hi!<a href=\"#hi-1\" aria-label=\"Link to heading 'Hi!'\" data-heading-content=\"Hi!\" class=\"anchor\"></a></h1>\n",
            "<h1 id=\"user-content-hi\">Hi!<a href=\"#hi\" aria-label=\"Link to heading 'Hi!'\" data-heading-content=\"Hi!\" class=\"anchor\"></a></h1>\n",
            "<h1 id=\"user-content-hi-2\">Hi!<a href=\"#hi-2\" aria-label=\"Link to heading 'Hi!'\" data-heading-content=\"Hi!\" class=\"anchor\"></a></h1>\n",
        ),
        false,
        |opts| {
            opts.extension.header_attributes = true;
            opts.extension.header_id_prefix = Some("user-content-".to_owned());
            opts.render.attributes = true;
            opts.render.r#unsafe = true;
        },
    );
}

#[test]
fn explicit_heading_ids_need_unsafe_to_be_reserved() {
    html_opts_i(
        "# Hi!\n\n# Hi! {#hi}\n",
        concat!(
            "<h1 id=\"user-content-hi\">Hi!<a href=\"#hi\" aria-label=\"Link to heading 'Hi!'\" data-heading-content=\"Hi!\" class=\"anchor\"></a></h1>\n",
            "<h1 id=\"user-content-hi-1\">Hi!<a href=\"#hi-1\" aria-label=\"Link to heading 'Hi!'\" data-heading-content=\"Hi!\" class=\"anchor\"></a></h1>\n",
        ),
        false,
        |opts| {
            opts.extension.header_attributes = true;
            opts.extension.header_id_prefix = Some("user-content-".to_owned());
            opts.render.attributes = true;
        },
    );
}

#[test]
fn link_tail_crossing_lines_sourcepos() {
    assert_ast_match!(
        [extension.link_attributes],
        "[a](\n"
        "/u){.c} x\n",
        (document (1:1-2:9) [
            (paragraph (1:1-2:9) [
                (link (1:1-2:7) {.c} "/u" [
                    (text (1:2-1:2) "a")
                ])
                (text (2:8-2:9) " x")
            ])
        ])
    );
}
