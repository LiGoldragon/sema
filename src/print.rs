//! A Document printed into Markdown, in the canonical form the reader takes:
//! one blank line between blocks, and a newline ending the text.

use crate::compose::ESCAPABLE;
use crate::document::{Block, Depth, Document, Markdown, Place, Row, Target, Text, TextPart};
use crate::programming_language::Spelled;

pub trait Printable {
    fn print(&self) -> Markdown;
}

impl Printable for Document {
    fn print(&self) -> Markdown {
        let body = self.0.as_slice().rendered(Depth(0));
        match body.is_empty() {
            true => Markdown(body),
            false => Markdown(body + "\n"),
        }
    }
}

/// Where words are printed, which decides what it escapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Surround {
    Prose,
    ReferenceText,
}

trait BlockRendering {
    fn rendered(&self, depth: Depth) -> String;
}

impl BlockRendering for [Block] {
    fn rendered(&self, depth: Depth) -> String {
        self.iter()
            .map(|block| block.rendered(depth))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl BlockRendering for Block {
    fn rendered(&self, depth: Depth) -> String {
        match self {
            Block::Statement(text) => text.rendered(Surround::Prose),
            Block::Section(section) => {
                let mut out =
                    "#".repeat(depth.0 + 1) + " " + &section.title.0.rendered(Surround::Prose);
                if !section.block_vector.is_empty() {
                    out += "\n\n";
                    out += &section.block_vector.as_slice().rendered(Depth(depth.0 + 1));
                }
                out
            }
            Block::Code(code) => match code.text.0.is_empty() {
                true => format!("```{}\n```", code.programming_language.spelling()),
                false => format!(
                    "```{}\n{}\n```",
                    code.programming_language.spelling(),
                    code.text.verbatim()
                ),
            },
            Block::List(items) => items
                .iter()
                .map(|item| item.rendered(Depth(0)).prefixed("- ", "  "))
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Quote(blocks) => blocks.as_slice().rendered(Depth(0)).prefixed("> ", "> "),
            Block::Table(rows) => {
                let mut lines: Vec<String> = rows.iter().map(Row::rendered).collect();
                if let Some(header) = rows.first() {
                    let delimiter = vec!["---"; header.0.len()].join(" | ");
                    lines.insert(1, format!("| {delimiter} |"));
                }
                lines.join("\n")
            }
        }
    }
}

trait RowRendering {
    fn rendered(&self) -> String;
}

impl RowRendering for Row {
    fn rendered(&self) -> String {
        let cells: Vec<String> = self
            .0
            .iter()
            .map(|cell| cell.rendered(Surround::Prose))
            .collect();
        format!("| {} |", cells.join(" | "))
    }
}

/// Every line of a block given a leading mark: the first line one, the rest
/// another. An empty line keeps the mark without its trailing space.
trait Prefixed {
    fn prefixed(&self, first: &str, rest: &str) -> String;
}

impl Prefixed for String {
    fn prefixed(&self, first: &str, rest: &str) -> String {
        self.split('\n')
            .enumerate()
            .map(|(index, line)| {
                let mark = if index == 0 { first } else { rest };
                match line.is_empty() {
                    true => mark.trim_end().to_string(),
                    false => format!("{mark}{line}"),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

trait TextRendering {
    fn rendered(&self, surround: Surround) -> String;
    /// The text as a fence holds it: its words unescaped.
    fn verbatim(&self) -> String;
}

impl TextRendering for Text {
    fn rendered(&self, surround: Surround) -> String {
        self.0
            .iter()
            .map(|text_part| text_part.rendered(surround))
            .collect()
    }

    fn verbatim(&self) -> String {
        self.0
            .iter()
            .map(|text_part| match text_part {
                TextPart::Words(words) => words.clone(),
                other => other.rendered(Surround::Prose),
            })
            .collect()
    }
}

/// UNIMPLEMENTED until designed (Fable ruling, inferred, pending the living).
/// A Markdown link target, anchors included, is read and printed as
/// `Target.Url`, the string as written; a Place is never composed from
/// Markdown. A Place is what Sema itself yields, in search results and edits,
/// named by titles from the root, and the conversion from a Place to an
/// anchor is not designed, so printing one stops here instead of guessing.
trait AnchorConversion {
    fn anchor(&self) -> String;
}

impl AnchorConversion for Place {
    fn anchor(&self) -> String {
        unimplemented!("Place-to-anchor conversion is not designed in Sema v0")
    }
}

trait TextPartRendering {
    fn rendered(&self, surround: Surround) -> String;
}

impl TextPartRendering for TextPart {
    fn rendered(&self, surround: Surround) -> String {
        match self {
            TextPart::Words(words) => words.escaped(surround),
            TextPart::Emphasis(text) => format!("*{}*", text.rendered(surround)),
            TextPart::Strong(text) => format!("**{}**", text.rendered(surround)),
            TextPart::Code(code) => format!("`{code}`"),
            TextPart::Reference(reference) => {
                let target = match &reference.target {
                    Target::Url(url) => url.clone(),
                    Target::Place(place) => place.anchor(),
                };
                format!(
                    "[{}]({target})",
                    reference.text.rendered(Surround::ReferenceText)
                )
            }
        }
    }
}

/// Words with the escapes the reader needs to read them back as words.
trait Escaped {
    fn escaped(&self, surround: Surround) -> String;
}

impl Escaped for String {
    fn escaped(&self, surround: Surround) -> String {
        let characters: Vec<char> = self.chars().collect();
        let mut out = String::new();
        for (index, character) in characters.iter().enumerate() {
            let following = characters.get(index + 1);
            let escape = match character {
                '*' | '`' => true,
                '\\' => following.is_none_or(|next| ESCAPABLE.contains(next)),
                '[' => characters[index..]
                    .iter()
                    .collect::<String>()
                    .contains("]("),
                ']' => surround == Surround::ReferenceText,
                _ => false,
            };
            if escape {
                out.push('\\');
            }
            out.push(*character);
        }
        out
    }
}
