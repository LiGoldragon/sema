//! A Document printed into Markdown. The Document is built into comrak's
//! CommonMark tree, and comrak prints the tree: a top section at `#`, one blank
//! line between blocks, and a newline ending the text.

use comrak::nodes::{
    AstNode, ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList, NodeTable,
    NodeValue, TableAlignment,
};
use comrak::{Arena, Options};

use crate::reading::{
    Bulleted, Configurable, Fenceable, Header, Parent, PrintStep, PrintTask, Row_Data, Spellable,
};
use crate::{Block, Depth, Document, Markdown, Printable, Target, Text, TextPart, Url};

impl Printable for Document {
    /// The tree is built step by step: a step appends its node under its
    /// parent at once, so siblings keep their order, and leaves what it holds
    /// as further steps under that node. Built nodes are named by their index.
    fn print(&self) -> Markdown {
        let arena = Arena::new();
        let root = arena.alloc(AstNode::from(NodeValue::Document));
        let mut nodes = vec![&*root];
        let mut add = |parent: &Parent, value: NodeValue| {
            let node = arena.alloc(AstNode::from(value));
            nodes[parent.0 as usize].append(node);
            nodes.push(node);
            Parent(nodes.len() as i64 - 1)
        };
        let mut steps: Vec<PrintTask> = self
            .0
            .iter()
            .rev()
            .map(|block| PrintTask {
                parent: Parent(0),
                print_step: PrintStep::Block(block.clone()),
                depth: Depth(1),
            })
            .collect();
        while let Some(PrintTask {
            parent,
            print_step,
            depth,
        }) = steps.pop()
        {
            match print_step {
                PrintStep::Block(Block::Statement(text)) => {
                    let paragraph = add(&parent, NodeValue::Paragraph);
                    steps.push(PrintTask {
                        parent: paragraph,
                        print_step: PrintStep::Text(text),
                        depth,
                    });
                }
                PrintStep::Block(Block::Section(section)) => {
                    let heading = add(
                        &parent,
                        NodeValue::Heading(NodeHeading {
                            level: depth.0 as u8,
                            setext: false,
                            closed: false,
                        }),
                    );
                    steps.extend(
                        section
                            .block_vector
                            .into_iter()
                            .rev()
                            .map(|inner| PrintTask {
                                parent,
                                print_step: PrintStep::Block(inner),
                                depth: Depth(depth.0 + 1),
                            }),
                    );
                    steps.push(PrintTask {
                        parent: heading,
                        print_step: PrintStep::Text(section.title.0),
                        depth,
                    });
                }
                PrintStep::Block(Block::CodeBlock(code)) => {
                    let mut literal = code.text.verbatim();
                    if !literal.is_empty() {
                        literal.push('\n');
                    }
                    add(
                        &parent,
                        NodeValue::CodeBlock(Box::new(NodeCodeBlock {
                            fenced: true,
                            fence_char: b'`',
                            fence_length: 3,
                            info: code.programming_language.spelling(),
                            literal,
                            closed: true,
                            ..NodeCodeBlock::default()
                        })),
                    );
                }
                PrintStep::Block(Block::List(items)) => {
                    let list = add(&parent, NodeValue::List(NodeList::bullet()));
                    for item in items {
                        let item_node = add(&list, NodeValue::Item(NodeList::bullet()));
                        steps.push(PrintTask {
                            parent: item_node,
                            print_step: PrintStep::Block(item),
                            depth,
                        });
                    }
                }
                PrintStep::Block(Block::Quote(blocks)) => {
                    let quote = add(&parent, NodeValue::BlockQuote);
                    steps.extend(blocks.into_iter().rev().map(|inner| PrintTask {
                        parent: quote,
                        print_step: PrintStep::Block(inner),
                        depth,
                    }));
                }
                PrintStep::Block(Block::Table(rows)) => {
                    let columns = rows.first().map_or(0, |row| row.0.len());
                    let table = add(
                        &parent,
                        NodeValue::Table(Box::new(NodeTable {
                            alignments: vec![TableAlignment::None; columns],
                            num_columns: columns,
                            num_rows: rows.len(),
                            num_nonempty_cells: 0,
                        })),
                    );
                    steps.extend(rows.into_iter().enumerate().rev().map(|(index, row)| {
                        PrintTask {
                            parent: table,
                            print_step: PrintStep::Row(Row_Data {
                                text_vector: row.0,
                                header: Header(index == 0),
                            }),
                            depth,
                        }
                    }));
                }
                PrintStep::Row(row) => {
                    let row_node = add(&parent, NodeValue::TableRow(row.header.0));
                    for cell in row.text_vector {
                        let cell_node = add(&row_node, NodeValue::TableCell);
                        steps.push(PrintTask {
                            parent: cell_node,
                            print_step: PrintStep::Text(cell),
                            depth,
                        });
                    }
                }
                PrintStep::Text(text) => {
                    for text_part in text.0 {
                        let inner = match text_part {
                            TextPart::Words(words) => {
                                for (index, line) in words.split('\n').enumerate() {
                                    if index > 0 {
                                        add(&parent, NodeValue::SoftBreak);
                                    }
                                    if !line.is_empty() {
                                        add(&parent, NodeValue::Text(line.to_string().into()));
                                    }
                                }
                                None
                            }
                            TextPart::InlineCode(literal) => {
                                add(
                                    &parent,
                                    NodeValue::Code(NodeCode {
                                        num_backticks: 1,
                                        literal,
                                    }),
                                );
                                None
                            }
                            TextPart::Emphasis(inner) => {
                                Some((add(&parent, NodeValue::Emph), inner))
                            }
                            TextPart::Strong(inner) => {
                                Some((add(&parent, NodeValue::Strong), inner))
                            }
                            TextPart::Reference(reference) => {
                                let Target(Url(url)) = &reference.target;
                                let url = url.clone();
                                let link = add(
                                    &parent,
                                    NodeValue::Link(Box::new(NodeLink {
                                        url,
                                        title: String::new(),
                                    })),
                                );
                                Some((link, reference.text))
                            }
                        };
                        if let Some((node, inner)) = inner {
                            steps.push(PrintTask {
                                parent: node,
                                print_step: PrintStep::Text(inner),
                                depth,
                            });
                        }
                    }
                }
            }
        }
        let mut out = String::new();
        comrak::format_commonmark(root, &Options::sema(), &mut out)
            .expect("formatting into a String cannot fail");
        Markdown(out)
    }
}

impl Bulleted for NodeList {
    /// A bullet list, tight, marked `-`.
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

impl Fenceable for Text {
    /// Its words unescaped.
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
