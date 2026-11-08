//! The one place a Language meets its spelling in a fence's info string.

use crate::document::Language;

pub(crate) trait Spelled {
    fn spelling(&self) -> &'static str;
}

impl Spelled for Language {
    fn spelling(&self) -> &'static str {
        match self {
            Language::Ethos => "ethos",
            Language::Datom => "datom",
            Language::Rust => "rust",
            Language::Nix => "nix",
            Language::Shell => "sh",
            Language::Ini => "ini",
            Language::Toml => "toml",
            Language::Mermaid => "mermaid",
            Language::Plain => "",
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = String;

    fn try_from(spelling: &str) -> Result<Self, String> {
        [
            Language::Ethos,
            Language::Datom,
            Language::Rust,
            Language::Nix,
            Language::Shell,
            Language::Ini,
            Language::Toml,
            Language::Mermaid,
            Language::Plain,
        ]
        .into_iter()
        .find(|language| language.spelling() == spelling)
        .ok_or_else(|| spelling.to_string())
    }
}
