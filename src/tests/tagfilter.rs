use super::*;

#[test]
fn tagfilter() {
    html_opts!(
        [render.unsafe_, extension.tagfilter],
        concat!("hi <xmp> ok\n", "\n", "<xmp>\n"),
        concat!("<p>hi &lt;xmp> ok</p>\n", "&lt;xmp>\n"),
    );
}
