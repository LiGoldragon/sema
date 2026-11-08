use sema::{
    Block, CodeBlock, Composable, Document, Line, Markdown, Printable, Problem,
    ProgrammingLanguage, Reference, Section, Text, TextPart, Title, Url,
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
    Text(vec![TextPart::Words(text.to_string())])
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
fn the_shallowest_heading_level_is_the_top() {
    // A shallower heading after the first makes the first a skipped level.
    assert_eq!(refused("## A\n\n# B\n"), unsupported(1, "## A"));
}

#[test]
fn a_module_opened_lower_prints_one_level_up() {
    let opened_lower = composed("## A\n\nOne.\n\n### B\n\n## C\n");
    assert_eq!(opened_lower, composed("# A\n\nOne.\n\n## B\n\n# C\n"));
    assert_eq!(opened_lower.print().0, "# A\n\nOne.\n\n## B\n\n# C\n");
}

#[test]
fn every_fence_spelling_round_trips() {
    round_trips(
        "```ethos\n```\n\n```datom\n```\n\n```rust\n```\n\n```nix\n```\n\n```sh\n```\n\n```ini\n```\n\n```toml\n```\n\n```mermaid\n```\n\n```\n```\n",
    );
}

#[test]
fn runs_are_typed() {
    let document = composed("A *b* **c** `d` [e](f) \\* g\n");
    let expected = Document(vec![Block::Statement(Text(vec![
        TextPart::Words("A ".into()),
        TextPart::Emphasis(words("b")),
        TextPart::Words(" ".into()),
        TextPart::Strong(words("c")),
        TextPart::Words(" ".into()),
        TextPart::InlineCode("d".into()),
        TextPart::Words(" ".into()),
        TextPart::Reference(Reference {
            text: words("e"),
            url: Url("f".into()),
        }),
        TextPart::Words(" * g".into()),
    ]))]);
    assert_eq!(document, expected);
}

#[test]
fn fence_holds_its_language() {
    let document = composed("```ethos\nLibrary\n[]\n```\n");
    let expected = Document(vec![Block::CodeBlock(CodeBlock {
        programming_language: ProgrammingLanguage::Ethos,
        string: "Library\n[]".to_string(),
    })]);
    assert_eq!(document, expected);
}

#[test]
fn canonical_text_round_trips_byte_for_byte() {
    round_trips(
        "# Title\n\nA line\nwrapped *across* two.\n\n## Sub\n\n- one\n- two `x`\n\n> quoted\n> \n> again\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n```mermaid\nflowchart TB\n```\n\n```\n```\n\nBrackets [stay] plain, snake_case_stays, and \\ too.\n",
    );
}

fn unsupported(line: i64, written: &str) -> (Line, Problem) {
    (Line(line), Problem::Unsupported(written.to_string()))
}

#[test]
fn front_matter_is_refused_as_a_document() {
    assert_eq!(
        refused("---\ndescription: x\n---\n\nBody.\n"),
        unsupported(1, "---")
    );
}

#[test]
fn refusals_carry_the_line_as_written() {
    assert_eq!(refused("# A\n\n### C\n"), unsupported(3, "### C"));
    assert_eq!(refused("```bash\nls\n```\n"), unsupported(1, "```bash"));
    assert_eq!(refused("1. one\n"), unsupported(1, "1. one"));
    assert_eq!(refused("Para.\n\n***\n"), unsupported(3, "***"));
    assert_eq!(refused("<div>\nx\n</div>\n"), unsupported(1, "<div>"));
    assert_eq!(
        refused("See ![a](b.png).\n"),
        unsupported(1, "See ![a](b.png).")
    );
    assert_eq!(refused("Hard  \nbreak\n"), unsupported(1, "Hard  "));
    assert_eq!(refused("[a](b \"t\")\n"), unsupported(1, "[a](b \"t\")"));
    assert_eq!(refused("- a\n\n  b\n"), unsupported(1, "- a"));
    assert_eq!(refused("> # A\n"), unsupported(1, "> # A"));
    assert_eq!(refused("| a |\n| :-- |\n| 1 |\n"), unsupported(1, "| a |"));
}

/// What the Document has a place for is read, whichever way it is written,
/// and printed in the one canonical form.
#[test]
fn every_spelling_of_a_construct_is_read() {
    let read = |text: &str| composed(text).print().0;
    assert_eq!(read("Title\n=====\n\nBody.\n"), "# Title\n\nBody.\n");
    assert_eq!(read("* a\n* b\n"), "- a\n- b\n");
    assert_eq!(read("~~~rust\nlet x;\n~~~\n"), "```rust\nlet x;\n```\n");
    assert_eq!(read("- a\nlazy\n"), "- a\n  lazy\n");
    assert_eq!(read("Para\n2 * 3\n"), "Para\n2 * 3\n");
}

#[test]
fn inline_html_is_unsupported() {
    assert_eq!(
        refused("Para.\n\nThe module vision:<topic> holds it.\n"),
        unsupported(3, "The module vision:<topic> holds it.")
    );
}

#[test]
fn an_unclosed_fence_is_code_to_the_end() {
    let document = composed("Para.\n\n```rust\nfn x\n");
    assert_eq!(document, composed("Para.\n\n```rust\nfn x\n```\n"));
}

/// Every link target, an anchor among them, reads as Url and prints back as written.
#[test]
fn an_anchor_reads_as_url_and_prints_back() {
    let source = "See [the signal](#signal).\n";
    let document = composed(source);
    let expected = Document(vec![Block::Statement(Text(vec![
        TextPart::Words("See ".into()),
        TextPart::Reference(Reference {
            text: words("the signal"),
            url: Url("#signal".into()),
        }),
        TextPart::Words(".".into()),
    ]))]);
    assert_eq!(document, expected);
    assert_eq!(document.print().0, source);
}
