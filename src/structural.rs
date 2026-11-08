//! What is asked of a document, by the titles above its blocks.
//!
//! Implemented: outline, code, references, search. Not yet: edit, whose
//! Mismatch names a Found block that no rule yet picks; it stops.

use crate::{
    Block, CodeBlock, Depth, Document, Edit, EditRefusal, Kind, Outline, Pattern, Place, Reference,
    Section, Structural, Target, Text, TextPart,
};

impl Structural for Document {
    fn outline(&self, depth: Depth) -> Vec<Outline> {
        self.0.as_slice().outlined(depth)
    }

    fn code(&self, place: Place) -> Vec<CodeBlock> {
        self.placed()
            .into_iter()
            .filter(|(above, _)| above.under(&place))
            .filter_map(|(_, block)| match block {
                Block::CodeBlock(code) => Some(code.clone()),
                _ => None,
            })
            .collect()
    }

    fn references(&self, place: Place) -> Vec<Reference> {
        self.placed()
            .into_iter()
            .filter(|(above, _)| above.under(&place))
            .flat_map(|(_, block)| block.own_texts())
            .flat_map(|text| text.references())
            .collect()
    }

    fn search(&self, kind: Kind, pattern: Pattern) -> Vec<Place> {
        let holds = |text: String| text.contains(&pattern.0);
        let mut places = Vec::new();
        for (above, block) in self.placed() {
            match (&kind, block) {
                (Kind::Headings, Block::Section(section)) if holds(section.title.0.plain()) => {
                    places.push(above.within(&section.title.0));
                }
                (Kind::Statements, Block::Statement(text)) if holds(text.plain()) => {
                    places.push(above);
                }
                (Kind::CodeBlocks, Block::CodeBlock(code)) if holds(code.text.plain()) => {
                    places.push(above)
                }
                (Kind::References, block) => {
                    let found = block
                        .own_texts()
                        .into_iter()
                        .flat_map(|text| text.references())
                        .any(|reference| {
                            let target = match &reference.target {
                                Target::Url(url) => url.clone(),
                                Target::Place(_) => String::new(),
                            };
                            holds(reference.text.plain()) || holds(target)
                        });
                    if found {
                        places.push(above);
                    }
                }
                _ => {}
            }
        }
        places
    }

    /// UNIMPLEMENTED: a Mismatch names a Found block, and no rule yet says
    /// which block is found when no block at the place equals the expected one.
    fn edit(&mut self, _edit: Edit) -> Result<Document, EditRefusal> {
        unimplemented!("which block a Mismatch names as Found is not designed in Sema v0")
    }
}

/// Every block of the document with the titles above it, in reading order.
trait Placing {
    fn placed(&self) -> Vec<(Place, &Block)>;
}

impl Placing for Document {
    fn placed(&self) -> Vec<(Place, &Block)> {
        let mut out = Vec::new();
        let mut pending: Vec<(Place, &Block)> = self
            .0
            .iter()
            .rev()
            .map(|block| (Place(Vec::new()), block))
            .collect();
        while let Some((above, block)) = pending.pop() {
            let children: Vec<(Place, &Block)> = match block {
                Block::Section(section) => {
                    let inner = above.within(&section.title.0);
                    section
                        .block_vector
                        .iter()
                        .map(|child| (inner.clone(), child))
                        .collect()
                }
                Block::List(blocks) | Block::Quote(blocks) => {
                    blocks.iter().map(|child| (above.clone(), child)).collect()
                }
                _ => Vec::new(),
            };
            pending.extend(children.into_iter().rev());
            out.push((above, block));
        }
        out
    }
}

trait PlaceRelation {
    fn under(&self, place: &Place) -> bool;
    fn within(&self, title: &Text) -> Place;
}

impl PlaceRelation for Place {
    fn under(&self, place: &Place) -> bool {
        self.0.starts_with(&place.0)
    }

    fn within(&self, title: &Text) -> Place {
        let mut titles = self.0.clone();
        titles.push(title.clone());
        Place(titles)
    }
}

/// The texts a block holds itself, apart from the blocks inside it.
trait OwnTexts {
    fn own_texts(&self) -> Vec<&Text>;
}

impl OwnTexts for Block {
    fn own_texts(&self) -> Vec<&Text> {
        match self {
            Block::Statement(text) => vec![text],
            Block::Section(section) => vec![&section.title.0],
            Block::Table(rows) => rows.iter().flat_map(|row| row.0.iter()).collect(),
            Block::CodeBlock(_) | Block::List(_) | Block::Quote(_) => Vec::new(),
        }
    }
}

trait TextReading {
    /// The words of a text without its markup.
    fn plain(&self) -> String;
    fn references(&self) -> Vec<Reference>;
}

impl TextReading for Text {
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

trait Outlining {
    fn outlined(&self, depth: Depth) -> Vec<Outline>;
}

impl Outlining for [Block] {
    fn outlined(&self, depth: Depth) -> Vec<Outline> {
        match depth.0 {
            0 => Vec::new(),
            _ => self
                .iter()
                .filter_map(|block| match block {
                    Block::Section(Section {
                        title,
                        block_vector,
                    }) => Some(Outline {
                        title: title.clone(),
                        outline_vector: block_vector.as_slice().outlined(Depth(depth.0 - 1)),
                    }),
                    _ => None,
                })
                .collect(),
        }
    }
}
