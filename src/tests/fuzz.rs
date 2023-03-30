use super::{html, html_opts};

/// To be used when moving from the all_options fuzz target to here.
/// Reduce to the actual necessary options to reproduce.
#[allow(dead_code)]
#[cfg(feature = "shortcodes")]
fn all_options() -> crate::ComrakOptions {
    crate::ComrakOptions {
        extension: crate::ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: Some("user-content-".to_string()),
            footnotes: true,
            description_lists: true,
            front_matter_delimiter: Some("---".to_string()),
            shortcodes: true,
        },
        parse: crate::ComrakParseOptions {
            smart: true,
            default_info_string: Some("rust".to_string()),
            relaxed_tasklist_matching: true,
        },
        render: crate::ComrakRenderOptions {
            hardbreaks: true,
            github_pre_lang: true,
            full_info_string: true,
            width: 80,
            unsafe_: true,
            escape: true,
            list_style: crate::ListStyleType::Star,
            sourcepos: true,
        },
    }
}

#[test]
fn pointy_brace_open() {
    html("<!-", "<p>&lt;!-</p>\n");
}

#[test]
fn tasklist() {
    html_opts!(
        [extension.tasklist, parse.relaxed_tasklist_matching],
        "* [*]",
        "<ul>\n<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> </li>\n</ul>\n",
    );
}

#[test]
fn table_nul() {
    html_opts!(
        [extension.table],
        "\0|.\n-|-\nZ",
        r##"<table>
<thead>
<tr>
<th>�</th>
<th>.</th>
</tr>
</thead>
<tbody>
<tr>
<td>Z</td>
<td></td>
</tr>
</tbody>
</table>
"##,
    );
}

#[test]
fn footnote_def() {
    html_opts!(
        [
            extension.autolink,
            extension.footnotes,
            render.sourcepos,
            render.hardbreaks
        ],
        "\u{15}\u{b}\r[^ ]:",
        "<p data-sourcepos=\"1:1-2:5\">\u{15}\u{b}<br data-sourcepos=\"1:3-1:3\" />\n[^ ]:</p>\n",
    );
}

#[test]
fn line_end() {
    html("\u{2}\n\\\n\t-", "<p>\u{2}\n<br />\n-</p>\n");
}

#[test]
fn bracket_match() {
    html("[;\0V\n]::g\n[;\0V\n]", "<p><a href=\":g\">;�V\n</a></p>\n");
}
