#[test]
fn it_works() {
    let arena = ::typed_arena::Arena::new();
    let n = ::parse_document(
        &arena,
        b"My **document**.\n\nIt's mine.\n\n> Yes.\n\n## Hi!\n\nOkay.",
        0);
    let m = ::format_document(n);
    assert_eq!(
        m,
        concat!(
            "<p>My <strong>document</strong>.</p>\n",
            "<p>It's mine.</p>\n",
            "<blockquote>\n",
            "<p>Yes.</p>\n",
            "</blockquote>\n",
            "<h2>Hi!</h2>\n",
            "<p>Okay.</p>\n"));
}
