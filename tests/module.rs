use sema::{
    Block, Composable, Dependencies, Description, Document, Front, Line, Markdown, Module,
    ModuleComposable, ModuleName, Placement, Printable, Problem, Run, Text,
};

fn module(text: &str) -> Result<Module, (Line, Problem)> {
    Markdown(text.to_string())
        .compose_module()
        .map_err(|refusal| (refusal.line, refusal.problem))
}

fn unsupported(line: i64, written: &str) -> Result<Module, (Line, Problem)> {
    Err((Line(line), Problem::Unsupported(written.to_string())))
}

const CANONICAL: &str = "---\ndescription: What it is: whole.\ndependencies: [vision-sema, intent-design]\nplacement: system\n---\n\nA statement.\n";

#[test]
fn a_module_reads_front_then_document() {
    let expected = Module {
        front: Front {
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
        module("---\ndescription: \"quoted\"\ndependencies: []\n---\n"),
        unsupported(2, "description: \"quoted\"")
    );
    assert_eq!(module("---\ndescription: d\n---\n"), unsupported(3, "---"));
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
