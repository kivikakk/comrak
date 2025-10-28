use pretty_assertions::assert_eq;
use std::sync::Arc;

use super::*;

#[test]
fn markdown_list_bullets() {
    let dash = concat!("- a\n");
    let plus = concat!("+ a\n");
    let star = concat!("* a\n");
    let mut dash_opts = Options::default();
    dash_opts.render.list_style = options::ListStyleType::Dash;
    let mut plus_opts = Options::default();
    plus_opts.render.list_style = options::ListStyleType::Plus;
    let mut star_opts = Options::default();
    star_opts.render.list_style = options::ListStyleType::Star;

    commonmark(dash, dash, Some(&dash_opts));
    commonmark(plus, dash, Some(&dash_opts));
    commonmark(star, dash, Some(&dash_opts));

    commonmark(dash, plus, Some(&plus_opts));
    commonmark(plus, plus, Some(&plus_opts));
    commonmark(star, plus, Some(&plus_opts));

    commonmark(dash, star, Some(&star_opts));
    commonmark(plus, star, Some(&star_opts));
    commonmark(star, star, Some(&star_opts));
}

#[test]
fn width_breaks() {
    let mut options = Options::default();
    options.render.width = 72;
    let input = concat!(
        "this should break because it has breakable characters. break right here newline\n",
        "\n",
        "don't break\n",
        "\n",
        "a-long-line-that-won't-break-because-there-is-no-character-it-can-break-on\n"
    );
    let output = concat!(
        "this should break because it has breakable characters. break right here\n",
        "newline\n",
        "\n",
        "don't break\n",
        "\n",
        "a-long-line-that-won't-break-because-there-is-no-character-it-can-break-on\n"
    );

    commonmark(input, output, Some(&options));
}

#[test]
fn smart_chars() {
    html_opts!(
        [parse.smart],
        "Why 'hello' \"there\". It's good.",
        "<p>Why ‘hello’ “there”. It’s good.</p>\n",
    );

    html_opts!(
        [parse.smart],
        "Hm. Hm.. hm... yes- indeed-- quite---!",
        "<p>Hm. Hm.. hm… yes- indeed– quite—!</p>\n",
    );
}

#[test]
fn broken_link_callback() {
    let cb = |link_ref: options::BrokenLinkReference| match link_ref.normalized {
        "foo" => Some(ResolvedReference {
            url: "https://www.rust-lang.org/".to_string(),
            title: "The Rust Language".to_string(),
        }),
        _ => None,
    };
    let mut options = Options::default();
    options.parse.broken_link_callback = Some(Arc::new(cb));

    let output = markdown_to_html(
        "# Cool input!\nWow look at this cool [link][foo]. A [broken link] renders as text.",
        &options,
    );
    assert_eq!(
        output,
        "<h1>Cool input!</h1>\n<p>Wow look at this cool \
        <a href=\"https://www.rust-lang.org/\" title=\"The Rust Language\">link</a>. \
        A [broken link] renders as text.</p>\n"
    );
}
