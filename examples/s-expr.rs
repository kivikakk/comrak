extern crate comrak;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};

fn iter_nodes<'a, W: Write>(node: &'a AstNode<'a>, writer: &mut W, indent: usize) -> io::Result<()> {

    use NodeValue::*;
    match &node.data.borrow().value {
        Text(t) => write!(writer, "{:?}", String::from_utf8_lossy(&t))?,
        n => {
            let has_blocks = node.children().any(|c| c.data.borrow().value.block());

            write!(writer, "({:?}", n)?;
            for child in node.children() {
                if has_blocks {
                    write!(writer, "\n{1:0$}", indent + 2, " ")?;
                } else {
                    write!(writer, " ")?;
                }
                iter_nodes(child, writer, indent + 2)?;
            }

            if indent == 0 {
                write!(writer, "\n)\n")?;
            } else if has_blocks {
                write!(writer, "\n{1:0$})", indent, " ")?;
            } else {
                write!(writer, ")")?;
            }
        }
    }

    Ok(())
}

fn dump(source: &str) -> io::Result<()> {

    let arena = Arena::new();

    let opts = ComrakOptions {
        ext_strikethrough: true,
        ext_tagfilter: true,
        ext_table: true,
        ext_autolink: true,
        ext_tasklist: true,
        ext_superscript: true,
        ext_footnotes: true,
        ext_description_lists: true,
        ..ComrakOptions::default()
    };

    let doc = parse_document(&arena, source, &opts);

    let mut output = BufWriter::new(io::stdout());
    iter_nodes(doc, &mut output, 0)
}

fn main() -> Result<(), Box<Error>> {

    let mut args = env::args_os().skip(1).peekable();
    let mut body = String::new();

    if args.peek().is_none() {
        io::stdin().read_to_string(&mut body)?;
        dump(&body)?;
    }

    for filename in args {
        body.clear();
        println!("{:?}", filename);
        let mut file = File::open(&filename)?;
        file.read_to_string(&mut body)?;
        dump(&body)?;
    }

    Ok(())
}
