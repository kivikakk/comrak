use super::*;

#[test]
fn default_info_string_is_trimmed() {
    html_opts!(
        [parse.default_info_string = Some(" \trust ignore \n".into())],
        "```\nx\n```\n",
        "<pre><code class=\"language-rust\">x\n</code></pre>\n",
    );
}

#[test]
fn default_info_string_only_whitespace() {
    html_opts!(
        [parse.default_info_string = Some("  ".into())],
        "```\nx\n```\n",
        "<pre><code>x\n</code></pre>\n",
    );
}

#[test]
fn default_info_string_is_verbatim() {
    html_opts!(
        [
            parse.default_info_string = Some("a&amp;\\*`\nb".into()),
            render.full_info_string = true
        ],
        "```\nx\n```\n",
        "<pre><code class=\"language-a&amp;amp;\\*`\" data-meta=\"b\">x\n</code></pre>\n",
    );
}
