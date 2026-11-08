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
    Code(Code_Data),
    List(Vec<Block>),
    Quote(Vec<Block>),
    Table(Vec<Row>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code_Data {
    pub language: Language,
    pub text: Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(pub Vec<Text>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ethos,
    Datom,
    Rust,
    Nix,
    Shell,
    Ini,
    Mermaid,
    Plain,
}

/// Inside a paragraph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(pub Vec<Run>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    Words(String),
    Emphasis(Text),
    Strong(Text),
    Code(String),
    /// A link.
    Reference(Reference_Data),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference_Data {
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
