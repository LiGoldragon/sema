#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct InlineMarkdown(pub String);
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
pub struct RenderedOutline {
    pub inline_markdown: InlineMarkdown,
    #[rkyv(omit_bounds)]
    pub rendered_outline_vector: std::vec::Vec<RenderedOutline>,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct RenderedPlace(pub std::vec::Vec<InlineMarkdown>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct MarkdownBlock(pub String);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct RenderedHit {
    pub rendered_place: RenderedPlace,
    pub markdown_block: MarkdownBlock,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum PlaceRefusal {
    NoSuchSection(RenderedPlace),
    AmbiguousSection(RenderedPlace),
}
#[rustfmt::skip]
pub trait InlineRenderable {
    fn inline(&self) -> InlineMarkdown;
}
#[rustfmt::skip]
pub trait OutlineRenderable {
    fn rendered_outline(&self) -> RenderedOutline;
}
#[rustfmt::skip]
pub trait PlaceRenderable {
    fn rendered_place(&self) -> RenderedPlace;
}
#[rustfmt::skip]
pub trait HitRenderable {
    fn rendered_hit(&self) -> RenderedHit;
}
#[rustfmt::skip]
pub trait PlaceResolvable {
    fn resolved(
        &self,
        input: RenderedPlace,
    ) -> std::result::Result<sema::Place, PlaceRefusal>;
}
#[rustfmt::skip]
const _: () = {
    fn assert_text_inlinerenderable<T: InlineRenderable>() {}
    let _ = assert_text_inlinerenderable::<sema::Text>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_outline_outlinerenderable<T: OutlineRenderable>() {}
    let _ = assert_outline_outlinerenderable::<sema::Outline>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_place_placerenderable<T: PlaceRenderable>() {}
    let _ = assert_place_placerenderable::<sema::Place>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_hit_hitrenderable<T: HitRenderable>() {}
    let _ = assert_hit_hitrenderable::<sema::Hit>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_document_placeresolvable<T: PlaceResolvable>() {}
    let _ = assert_document_placeresolvable::<sema::Document>;
};
