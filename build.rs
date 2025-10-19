use entities::ENTITIES;
use std::io::Write;
use std::{env, path::PathBuf};

fn main() {
    let out_dir: PathBuf = env::var("OUT_DIR").unwrap().parse().unwrap();

    // entity::lookup is handed just the inner entity name, like "amp" for
    // "&amp;"; we only match those with a trailing ";".
    //
    // entities::ENTITIES includes many both with and without a trailing ";".
    // Exclude those without, and then write to source only the name, without
    // the leading or trailing "&" or ";".
    //
    // It's also not sorted; upper- and lower-case variants are interleaved.
    // Sort it for binary search.
    let mut translated_entities = ENTITIES
        .iter()
        .filter(|e| e.entity.starts_with("&") && e.entity.ends_with(";"))
        .collect::<Vec<_>>();
    translated_entities.sort_by_key(|e| e.entity);

    let out = std::fs::File::create(out_dir.join("entitydata.rs")).unwrap();
    let mut bw = std::io::BufWriter::new(out);
    write!(bw, "mod entitydata {{\n").unwrap();
    write!(
        bw,
        "    pub static TRANSLATED_ENTITIES: &[(&'static str, &'static str); {}] = &[\n",
        translated_entities.len()
    )
    .unwrap();
    for e in translated_entities {
        write!(
            bw,
            "        ({:?}, {:?}),\n",
            &e.entity[1..e.entity.len() - 1],
            &e.characters
        )
        .unwrap();
    }
    write!(bw, "    ];\n").unwrap();
    write!(bw, "}}\n").unwrap();
}
