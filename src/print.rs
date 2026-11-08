//! A Document printed into Markdown. The Document is built into comrak's
//! CommonMark tree, and comrak prints the tree: a top section at `#`, one blank
//! line between blocks, and a newline ending the text.

use comrak::nodes::{
    AstNode, ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList, NodeTable,
    NodeValue, TableAlignment,
};
use comrak::{Arena, Options};

use crate::commonmark::Configured;
use crate::document::{Block, Depth, Document, Markdown, Place, Row, Target, Text, TextPart};
use crate::programming_language::Spelled;

pub trait Printable {
    fn print(&self) -> Markdown;
}

impl Printable for Document {
    fn print(&self) -> Markdown {
        let arena = Arena::new();
        let builder = Builder { arena: &arena };
        let root = builder.node(NodeValue::Document);
        for block in &self.0 {
            builder.block(root, block, Depth(1));
        }
        let mut out = String::new();
        comrak::format_commonmark(root, &Options::sema(), &mut out)
            .expect("formatting into a String cannot fail");
        Markdown(out)
    }
}

/// Builds comrak's tree from the Document, its nodes held by one arena.
struct Builder<'a> {
    arena: &'a Arena<'a>,
}

type Node<'a> = &'a AstNode<'a>;

trait Building<'a> {
    fn node(&self, value: NodeValue) -> Node<'a>;
    fn block(&self, parent: Node<'a>, block: &Block, depth: Depth);
    fn row(&self, table: Node<'a>, row: &Row, header: bool);
    fn text(&self, parent: Node<'a>, text: &Text);
    fn words(&self, parent: Node<'a>, words: &str);
}

/// A bullet list, tight, marked `-`.
trait Bullet {
    fn bullet() -> Self;
}

impl Bullet for NodeList {
    fn bullet() -> Self {
        NodeList {
            list_type: ListType::Bullet,
            bullet_char: b'-',
            tight: true,
            padding: 2,
            ..NodeList::default()
        }
    }
}

impl<'a> Building<'a> for Builder<'a> {
    fn node(&self, value: NodeValue) -> Node<'a> {
        self.arena.alloc(AstNode::from(value))
    }

    fn block(&self, parent: Node<'a>, block: &Block, depth: Depth) {
        match block {
            Block::Statement(text) => {
                let paragraph = self.node(NodeValue::Paragraph);
                self.text(paragraph, text);
                parent.append(paragraph);
            }
            Block::Section(section) => {
                let heading = self.node(NodeValue::Heading(NodeHeading {
                    level: depth.0 as u8,
                    setext: false,
                    closed: false,
                }));
                self.text(heading, &section.title.0);
                parent.append(heading);
                for inner in &section.block_vector {
                    self.block(parent, inner, Depth(depth.0 + 1));
                }
            }
            Block::CodeBlock(code) => {
                let mut literal = code.text.verbatim();
                if !literal.is_empty() {
                    literal.push('\n');
                }
                parent.append(self.node(NodeValue::CodeBlock(Box::new(NodeCodeBlock {
                    fenced: true,
                    fence_char: b'`',
                    fence_length: 3,
                    info: code.programming_language.spelling().to_string(),
                    literal,
                    closed: true,
                    ..NodeCodeBlock::default()
                }))));
            }
            Block::List(items) => {
                let list = self.node(NodeValue::List(NodeList::bullet()));
                for item in items {
                    let item_node = self.node(NodeValue::Item(NodeList::bullet()));
                    self.block(item_node, item, depth);
                    list.append(item_node);
                }
                parent.append(list);
            }
            Block::Quote(blocks) => {
                let quote = self.node(NodeValue::BlockQuote);
                for inner in blocks {
                    self.block(quote, inner, depth);
                }
                parent.append(quote);
            }
            Block::Table(rows) => {
                let columns = rows.first().map_or(0, |row| row.0.len());
                let table = self.node(NodeValue::Table(Box::new(NodeTable {
                    alignments: vec![TableAlignment::None; columns],
                    num_columns: columns,
                    num_rows: rows.len(),
                    num_nonempty_cells: 0,
                })));
                for (index, row) in rows.iter().enumerate() {
                    self.row(table, row, index == 0);
                }
                parent.append(table);
            }
        }
    }

    fn row(&self, table: Node<'a>, row: &Row, header: bool) {
        let row_node = self.node(NodeValue::TableRow(header));
        for cell in &row.0 {
            let cell_node = self.node(NodeValue::TableCell);
            self.text(cell_node, cell);
            row_node.append(cell_node);
        }
        table.append(row_node);
    }

    fn text(&self, parent: Node<'a>, text: &Text) {
        for text_part in &text.0 {
            match text_part {
                TextPart::Words(words) => self.words(parent, words),
                TextPart::InlineCode(code) => parent.append(self.node(NodeValue::Code(NodeCode {
                    num_backticks: 1,
                    literal: code.clone(),
                }))),
                TextPart::Emphasis(inner) => {
                    let emphasis = self.node(NodeValue::Emph);
                    self.text(emphasis, inner);
                    parent.append(emphasis);
                }
                TextPart::Strong(inner) => {
                    let strong = self.node(NodeValue::Strong);
                    self.text(strong, inner);
                    parent.append(strong);
                }
                TextPart::Reference(reference) => {
                    let url = match &reference.target {
                        Target::Url(url) => url.clone(),
                        Target::Place(place) => place.anchor(),
                    };
                    let link = self.node(NodeValue::Link(Box::new(NodeLink {
                        url,
                        title: String::new(),
                    })));
                    self.text(link, &reference.text);
                    parent.append(link);
                }
            }
        }
    }

    /// Words, a newline among them a soft line break, and a tag written in
    /// prose printed as written, as it was read.
    fn words(&self, parent: Node<'a>, words: &str) {
        for (index, line) in words.split('\n').enumerate() {
            if index > 0 {
                parent.append(self.node(NodeValue::SoftBreak));
            }
            for segment in line.segments() {
                let value = match segment {
                    Segment::Words(words) => NodeValue::Text(words.to_string().into()),
                    Segment::Tag(tag) => NodeValue::HtmlInline(tag.to_string()),
                };
                parent.append(self.node(value));
            }
        }
    }
}

/// A line of words split where a tag is written: `<` and a letter or `/`,
/// then anything but a bracket or a newline, then `>`.
enum Segment<'s> {
    Words(&'s str),
    Tag(&'s str),
}

trait Segmented {
    fn segments(&self) -> Vec<Segment<'_>>;
}

impl Segmented for str {
    fn segments(&self) -> Vec<Segment<'_>> {
        let mut segments = Vec::new();
        let mut start = 0;
        let mut cursor = 0;
        while let Some(offset) = self[cursor..].find('<') {
            let open = cursor + offset;
            let rest = &self[open + 1..];
            let tag_end = rest
                .chars()
                .next()
                .filter(|first| first.is_ascii_alphabetic() || *first == '/')
                .and_then(|_| rest.find(['<', '>']))
                .filter(|end| rest.as_bytes()[*end] == b'>');
            match tag_end {
                Some(end) => {
                    if start < open {
                        segments.push(Segment::Words(&self[start..open]));
                    }
                    let close = open + 1 + end + 1;
                    segments.push(Segment::Tag(&self[open..close]));
                    start = close;
                    cursor = close;
                }
                None => cursor = open + 1,
            }
        }
        if start < self.len() {
            segments.push(Segment::Words(&self[start..]));
        }
        segments
    }
}

trait Verbatim {
    /// The text as a fence holds it: its words unescaped.
    fn verbatim(&self) -> String;
}

impl Verbatim for Text {
    fn verbatim(&self) -> String {
        self.0
            .iter()
            .map(|text_part| match text_part {
                TextPart::Words(words) | TextPart::InlineCode(words) => words.clone(),
                TextPart::Emphasis(inner) | TextPart::Strong(inner) => inner.verbatim(),
                TextPart::Reference(reference) => reference.text.verbatim(),
            })
            .collect()
    }
}

/// UNIMPLEMENTED until designed (Fable ruling, inferred, pending the living).
/// A Markdown link target, anchors included, is read and printed as
/// `Target.Url`, the string as written; a Place is never composed from
/// Markdown. A Place is what Sema itself yields, in search results and edits,
/// named by titles from the root, and the conversion from a Place to an
/// anchor is not designed, so printing one stops here instead of guessing.
trait AnchorConversion {
    fn anchor(&self) -> String;
}

impl AnchorConversion for Place {
    fn anchor(&self) -> String {
        unimplemented!("Place-to-anchor conversion is not designed in Sema v0")
    }
}
