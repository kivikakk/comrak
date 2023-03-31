use super::*;

#[test]
fn markdown_list_bullets() {
    let dash = concat!("- a\n");
    let plus = concat!("+ a\n");
    let star = concat!("* a\n");
    let mut dash_opts = ComrakOptions::default();
    dash_opts.render.list_style = ListStyleType::Dash;
    let mut plus_opts = ComrakOptions::default();
    plus_opts.render.list_style = ListStyleType::Plus;
    let mut star_opts = ComrakOptions::default();
    star_opts.render.list_style = ListStyleType::Star;

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
    let mut options = ComrakOptions::default();
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
