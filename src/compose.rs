//! Markdown composed into a Document. comrak reads the text into its
//! CommonMark tree; that tree is walked into the Document, and a construct the
//! Document has no place for is refused with the line it starts on.

use comrak::nodes::{AstNode, ListType, NodeCode, NodeHeading, NodeValue, TableAlignment};
use comrak::{Arena, Options};

use crate::commonmark::Configured;
use crate::document::{
    Block, Code, Depth, Document, Markdown, ProgrammingLanguage, Reference, Row, Section, Target,
    Text, TextPart, Title,
};
use crate::error::{Line, Problem, ReadRefusal};

pub trait Composable {
    fn compose(&self) -> Result<Document, ReadRefusal>;
}

impl Composable for Markdown {
    fn compose(&self) -> Result<Document, ReadRefusal> {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &self.0, &Options::sema());
        let reader = Reader {
            source: self.0.split('\n').map(str::to_string).collect(),
        };
        let pieces = reader.pieces(root)?;
        reader.nested(pieces).map(Document)
    }
}

/// A heading or a block, before headings are nested into sections.
enum Piece {
    Heading(Depth, Title, Line),
    Block(Block),
}

/// A section still open while its heading's blocks are read.
struct Open {
    depth: Depth,
    title: Title,
    block_vector: Vec<Block>,
}

/// The walk from comrak's tree into the Document, holding the text as written
/// for what a refusal carries.
struct Reader {
    source: Vec<String>,
}

type Node<'a> = &'a AstNode<'a>;

trait Reading {
    fn pieces<'a>(&self, container: Node<'a>) -> Result<Vec<Piece>, ReadRefusal>;
    fn piece<'a>(&self, node: Node<'a>) -> Result<Piece, ReadRefusal>;
    fn nested(&self, pieces: Vec<Piece>) -> Result<Vec<Block>, ReadRefusal>;
    /// The blocks of a list item or a quote, where a heading has no place.
    fn contained<'a>(&self, container: Node<'a>) -> Result<Vec<Block>, ReadRefusal>;
    fn list<'a>(&self, list: Node<'a>) -> Result<Block, ReadRefusal>;
    fn table<'a>(&self, table: Node<'a>) -> Result<Block, ReadRefusal>;
    fn text<'a>(&self, container: Node<'a>) -> Result<Text, ReadRefusal>;
    fn text_part<'a>(&self, node: Node<'a>) -> Result<TextPart, ReadRefusal>;
    fn refused<'a>(&self, node: Node<'a>) -> ReadRefusal;
}

trait Located {
    fn line(&self) -> Line;
}

impl Located for AstNode<'_> {
    fn line(&self) -> Line {
        Line(self.data.borrow().sourcepos.start.line as i64)
    }
}

impl Reading for Reader {
    fn pieces<'a>(&self, container: Node<'a>) -> Result<Vec<Piece>, ReadRefusal> {
        container.children().map(|node| self.piece(node)).collect()
    }

    fn piece<'a>(&self, node: Node<'a>) -> Result<Piece, ReadRefusal> {
        let value = node.data.borrow().value.clone();
        let block = match value {
            NodeValue::Heading(NodeHeading { level, .. }) => {
                return Ok(Piece::Heading(
                    Depth(level as usize),
                    Title(self.text(node)?),
                    node.line(),
                ));
            }
            NodeValue::Paragraph => Block::Statement(self.text(node)?),
            NodeValue::CodeBlock(code) => {
                let programming_language = ProgrammingLanguage::try_from(code.info.as_str())
                    .map_err(|_| self.refused(node))?;
                let literal = code.literal.strip_suffix('\n').unwrap_or(&code.literal);
                let text = match literal.is_empty() {
                    true => Text(Vec::new()),
                    false => Text(vec![TextPart::Words(literal.to_string())]),
                };
                Block::Code(Code {
                    programming_language,
                    text,
                })
            }
            NodeValue::List(list) if list.list_type == ListType::Bullet => self.list(node)?,
            NodeValue::BlockQuote => Block::Quote(self.contained(node)?),
            NodeValue::Table(_) => self.table(node)?,
            _ => return Err(self.refused(node)),
        };
        Ok(Piece::Block(block))
    }

    /// Depth is the nesting, read relative: the shallowest heading in the
    /// document is the top and each deeper level a child.
    fn nested(&self, pieces: Vec<Piece>) -> Result<Vec<Block>, ReadRefusal> {
        let mut root = Vec::new();
        let mut open: Vec<Open> = Vec::new();
        let close = |open: &mut Vec<Open>, root: &mut Vec<Block>| {
            if let Some(section) = open.pop() {
                let block = Block::Section(Section {
                    title: section.title,
                    block_vector: section.block_vector,
                });
                match open.last_mut() {
                    Some(parent) => parent.block_vector.push(block),
                    None => root.push(block),
                }
            }
        };
        let top = pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Heading(written, _, _) => Some(*written),
                Piece::Block(_) => None,
            })
            .min()
            .unwrap_or(Depth(1));
        for piece in pieces {
            match piece {
                Piece::Block(block) => match open.last_mut() {
                    Some(section) => section.block_vector.push(block),
                    None => root.push(block),
                },
                Piece::Heading(written, title, line) => {
                    let depth = Depth(written.0 - top.0 + 1);
                    while open.last().is_some_and(|section| section.depth >= depth) {
                        close(&mut open, &mut root);
                    }
                    let above = open.last().map_or(Depth(0), |section| section.depth);
                    if depth.0 != above.0 + 1 {
                        return Err(line.refusal(&self.source));
                    }
                    open.push(Open {
                        depth,
                        title,
                        block_vector: Vec::new(),
                    });
                }
            }
        }
        while !open.is_empty() {
            close(&mut open, &mut root);
        }
        Ok(root)
    }

    fn contained<'a>(&self, container: Node<'a>) -> Result<Vec<Block>, ReadRefusal> {
        self.pieces(container)?
            .into_iter()
            .map(|piece| match piece {
                Piece::Block(block) => Ok(block),
                Piece::Heading(_, _, line) => Err(line.refusal(&self.source)),
            })
            .collect()
    }

    /// A list item holds one block.
    fn list<'a>(&self, list: Node<'a>) -> Result<Block, ReadRefusal> {
        let items = list
            .children()
            .map(|item| {
                let mut blocks = self.contained(item)?;
                match blocks.len() {
                    1 => Ok(blocks.remove(0)),
                    _ => Err(self.refused(item)),
                }
            })
            .collect::<Result<Vec<Block>, ReadRefusal>>()?;
        Ok(Block::List(items))
    }

    /// A table without alignment: its rows, the header first.
    fn table<'a>(&self, table: Node<'a>) -> Result<Block, ReadRefusal> {
        if let NodeValue::Table(shape) = &table.data.borrow().value
            && shape.alignments.iter().any(|a| *a != TableAlignment::None)
        {
            return Err(self.refused(table));
        }
        let rows = table
            .children()
            .map(|row| {
                row.children()
                    .map(|cell| self.text(cell))
                    .collect::<Result<Vec<Text>, ReadRefusal>>()
                    .map(Row)
            })
            .collect::<Result<Vec<Row>, ReadRefusal>>()?;
        Ok(Block::Table(rows))
    }

    /// The inside of a paragraph, a heading or a cell; adjacent words are one
    /// TextPart, and a soft line break is a newline among them.
    fn text<'a>(&self, container: Node<'a>) -> Result<Text, ReadRefusal> {
        let mut text_parts: Vec<TextPart> = Vec::new();
        for node in container.children() {
            match (text_parts.last_mut(), self.text_part(node)?) {
                (Some(TextPart::Words(last)), TextPart::Words(more)) => last.push_str(&more),
                (_, text_part) => text_parts.push(text_part),
            }
        }
        Ok(Text(text_parts))
    }

    fn text_part<'a>(&self, node: Node<'a>) -> Result<TextPart, ReadRefusal> {
        let value = node.data.borrow().value.clone();
        Ok(match value {
            NodeValue::Text(words) => TextPart::Words(words.to_string()),
            NodeValue::SoftBreak => TextPart::Words("\n".to_string()),
            NodeValue::Code(NodeCode { literal, .. }) => TextPart::Code(literal),
            NodeValue::Emph => TextPart::Emphasis(self.text(node)?),
            NodeValue::Strong => TextPart::Strong(self.text(node)?),
            NodeValue::Link(link) if link.title.is_empty() => TextPart::Reference(Reference {
                text: self.text(node)?,
                target: Target::Url(link.url.clone()),
            }),
            _ => return Err(self.refused(node)),
        })
    }

    fn refused<'a>(&self, node: Node<'a>) -> ReadRefusal {
        node.line().refusal(&self.source)
    }
}

/// A refusal carrying the line as written.
pub(crate) trait Refusing {
    fn refusal(&self, source: &[String]) -> ReadRefusal;
}

impl Refusing for Line {
    fn refusal(&self, source: &[String]) -> ReadRefusal {
        let written = usize::try_from(self.0 - 1)
            .ok()
            .and_then(|index| source.get(index))
            .cloned()
            .unwrap_or_default();
        ReadRefusal {
            line: *self,
            problem: Problem::Unsupported(written),
        }
    }
}
