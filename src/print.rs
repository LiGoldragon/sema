//! A Document printed into Markdown, in the canonical form the reader takes:
//! one blank line between blocks, and a newline ending the text.

use crate::compose::ESCAPABLE;
use crate::document::{Block, Depth, Document, Markdown, Place, Row, Run, Target, Text};
use crate::language::Spelled;

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

/// Where a run of words is printed, which decides what it escapes.
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
                true => format!("```{}\n```", code.language.spelling()),
                false => format!(
                    "```{}\n{}\n```",
                    code.language.spelling(),
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
        self.0.iter().map(|run| run.rendered(surround)).collect()
    }

    fn verbatim(&self) -> String {
        self.0
            .iter()
            .map(|run| match run {
                Run::Words(words) => words.clone(),
                other => other.rendered(Surround::Prose),
            })
            .collect()
    }
}

/// UNKNOWN, pending judgment. The vision gives `Target.Place`, the titles
/// above a block from the root, and gives no Markdown form for it: a Markdown
/// link names an anchor, and neither direction between a Place and an anchor
/// is stated. The reader never composes a Place. `print.[ Markdown ]` is total,
/// so a Place handed to the printer is written in this assumed form, `#` and
/// the titles joined by `/`, which is not approved and is not read back.
trait ProvisionalTarget {
    fn provisional_target(&self) -> String;
}

impl ProvisionalTarget for Place {
    fn provisional_target(&self) -> String {
        let titles: Vec<String> = self
            .0
            .iter()
            .map(|title| title.rendered(Surround::Prose))
            .collect();
        format!("#{}", titles.join("/"))
    }
}

trait RunRendering {
    fn rendered(&self, surround: Surround) -> String;
}

impl RunRendering for Run {
    fn rendered(&self, surround: Surround) -> String {
        match self {
            Run::Words(words) => words.escaped(surround),
            Run::Emphasis(text) => format!("*{}*", text.rendered(surround)),
            Run::Strong(text) => format!("**{}**", text.rendered(surround)),
            Run::Code(code) => format!("`{code}`"),
            Run::Reference(reference) => {
                let target = match &reference.target {
                    Target::Url(url) => url.clone(),
                    Target::Place(place) => place.provisional_target(),
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
