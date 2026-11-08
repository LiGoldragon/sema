//! Markdown composed into a Document. comrak reads the text into its
//! CommonMark tree; each node of that tree reads itself into the Document,
//! and a construct the Document has no place for is refused at the line it
//! starts on, which the source then quotes.

use comrak::nodes::{AstNode, ListType, NodeCode, NodeHeading, NodeValue, TableAlignment};
use comrak::{Arena, Options};

use crate::reading::{
    Configurable, Heading_Data, Nestable, OpenSection, Piece, Pieces, Quotable, Readable,
    SourceLines, Spellable,
};
use crate::{
    Block, CodeBlock, Composable, Depth, Document, Line, Markdown, Problem, ProgrammingLanguage,
    ReadRefusal, Reference, Row, Section, Target, Text, TextPart, Title,
};

impl Composable for Markdown {
    fn compose(&self) -> Result<Document, ReadRefusal> {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &self.0, &Options::sema());
        let source = SourceLines(self.0.split('\n').map(str::to_string).collect());
        root.pieces()
            .and_then(|pieces| Pieces(pieces).nested())
            .map(Document)
            .map_err(|line| source.quote(line.0))
    }
}

impl<'a> Readable for &'a AstNode<'a> {
    fn pieces(&self) -> Result<Vec<Piece>, Line> {
        (*self).children().map(|node| node.piece()).collect()
    }

    fn piece(&self) -> Result<Piece, Line> {
        let value = self.data.borrow().value.clone();
        let block = match value {
            NodeValue::Heading(NodeHeading { level, .. }) => {
                return Ok(Piece::Heading(Heading_Data {
                    depth: Depth(i64::from(level)),
                    title: Title(self.text()?),
                    line: self.line(),
                }));
            }
            NodeValue::Paragraph => Block::Statement(self.text()?),
            NodeValue::CodeBlock(code) => {
                let programming_language =
                    ProgrammingLanguage::spelled(code.info.clone()).ok_or_else(|| self.line())?;
                let literal = code.literal.strip_suffix('\n').unwrap_or(&code.literal);
                let text = match literal.is_empty() {
                    true => Text(Vec::new()),
                    false => Text(vec![TextPart::Words(literal.to_string())]),
                };
                Block::CodeBlock(CodeBlock {
                    programming_language,
                    text,
                })
            }
            NodeValue::List(list) if list.list_type == ListType::Bullet => self.list()?,
            NodeValue::BlockQuote => Block::Quote(self.contained()?),
            NodeValue::Table(_) => self.table()?,
            _ => return Err(self.line()),
        };
        Ok(Piece::Block(block))
    }

    fn contained(&self) -> Result<Vec<Block>, Line> {
        self.pieces()?
            .into_iter()
            .map(|piece| match piece {
                Piece::Block(block) => Ok(block),
                Piece::Heading(heading) => Err(heading.line),
            })
            .collect()
    }

    fn list(&self) -> Result<Block, Line> {
        let items = (*self)
            .children()
            .map(|item| {
                let mut blocks = item.contained()?;
                match blocks.len() {
                    1 => Ok(blocks.remove(0)),
                    _ => Err(item.line()),
                }
            })
            .collect::<Result<Vec<Block>, Line>>()?;
        Ok(Block::List(items))
    }

    /// A table without alignment: its rows, the header first.
    fn table(&self) -> Result<Block, Line> {
        if let NodeValue::Table(shape) = &self.data.borrow().value
            && shape.alignments.iter().any(|a| *a != TableAlignment::None)
        {
            return Err(self.line());
        }
        let rows = (*self)
            .children()
            .map(|row| {
                row.children()
                    .map(|cell| cell.text())
                    .collect::<Result<Vec<Text>, Line>>()
                    .map(Row)
            })
            .collect::<Result<Vec<Row>, Line>>()?;
        Ok(Block::Table(rows))
    }

    /// Adjacent words are one TextPart, and a soft line break is a newline
    /// among them.
    fn text(&self) -> Result<Text, Line> {
        let mut text_parts: Vec<TextPart> = Vec::new();
        for node in (*self).children() {
            match (text_parts.last_mut(), node.text_part()?) {
                (Some(TextPart::Words(last)), TextPart::Words(more)) => last.push_str(&more),
                (_, text_part) => text_parts.push(text_part),
            }
        }
        Ok(Text(text_parts))
    }

    fn text_part(&self) -> Result<TextPart, Line> {
        let value = self.data.borrow().value.clone();
        Ok(match value {
            NodeValue::Text(words) => TextPart::Words(words.to_string()),
            NodeValue::SoftBreak => TextPart::Words("\n".to_string()),
            NodeValue::Code(NodeCode { literal, .. }) => TextPart::InlineCode(literal),
            NodeValue::Emph => TextPart::Emphasis(self.text()?),
            NodeValue::Strong => TextPart::Strong(self.text()?),
            NodeValue::Link(link) if link.title.is_empty() => TextPart::Reference(Reference {
                text: self.text()?,
                target: Target::Url(link.url.clone()),
            }),
            _ => return Err(self.line()),
        })
    }

    fn line(&self) -> Line {
        Line(self.data.borrow().sourcepos.start.line as i64)
    }
}

impl Nestable for Pieces {
    /// Depth is the nesting, read relative: the shallowest heading in the
    /// document is the top and each deeper level a child.
    fn nested(&self) -> Result<Vec<Block>, Line> {
        let mut root = Vec::new();
        let mut open: Vec<OpenSection> = Vec::new();
        let close = |open: &mut Vec<OpenSection>, root: &mut Vec<Block>| {
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
        let top = self
            .0
            .iter()
            .filter_map(|piece| match piece {
                Piece::Heading(heading) => Some(heading.depth),
                Piece::Block(_) => None,
            })
            .min()
            .unwrap_or(Depth(1));
        for piece in self.0.iter().cloned() {
            match piece {
                Piece::Block(block) => match open.last_mut() {
                    Some(section) => section.block_vector.push(block),
                    None => root.push(block),
                },
                Piece::Heading(Heading_Data {
                    depth: written,
                    title,
                    line,
                }) => {
                    let depth = Depth(written.0 - top.0 + 1);
                    while open.last().is_some_and(|section| section.depth >= depth) {
                        close(&mut open, &mut root);
                    }
                    let above = open.last().map_or(Depth(0), |section| section.depth);
                    if depth.0 != above.0 + 1 {
                        return Err(line);
                    }
                    open.push(OpenSection {
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
}

impl Quotable for SourceLines {
    fn quote(&self, line: i64) -> ReadRefusal {
        let written = usize::try_from(line - 1)
            .ok()
            .and_then(|index| self.0.get(index))
            .cloned()
            .unwrap_or_default();
        ReadRefusal {
            line: Line(line),
            problem: Problem::Unsupported(written),
        }
    }
}

impl std::fmt::Display for ReadRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.problem {
            Problem::Unsupported(written) => {
                write!(formatter, "line {}: Unsupported «{written}»", self.line.0)
            }
            Problem::Missing(field) => write!(formatter, "line {}: Missing {field:?}", self.line.0),
        }
    }
}

impl std::error::Error for ReadRefusal {}
