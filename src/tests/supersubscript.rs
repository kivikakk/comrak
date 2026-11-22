use super::*;

#[test]
fn superscript() {
    html_opts!(
        [extension.superscript],
        "e = mc^2^.\n",
        "<p>e = mc<sup>2</sup>.</p>\n",
    );
}

#[test]
fn superscript_negative_exponents_are_real() {
    html_opts!(
        [extension.superscript],
        "i^-2^ = -1.\n",
        "<p>i<sup>-2</sup> = -1.</p>\n",
    );
}

#[test]
fn subscript() {
    html_opts!(
        [extension.subscript],
        "f~i~ = f~i - 2~ + f~i - 1~.\n",
        "<p>f<sub>i</sub> = f<sub>i - 2</sub> + f<sub>i - 1</sub>.</p>\n",
    );
}

#[test]
fn subscript_negative_terms_are_real() {
    html_opts!(
        [extension.subscript],
        "f~i~ = f~-2 + i~ + f~-1 + i~.\n",
        "<p>f<sub>i</sub> = f<sub>-2 + i</sub> + f<sub>-1 + i</sub>.</p>\n",
    );
}

#[test]
fn superscript_inside_brackets() {
    html_opts!(
        [extension.superscript],
        "[a^2^](/u) and ![b^2^](/i) and [c^2^] and [ d^2^\n",
        concat!(
            "<p><a href=\"/u\">a<sup>2</sup></a> and <img src=\"/i\" alt=\"b2\" /> ",
            "and [c<sup>2</sup>] and [ d<sup>2</sup></p>\n"
        ),
    );
}

#[test]
fn superscript_beside_footnote_references() {
    html_opts!(
        [extension.superscript, extension.footnotes],
        "a[^1] b^c^ [^1]^d^ [^x] e^f^\n\n[^1]: one\n",
        concat!(
            "<p>a<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\" data-footnote-ref>1</a></sup> ",
            "b<sup>c</sup> ",
            "<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1-2\" data-footnote-ref>1</a></sup><sup>d</sup> ",
            "[<sup>x] e</sup>f^</p>\n",
            "<section class=\"footnotes\" data-footnotes>\n",
            "<ol>\n",
            "<li id=\"fn-1\">\n",
            "<p>one <a href=\"#fnref-1\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1\" aria-label=\"Back to reference 1\">↩</a> ",
            "<a href=\"#fnref-1-2\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"1-2\" aria-label=\"Back to reference 1-2\">↩<sup class=\"footnote-ref\">2</sup></a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}
