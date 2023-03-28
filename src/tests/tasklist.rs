use super::*;

#[test]
fn tasklist() {
    html_opts!(
        [
            render.unsafe_,
            extension.tasklist,
            parse.relaxed_tasklist_matching
        ],
        concat!(
            "* [ ] Red\n",
            "* [x] Green\n",
            "* [ ] Blue\n",
            "* [!] Papayawhip\n",
            "<!-- end list -->\n",
            "1. [ ] Bird\n",
            "2. [ ] McHale\n",
            "3. [x] Parish\n",
            "<!-- end list -->\n",
            "* [ ] Red\n",
            "  * [x] Green\n",
            "    * [ ] Blue\n"
        ),
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Red</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Green</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Blue</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Papayawhip</li>\n",
            "</ul>\n",
            "<!-- end list -->\n",
            "<ol>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Bird</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> McHale</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Parish</li>\n",
            "</ol>\n",
            "<!-- end list -->\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Red\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Green\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Blue</li>\n",
            "</ul>\n",
            "</li>\n",
            "</ul>\n",
            "</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn tasklist_relaxed_regression() {
    html_opts!(
        [extension.tasklist, parse.relaxed_tasklist_matching],
        "* [!] Red\n",
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Red</li>\n",
            "</ul>\n"
        ),
    );

    html_opts!(
        [extension.tasklist],
        "* [!] Red\n",
        concat!("<ul>\n", "<li>[!] Red</li>\n", "</ul>\n"),
    );

    html_opts!(
        [extension.tasklist, parse.relaxed_tasklist_matching],
        "* [!] Red\n",
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Red</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn tasklist_32() {
    html_opts!(
        [render.unsafe_, extension.tasklist],
        concat!(
            "- [ ] List item 1\n",
            "- [ ] This list item is **bold**\n",
            "- [x] There is some `code` here\n"
        ),
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> List item 1</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> This list item is <strong>bold</strong></li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> There is some <code>code</code> here</li>\n",
            "</ul>\n"
        ),
    );
}
