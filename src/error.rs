//! Why reading Markdown stopped, as ethos/markdown.ethos declares it (ruled by
//! Psyche core Fable, inferred, pending the living).

/// Where reading stopped, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadRefusal {
    pub line: Line,
    pub problem: Problem,
}

/// A line of the text, counted from one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A construct the Document has no place for, as written: the line it
    /// stands on.
    Unsupported(String),
    /// A front-matter field absent, or empty.
    Missing(FrontMatterField),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontMatterField {
    Description,
    Dependencies,
}

impl std::fmt::Display for ReadRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.problem {
            Problem::Unsupported(written) => {
                write!(formatter, "line {}: Unsupported «{written}»", self.line.0)
            }
            Problem::Missing(field) => write!(formatter, "line {}: Missing {field:?}", self.line.0),
        }
    }
}

impl std::error::Error for ReadRefusal {}
