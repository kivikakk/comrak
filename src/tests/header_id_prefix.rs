use super::*;

#[test]
fn header_id_prefix() {
    html_opts_i(
        concat!(
            "# Hi.\n",
            "## Hi 1.\n",
            "### Hi.\n",
            "#### Hello.\n",
            "##### Hi.\n",
            "###### Hello.\n",
            "# Isn't it grand?"
        ),
        concat!(
            "<h1 id=\"user-content-hi\">Hi.<a href=\"#hi\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h1>\n",
            "<h2 id=\"user-content-hi-1\">Hi 1.<a href=\"#hi-1\" aria-label=\"Link to heading 'Hi 1.'\" data-heading-content=\"Hi 1.\" class=\"anchor\"></a></h2>\n",
            "<h3 id=\"user-content-hi-2\">Hi.<a href=\"#hi-2\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h3>\n",
            "<h4 id=\"user-content-hello\">Hello.<a href=\"#hello\" aria-label=\"Link to heading 'Hello.'\" data-heading-content=\"Hello.\" class=\"anchor\"></a></h4>\n",
            "<h5 id=\"user-content-hi-3\">Hi.<a href=\"#hi-3\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h5>\n",
            "<h6 id=\"user-content-hello-1\">Hello.<a href=\"#hello-1\" aria-label=\"Link to heading 'Hello.'\" data-heading-content=\"Hello.\" class=\"anchor\"></a></h6>\n",
            "<h1 id=\"user-content-isnt-it-grand\">Isn't it grand?<a href=\"#isnt-it-grand\" aria-label=\"Link to heading 'Isn't it grand?'\" data-heading-content=\"Isn't it grand?\" class=\"anchor\"></a></h1>\n"
        ),
        true,
        |opts| opts.extension.header_id_prefix = Some("user-content-".to_owned()),
    );
}

#[test]
fn header_ids_prefix_in_href() {
    html_opts_i(
        concat!(
            "# Hi.\n",
            "## Hi 1.\n",
            "### Hi.\n",
            "#### Hello.\n",
            "##### Hi.\n",
            "###### Hello.\n",
            "# Isn't it grand?"
        ),
        concat!(
            "<h1 id=\"user-content-hi\">Hi.<a href=\"#user-content-hi\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h1>\n",
            "<h2 id=\"user-content-hi-1\">Hi 1.<a href=\"#user-content-hi-1\" aria-label=\"Link to heading 'Hi 1.'\" data-heading-content=\"Hi 1.\" class=\"anchor\"></a></h2>\n",
            "<h3 id=\"user-content-hi-2\">Hi.<a href=\"#user-content-hi-2\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h3>\n",
            "<h4 id=\"user-content-hello\">Hello.<a href=\"#user-content-hello\" aria-label=\"Link to heading 'Hello.'\" data-heading-content=\"Hello.\" class=\"anchor\"></a></h4>\n",
            "<h5 id=\"user-content-hi-3\">Hi.<a href=\"#user-content-hi-3\" aria-label=\"Link to heading 'Hi.'\" data-heading-content=\"Hi.\" class=\"anchor\"></a></h5>\n",
            "<h6 id=\"user-content-hello-1\">Hello.<a href=\"#user-content-hello-1\" aria-label=\"Link to heading 'Hello.'\" data-heading-content=\"Hello.\" class=\"anchor\"></a></h6>\n",
            "<h1 id=\"user-content-isnt-it-grand\">Isn't it grand?<a href=\"#user-content-isnt-it-grand\" aria-label=\"Link to heading 'Isn't it grand?'\" data-heading-content=\"Isn't it grand?\" class=\"anchor\"></a></h1>\n"
        ),
        true,
        |opts| {
            opts.extension.header_id_prefix = Some("user-content-".to_owned());
            opts.extension.header_id_prefix_in_href = true;
        },
    );
}

#[test]
fn header_id_prefix_in_href_without_prefix() {
    // When header_id_prefix is None, header_id_prefix_in_href has no effect
    html_opts_i("# Hi.\n", "<h1>Hi.</h1>\n", true, |opts| {
        opts.extension.header_id_prefix_in_href = true;
    });
}
