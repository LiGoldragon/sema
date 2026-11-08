//! Sema version 0: Markdown read structurally. Markdown composes into a
//! Document, and a Document prints back into Markdown: one walk, two directions.

mod compose;
mod document;
mod error;
mod language;
mod print;
mod structural;

pub use compose::Composable;
pub use document::{
    Block, Code_Data, Depth, Document, Kind, Language, Markdown, Pattern, Place, Reference_Data,
    Row, Run, Section, Target, Text, Title,
};
pub use error::{Construct, Delimiter, Error, Line, Problem};
pub use print::Printable;
pub use structural::Structural;
