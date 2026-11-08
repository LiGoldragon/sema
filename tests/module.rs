use sema::module::{Composable as _, Printable as _};
use sema::{
    Block, Composable, Dependencies, Description, Document, FrontField, FrontMatter, Line,
    Markdown, Module, ModuleName, ModuleText, Placement, Problem, Run, Text,
};

fn module(text: &str) -> Result<Module, (Line, Problem)> {
    ModuleText(text.to_string())
        .compose()
        .map_err(|refusal| (refusal.line, refusal.problem))
}

fn unsupported(line: i64, written: &str) -> Result<Module, (Line, Problem)> {
    Err((Line(line), Problem::Unsupported(written.to_string())))
}

const CANONICAL: &str = "---\ndescription: What it is: whole.\ndependencies: [vision-sema, intent-design]\nplacement: system\n---\n\nA statement.\n";

#[test]
fn a_module_reads_front_then_document() {
    let expected = Module {
        front_matter: FrontMatter {
            description: Description("What it is: whole.".into()),
            dependencies: Dependencies(vec![
                ModuleName("vision-sema".into()),
                ModuleName("intent-design".into()),
            ]),
            placement_option: Some(Placement::SystemPrompt),
        },
        document: Document(vec![Block::Statement(Text(vec![Run::Words(
            "A statement.".into(),
        )]))]),
    };
    assert_eq!(module(CANONICAL), Ok(expected));
}

#[test]
fn a_canonical_module_round_trips_byte_for_byte() {
    for text in [
        CANONICAL,
        "---\ndescription: d\ndependencies: []\n---\n\nBody.\n",
        "---\ndescription: d\ndependencies: [a]\nplacement: skill\n---\n",
    ] {
        let read = module(text).expect("reads");
        assert_eq!(read.print().0, text);
        assert_eq!(module(&read.print().0), Ok(read));
    }
}

#[test]
fn fields_print_in_type_order() {
    let read = module("---\nplacement: skill\ndependencies: []\ndescription: d\n---\n\nBody.\n")
        .expect("reads");
    assert_eq!(
        read.print().0,
        "---\ndescription: d\ndependencies: []\nplacement: skill\n---\n\nBody.\n"
    );
}

#[test]
fn front_refusals_carry_the_line() {
    assert_eq!(
        module("---\ndescription: d\ndependencies: []\ntitle: t\n---\n"),
        unsupported(4, "title: t")
    );
    assert_eq!(
        module("---\ndescription: d\ndependencies: []\nplacement: elsewhere\n---\n"),
        unsupported(4, "placement: elsewhere")
    );
    assert_eq!(
        module("---\ndescription: \"open\ndependencies: []\n---\n"),
        unsupported(2, "description: \"open")
    );
    assert_eq!(
        module("---\ndescription: d\n---\n"),
        Err((Line(3), Problem::Missing(FrontField::Dependencies)))
    );
    assert_eq!(
        module("---\ndependencies: []\n---\n"),
        Err((Line(3), Problem::Missing(FrontField::Description)))
    );
    assert_eq!(
        module("---\ndescription:\ndependencies: []\n---\n"),
        Err((Line(2), Problem::Missing(FrontField::Description)))
    );
    assert_eq!(
        module("---\ndescription: \"\"\ndependencies: []\n---\n"),
        Err((Line(2), Problem::Missing(FrontField::Description)))
    );
    assert_eq!(
        module("---\ndescription: d\ndependencies: []\n"),
        Err((Line(1), Problem::UnclosedFence))
    );
    assert_eq!(module("Body.\n"), unsupported(1, "Body."));
}

#[test]
fn body_refusals_count_lines_from_the_file_start() {
    assert_eq!(
        module("---\ndescription: d\ndependencies: []\n---\n\n# A\n\n### C\n"),
        unsupported(8, "### C")
    );
}

#[test]
fn a_document_alone_still_refuses_front_matter() {
    assert!(Markdown(CANONICAL.to_string()).compose().is_err());
}

#[test]
fn a_quoted_description_prints_plain_when_plain_reads_back_the_same() {
    let read = module("---\ndescription: \"What it is\"\ndependencies: []\n---\n").expect("reads");
    assert_eq!(
        read.front_matter.description,
        Description("What it is".into())
    );
    assert_eq!(
        read.print().0,
        "---\ndescription: What it is\ndependencies: []\n---\n"
    );
    let single = module("---\ndescription: 'it''s'\ndependencies: []\n---\n").expect("reads");
    assert_eq!(single.front_matter.description, Description("it's".into()));
}

#[test]
fn a_description_that_plain_cannot_carry_prints_quoted() {
    for (text, description) in [
        (
            "---\ndescription: \"#hash\"\ndependencies: []\n---\n",
            "#hash",
        ),
        (
            "---\ndescription: \"say \\\"x\\\" \"\ndependencies: []\n---\n",
            "say \"x\" ",
        ),
    ] {
        let read = module(text).expect("reads");
        assert_eq!(
            read.front_matter.description,
            Description(description.into())
        );
        assert_eq!(read.print().0, text);
        assert_eq!(module(&read.print().0), Ok(read));
    }
}
