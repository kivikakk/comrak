use super::*;

#[test]
fn base() {
    let input = concat!(
        "foo *bar*\n",
        "\n",
        "paragraph 2\n",
        "\n",
        "```\n",
        "code\n",
        "```\n",
    );

    xml(
        input,
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <paragraph>\n",
            "    <text xml:space=\"preserve\">foo </text>\n",
            "    <emph>\n",
            "      <text xml:space=\"preserve\">bar</text>\n",
            "    </emph>\n",
            "  </paragraph>\n",
            "  <paragraph>\n",
            "    <text xml:space=\"preserve\">paragraph 2</text>\n",
            "  </paragraph>\n",
            "  <code_block xml:space=\"preserve\">code\n",
            "</code_block>\n",
            "</document>\n",
        ),
    );
}

#[test]
fn code_block_info_escaped() {
    xml(
        "``` a\"b<c>&d\te\ncode\n```\n",
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <code_block info=\"a&quot;b&lt;c&gt;&amp;d&#9;e\" xml:space=\"preserve\">code\n",
            "</code_block>\n",
            "</document>\n",
        ),
    );
}

#[test]
fn sourcepos() {
    let input = concat!(
        "foo *bar*\n",
        "\n",
        "paragraph 2\n",
        "\n",
        "```\n",
        "code\n",
        "```\n",
    );

    xml_opts(
        input,
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document sourcepos=\"1:1-7:3\" xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <paragraph sourcepos=\"1:1-1:9\">\n",
            "    <text sourcepos=\"1:1-1:4\" xml:space=\"preserve\">foo </text>\n",
            "    <emph sourcepos=\"1:5-1:9\">\n",
            "      <text sourcepos=\"1:6-1:8\" xml:space=\"preserve\">bar</text>\n",
            "    </emph>\n",
            "  </paragraph>\n",
            "  <paragraph sourcepos=\"3:1-3:11\">\n",
            "    <text sourcepos=\"3:1-3:11\" xml:space=\"preserve\">paragraph 2</text>\n",
            "  </paragraph>\n",
            "  <code_block sourcepos=\"5:1-7:3\" xml:space=\"preserve\">code\n",
            "</code_block>\n",
            "</document>\n",
        ),
        |opts| opts.render.sourcepos = true,
    );
}

#[test]
fn description_item_tight() {
    xml_opts(
        "a\n: b\n\nc\n\n: d\n",
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <description_list>\n",
            "    <description_item tight=\"true\">\n",
            "      <description_term>\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">a</text>\n",
            "        </paragraph>\n",
            "      </description_term>\n",
            "      <description_details>\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">b</text>\n",
            "        </paragraph>\n",
            "      </description_details>\n",
            "    </description_item>\n",
            "    <description_item tight=\"false\">\n",
            "      <description_term>\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">c</text>\n",
            "        </paragraph>\n",
            "      </description_term>\n",
            "      <description_details>\n",
            "        <paragraph>\n",
            "          <text xml:space=\"preserve\">d</text>\n",
            "        </paragraph>\n",
            "      </description_details>\n",
            "    </description_item>\n",
            "  </description_list>\n",
            "</document>\n",
        ),
        |opts| opts.extension.description_lists = true,
    );
}

#[test]
fn escaped_tag() {
    xml_opts(
        "~~a~~ and ~b~~\n",
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n",
            "<document xmlns=\"http://commonmark.org/xml/1.0\">\n",
            "  <paragraph>\n",
            "    <escaped_tag tag=\"~~\">\n",
            "      <text xml:space=\"preserve\">a</text>\n",
            "    </escaped_tag>\n",
            "    <text xml:space=\"preserve\"> and ~b~~</text>\n",
            "  </paragraph>\n",
            "</document>\n",
        ),
        |opts| opts.extension.subscript = true,
    );
}
