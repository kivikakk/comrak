extern crate comrak;

use std::io::{self, Write};

use comrak::{
    adapters::{ImageAdapter, ImageMeta},
    html::{escape, escape_href},
    markdown_to_html_with_plugins,
    nodes::Sourcepos,
    ComrakOptions, ComrakPlugins,
};

struct CustomImages;

impl ImageAdapter for CustomImages {
    fn render(
        &self,
        output: &mut dyn Write,
        img_meta: ImageMeta,
        sourcepos: Option<Sourcepos>,
    ) -> io::Result<()> {
        output.write_all(b"<figure")?;
        if let Some(sourcepos) = sourcepos {
            write!(output, " data-sourcepos=\"{}\"", sourcepos)?;
        }
        output.write_all(b"><a href=\"")?;
        escape_href(output, img_meta.url.as_bytes())?;
        output.write_all(b"\" target=\"_blank\"><img src=\"")?;
        escape_href(output, img_meta.url.as_bytes())?;
        output.write_all(b"\"></a>")?;
        if !img_meta.title.is_empty() {
            output.write_all(b"<figcaption>")?;
            escape(output, img_meta.title.as_bytes())?;
            output.write_all(b"</figcaption>")?;
        };
        output.write_all(b"</figure>")?;
        Ok(())
    }
}

fn main() {
    let adapter = CustomImages;

    let mut options = ComrakOptions::default();
    options.render.sourcepos = true;
    let mut plugins = ComrakPlugins::default();
    plugins.render.image_adapter = Some(&adapter);

    let input = "![Here is a caption](/img/logo.png)";

    let formatted = markdown_to_html_with_plugins(input, &options, &plugins);

    println!("{}", formatted);
}
