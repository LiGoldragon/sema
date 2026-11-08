//! Measures Markdown libraries as the codec under sema's Document: each body
//! is read into a Document, printed back, and read again. `sema` is the lane's
//! own codec, comrak; `markdown-rs` is markdown with mdast_util_to_markdown.
//!
//! usage: sema-codec-measure <module.md>...
//! prints one tab-separated row per module and library, then the totals.

use sema::{
    Block, Code, Composable, Document, Markdown, Printable, ProgrammingLanguage, Reference, Row,
    Section, Target, Text, TextPart, Title,
};

const SPELLINGS: [(ProgrammingLanguage, &str); 9] = [
    (ProgrammingLanguage::Ethos, "ethos"),
    (ProgrammingLanguage::Datom, "datom"),
    (ProgrammingLanguage::Rust, "rust"),
    (ProgrammingLanguage::Nix, "nix"),
    (ProgrammingLanguage::Shell, "sh"),
    (ProgrammingLanguage::Ini, "ini"),
    (ProgrammingLanguage::Toml, "toml"),
    (ProgrammingLanguage::Mermaid, "mermaid"),
    (ProgrammingLanguage::Plain, ""),
];

fn language(info: &str) -> Option<ProgrammingLanguage> {
    SPELLINGS.iter().find(|(_, s)| *s == info).map(|(l, _)| *l)
}

fn spelling(language: ProgrammingLanguage) -> &'static str {
    SPELLINGS.iter().find(|(l, _)| *l == language).unwrap().1
}

/// A construct the Document has no place for: its kind and its line.
#[derive(Debug, Clone)]
struct Unread {
    kind: String,
    line: usize,
}

#[derive(Default)]
struct Reading {
    unread: Vec<Unread>,
}

impl Reading {
    fn unread(&mut self, kind: &str, line: usize) {
        self.unread.push(Unread {
            kind: kind.to_string(),
            line,
        });
    }
}

enum Piece {
    Heading(usize, Text, usize),
    Block(Block),
}

/// Headings nested into sections: the first heading level is the top, each
/// deeper level a child; a heading above the top or skipping a level is unread.
fn nest(pieces: Vec<Piece>, reading: &mut Reading) -> Vec<Block> {
    struct Open {
        depth: usize,
        title: Text,
        blocks: Vec<Block>,
    }
    fn close(open: &mut Vec<Open>, root: &mut Vec<Block>) {
        if let Some(section) = open.pop() {
            let block = Block::Section(Section {
                title: Title(section.title),
                block_vector: section.blocks,
            });
            match open.last_mut() {
                Some(parent) => parent.blocks.push(block),
                None => root.push(block),
            }
        }
    }
    let mut root = Vec::new();
    let mut open: Vec<Open> = Vec::new();
    let mut top = None;
    for piece in pieces {
        match piece {
            Piece::Block(block) => match open.last_mut() {
                Some(section) => section.blocks.push(block),
                None => root.push(block),
            },
            Piece::Heading(written, title, line) => {
                let top = *top.get_or_insert(written);
                if written < top {
                    reading.unread("heading above the first level", line);
                    continue;
                }
                let depth = written - top + 1;
                while open.last().is_some_and(|s| s.depth >= depth) {
                    close(&mut open, &mut root);
                }
                let above = open.last().map_or(0, |s| s.depth);
                if depth != above + 1 {
                    reading.unread("heading skipping a level", line);
                    continue;
                }
                open.push(Open {
                    depth,
                    title,
                    blocks: Vec::new(),
                });
            }
        }
    }
    while !open.is_empty() {
        close(&mut open, &mut root);
    }
    root
}

/// Adjacent words joined, as one TextPart.
fn joined(parts: Vec<TextPart>) -> Text {
    let mut out: Vec<TextPart> = Vec::new();
    for part in parts {
        match (out.last_mut(), part) {
            (Some(TextPart::Words(last)), TextPart::Words(more)) => last.push_str(&more),
            (_, part) => out.push(part),
        }
    }
    Text(out)
}

/// Blocks inside a list item or a quote: a heading there is unread.
fn flat(pieces: Vec<Piece>, reading: &mut Reading) -> Vec<Block> {
    pieces
        .into_iter()
        .filter_map(|piece| match piece {
            Piece::Block(block) => Some(block),
            Piece::Heading(_, _, line) => {
                reading.unread("heading inside a list or quote", line);
                None
            }
        })
        .collect()
}

fn code_block(info: &str, literal: &str, line: usize, reading: &mut Reading) -> Option<Block> {
    match language(info) {
        Some(programming_language) => {
            let literal = literal.strip_suffix('\n').unwrap_or(literal);
            let text = match literal.is_empty() {
                true => Text(Vec::new()),
                false => Text(vec![TextPart::Words(literal.to_string())]),
            };
            Some(Block::Code(Code {
                programming_language,
                text,
            }))
        }
        None => {
            reading.unread(&format!("fence language «{info}»"), line);
            None
        }
    }
}

trait Codec {
    fn name(&self) -> &'static str;
    fn read(&self, body: &str) -> (Document, Vec<Unread>);
    fn print(&self, document: &Document) -> Result<String, String>;
}

// ---------------------------------------------------------------- markdown-rs

mod with_markdown {
    use super::*;
    use markdown::mdast::{self, Node};
    use markdown::{Constructs, ParseOptions};

    pub struct MarkdownRs;

    fn parse_options() -> ParseOptions {
        ParseOptions {
            constructs: Constructs {
                gfm_table: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        }
    }

    fn line(node: &Node) -> usize {
        node.position().map_or(0, |p| p.start.line)
    }

    fn pieces(children: &[Node], body: &[&str], reading: &mut Reading) -> Vec<Piece> {
        let mut out = Vec::new();
        for child in children {
            let at = line(child);
            match child {
                Node::Paragraph(p) => {
                    out.push(Piece::Block(Block::Statement(inlines(&p.children, reading))))
                }
                Node::Heading(h) => {
                    let written = body.get(at.wrapping_sub(1)).copied().unwrap_or("");
                    if !written.trim_start().starts_with('#') {
                        reading.unread("setext heading", at);
                    }
                    out.push(Piece::Heading(
                        h.depth as usize,
                        inlines(&h.children, reading),
                        at,
                    ));
                }
                Node::Code(code) => {
                    let written = body.get(at.wrapping_sub(1)).copied().unwrap_or("");
                    let info = match &code.meta {
                        Some(meta) => format!("{} {meta}", code.lang.clone().unwrap_or_default()),
                        None => code.lang.clone().unwrap_or_default(),
                    };
                    if written.trim_start().starts_with("~~~") {
                        reading.unread("tilde fence", at);
                    } else if !written.trim_start().starts_with("```") {
                        reading.unread("indented code", at);
                    } else if let Some(block) = code_block(&info, &code.value, at, reading) {
                        out.push(Piece::Block(block));
                    }
                }
                Node::List(list) => {
                    if list.ordered {
                        reading.unread("ordered list", at);
                        continue;
                    }
                    let written = body.get(at.wrapping_sub(1)).copied().unwrap_or("");
                    if !written.trim_start().starts_with('-') {
                        reading.unread("list marker other than -", at);
                    }
                    let mut items = Vec::new();
                    for item in &list.children {
                        let item_children = item.children().map(Vec::as_slice).unwrap_or(&[]);
                        let mut blocks = flat(pieces(item_children, body, reading), reading);
                        match blocks.len() {
                            1 => items.extend(blocks.pop()),
                            n => reading.unread(&format!("list item of {n} blocks"), line(item)),
                        }
                    }
                    out.push(Piece::Block(Block::List(items)));
                }
                Node::Blockquote(quote) => out.push(Piece::Block(Block::Quote(flat(
                    pieces(&quote.children, body, reading),
                    reading,
                )))),
                Node::Table(table) => {
                    if table.align.iter().any(|a| *a != mdast::AlignKind::None) {
                        reading.unread("table alignment", at);
                    }
                    let rows = table
                        .children
                        .iter()
                        .map(|row| {
                            Row(row
                                .children()
                                .map(Vec::as_slice)
                                .unwrap_or(&[])
                                .iter()
                                .map(|cell| {
                                    inlines(cell.children().map(Vec::as_slice).unwrap_or(&[]), reading)
                                })
                                .collect())
                        })
                        .collect();
                    out.push(Piece::Block(Block::Table(rows)));
                }
                Node::ThematicBreak(_) => reading.unread("thematic break", at),
                Node::Html(_) => reading.unread("html block", at),
                other => reading.unread(&format!("block {}", variant(other)), at),
            }
        }
        out
    }

    fn variant(node: &Node) -> String {
        format!("{node:?}").split(['(', ' ', '{']).next().unwrap_or("").to_string()
    }

    fn inlines(children: &[Node], reading: &mut Reading) -> Text {
        let mut parts = Vec::new();
        for child in children {
            let at = line(child);
            match child {
                Node::Text(text) => parts.push(TextPart::Words(text.value.clone())),
                Node::InlineCode(code) => parts.push(TextPart::Code(code.value.clone())),
                Node::Emphasis(e) => parts.push(TextPart::Emphasis(inlines(&e.children, reading))),
                Node::Strong(s) => parts.push(TextPart::Strong(inlines(&s.children, reading))),
                Node::Link(link) => {
                    if link.title.is_some() {
                        reading.unread("link title", at);
                    }
                    parts.push(TextPart::Reference(Reference {
                        text: inlines(&link.children, reading),
                        target: Target::Url(link.url.clone()),
                    }));
                }
                Node::Break(_) => reading.unread("hard line break", at),
                Node::Image(_) => reading.unread("image", at),
                Node::Html(raw) => parts.push(TextPart::Words(raw.value.clone())),
                other => reading.unread(&format!("inline {}", variant(other)), at),
            }
        }
        joined(parts)
    }

    fn text_nodes(text: &Text) -> Vec<Node> {
        text.0
            .iter()
            .map(|part| match part {
                TextPart::Words(words) => Node::Text(mdast::Text {
                    value: words.clone(),
                    position: None,
                }),
                TextPart::Code(code) => Node::InlineCode(mdast::InlineCode {
                    value: code.clone(),
                    position: None,
                }),
                TextPart::Emphasis(inner) => Node::Emphasis(mdast::Emphasis {
                    children: text_nodes(inner),
                    position: None,
                }),
                TextPart::Strong(inner) => Node::Strong(mdast::Strong {
                    children: text_nodes(inner),
                    position: None,
                }),
                TextPart::Reference(reference) => Node::Link(mdast::Link {
                    children: text_nodes(&reference.text),
                    url: match &reference.target {
                        Target::Url(url) => url.clone(),
                        Target::Place(_) => String::new(),
                    },
                    title: None,
                    position: None,
                }),
            })
            .collect()
    }

    fn block_nodes(block: &Block, depth: u8, out: &mut Vec<Node>) {
        match block {
            Block::Statement(text) => out.push(Node::Paragraph(mdast::Paragraph {
                children: text_nodes(text),
                position: None,
            })),
            Block::Section(section) => {
                out.push(Node::Heading(mdast::Heading {
                    children: text_nodes(&section.title.0),
                    position: None,
                    depth,
                }));
                for inner in &section.block_vector {
                    block_nodes(inner, depth + 1, out);
                }
            }
            Block::Code(code) => {
                let value: String = code
                    .text
                    .0
                    .iter()
                    .map(|part| match part {
                        TextPart::Words(words) => words.clone(),
                        _ => String::new(),
                    })
                    .collect();
                let info = spelling(code.programming_language);
                out.push(Node::Code(mdast::Code {
                    value,
                    position: None,
                    lang: (!info.is_empty()).then(|| info.to_string()),
                    meta: None,
                }));
            }
            Block::List(items) => out.push(Node::List(mdast::List {
                children: items
                    .iter()
                    .map(|item| {
                        let mut children = Vec::new();
                        block_nodes(item, depth, &mut children);
                        Node::ListItem(mdast::ListItem {
                            children,
                            position: None,
                            spread: false,
                            checked: None,
                        })
                    })
                    .collect(),
                position: None,
                ordered: false,
                start: None,
                spread: false,
            })),
            Block::Quote(blocks) => {
                let mut children = Vec::new();
                for inner in blocks {
                    block_nodes(inner, depth, &mut children);
                }
                out.push(Node::Blockquote(mdast::Blockquote {
                    children,
                    position: None,
                }));
            }
            Block::Table(rows) => {
                let columns = rows.first().map_or(0, |row| row.0.len());
                out.push(Node::Table(mdast::Table {
                    children: rows
                        .iter()
                        .map(|row| {
                            Node::TableRow(mdast::TableRow {
                                children: row
                                    .0
                                    .iter()
                                    .map(|cell| {
                                        Node::TableCell(mdast::TableCell {
                                            children: text_nodes(cell),
                                            position: None,
                                        })
                                    })
                                    .collect(),
                                position: None,
                            })
                        })
                        .collect(),
                    position: None,
                    align: vec![mdast::AlignKind::None; columns],
                }));
            }
        }
    }

    impl Codec for MarkdownRs {
        fn name(&self) -> &'static str {
            "markdown-rs"
        }

        fn read(&self, body: &str) -> (Document, Vec<Unread>) {
            let mut reading = Reading::default();
            let lines: Vec<&str> = body.split('\n').collect();
            let tree = markdown::to_mdast(body, &parse_options()).expect("plain markdown parses");
            let children = tree.children().map(Vec::as_slice).unwrap_or(&[]);
            let pieces = pieces(children, &lines, &mut reading);
            let blocks = nest(pieces, &mut reading);
            (Document(blocks), reading.unread)
        }

        fn print(&self, document: &Document) -> Result<String, String> {
            let mut children = Vec::new();
            for block in &document.0 {
                block_nodes(block, 1, &mut children);
            }
            let root = Node::Root(mdast::Root {
                children,
                position: None,
            });
            let options = mdast_util_to_markdown::Options {
                bullet: '-',
                emphasis: '*',
                strong: '*',
                fence: '`',
                fences: true,
                list_item_indent: mdast_util_to_markdown::IndentOptions::One,
                ..mdast_util_to_markdown::Options::default()
            };
            mdast_util_to_markdown::to_markdown_with_options(&root, &options)
                .map_err(|e| e.to_string())
        }
    }
}

// ---------------------------------------------------------------- the hand reader, as a baseline

/// sema's own codec, as the lane builds it.
struct Sema;

impl Codec for Sema {
    fn name(&self) -> &'static str {
        "sema"
    }

    fn read(&self, body: &str) -> (Document, Vec<Unread>) {
        match Markdown(body.to_string()).compose() {
            Ok(document) => (document, Vec::new()),
            Err(refusal) => (
                Document(Vec::new()),
                vec![Unread {
                    kind: format!("{:?}", refusal.problem)
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .to_string(),
                    line: refusal.line.0 as usize,
                }],
            ),
        }
    }

    fn print(&self, document: &Document) -> Result<String, String> {
        Ok(document.print().0)
    }
}

// ---------------------------------------------------------------- the measure

/// The text after the front matter's closing delimiter line, its leading blank
/// line set aside: what the library codes. The front matter is sema's own.
fn body(whole: &str) -> &str {
    let rest = whole
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or(whole, |(_, body)| body);
    rest.strip_prefix('\n').unwrap_or(rest)
}

fn main() {
    let codecs: Vec<Box<dyn Codec>> = vec![
        Box::new(Sema),
        Box::new(with_markdown::MarkdownRs),
    ];
    let paths: Vec<String> = std::env::args().skip(1).collect();
    let mut totals: Vec<(usize, usize, usize, std::collections::BTreeMap<String, usize>)> =
        codecs.iter().map(|_| (0, 0, 0, Default::default())).collect();
    println!("module\tcodec\tresult\tunread");
    for path in &paths {
        let whole = std::fs::read_to_string(path).expect("module reads");
        let body = body(&whole);
        let name = path.rsplit('/').next().unwrap_or(path);
        for (index, codec) in codecs.iter().enumerate() {
            let (document, unread) = codec.read(body);
            let total = &mut totals[index];
            for u in &unread {
                *total.3.entry(u.kind.clone()).or_default() += 1;
            }
            let result = match unread.is_empty() {
                false => {
                    total.2 += 1;
                    "unread".to_string()
                }
                true => match codec.print(&document) {
                    Err(error) => format!("print failed: {}", error.replace('\n', " ")),
                    Ok(printed) => {
                        if let Ok(dir) = std::env::var("SEMA_MEASURE_OUT") {
                            let file = format!("{dir}/{}.{name}", codec.name());
                            std::fs::write(file, &printed).expect("printed text writes");
                        }
                        let (again, again_unread) = codec.read(&printed);
                        match (again == document && again_unread.is_empty(), printed == body) {
                            (true, true) => {
                                total.0 += 1;
                                total.1 += 1;
                                "byte-identical".to_string()
                            }
                            (true, false) => {
                                total.1 += 1;
                                let at = printed
                                    .bytes()
                                    .zip(body.bytes())
                                    .take_while(|(a, b)| a == b)
                                    .count();
                                let line = body[..at.min(body.len())].matches('\n').count() + 1;
                                format!("document-equal, bytes differ from body line {line}")
                            }
                            (false, _) => "printed text reads back different".to_string(),
                        }
                    }
                },
            };
            let unread_text = unread
                .iter()
                .map(|u| format!("{} (line {})", u.kind, u.line))
                .collect::<Vec<_>>()
                .join("; ");
            println!("{name}\t{}\t{result}\t{unread_text}", codec.name());
        }
    }
    println!();
    println!("codec\tbyte-identical\tdocument-equal\tmodules with unread\tunread constructs");
    for (index, codec) in codecs.iter().enumerate() {
        let (bytes, equal, unread, kinds) = &totals[index];
        let kinds = kinds
            .iter()
            .map(|(k, n)| format!("{k}: {n}"))
            .collect::<Vec<_>>()
            .join("; ");
        println!(
            "{}\t{bytes}\t{equal}\t{unread}\t{kinds}",
            codec.name()
        );
    }
}
