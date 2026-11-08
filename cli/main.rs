//! sema reads a Markdown file, prints the typed tree and the Markdown printed
//! from it, and compares the printed text with what was read.
//!
//! sema module <path>     the whole file as a Module: front matter, then body
//! sema document <path>   the whole file as a Document; front matter is refused
//! sema body <path>       a test harness: the text after the front matter's
//!                        closing delimiter line, read as a Document
//!
//! A query reads the file as a Module and asks its Document; each argument is
//! a datom, and the answer prints as datom in the translator's forms: a title
//! as its Markdown inline string, the marks kept; a place as the vector of
//! its titles; a hit as its place and its block printed as Markdown. A typed
//! place is matched on the plain words of the headings.
//!
//! sema outline <path> <Depth>             e.g. 2
//! sema search <path> <Kind> <Pattern>     e.g. Headings Signal
//! sema code <path> <RenderedPlace>        e.g. [ «The Belief Nexus» Memory ]

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Path, Potential};
use protos::{Protosizable, ReaderBudget, Textualizable};
use sema::module::{Composable as _, Printable as _};
use sema::{
    Composable, Depth, Document, HitRenderable, Kind, Markdown, ModuleText, OutlineRenderable,
    Pattern, PlaceResolvable, Printable, RenderedPlace, Structural,
};
use std::fmt::Debug;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [query, path, inputs @ ..] if ["outline", "search", "code"].contains(&query.as_str()) => {
            return ask(query, path, inputs);
        }
        _ => {}
    }
    let [mode, path] = arguments.as_slice() else {
        return usage();
    };
    let whole = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::from(2);
        }
    };
    println!("file: {path}, {} bytes", whole.len());
    let markdown = Markdown(whole.clone());
    match mode.as_str() {
        "module" => report(
            &whole,
            ModuleText(whole.clone()).compose(),
            |module| module.print().0,
            |text| ModuleText(text.to_string()).compose(),
        ),
        "document" => report(
            &whole,
            markdown.compose(),
            |document| document.print().0,
            |text| Markdown(text.to_string()).compose(),
        ),
        "body" => {
            let body = whole
                .strip_prefix("---\n")
                .and_then(|rest| rest.split_once("\n---\n"))
                .map_or(whole.as_str(), |(_, body)| body);
            println!(
                "harness: {} bytes set aside through the closing delimiter line; body {} bytes",
                whole.len() - body.len(),
                body.len()
            );
            report(
                body,
                Markdown(body.to_string()).compose(),
                |document| document.print().0,
                |text| Markdown(text.to_string()).compose(),
            )
        }
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: sema (module|document|body) <path>\n       sema outline <path> <Depth>\n       sema search <path> <Kind> <Pattern>\n       sema code <path> <Place>"
    );
    ExitCode::from(2)
}

/// One Structural capability over the file's Document, its answer as datom.
fn ask(query: &str, path: &str, inputs: &[String]) -> ExitCode {
    let document: Document = match std::fs::read_to_string(path)
        .map_err(|error| error.to_string())
        .and_then(|text| {
            ModuleText(text)
                .compose()
                .map_err(|refusal| refusal.to_string())
        }) {
        Ok(module) => module.document,
        Err(problem) => {
            eprintln!("{path}: {problem}");
            return ExitCode::from(1);
        }
    };
    let answer = match (query, inputs) {
        ("outline", [depth]) => actual::<Depth>(depth).map(|depth| {
            let outlines: Vec<_> = document
                .outline(depth)
                .iter()
                .map(|outline| outline.rendered_outline())
                .collect();
            datom(&outlines)
        }),
        ("search", [kind, pattern]) => actual::<Kind>(kind).and_then(|kind| {
            actual::<Pattern>(pattern).map(|pattern| {
                let hits: Vec<_> = document
                    .search(kind, pattern)
                    .iter()
                    .map(|hit| hit.rendered_hit())
                    .collect();
                datom(&hits)
            })
        }),
        ("code", [place]) => actual::<RenderedPlace>(place).and_then(|place| {
            document
                .resolved(place)
                .map(|place| datom(&document.code(place)))
                .map_err(|refusal| datom(&refusal))
        }),
        _ => return usage(),
    };
    match answer {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::from(2)
        }
    }
}

fn actual<T: Composing>(text: &str) -> Result<T, String> {
    let mut budget = Budget {
        remaining: 4_096,
        reader: ReaderBudget::try_from(4_096).map_err(|error| format!("{error:?}"))?,
        depth: 0,
        maximum_depth: 64,
    };
    Potential::<T>::from(text)
        .actualize(&mut budget)
        .map_err(|error| format!("{text}: {error:?}"))
}

fn datom<T: Datomizable>(value: &T) -> String {
    value.datomize(Path::new()).protosize().textualize()
}

fn report<T: Debug + PartialEq, E: std::fmt::Display>(
    source: &str,
    read: Result<T, E>,
    print: impl Fn(&T) -> String,
    again: impl Fn(&str) -> Result<T, E>,
) -> ExitCode {
    let tree = match read {
        Ok(tree) => tree,
        Err(refusal) => {
            println!("refused: {refusal}");
            return ExitCode::from(1);
        }
    };
    println!("--- tree\n{tree:#?}");
    let printed = print(&tree);
    println!("--- printed Markdown\n{printed}");
    println!("--- comparison");
    println!(
        "read bytes {}, printed bytes {}",
        source.len(),
        printed.len()
    );
    match source == printed {
        true => println!("bytes: identical"),
        false => {
            let common = source
                .bytes()
                .zip(printed.bytes())
                .take_while(|(left, right)| left == right)
                .count();
            println!("bytes: differ, first difference at byte offset {common}");
        }
    }
    match again(&printed) {
        Ok(reread) if reread == tree => println!("tree: the printed text reads back equal"),
        Ok(_) => println!("tree: the printed text reads back different"),
        Err(refusal) => println!("tree: the printed text is refused: {refusal}"),
    }
    ExitCode::SUCCESS
}
