//! The one place a ProgrammingLanguage meets its spelling in a fence's info string.

use crate::ProgrammingLanguage;

pub(crate) trait Spelled {
    fn spelling(&self) -> &'static str;
}

impl Spelled for ProgrammingLanguage {
    fn spelling(&self) -> &'static str {
        match self {
            ProgrammingLanguage::Ethos => "ethos",
            ProgrammingLanguage::Datom => "datom",
            ProgrammingLanguage::Rust => "rust",
            ProgrammingLanguage::Nix => "nix",
            ProgrammingLanguage::Shell => "sh",
            ProgrammingLanguage::Ini => "ini",
            ProgrammingLanguage::Toml => "toml",
            ProgrammingLanguage::Mermaid => "mermaid",
            ProgrammingLanguage::Plain => "",
        }
    }
}

impl TryFrom<&str> for ProgrammingLanguage {
    type Error = String;

    fn try_from(spelling: &str) -> Result<Self, String> {
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
        .ok_or_else(|| spelling.to_string())
    }
}
