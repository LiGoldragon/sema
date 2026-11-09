#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Heading_Data {
    pub depth: sema::Depth,
    pub title: sema::Title,
    pub line: sema::Line,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Piece {
    Heading(Heading_Data),
    Block(sema::Block),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Pieces(pub std::vec::Vec<Piece>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct OpenSection {
    pub depth: sema::Depth,
    pub title: sema::Title,
    pub block_vector: std::vec::Vec<sema::Block>,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct SourceLines(pub std::vec::Vec<String>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Close(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct FrontLines {
    pub source_lines: SourceLines,
    pub close: Close,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Index(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct MissingField {
    pub index: Index,
    pub front_matter_field: sema::FrontMatterField,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct PlacedBlock {
    pub place: sema::Place,
    pub block: sema::Block,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Remaining(pub std::vec::Vec<sema::Text>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct PendingEdit {
    pub edit: sema::Edit,
    pub remaining: Remaining,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Parent(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct PrintTask {
    pub parent: Parent,
    pub print_step: PrintStep,
    pub depth: sema::Depth,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Header(pub bool);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Row_Data {
    pub text_vector: std::vec::Vec<sema::Text>,
    pub header: Header,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum PrintStep {
    Block(sema::Block),
    Text(sema::Text),
    Row(Row_Data),
}
#[rustfmt::skip]
pub trait Readable {
    fn pieces(&self) -> std::result::Result<std::vec::Vec<Piece>, sema::Line>;
    fn piece(&self) -> std::result::Result<Piece, sema::Line>;
    fn contained(&self) -> std::result::Result<std::vec::Vec<sema::Block>, sema::Line>;
    fn list(&self) -> std::result::Result<sema::Block, sema::Line>;
    fn table(&self) -> std::result::Result<sema::Block, sema::Line>;
    fn text(&self) -> std::result::Result<sema::Text, sema::Line>;
    fn text_part(&self) -> std::result::Result<sema::TextPart, sema::Line>;
    fn line(&self) -> sema::Line;
}
#[rustfmt::skip]
pub trait Nestable {
    fn nested(&self) -> std::result::Result<std::vec::Vec<sema::Block>, sema::Line>;
}
#[rustfmt::skip]
pub trait Quotable {
    fn quote(&self, input: i64) -> sema::ReadRefusal;
}
#[rustfmt::skip]
pub trait Configurable {
    fn sema() -> Self
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait FrontMatterReadable {
    fn front_matter(&self) -> std::result::Result<sema::FrontMatter, sema::ReadRefusal>;
    fn refused(&self, input: i64) -> sema::ReadRefusal;
    fn missing(&self, input: MissingField) -> sema::ReadRefusal;
}
#[rustfmt::skip]
pub trait ScalarReadable {
    fn plain_scalar(&self) -> bool;
    fn description(&self) -> std::option::Option<String>;
    fn printed_description(&self) -> String;
    fn names(&self) -> std::option::Option<std::vec::Vec<sema::ModuleName>>;
    fn name(&self) -> std::option::Option<sema::ModuleName>;
}
#[rustfmt::skip]
pub trait Spellable {
    fn spelling(&self) -> String;
    fn spelled(input: String) -> std::option::Option<Self>
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Bulleted {
    fn bullet() -> Self
    where
        Self: Sized;
}
#[rustfmt::skip]
pub trait Placeable {
    fn placed(&self) -> std::vec::Vec<PlacedBlock>;
}
#[rustfmt::skip]
pub trait Relatable {
    fn under(&self, input: std::vec::Vec<sema::Text>) -> bool;
    fn within(&self, input: std::vec::Vec<sema::Text>) -> sema::Place;
}
#[rustfmt::skip]
pub trait Textual {
    fn own_texts(&self) -> std::vec::Vec<sema::Text>;
}
#[rustfmt::skip]
pub trait Searchable {
    fn plain(&self) -> String;
    fn references(&self) -> std::vec::Vec<sema::Reference>;
    fn inline_codes(&self) -> std::vec::Vec<String>;
}
#[rustfmt::skip]
pub trait Outlinable {
    fn outlined(&self, input: i64) -> std::vec::Vec<sema::Outline>;
}
#[rustfmt::skip]
pub trait Editable {
    fn apply(&mut self, input: PendingEdit) -> std::option::Option<sema::EditRefusal>;
}
#[rustfmt::skip]
const _: () = {
    fn assert_sourcelines_quotable<T: Quotable>() {}
    let _ = assert_sourcelines_quotable::<SourceLines>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_frontlines_frontmatterreadable<T: FrontMatterReadable>() {}
    let _ = assert_frontlines_frontmatterreadable::<FrontLines>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_document_placeable<T: Placeable>() {}
    let _ = assert_document_placeable::<sema::Document>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_place_relatable<T: Relatable>() {}
    let _ = assert_place_relatable::<sema::Place>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_block_textual<T: Textual>() {}
    let _ = assert_block_textual::<sema::Block>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_text_searchable<T: Searchable>() {}
    let _ = assert_text_searchable::<sema::Text>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_programminglanguage_spellable<T: Spellable>() {}
    let _ = assert_programminglanguage_spellable::<sema::ProgrammingLanguage>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_placement_spellable<T: Spellable>() {}
    let _ = assert_placement_spellable::<sema::Placement>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_pieces_nestable<T: Nestable>() {}
    let _ = assert_pieces_nestable::<Pieces>;
};
