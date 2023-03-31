use comrak::{
    adapters::{HeadingAdapter, HeadingMeta},
    markdown_to_html_with_plugins,
    nodes::Sourcepos,
    ComrakOptions, ComrakPlugins,
};
use std::io::{self, Write};

fn main() {
    let adapter = CustomHeadingAdapter;
    let mut options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    plugins.render.heading_adapter = Some(&adapter);

    print_html(
        "Some text.\n\n## Please hide me from search\n\nSome other text",
        &options,
        &plugins,
    );
    print_html(
        "Some text.\n\n### Here is some `code`\n\nSome other text",
        &options,
        &plugins,
    );
    print_html(
        "Some text.\n\n### Here is some **bold** text and some *italicized* text\n\nSome other text",
        &options,
        &plugins
    );
    options.render.sourcepos = true;
    print_html("# Here is a [link](/)", &options, &plugins);
}

struct CustomHeadingAdapter;

impl HeadingAdapter for CustomHeadingAdapter {
    fn enter(
        &self,
        output: &mut dyn Write,
        heading: &HeadingMeta,
        sourcepos: Option<Sourcepos>,
    ) -> io::Result<()> {
        let id = slug::slugify(&heading.content);

        let search_include = !&heading.content.contains("hide");

        write!(output, "<h{}", heading.level)?;

        if let Some(sourcepos) = sourcepos {
            write!(output, " data-sourcepos=\"{}\"", sourcepos)?;
        }

        write!(
            output,
            " id=\"{}\" data-search-include=\"{}\">",
            id, search_include
        )
    }

    fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> io::Result<()> {
        write!(output, "</h{}>", heading.level)
    }
}

fn print_html(document: &str, options: &ComrakOptions, plugins: &ComrakPlugins) {
    let html = markdown_to_html_with_plugins(document, options, plugins);
    println!("{}", html);
}
