//! The one place a ProgrammingLanguage meets its spelling in a fence's info string.

use crate::ProgrammingLanguage;
use crate::reading::Spellable;

impl Spellable for ProgrammingLanguage {
    fn spelling(&self) -> String {
        let spelling = match self {
            ProgrammingLanguage::Ethos => "ethos",
            ProgrammingLanguage::Datom => "datom",
            ProgrammingLanguage::Rust => "rust",
            ProgrammingLanguage::Nix => "nix",
            ProgrammingLanguage::Shell => "sh",
            ProgrammingLanguage::Ini => "ini",
            ProgrammingLanguage::Toml => "toml",
            ProgrammingLanguage::Mermaid => "mermaid",
            ProgrammingLanguage::Plain => "",
        };
        spelling.to_string()
    }

    fn spelled(spelling: String) -> Option<Self> {
        [
            ProgrammingLanguage::Ethos,
            ProgrammingLanguage::Datom,
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Nix,
            ProgrammingLanguage::Shell,
            ProgrammingLanguage::Ini,
            ProgrammingLanguage::Toml,
            ProgrammingLanguage::Mermaid,
            ProgrammingLanguage::Plain,
        ]
        .into_iter()
        .find(|programming_language| programming_language.spelling() == spelling)
    }
}
