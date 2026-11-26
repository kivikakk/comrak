use super::*;

#[test]
fn full_info_string_skips_separating_whitespace() {
    html_opts!(
        [render.full_info_string],
        "``` rust   a   b\nx\n```\n\n``` rust\t  c\nx\n```\n",
        concat!(
            "<pre><code class=\"language-rust\" data-meta=\"a   b\">x\n</code></pre>\n",
            "<pre><code class=\"language-rust\" data-meta=\"c\">x\n</code></pre>\n",
        ),
    );
}

#[test]
fn full_info_string_keeps_unicode_whitespace() {
    html_opts!(
        [render.full_info_string],
        "``` rust \u{a0}a\nx\n```\n\n``` rust b\u{a0}\nx\n```\n",
        concat!(
            "<pre><code class=\"language-rust\" data-meta=\"\u{a0}a\">x\n</code></pre>\n",
            "<pre><code class=\"language-rust\" data-meta=\"b\u{a0}\">x\n</code></pre>\n",
        ),
    );
}
