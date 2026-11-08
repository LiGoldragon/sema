//! The translator's forms of a listing: a Text is shown as its Markdown inline
//! string with the marks kept, since Markdown is the translator's language,
//! and a place typed by a reader is found among the headings by their plain
//! words.

use crate::reading::Searchable;
use crate::translation::{
    HitRenderable, InlineMarkdown, InlineRenderable, MarkdownBlock, OutlineRenderable,
    PlaceRefusal, PlaceRenderable, PlaceResolvable, RenderedHit, RenderedOutline, RenderedPlace,
};
use crate::{Block, Composable, Document, Hit, Markdown, Outline, Place, Printable, Text};

impl InlineRenderable for Text {
    /// The text printed as a paragraph of its own, without the line end.
    fn inline(&self) -> InlineMarkdown {
        let printed = Document(vec![Block::Statement(self.clone())]).print().0;
        InlineMarkdown(printed.trim_end_matches('\n').to_string())
    }
}

impl OutlineRenderable for Outline {
    fn rendered_outline(&self) -> RenderedOutline {
        RenderedOutline {
            inline_markdown: self.title.0.inline(),
            rendered_outline_vector: self
                .outline_vector
                .iter()
                .map(Outline::rendered_outline)
                .collect(),
        }
    }
}

impl PlaceRenderable for Place {
    fn rendered_place(&self) -> RenderedPlace {
        RenderedPlace(self.0.iter().map(Text::inline).collect())
    }
}

impl HitRenderable for Hit {
    /// Its place, and its block printed as Markdown, without the last line end.
    fn rendered_hit(&self) -> RenderedHit {
        let printed = Document(vec![self.block.clone()]).print().0;
        RenderedHit {
            rendered_place: self.place.rendered_place(),
            markdown_block: MarkdownBlock(printed.trim_end_matches('\n').to_string()),
        }
    }
}

impl PlaceResolvable for Document {
    /// Each typed title is read as inline Markdown and matched by its plain
    /// words against the plain words of the headings at that level; exactly
    /// one section is descended into. A title that is not one paragraph of
    /// inline Markdown matches no heading.
    fn resolved(&self, input: RenderedPlace) -> Result<Place, PlaceRefusal> {
        let mut blocks = &self.0;
        let mut titles = Vec::new();
        for (index, typed) in input.0.iter().enumerate() {
            let so_far = || RenderedPlace(input.0[..=index].to_vec());
            let words = match Markdown(typed.0.clone()).compose() {
                Ok(Document(read)) => match read.as_slice() {
                    [Block::Statement(text)] => text.plain(),
                    _ => return Err(PlaceRefusal::NoSuchSection(so_far())),
                },
                Err(_) => return Err(PlaceRefusal::NoSuchSection(so_far())),
            };
            let matching: Vec<_> = blocks
                .iter()
                .filter_map(|block| match block {
                    Block::Section(section) if section.title.0.plain() == words => Some(section),
                    _ => None,
                })
                .collect();
            let section = match matching.as_slice() {
                [section] => *section,
                [] => return Err(PlaceRefusal::NoSuchSection(so_far())),
                _ => return Err(PlaceRefusal::AmbiguousSection(so_far())),
            };
            titles.push(section.title.0.clone());
            blocks = &section.block_vector;
        }
        Ok(Place(titles))
    }
}
