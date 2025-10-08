use super::*;

// A series of tests ensuring the default HTML formatter doesn't panic on
// unusual but still believable ASTs.  We also assert expected panics give at
// least helpful messages.

#[test]
fn test_paragraph_at_root_crash() {
    let options = Options {
        ..Default::default()
    };
    let mut arena = Arena::new();

    let doc = parse_document(&mut arena, "para", &options);

    let para = doc.first_child(&arena).unwrap();

    // This is the bit we don't expect: what if the paragraph doesn't have a
    // parent at all?  Normally it should have at least a Document.
    doc.remove_self(&mut arena);

    let mut output = String::new();
    html::format_document(&arena, para, &options, &mut output).unwrap();
}

#[test]
fn test_empty_table_crash() {
    let options = Options {
        extension: ExtensionOptions {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut arena = Arena::new();

    let table = parse_document(&mut arena, "| x |\n| - |\n| z |", &options)
        .first_child(&arena)
        .unwrap();

    // What if the table has been emptied of *all* children?
    table
        .first_child(&arena)
        .unwrap()
        .remove_subtree(&mut arena);

    let mut output = String::new();
    html::format_document(&arena, table, &options, &mut output).unwrap();
}

#[test]
#[should_panic(expected = "rendered a table cell without a containing table")]
fn test_table_cell_out_of_water_crash() {
    let options = Options {
        extension: ExtensionOptions {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut arena = Arena::new();

    let doc = parse_document(&mut arena, "| x |\n| - |\n| z |", &options);

    let table = doc.first_child(&arena).unwrap();
    let table_row = table.last_child(&arena).unwrap();
    let table_cell = table_row.first_child(&arena).unwrap();

    // What if the table cell has no owning table?
    doc.remove_self(&mut arena);
    table.remove_self(&mut arena);

    let mut output = String::new();
    html::format_document(&arena, table_cell, &options, &mut output).unwrap();
}

#[test]
#[should_panic(expected = "rendered a table cell without a containing table row")]
fn test_table_cell_out_of_school_crash() {
    let options = Options {
        extension: ExtensionOptions {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut arena = Arena::new();

    let doc = parse_document(&mut arena, "| x |\n| - |\n| z |", &options);

    let table = doc.first_child(&arena).unwrap();
    let table_row = table.last_child(&arena).unwrap();
    let table_cell = table_row.first_child(&arena).unwrap();

    // What if the table cell has no owning table row?
    doc.remove_self(&mut arena);
    table.remove_self(&mut arena);
    table_row.remove_self(&mut arena);

    let mut output = String::new();
    html::format_document(&arena, table_cell, &options, &mut output).unwrap();
}
