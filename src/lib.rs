//! Sema version 0: Markdown read structurally. Markdown composes into a
//! Document, and a Document prints back into Markdown: one walk, two directions.

mod compose;
mod document;
mod error;
mod front;
mod language;
pub mod module;
mod print;
mod structural;

pub use compose::Composable;
pub use document::{
    Block, Code_Data, Depth, Document, Kind, Language, Markdown, Outline, Pattern, Place,
    Reference_Data, Row, Run, Section, Target, Text, Title,
};
pub use error::{FrontField, Line, Problem, ReadRefusal};
pub use module::{Dependencies, Description, Front, Module, ModuleName, ModuleText, Placement};
pub use print::Printable;
pub use structural::Structural;
