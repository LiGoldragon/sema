//! The types of ethos/module.ethos: a belief module is its front matter, typed,
//! then its body (ruled by Psyche core Fable, inferred, pending the living).

use crate::document::Document;

/// A module's name, kind-subject, as written in the YAML: vision-sema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleName(pub String);

/// Where the module is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    SystemPrompt,
    Skill,
}

/// The front matter, typed; field order is print order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Front {
    pub description: Description,
    pub dependencies: Dependencies,
    pub placement_option: Option<Placement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependencies(pub Vec<ModuleName>);

/// A belief module: its front matter, then its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub front: Front,
    pub document: Document,
}
