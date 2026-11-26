use pretty_assertions::assert_eq;

use super::*;

#[test]
fn hardbreaks_keeps_consecutive_breaks() {
    html_opts!(
        [render.hardbreaks],
        "a\\\n\\\nb\n",
        "<p>a<br />\n<br />\nb</p>\n",
    );
}

#[test]
fn hardbreaks_writes_every_break_with_a_backslash() {
    let mut options = Options::default();
    options.render.hardbreaks = true;
    commonmark(
        "a\nb  \nc\\\n\\\nd\n\n> e\nf\n",
        "a\\\nb\\\nc\\\n\\\nd\n\n> e\\\n> f\n",
        Some(&options),
    );
}

#[test]
fn hardbreaks_commonmark_means_the_same_without_the_option() {
    let mut options = Options::default();
    options.render.hardbreaks = true;
    let input = "> a\nb\\\n> \\\n> c\n\n- d\n  e\n\n[f\ng](/u)\n";

    let arena = Arena::new();
    let root = parse_document(&arena, input, &options);
    let mut output = String::new();
    cm::format_document(root, &options, &mut output).unwrap();

    let expected = markdown_to_html(input, &options);
    assert_eq!(markdown_to_html(&output, &options), expected);
    assert_eq!(markdown_to_html(&output, &Options::default()), expected);
}

#[test]
fn hardbreaks_does_not_wrap() {
    let mut options = Options::default();
    options.render.hardbreaks = true;
    options.render.width = 5;
    commonmark("aaa bbb ccc\nd\n", "aaa bbb ccc\\\nd\n", Some(&options));

    options.render.experimental_minimize_commonmark = true;
    commonmark("aaa bbb ccc\nd\n", "aaa bbb ccc\\\nd\n", Some(&options));
}

#[test]
fn hardbreaks_minimize_keeps_break_backslashes() {
    let mut options = Options::default();
    options.render.hardbreaks = true;
    options.render.experimental_minimize_commonmark = true;
    commonmark("a\nb\\\n\\\nc\\.\n", "a\\\nb\\\n\\\nc.\n", Some(&options));
}
