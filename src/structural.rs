//! What is asked of a document, by the titles above its blocks.
//!
//! A Place names the section whose titles, from the root, it holds: the block
//! that stands at a Place is that section. A search yields a heading's place
//! ending in its own title, and a statement's place as the titles above it.

use crate::reading::{
    Editable, Outlinable, PendingEdit, Placeable, PlacedBlock, Relatable, Remaining, Searchable,
    Textual,
};
use crate::{
    Block, CodeBlock, Depth, Document, Edit, EditRefusal, Expected, Found, Kind, Mismatch_Data,
    Outline, Pattern, Place, PlainTitle, Reference, Section, Structural, Target, Text, TextPart,
    Url,
};

impl Structural for Document {
    /// One outline per top-level section, to the depth; empty when there is none.
    fn outline(&self, depth: Depth) -> Vec<Outline> {
        self.0.outlined(depth.0)
    }

    /// The code blocks under a section, its subsections included.
    fn code(&self, place: Place) -> Vec<CodeBlock> {
        self.placed()
            .into_iter()
            .filter(|placed| placed.place.under(place.0.clone()))
            .filter_map(|placed| match placed.block {
                Block::CodeBlock(code) => Some(code),
                _ => None,
            })
            .collect()
    }

    /// The links under a section, its subsections and titles included.
    fn references(&self, place: Place) -> Vec<Reference> {
        self.placed()
            .into_iter()
            .filter(|placed| placed.place.under(place.0.clone()))
            .flat_map(|placed| placed.block.own_texts())
            .flat_map(|text| text.references())
            .collect()
    }

    /// The place of each block of the kind whose text holds the pattern.
    fn search(&self, kind: Kind, pattern: Pattern) -> Vec<Place> {
        let holds = |text: String| text.contains(&pattern.0);
        let mut places = Vec::new();
        for PlacedBlock { place, block } in self.placed() {
            match (&kind, &block) {
                (Kind::Headings, Block::Section(section)) if holds(section.title.0.plain()) => {
                    places.push(place.within(vec![section.title.0.plain()]));
                }
                (Kind::Statements, Block::Statement(text)) if holds(text.plain()) => {
                    places.push(place);
                }
                (Kind::CodeBlocks, Block::CodeBlock(code)) if holds(code.text.plain()) => {
                    places.push(place)
                }
                (Kind::References, block) => {
                    let found = block
                        .own_texts()
                        .into_iter()
                        .flat_map(|text| text.references())
                        .any(|reference| {
                            let Target(Url(target)) = &reference.target;
                            holds(reference.text.plain()) || holds(target.clone())
                        });
                    if found {
                        places.push(place);
                    }
                }
                _ => {}
            }
        }
        places
    }

    /// An edit lands on a copy, and the document becomes it only when the
    /// edit is not refused.
    fn edit(&mut self, edit: Edit) -> Result<Document, EditRefusal> {
        let place = match &edit {
            Edit::Insert(insert) => insert.place.clone(),
            Edit::Replace(replace) => replace.place.clone(),
            Edit::Remove(remove) => remove.place.clone(),
        };
        let mut blocks = self.0.clone();
        match blocks.apply(PendingEdit {
            edit,
            remaining: Remaining(place.0),
        }) {
            Some(refusal) => Err(refusal),
            None => {
                self.0 = blocks;
                Ok(self.clone())
            }
        }
    }
}

impl Editable for Vec<Block> {
    /// The first section titled as the next remaining title is descended
    /// into; at the last title it is the block at the place. Insert appends
    /// to that section, or to these blocks when no title remains; Replace and
    /// Remove act on that section when it is the one expected.
    fn apply(&mut self, pending: PendingEdit) -> Option<EditRefusal> {
        let PendingEdit { edit, remaining } = pending;
        let place = match &edit {
            Edit::Insert(insert) => insert.place.clone(),
            Edit::Replace(replace) => replace.place.clone(),
            Edit::Remove(remove) => remove.place.clone(),
        };
        let mut titles = remaining.0;
        if titles.is_empty() {
            return match edit {
                Edit::Insert(insert) => {
                    self.push(insert.block);
                    None
                }
                Edit::Replace(_) | Edit::Remove(_) => Some(EditRefusal::NoSuchPlace(place)),
            };
        }
        let title = titles.remove(0);
        let Some(index) = self.iter().position(
            |block| matches!(block, Block::Section(section) if section.title.0.plain() == title),
        ) else {
            return Some(EditRefusal::NoSuchPlace(place));
        };
        let at_place = titles.is_empty();
        match (edit, at_place) {
            (Edit::Replace(replace), true) => match self[index] == replace.old.0 {
                true => {
                    self[index] = replace.new.0;
                    None
                }
                false => Some(EditRefusal::Mismatch(Mismatch_Data {
                    place,
                    expected: Expected(replace.old.0),
                    found: Found(self[index].clone()),
                })),
            },
            (Edit::Remove(remove), true) => match self[index] == remove.block {
                true => {
                    self.remove(index);
                    None
                }
                false => Some(EditRefusal::Mismatch(Mismatch_Data {
                    place,
                    expected: Expected(remove.block),
                    found: Found(self[index].clone()),
                })),
            },
            (edit, _) => match &mut self[index] {
                Block::Section(section) => section.block_vector.apply(PendingEdit {
                    edit,
                    remaining: Remaining(titles),
                }),
                _ => Some(EditRefusal::NoSuchPlace(place)),
            },
        }
    }
}

impl Placeable for Document {
    /// Every block of the document with the titles above it, in reading order.
    fn placed(&self) -> Vec<PlacedBlock> {
        let mut out = Vec::new();
        let mut pending: Vec<PlacedBlock> = self
            .0
            .iter()
            .rev()
            .map(|block| PlacedBlock {
                place: Place(Vec::new()),
                block: block.clone(),
            })
            .collect();
        while let Some(placed) = pending.pop() {
            let children: Vec<PlacedBlock> = match &placed.block {
                Block::Section(section) => {
                    let inner = placed.place.within(vec![section.title.0.plain()]);
                    section
                        .block_vector
                        .iter()
                        .map(|child| PlacedBlock {
                            place: inner.clone(),
                            block: child.clone(),
                        })
                        .collect()
                }
                Block::List(blocks) | Block::Quote(blocks) => blocks
                    .iter()
                    .map(|child| PlacedBlock {
                        place: placed.place.clone(),
                        block: child.clone(),
                    })
                    .collect(),
                _ => Vec::new(),
            };
            pending.extend(children.into_iter().rev());
            out.push(placed);
        }
        out
    }
}

impl Relatable for Place {
    fn under(&self, titles: Vec<String>) -> bool {
        self.0.starts_with(&titles)
    }

    fn within(&self, titles: Vec<String>) -> Place {
        let mut within = self.0.clone();
        within.extend(titles);
        Place(within)
    }
}

impl Textual for Block {
    /// The texts a block holds itself, apart from the blocks inside it.
    fn own_texts(&self) -> Vec<Text> {
        match self {
            Block::Statement(text) => vec![text.clone()],
            Block::Section(section) => vec![section.title.0.clone()],
            Block::Table(rows) => rows.iter().flat_map(|row| row.0.clone()).collect(),
            Block::CodeBlock(_) | Block::List(_) | Block::Quote(_) => Vec::new(),
        }
    }
}

impl Searchable for Text {
    /// The words of a text without its markup.
    fn plain(&self) -> String {
        self.0
            .iter()
            .map(|text_part| match text_part {
                TextPart::Words(words) | TextPart::InlineCode(words) => words.clone(),
                TextPart::Emphasis(text) | TextPart::Strong(text) => text.plain(),
                TextPart::Reference(reference) => reference.text.plain(),
            })
            .collect()
    }

    fn references(&self) -> Vec<Reference> {
        self.0
            .iter()
            .flat_map(|text_part| match text_part {
                TextPart::Reference(reference) => {
                    let mut found = vec![reference.clone()];
                    found.extend(reference.text.references());
                    found
                }
                TextPart::Emphasis(text) | TextPart::Strong(text) => text.references(),
                TextPart::Words(_) | TextPart::InlineCode(_) => Vec::new(),
            })
            .collect()
    }
}

impl Outlinable for Vec<Block> {
    fn outlined(&self, depth: i64) -> Vec<Outline> {
        match depth {
            0 => Vec::new(),
            _ => self
                .iter()
                .filter_map(|block| match block {
                    Block::Section(Section {
                        title,
                        block_vector,
                    }) => Some(Outline {
                        plain_title: PlainTitle(title.0.plain()),
                        outline_vector: block_vector.outlined(depth - 1),
                    }),
                    _ => None,
                })
                .collect(),
        }
    }
}
