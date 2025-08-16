use super::*;

// A series of tests ensuring the default HTML formatter doesn't panic on
// unusual but still believable ASTs.  We also assert expected panics give at
// least helpful messages.

#[test]
fn test_paragraph_at_root_crash() {
    let options = Options {
        ..Default::default()
    };
    let arena = Arena::new();

    let para = parse_document(&arena, "para", &options)
        .first_child()
        .unwrap();

    // This is the bit we don't expect: what if the paragraph doesn't have a
    // parent at all?  Normally it should have at least a Document.
    para.detach();

    let mut output = vec![];
    html::format_document(para, &options, &mut output).unwrap();
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
    let arena = Arena::new();

    let table = parse_document(&arena, "| x |\n| - |\n| z |", &options)
        .first_child()
        .unwrap();

    // What if the table has been emptied of *all* children?
    while let Some(child) = table.first_child() {
        child.detach();
    }

    let mut output = vec![];
    html::format_document(table, &options, &mut output).unwrap();
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
    let arena = Arena::new();

    let doc = parse_document(&arena, "| x |\n| - |\n| z |", &options);

    let table_row = doc
        .first_child() // table
        .unwrap()
        .last_child() // table row
        .unwrap();

    let table_cell = table_row
        .first_child() // table cell
        .unwrap();

    // What if the table cell has no owning table?
    table_row.detach();

    let mut output = vec![];
    html::format_document(table_cell, &options, &mut output).unwrap();
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
    let arena = Arena::new();

    let doc = parse_document(&arena, "| x |\n| - |\n| z |", &options);

    let table_row = doc
        .first_child() // table
        .unwrap()
        .last_child() // table row
        .unwrap();

    let table_cell = table_row
        .first_child() // table cell
        .unwrap();

    // What if the table cell has no owning table row?
    table_cell.detach();

    let mut output = vec![];
    html::format_document(table_cell, &options, &mut output).unwrap();
}
