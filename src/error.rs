//! What reading Markdown refuses, and where. Each problem names a text the
//! Document cannot hold as it was written; the line is where it starts.

use crate::document::Depth;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub line: Line,
    pub problem: Problem,
}

/// A line of the Markdown, counted from one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The text opens with a front-matter delimiter; a Document is a body alone.
    FrontMatter,
    /// A heading more than one level under the section it would open in.
    SkippedDepth {
        above: Depth,
        heading: Depth,
    },
    EmptyHeading,
    /// A fence whose info string names no Language.
    UnknownLanguage(String),
    UnclosedFence,
    UnclosedRun(Delimiter),
    /// A list item holds exactly one block; this one held the count given.
    ItemBlockCount(usize),
    /// A table row whose cell count differs from its header's.
    RaggedTable {
        header: usize,
        row: usize,
    },
    /// A table whose second line is not a delimiter row.
    MissingTableDelimiter,
    Unsupported(Construct),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Emphasis,
    Strong,
    Code,
    Reference,
}

/// Markdown forms the Document has no place for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construct {
    ThematicBreak,
    SetextHeading,
    OrderedList,
    ListMarker,
    IndentedBlock,
    TildeFence,
    DoubleBacktick,
    TableAlignment,
    LazyContinuation,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}: {:?}", self.line.0, self.problem)
    }
}

impl std::error::Error for Error {}
