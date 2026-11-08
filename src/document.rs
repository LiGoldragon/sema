//! The typed tree of ethos/document.ethos. Each declaration is one Rust type;
//! an inline payload carries the `_Data` name the ethos rules derive for it.
#![allow(non_camel_case_types)]

/// The text, as it is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Markdown(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document(pub Vec<Block>);

/// A heading and what is under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: Title,
    pub block_vector: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(pub Text);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A paragraph.
    Statement(Text),
    Section(Section),
    /// A fence; Mermaid is a figure.
    Code(Code),
    List(Vec<Block>),
    Quote(Vec<Block>),
    Table(Vec<Row>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    pub programming_language: ProgrammingLanguage,
    pub text: Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(pub Vec<Text>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgrammingLanguage {
    Ethos,
    Datom,
    Rust,
    Nix,
    Shell,
    Ini,
    Toml,
    Mermaid,
    Plain,
}

/// Inside a paragraph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(pub Vec<TextPart>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPart {
    Words(String),
    Emphasis(Text),
    Strong(Text),
    Code(String),
    /// A link.
    Reference(Reference),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub text: Text,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Place(Place),
    Url(String),
}

/// The titles above a block, from the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place(pub Vec<Text>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Depth(pub usize);

/// The kind of block a search looks in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Headings,
    Statements,
    Code,
    References,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern(pub String);

/// A heading and the headings under it, to a depth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    pub title: Title,
    pub outline_vector: Vec<Outline>,
}
