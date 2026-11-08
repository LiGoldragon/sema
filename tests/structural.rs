use sema::{
    Code, Composable, Document, Kind, Markdown, Pattern, Place, ProgrammingLanguage, Run,
    Structural, Text,
};

fn words(text: &str) -> Text {
    Text(vec![Run::Words(text.to_string())])
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
    let expected = vec![Code {
        programming_language: ProgrammingLanguage::Ethos,
        text: words("Signal"),
    }];
    assert_eq!(
        module().code(&place(&["The Belief Nexus", "Signal"])),
        expected
    );
    assert_eq!(module().code(&place(&[])).len(), 2);
    assert_eq!(module().code(&place(&["Absent"])), vec![]);
}

#[test]
fn references_under_a_section() {
    let all = module().references(&place(&["The Belief Nexus"]));
    assert_eq!(all.len(), 2);
    assert_eq!(
        module().references(&place(&["The Belief Nexus", "Memory"])),
        vec![]
    );
}

#[test]
fn search_over_one_kind() {
    let pattern = |text: &str| Pattern(text.to_string());
    assert_eq!(
        module().search(Kind::Headings, &pattern("Signal")),
        vec![place(&["The Belief Nexus", "Signal"])]
    );
    assert_eq!(
        module().search(Kind::Statements, &pattern("beliefs")),
        vec![place(&["The Belief Nexus"])]
    );
    assert_eq!(
        module().search(Kind::Code, &pattern("memory")),
        vec![place(&["The Belief Nexus", "Memory"])]
    );
    assert_eq!(
        module().search(Kind::References, &pattern("#memory")),
        vec![place(&["The Belief Nexus", "Signal"])]
    );
    assert_eq!(module().search(Kind::Headings, &pattern("Absent")), vec![]);
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
