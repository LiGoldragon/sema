#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Document(pub std::vec::Vec<Block>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub struct Section {
    pub title: Title,
    #[rkyv(omit_bounds)]
    pub block_vector: std::vec::Vec<Block>,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub enum Block {
    Statement(Text),
    Section(#[rkyv(omit_bounds)] Section),
    CodeBlock(CodeBlock),
    List(#[rkyv(omit_bounds)] std::vec::Vec<Block>),
    Quote(#[rkyv(omit_bounds)] std::vec::Vec<Block>),
    Table(std::vec::Vec<Row>),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Row(pub std::vec::Vec<Text>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Title(pub Text);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
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
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Text(pub std::vec::Vec<TextPart>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub enum TextPart {
    Words(String),
    Emphasis(#[rkyv(omit_bounds)] Text),
    Strong(#[rkyv(omit_bounds)] Text),
    InlineCode(String),
    Reference(#[rkyv(omit_bounds)] Reference),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct CodeBlock {
    pub programming_language: ProgrammingLanguage,
    pub text: Text,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub struct Reference {
    #[rkyv(omit_bounds)]
    pub text: Text,
    #[rkyv(omit_bounds)]
    pub target: Target,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub enum Target {
    Place(#[rkyv(omit_bounds)] Place),
    Url(String),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Place(pub std::vec::Vec<Text>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Depth(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
#[rkyv(
    serialize_bounds(
        __S:rkyv::ser::Writer+rkyv::ser::Allocator,
        __S::Error:rkyv::rancor::Source
    )
)]
#[rkyv(deserialize_bounds(__D::Error:rkyv::rancor::Source))]
#[rkyv(
    bytecheck(
        bounds(__C:rkyv::validation::ArchiveContext, __C::Error:rkyv::rancor::Source)
    )
)]
pub struct Outline {
    pub title: Title,
    #[rkyv(omit_bounds)]
    pub outline_vector: std::vec::Vec<Outline>,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Kind {
    Headings,
    Statements,
    CodeBlocks,
    References,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Pattern(pub String);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Insert_Data {
    pub place: Place,
    pub block: Block,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Old(pub Block);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct New(pub Block);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Replace_Data {
    pub place: Place,
    pub old: Old,
    pub new: New,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Remove_Data {
    pub place: Place,
    pub block: Block,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Edit {
    Insert(Insert_Data),
    Replace(Replace_Data),
    Remove(Remove_Data),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Expected(pub Block);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Found(pub Block);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Mismatch_Data {
    pub place: Place,
    pub expected: Expected,
    pub found: Found,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum EditRefusal {
    NoSuchPlace(Place),
    Mismatch(Mismatch_Data),
}
#[rustfmt::skip]
pub trait Structural {
    fn outline(&self, input: Depth) -> std::vec::Vec<Outline>;
    fn code(&self, input: Place) -> std::vec::Vec<CodeBlock>;
    fn references(&self, input: Place) -> std::vec::Vec<Reference>;
    fn search(&self, input_0: Kind, input_1: Pattern) -> std::vec::Vec<Place>;
    fn edit(&mut self, input: Edit) -> std::result::Result<Document, EditRefusal>;
}
#[rustfmt::skip]
const _: () = {
    fn assert_document_structural<T: Structural>() {}
    let _ = assert_document_structural::<Document>;
};
