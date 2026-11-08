//! sema reads a Markdown file, prints the typed tree and the Markdown printed
//! from it, and compares the printed text with what was read.
//!
//! sema module <path>     the whole file as a Module: front matter, then body
//! sema document <path>   the whole file as a Document; front matter is refused
//! sema body <path>       a test harness: the text after the front matter's
//!                        closing delimiter line, read as a Document

use sema::module::{Composable as _, Printable as _};
use sema::{Composable, Markdown, ModuleText, Printable};
use std::fmt::Debug;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
    eprintln!("usage: sema (module|document|body) <path>");
    ExitCode::from(2)
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
