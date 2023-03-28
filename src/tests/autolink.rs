use super::*;

#[test]
fn autolink_www() {
    html_opts!(
        [extension.autolink],
        concat!("www.autolink.com\n"),
        concat!("<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n"),
    );
}

#[test]
fn autolink_email() {
    html_opts!(
        [extension.autolink],
        concat!("john@smith.com\n"),
        concat!("<p><a href=\"mailto:john@smith.com\">john@smith.com</a></p>\n"),
    );
}

#[test]
fn autolink_scheme() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\n"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a></p>\n"
        ),
    );
}

#[test]
fn autolink_scheme_multiline() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\nhttps://www.google.com/maps"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a>\n<a href=\"https://www.google.com/maps\">\
             https://www.google.com/maps</a></p>\n"
        ),
    );
}

#[test]
fn autolink_no_link_bad() {
    html_opts!(
        [extension.autolink],
        concat!("@a.b.c@. x\n", "\n", "n@. x\n"),
        concat!("<p>@a.b.c@. x</p>\n", "<p>n@. x</p>\n"),
    );
}
