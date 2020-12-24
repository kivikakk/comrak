#[cfg(feature = "cbindgen")]
fn build_header() {
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use std::io::Write;


    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var is not defined");
    let target_dir = if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    };

    let output_file = target_dir
        .join(format!("{}.h", "comrak_ffi"))
        .display()
        .to_string();
    let config = cbindgen::Config::from_file("cbindgen.toml").expect("Unable to find cbindgen.toml configuration file");

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);

    // cbindgen doesn't understand replacing generics, so rewrite
    //  this syntax into something C understands
    let mut contents = fs::read_to_string(&output_file).unwrap();
    contents = contents.replace("typedef Arena<AstNode>", "typedef struct Arena_AstNode Arena_AstNode;\ntypedef Arena_AstNode");
    let mut file = File::create(output_file).unwrap();
    file.write_all(&contents.as_bytes()).unwrap();
    // write!(&output_file, "{}", contents).unwrap();
}

fn main() {
    #[cfg(feature = "cbindgen")]
    build_header()
}
