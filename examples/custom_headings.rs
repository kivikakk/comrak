extern crate comrak;
extern crate slug;

use comrak::{
    adapters::{HeadingAdapter, HeadingMeta},
    markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins,
};

fn main() {
    println!(
        "{}",
        render_heading("Some text.\n\n## Please hide me from search\n\nSome other text")
    );
    println!(
        "{}",
        render_heading("Some text.\n\n### Here is some `code`\n\nSome other text")
    );
    println!(
        "{}",
        render_heading("Some text.\n\n### Here is some **bold** text and some *italicized* text\n\nSome other text")
    );
    println!("{}", render_heading("# Here is a [link](/)"));
}

struct CustomHeadingAdapter;

impl HeadingAdapter for CustomHeadingAdapter {
    fn render(&self, heading: &HeadingMeta) -> String {
        let id = slug::slugify(&heading.content);

        let search_include = !&heading.content.contains("hide");

        format!(
            "<h{} id=\"{}\" data-search-include=\"{}\">",
            heading.level, id, search_include
        )
    }
}

fn render_heading(document: &str) -> String {
    let adapter = CustomHeadingAdapter {};
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();

    plugins.render.heading_adapter = Some(&adapter);
    markdown_to_html_with_plugins(document, &options, &plugins)
}
