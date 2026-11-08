//! A whole module read and printed: the front matter first, then the body.
//!
//! The front matter is read as the three fields of FrontMatter, in any order,
//! and printed back canonically in the type's order. The dependencies are read
//! as a flow sequence on their line or a block sequence below it, and printed
//! as a flow sequence. Any other line in it is refused as Unsupported,
//! carrying that line.

use crate::compose::Refusing;
use crate::module;
use crate::reading::{Close, FrontLines, SourceLines};
use crate::{
    Composable, Dependencies, Description, FrontMatter, FrontMatterField, Line, Markdown, Module,
    ModuleName, ModuleText, Placement, Printable, Problem, ReadRefusal,
};

/// The front-matter delimiter line.
const DELIMITER: &str = "---";

/// The one place a field meets its YAML key.
const DESCRIPTION: &str = "description:";
const DEPENDENCIES: &str = "dependencies:";
/// An entry of a block sequence.
const ENTRY: &str = "- ";
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
            .ok_or_else(|| Line(1).refusal(&lines))?;
        let front_matter = FrontLines {
            source_lines: SourceLines(lines.clone()),
            close: Close(close as i64),
        }
        .front_matter()?;
        let rest = Markdown(lines[close + 1..].join("\n"));
        let document = rest.compose().map_err(|refusal| ReadRefusal {
            line: Line(refusal.line.0 + close as i64 + 1),
            problem: refusal.problem,
        })?;
        Ok(Module {
            front_matter,
            document,
        })
    }
}

trait FrontReading {
    fn front_matter(&self) -> Result<FrontMatter, ReadRefusal>;
    fn refused(&self, index: usize) -> ReadRefusal;
    /// A field absent, refused at the closing delimiter, or empty, at its line.
    fn missing(&self, index: usize, field: FrontMatterField) -> ReadRefusal;
}

impl FrontReading for FrontLines {
    fn front_matter(&self) -> Result<FrontMatter, ReadRefusal> {
        let source = &self.source_lines.0;
        let close = self.close.0 as usize;
        let mut description = None;
        let mut dependencies = None;
        let mut placement = None;
        let mut index = 1;
        while index < close {
            let line = source[index].as_str();
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
                        return Err(self.missing(index, FrontMatterField::Description));
                    }
                    description = Some(Description(read));
                }
                (_, Some(""), _) if dependencies.is_none() => {
                    let mut names = Vec::new();
                    while index + 1 < close {
                        let Some(entry) = source[index + 1].trim_start().strip_prefix(ENTRY) else {
                            break;
                        };
                        let name = entry.name().ok_or_else(|| self.refused(index + 1))?;
                        names.push(name);
                        index += 1;
                    }
                    if names.is_empty() {
                        return Err(self.missing(index, FrontMatterField::Dependencies));
                    }
                    dependencies = Some(Dependencies(names));
                }
                (_, Some(value), _) if dependencies.is_none() => {
                    let names = value
                        .strip_prefix(' ')
                        .and_then(Scalar::names)
                        .ok_or_else(|| self.refused(index))?;
                    dependencies = Some(Dependencies(names));
                }
                (_, _, Some(value)) if placement.is_none() => {
                    let read = Placement::try_from(value).map_err(|_| self.refused(index))?;
                    placement = Some(read);
                }
                _ => return Err(self.refused(index)),
            }
            index += 1;
        }
        let description =
            description.ok_or_else(|| self.missing(close, FrontMatterField::Description))?;
        let dependencies =
            dependencies.ok_or_else(|| self.missing(close, FrontMatterField::Dependencies))?;
        Ok(FrontMatter {
            description,
            dependencies,
            placement_option: placement,
        })
    }

    fn refused(&self, index: usize) -> ReadRefusal {
        Line(index as i64 + 1).refusal(&self.source_lines.0)
    }

    fn missing(&self, index: usize, field: FrontMatterField) -> ReadRefusal {
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
    /// A module name as written: a plain scalar that also stands in a flow
    /// sequence, `vision-sema` or `intent:energy`.
    fn name(&self) -> Option<ModuleName>;
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
            false => inner.split(", ").map(Scalar::name).collect(),
        }
    }

    fn name(&self) -> Option<ModuleName> {
        let flow_safe = !self.contains([',', '[', ']', '{', '}']) && !self.contains(": ");
        (self.plain_scalar() && flow_safe && !self.ends_with(':'))
            .then(|| ModuleName(self.to_string()))
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
            .front_matter
            .dependencies
            .0
            .iter()
            .map(|name| name.0.as_str())
            .collect();
        let mut out = format!(
            "{DELIMITER}\n{DESCRIPTION} {}\n{DEPENDENCIES} [{}]\n",
            self.front_matter.description.0.printed_description(),
            names.join(", ")
        );
        if let Some(placement) = &self.front_matter.placement_option {
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
