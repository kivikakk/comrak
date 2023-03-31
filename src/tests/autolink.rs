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

#[test]
fn sourcepos_correctly_restores_context() {
    assert_ast_match!(
        [],
        "ab _cde_ f@g.ee *hijkl* m",
        (document (1:1-1:25) [
            (paragraph (1:1-1:25) [
                (text (1:1-1:3) "ab ")
                (emph (1:4-1:8) [
                    (text (1:5-1:7) "cde")
                ])
                (text (1:9-1:16) " f@g.ee ")
                (emph (1:17-1:23) [
                    (text (1:18-1:22) "hijkl")
                ])
                (text (1:24-1:25) " m")
            ])
        ])
    );

    assert_ast_match!(
        [extension.autolink],
        "ab _cde_ f@g.ee *hijkl* m",
        (document (1:1-1:25) [
            (paragraph (1:1-1:25) [
                (text (1:1-1:3) "ab ")
                (emph (1:4-1:8) [
                    (text (1:5-1:7) "cde")
                ])
                (text (1:9-1:10) " ")
                (link (1:10-1:15) [
                    (text (1:10-1:15) "f@g.ee")
                ])
                (text (1:16-1:16) " ")
                (emph (1:17-1:23) [
                    (text (1:18-1:22) "hijkl")
                ])
                (text (1:24-1:25) " m")
            ])
        ])
    );
}
