#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Markdown(pub String);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Line(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ReadRefusal {
    pub line: Line,
    pub problem: Problem,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Problem {
    Unsupported(String),
    Missing(FrontMatterField),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum FrontMatterField {
    Description,
    Dependencies,
}
#[rustfmt::skip]
pub trait Composable {
    fn compose(&self) -> std::result::Result<sema::Document, ReadRefusal>;
}
#[rustfmt::skip]
pub trait Printable {
    fn print(&self) -> Markdown;
}
#[rustfmt::skip]
const _: () = {
    fn assert_markdown_composable<T: Composable>() {}
    let _ = assert_markdown_composable::<Markdown>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_document_printable<T: Printable>() {}
    let _ = assert_document_printable::<sema::Document>;
};
