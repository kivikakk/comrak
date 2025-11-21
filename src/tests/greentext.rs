use super::*;

#[test]
fn greentext_preserved() {
    html_opts!(
        [extension.greentext, render.hardbreaks],
        ">implying\n>>implying",
        "<p>&gt;implying<br />\n&gt;&gt;implying</p>\n"
    );
}

#[test]
fn empty_line() {
    html_opts!([extension.greentext], ">", "<p>&gt;</p>\n");
}

#[test]
fn separate_quotes_on_line_end() {
    html_opts!(
        [extension.greentext],
        "> 1\n>\n> 2",
        "<blockquote>\n<p>1</p>\n</blockquote>\n<p>&gt;</p>\n<blockquote>\n<p>2</p>\n</blockquote>\n"
    );
}

#[test]
fn unnest_quotes_on_line_end() {
    html_opts!(
        [extension.greentext],
        "> 1\n> > 2\n> 1",
        "<blockquote>\n<p>1</p>\n<blockquote>\n<p>2</p>\n</blockquote>\n<p>1</p>\n</blockquote>\n"
    );
}

#[test]
fn unnest_quotes_on_line_end_commonmark() {
    html_opts!(
        [extension.greentext],
        "> 1\n> > 2\n> \n> 1",
        "<blockquote>\n<p>1</p>\n<blockquote>\n<p>2</p>\n</blockquote>\n<p>1</p>\n</blockquote>\n"
    );
}
