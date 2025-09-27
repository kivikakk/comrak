use super::*;

#[test]
fn superscript() {
    html_opts!(
        [extension.superscript],
        concat!("e = mc^2^.\n"),
        concat!("<p>e = mc<sup>2</sup>.</p>\n"),
    );
}

#[test]
fn superscript_negative_exponents_are_real() {
    html_opts!(
        [extension.superscript],
        concat!("i^-2^ = -1.\n"),
        concat!("<p>i<sup>-2</sup> = -1.</p>\n"),
    );
}

#[test]
fn subscript() {
    html_opts!(
        [extension.subscript],
        concat!("f~i~ = f~i - 2~ + f~i - 1~.\n"),
        concat!("<p>f<sub>i</sub> = f<sub>i - 2</sub> + f<sub>i - 1</sub>.</p>\n"),
    );
}

#[test]
fn subscript_negative_terms_are_real() {
    html_opts!(
        [extension.subscript],
        concat!("f~i~ = f~-2 + i~ + f~-1 + i~.\n"),
        concat!("<p>f<sub>i</sub> = f<sub>-2 + i</sub> + f<sub>-1 + i</sub>.</p>\n"),
    );
}
