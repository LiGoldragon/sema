//! A whole module read and printed: the front matter first, then the body.
//!
//! The front matter is read as three YAML lines, each a field of Front, in any
//! order, and printed back canonically in the type's order. Any other line in
//! it is refused as Unsupported, carrying that line.

use crate::compose::{Composable, Refusing};
use crate::document::Markdown;
use crate::error::{FrontField, Line, Problem, ReadRefusal};
use crate::module::{
    self, Dependencies, Description, Front, Module, ModuleName, ModuleText, Placement,
};
use crate::print::Printable;

/// The front-matter delimiter line.
const DELIMITER: &str = "---";

/// The one place a field meets its YAML key.
const DESCRIPTION: &str = "description:";
const DEPENDENCIES: &str = "dependencies: ";
const PLACEMENT: &str = "placement: ";

impl module::Composable for ModuleText {
    /// The front matter is composed here; the text after its closing
    /// delimiter line is handed to Markdown, and a refusal there is counted
    /// from the start of the module text.
    fn compose(&self) -> Result<Module, ReadRefusal> {
        let lines: Vec<String> = self.0.split('\n').map(str::to_string).collect();
        if lines.first().map(String::as_str) != Some(DELIMITER) {
            return Err(Line(1).refusal(&lines));
        }
        let close = (1..lines.len())
            .find(|index| lines[*index] == DELIMITER)
            .ok_or(ReadRefusal {
                line: Line(1),
                problem: Problem::UnclosedFence,
            })?;
        let front = FrontLines {
            source: &lines,
            close,
        }
        .front()?;
        let rest = Markdown(lines[close + 1..].join("\n"));
        let document = rest.compose().map_err(|refusal| ReadRefusal {
            line: Line(refusal.line.0 + close as i64 + 1),
            problem: refusal.problem,
        })?;
        Ok(Module { front, document })
    }
}

/// The lines between the two delimiters; `close` is the closing one's index.
struct FrontLines<'a> {
    source: &'a [String],
    close: usize,
}

trait FrontReading {
    fn front(&self) -> Result<Front, ReadRefusal>;
    fn refused(&self, index: usize) -> ReadRefusal;
    /// A field absent, refused at the closing delimiter, or empty, at its line.
    fn missing(&self, index: usize, field: FrontField) -> ReadRefusal;
}

impl FrontReading for FrontLines<'_> {
    fn front(&self) -> Result<Front, ReadRefusal> {
        let mut description = None;
        let mut dependencies = None;
        let mut placement = None;
        for index in 1..self.close {
            let line = self.source[index].as_str();
            match (
                line.strip_prefix(DESCRIPTION),
                line.strip_prefix(DEPENDENCIES),
                line.strip_prefix(PLACEMENT),
            ) {
                (Some(value), _, _) if description.is_none() => {
                    let value = match value.strip_prefix(' ') {
                        Some(value) => value,
                        None if value.is_empty() => value,
                        None => return Err(self.refused(index)),
                    };
                    let read = value.description().ok_or_else(|| self.refused(index))?;
                    if read.is_empty() {
                        return Err(self.missing(index, FrontField::Description));
                    }
                    description = Some(Description(read));
                }
                (_, Some(value), _) if dependencies.is_none() => {
                    let names = value.names().ok_or_else(|| self.refused(index))?;
                    dependencies = Some(Dependencies(names));
                }
                (_, _, Some(value)) if placement.is_none() => {
                    let read = Placement::try_from(value).map_err(|_| self.refused(index))?;
                    placement = Some(read);
                }
                _ => return Err(self.refused(index)),
            }
        }
        let description =
            description.ok_or_else(|| self.missing(self.close, FrontField::Description))?;
        let dependencies =
            dependencies.ok_or_else(|| self.missing(self.close, FrontField::Dependencies))?;
        Ok(Front {
            description,
            dependencies,
            placement_option: placement,
        })
    }

    fn refused(&self, index: usize) -> ReadRefusal {
        Line(index as i64 + 1).refusal(self.source)
    }

    fn missing(&self, index: usize, field: FrontField) -> ReadRefusal {
        ReadRefusal {
            line: Line(index as i64 + 1),
            problem: Problem::Missing(field),
        }
    }
}

/// A YAML value read only in the plain forms the printer writes.
trait Scalar {
    /// A plain scalar on one line: not empty, not quoted, not a block or flow
    /// opener, and so the string as written.
    fn plain_scalar(&self) -> bool;
    /// A description as written: plain, double-quoted with `\\` and `\"`
    /// escaped, or single-quoted with `''` for a quote; None when malformed.
    fn description(&self) -> Option<String>;
    /// The description as printed: plain when the plain form reads back the
    /// same, double-quoted otherwise.
    fn printed_description(&self) -> String;
    /// A flow sequence of plain names, `[]` or `[a, b]`.
    fn names(&self) -> Option<Vec<ModuleName>>;
}

impl Scalar for str {
    fn plain_scalar(&self) -> bool {
        !self.is_empty()
            && !self.starts_with([
                ' ', '"', '\'', '|', '>', '[', '{', '&', '*', '!', '#', '%', '@', '`',
            ])
            && self.trim_end() == self
            && !self.contains(" #")
    }

    fn description(&self) -> Option<String> {
        if let Some(inner) = self.strip_prefix('"') {
            let inner = inner.strip_suffix('"')?;
            let mut out = String::new();
            let mut characters = inner.chars();
            while let Some(character) = characters.next() {
                match character {
                    '\\' => match characters.next()? {
                        escaped @ ('\\' | '"') => out.push(escaped),
                        _ => return None,
                    },
                    '"' => return None,
                    other => out.push(other),
                }
            }
            Some(out)
        } else if let Some(inner) = self.strip_prefix('\'') {
            let inner = inner.strip_suffix('\'')?;
            let doubled = inner.replace("''", "");
            (!doubled.contains('\'')).then(|| inner.replace("''", "'"))
        } else if self.is_empty() || self.plain_scalar() {
            Some(self.to_string())
        } else {
            None
        }
    }

    fn printed_description(&self) -> String {
        match self.plain_scalar() {
            true => self.to_string(),
            false => format!("\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\"")),
        }
    }

    fn names(&self) -> Option<Vec<ModuleName>> {
        let inner = self.strip_prefix('[')?.strip_suffix(']')?;
        match inner.is_empty() {
            true => Some(Vec::new()),
            false => inner
                .split(", ")
                .map(|name| {
                    let plain = !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
                    plain.then(|| ModuleName(name.to_string()))
                })
                .collect(),
        }
    }
}

/// The one place a Placement meets its YAML spelling.
pub(crate) trait PlacementSpelled {
    fn spelling(&self) -> &'static str;
}

impl PlacementSpelled for Placement {
    fn spelling(&self) -> &'static str {
        match self {
            Placement::SystemPrompt => "system",
            Placement::Skill => "skill",
        }
    }
}

impl TryFrom<&str> for Placement {
    type Error = String;

    fn try_from(spelling: &str) -> Result<Self, String> {
        [Placement::SystemPrompt, Placement::Skill]
            .into_iter()
            .find(|placement| placement.spelling() == spelling)
            .ok_or_else(|| spelling.to_string())
    }
}

impl module::Printable for Module {
    fn print(&self) -> ModuleText {
        let names: Vec<&str> = self
            .front
            .dependencies
            .0
            .iter()
            .map(|name| name.0.as_str())
            .collect();
        let mut out = format!(
            "{DELIMITER}\n{DESCRIPTION} {}\n{DEPENDENCIES}[{}]\n",
            self.front.description.0.printed_description(),
            names.join(", ")
        );
        if let Some(placement) = self.front.placement_option {
            out += &format!("{PLACEMENT}{}\n", placement.spelling());
        }
        out += DELIMITER;
        out += "\n";
        let body = self.document.print();
        if !body.0.is_empty() {
            out += "\n";
            out += &body.0;
        }
        ModuleText(out)
    }
}
