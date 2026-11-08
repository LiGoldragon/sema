//! sema reads a Markdown file into a Document, prints the tree and the
//! Markdown printed from it, and compares the two texts.
//!
//! sema document <path>   the whole file is the Markdown
//! sema body <path>       a front-matter block is set aside first, as a test
//!                        harness only: a module's front matter is not yet
//!                        part of the approved Sema v0 contract

use sema::{Composable, Markdown, Printable};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (mode, path) = match arguments.as_slice() {
        [mode, path] if mode == "document" || mode == "body" => (mode.as_str(), path.as_str()),
        _ => {
            eprintln!("usage: sema (document|body) <path>");
            return ExitCode::from(2);
        }
    };
    let whole = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::from(2);
        }
    };
    println!("file: {path}, {} bytes", whole.len());
    let source = match mode {
        "body" => match whole
            .strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---\n"))
        {
            Some((front, body)) => {
                println!(
                    "harness: front matter set aside, {} bytes through its closing delimiter; body {} bytes",
                    front.len() + "---\n".len() + "\n---\n".len(),
                    body.len()
                );
                body.to_string()
            }
            None => {
                println!("harness: no front matter found; the whole file is the body");
                whole.clone()
            }
        },
        _ => whole.clone(),
    };
    let document = match Markdown(source.clone()).compose() {
        Ok(document) => document,
        Err(error) => {
            println!("refused: {error}");
            return ExitCode::from(1);
        }
    };
    println!("--- Document\n{document:#?}");
    let printed = document.print();
    println!("--- printed Markdown\n{}", printed.0);
    println!("--- comparison");
    println!(
        "source bytes {}, printed bytes {}",
        source.len(),
        printed.0.len()
    );
    match source == printed.0 {
        true => println!("bytes: identical"),
        false => {
            let common = source
                .bytes()
                .zip(printed.0.bytes())
                .take_while(|(left, right)| left == right)
                .count();
            println!("bytes: differ, first difference at byte offset {common}");
        }
    }
    match printed.compose() {
        Ok(again) if again == document => {
            println!("tree: printed Markdown composes to an equal Document")
        }
        Ok(_) => println!("tree: printed Markdown composes to a different Document"),
        Err(error) => println!("tree: printed Markdown is refused: {error}"),
    }
    ExitCode::SUCCESS
}
