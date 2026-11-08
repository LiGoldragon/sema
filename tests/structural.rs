use sema::{
    CodeBlock, Composable, Document, Kind, Markdown, Pattern, Place, ProgrammingLanguage,
    Structural, Text, TextPart,
};

fn words(text: &str) -> Text {
    Text(vec![TextPart::Words(text.to_string())])
}

fn place(titles: &[&str]) -> Place {
    Place(titles.iter().map(|title| words(title)).collect())
}

fn module() -> Document {
    Markdown(
        "# The Belief Nexus\n\nIt holds beliefs, see [the spec](https://example.org).\n\n## Signal\n\n```ethos\nSignal\n```\n\n- A [list link](#memory)\n\n## Memory\n\n```rust\nlet memory = 1;\n```\n"
            .to_string(),
    )
    .compose()
    .expect("composes")
}

#[test]
fn code_under_a_section() {
    let expected = vec![CodeBlock {
        programming_language: ProgrammingLanguage::Ethos,
        string: "Signal".to_string(),
    }];
    assert_eq!(
        module().code(place(&["The Belief Nexus", "Signal"])),
        expected
    );
    assert_eq!(module().code(place(&[])).len(), 2);
    assert_eq!(module().code(place(&["Absent"])), vec![]);
}

#[test]
fn references_under_a_section() {
    let all = module().references(place(&["The Belief Nexus"]));
    assert_eq!(all.len(), 2);
    assert_eq!(
        module().references(place(&["The Belief Nexus", "Memory"])),
        vec![]
    );
}

#[test]
fn search_over_one_kind() {
    let pattern = |text: &str| Pattern(text.to_string());
    let places = |hits: Vec<sema::Hit>| hits.into_iter().map(|hit| hit.place).collect::<Vec<_>>();
    assert_eq!(
        places(module().search(Kind::Headings, pattern("Signal"))),
        vec![place(&["The Belief Nexus", "Signal"])]
    );
    assert_eq!(
        places(module().search(Kind::Statements, pattern("beliefs"))),
        vec![place(&["The Belief Nexus"])]
    );
    assert_eq!(
        places(module().search(Kind::CodeBlocks, pattern("memory"))),
        vec![place(&["The Belief Nexus", "Memory"])]
    );
    assert_eq!(
        places(module().search(Kind::References, pattern("#memory"))),
        vec![place(&["The Belief Nexus", "Signal"])]
    );
    assert_eq!(module().search(Kind::Headings, pattern("Absent")), vec![]);
}

#[test]
fn outline_is_one_per_top_level_section() {
    use sema::{Depth, Outline, Title};
    let outline = |title: &str, inner: Vec<Outline>| Outline {
        title: Title(words(title)),
        outline_vector: inner,
    };
    assert_eq!(
        module().outline(Depth(2)),
        vec![outline(
            "The Belief Nexus",
            vec![outline("Signal", vec![]), outline("Memory", vec![])]
        )]
    );
    assert_eq!(
        module().outline(Depth(1)),
        vec![outline("The Belief Nexus", vec![])]
    );
    assert_eq!(module().outline(Depth(0)), vec![]);
    let flat = Markdown("Only a statement.\n".to_string())
        .compose()
        .expect("composes");
    assert_eq!(flat.outline(Depth(3)), vec![]);
    let two = Markdown("# A\n\n# B\n".to_string())
        .compose()
        .expect("composes");
    assert_eq!(
        two.outline(Depth(1)),
        vec![outline("A", vec![]), outline("B", vec![])]
    );
}

fn read(text: &str) -> Document {
    Markdown(text.to_string()).compose().expect("composes")
}

fn section(text: &str) -> sema::Block {
    read(text).0.remove(0)
}

#[test]
fn edits_land_at_the_place() {
    use sema::{Block, Edit, Insert_Data, New, Old, Remove_Data, Replace_Data};
    let mut document = module();
    let inserted = document
        .edit(Edit::Insert(Insert_Data {
            place: place(&["The Belief Nexus", "Memory"]),
            block: Block::Statement(words("Kept.")),
        }))
        .expect("an insert at a section lands");
    assert_eq!(inserted, document);
    assert!(
        sema::Printable::print(&document)
            .0
            .ends_with("let memory = 1;\n```\n\nKept.\n")
    );

    let mut document = read("# A\n\nOne.\n\n# B\n\nTwo.\n");
    let replaced = document
        .edit(Edit::Replace(Replace_Data {
            place: place(&["B"]),
            old: Old(section("# B\n\nTwo.\n")),
            new: New(section("# C\n\nThree.\n")),
        }))
        .expect("a replace of the expected block lands");
    assert_eq!(replaced, read("# A\n\nOne.\n\n# C\n\nThree.\n"));

    let removed = document
        .edit(Edit::Remove(Remove_Data {
            place: place(&["A"]),
            block: section("# A\n\nOne.\n"),
        }))
        .expect("a remove of the expected block lands");
    assert_eq!(removed, read("# C\n\nThree.\n"));

    let mut root = read("One.\n");
    root.edit(Edit::Insert(Insert_Data {
        place: place(&[]),
        block: Block::Statement(words("Two.")),
    }))
    .expect("an insert at the root lands");
    assert_eq!(root, read("One.\n\nTwo.\n"));
}

#[test]
fn an_edit_at_a_place_with_no_block_is_no_such_place() {
    use sema::{Block, Edit, EditRefusal, Insert_Data, Remove_Data};
    let mut document = module();
    let before = document.clone();
    for (edit, at) in [
        (
            Edit::Insert(Insert_Data {
                place: place(&["The Belief Nexus", "Absent"]),
                block: Block::Statement(words("x")),
            }),
            place(&["The Belief Nexus", "Absent"]),
        ),
        (
            Edit::Remove(Remove_Data {
                place: place(&[]),
                block: Block::Statement(words("x")),
            }),
            place(&[]),
        ),
    ] {
        assert_eq!(document.edit(edit), Err(EditRefusal::NoSuchPlace(at)));
    }
    assert_eq!(document, before);
}

#[test]
fn an_edit_expecting_another_block_is_a_mismatch_naming_the_found_one() {
    use sema::{Edit, EditRefusal, Expected, Found, Mismatch_Data, New, Old, Replace_Data};
    let mut document = read("# A\n\nOne.\n");
    let before = document.clone();
    assert_eq!(
        document.edit(Edit::Replace(Replace_Data {
            place: place(&["A"]),
            old: Old(section("# A\n\nOther.\n")),
            new: New(section("# A\n\nNew.\n")),
        })),
        Err(EditRefusal::Mismatch(Mismatch_Data {
            place: place(&["A"]),
            expected: Expected(section("# A\n\nOther.\n")),
            found: Found(section("# A\n\nOne.\n")),
        }))
    );
    assert_eq!(document, before);
}

/// A search hit carries its place, typed as the Document holds its titles,
/// and the block it matched in; a styled heading matches by its words.
#[test]
fn a_hit_carries_the_typed_place_and_its_block() {
    use sema::{Block, Edit, Insert_Data, Printable as _};
    let mut document = Markdown("# The *Belief* `Nexus`\n\nRun `nix build` now.\n".to_string())
        .compose()
        .expect("composes");
    let title = Text(vec![
        TextPart::Words("The ".into()),
        TextPart::Emphasis(words("Belief")),
        TextPart::Words(" ".into()),
        TextPart::InlineCode("Nexus".into()),
    ]);
    let hits = document.search(Kind::Headings, Pattern("Belief Nexus".to_string()));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].place, Place(vec![title.clone()]));
    assert!(matches!(&hits[0].block, Block::Section(section) if section.title.0 == title));
    let code = document.search(Kind::InlineCode, Pattern("nix".to_string()));
    assert_eq!(code.len(), 1);
    assert_eq!(code[0].place, Place(vec![title.clone()]));
    assert!(matches!(&code[0].block, Block::Statement(_)));
    assert_eq!(
        document.search(Kind::InlineCode, Pattern("Run".to_string())),
        vec![]
    );
    let edited = document
        .edit(Edit::Insert(Insert_Data {
            place: hits[0].place.clone(),
            block: Block::Statement(words("Added.")),
        }))
        .expect("the hit's place is found");
    assert_eq!(
        edited.print().0,
        "# The *Belief* `Nexus`\n\nRun `nix build` now.\n\nAdded.\n"
    );
}
