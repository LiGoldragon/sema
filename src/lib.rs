//! Sema version 0: Markdown read structurally. Markdown composes into a
//! Document, and a Document prints back into Markdown: one walk, two directions.

mod commonmark;
mod compose;
mod document;
mod error;
mod front_matter;
pub mod module;
mod print;
mod programming_language;
mod structural;

pub use compose::Composable;
pub use document::{
    Block, Code, Depth, Document, Kind, Markdown, Outline, Pattern, Place, ProgrammingLanguage,
    Reference, Row, Section, Target, Text, TextPart, Title,
};
pub use error::{FrontMatterField, Line, Problem, ReadRefusal};
pub use module::{
    Dependencies, Description, FrontMatter, Module, ModuleName, ModuleText, Placement,
};
pub use print::Printable;
pub use structural::Structural;
