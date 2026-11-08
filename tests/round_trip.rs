use sema::{
    Block, Code_Data, Composable, Construct, Delimiter, Depth, Document, Language, Line, Markdown,
    Printable, Problem, Reference_Data, Run, Section, Target, Text, Title,
};

fn composed(text: &str) -> Document {
    Markdown(text.to_string()).compose().expect("composes")
}

fn refused(text: &str) -> (Line, Problem) {
    let error = Markdown(text.to_string()).compose().expect_err("refused");
    (error.line, error.problem)
}

fn round_trips(text: &str) {
    let document = composed(text);
    assert_eq!(
        document.print().0,
        text,
        "printed bytes differ for {text:?}"
    );
    assert_eq!(
        document.print().compose().expect("printed composes"),
        document
    );
}

fn words(text: &str) -> Text {
    Text(vec![Run::Words(text.to_string())])
}

#[test]
fn sections_nest_by_depth() {
    let document = composed("# A\n\nOne.\n\n## B\n\nTwo.\n\n# C\n");
    let expected = Document(vec![
        Block::Section(Section {
            title: Title(words("A")),
            block_vector: vec![
                Block::Statement(words("One.")),
                Block::Section(Section {
                    title: Title(words("B")),
                    block_vector: vec![Block::Statement(words("Two."))],
                }),
            ],
        }),
        Block::Section(Section {
            title: Title(words("C")),
            block_vector: vec![],
        }),
    ]);
    assert_eq!(document, expected);
}

#[test]
fn runs_are_typed() {
    let document = composed("A *b* **c** `d` [e](f) \\* g\n");
    let expected = Document(vec![Block::Statement(Text(vec![
        Run::Words("A ".into()),
        Run::Emphasis(words("b")),
        Run::Words(" ".into()),
        Run::Strong(words("c")),
        Run::Words(" ".into()),
        Run::Code("d".into()),
        Run::Words(" ".into()),
        Run::Reference(Reference_Data {
            text: words("e"),
            target: Target::Url("f".into()),
        }),
        Run::Words(" * g".into()),
    ]))]);
    assert_eq!(document, expected);
}

#[test]
fn fence_holds_its_language() {
    let document = composed("```ethos\nLibrary\n[]\n```\n");
    let expected = Document(vec![Block::Code(Code_Data {
        language: Language::Ethos,
        text: words("Library\n[]"),
    })]);
    assert_eq!(document, expected);
}

#[test]
fn canonical_text_round_trips_byte_for_byte() {
    round_trips(
        "# Title\n\nA line\nwrapped *across* two.\n\n## Sub\n\n- one\n- two `x`\n\n> quoted\n>\n> again\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n```mermaid\nflowchart TB\n```\n\n```\n```\n\nBrackets [stay] plain, snake_case_stays, and \\ too.\n",
    );
}

#[test]
fn front_matter_is_refused() {
    assert_eq!(
        refused("---\ndescription: x\n---\n\nBody.\n"),
        (Line(1), Problem::FrontMatter)
    );
}

#[test]
fn skipped_depth_is_refused() {
    assert_eq!(
        refused("# A\n\n### C\n"),
        (
            Line(3),
            Problem::SkippedDepth {
                above: Depth(1),
                heading: Depth(3)
            }
        )
    );
}

#[test]
fn unknown_language_is_refused() {
    assert_eq!(
        refused("```sh\nls\n```\n"),
        (Line(1), Problem::UnknownLanguage("sh".into()))
    );
}

#[test]
fn unclosed_emphasis_is_refused() {
    assert_eq!(
        refused("Para\n2 * 3\n"),
        (Line(2), Problem::UnclosedRun(Delimiter::Emphasis))
    );
}

#[test]
fn unsupported_forms_are_refused() {
    assert_eq!(
        refused("1. one\n"),
        (Line(1), Problem::Unsupported(Construct::OrderedList))
    );
    assert_eq!(
        refused("Title\n---\n"),
        (Line(2), Problem::Unsupported(Construct::SetextHeading))
    );
    assert_eq!(
        refused("- a\nlazy\n"),
        (Line(2), Problem::Unsupported(Construct::LazyContinuation))
    );
}
