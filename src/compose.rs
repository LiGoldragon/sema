//! Markdown composed into a Document. The reader takes the canonical form the
//! printer writes, and refuses, with the line, what the Document cannot hold.

use crate::document::{
    Block, Code, Depth, Document, Markdown, ProgrammingLanguage, Reference, Row, Run, Section,
    Target, Text, Title,
};
use crate::error::{Line, Problem, ReadRefusal};

pub trait Composable {
    fn compose(&self) -> Result<Document, ReadRefusal>;
}

impl Composable for Markdown {
    fn compose(&self) -> Result<Document, ReadRefusal> {
        let lines: Vec<String> = self.0.split('\n').map(str::to_string).collect();
        let region = Region {
            lines: lines.clone(),
            first: Line(1),
            source: &lines,
        };
        match lines.first().map(String::as_str) {
            Some("---") => Err(region.refused(0)),
            _ => region.blocks().map(Document),
        }
    }
}

/// Lines read as one container: the document, a list item or a quote.
pub(crate) struct Region<'a> {
    pub(crate) lines: Vec<String>,
    pub(crate) first: Line,
    /// Every line of the text as written, for what a refusal carries.
    pub(crate) source: &'a [String],
}

/// The forms a line can open that the Document has no place for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Construct {
    ThematicBreak,
    TildeFence,
    ListMarker,
    OrderedList,
}

/// What a line opens when a block begins on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opening {
    Blank,
    Indented,
    Heading(Depth),
    Fence,
    Item,
    Quote,
    Table,
    Underline,
    Paragraph,
    Refused(Construct),
}

trait Opens {
    fn opening(&self) -> Opening;
}

impl Opens for str {
    fn opening(&self) -> Opening {
        let hashes = self.chars().take_while(|c| *c == '#').count();
        let marks: Vec<char> = self.chars().filter(|c| *c != ' ').collect();
        let ordinal = self.chars().take_while(char::is_ascii_digit).count();
        if self.trim().is_empty() {
            Opening::Blank
        } else if self.starts_with([' ', '\t']) {
            Opening::Indented
        } else if (1..=6).contains(&hashes)
            && (self.len() == hashes || self[hashes..].starts_with(' '))
        {
            Opening::Heading(Depth(hashes))
        } else if self.starts_with("```") {
            Opening::Fence
        } else if self.starts_with("~~~") {
            Opening::Refused(Construct::TildeFence)
        } else if marks.len() >= 3 && ['-', '*', '_'].iter().any(|m| marks.iter().all(|c| c == m)) {
            Opening::Refused(Construct::ThematicBreak)
        } else if self == "-" || self.starts_with("- ") {
            Opening::Item
        } else if self.starts_with("* ") || self.starts_with("+ ") {
            Opening::Refused(Construct::ListMarker)
        } else if ordinal > 0
            && (self[ordinal..].starts_with(". ") || self[ordinal..].starts_with(") "))
        {
            Opening::Refused(Construct::OrderedList)
        } else if self.starts_with('>') {
            Opening::Quote
        } else if self.starts_with('|') {
            Opening::Table
        } else if self.chars().all(|c| c == '=') {
            Opening::Underline
        } else {
            Opening::Paragraph
        }
    }
}

/// A heading or a block, before headings are nested into sections.
pub(crate) enum Piece {
    Heading(Depth, Title, Line),
    Block(Block),
}

/// A section still open while its heading's blocks are read.
struct Open {
    depth: Depth,
    title: Title,
    block_vector: Vec<Block>,
}

pub(crate) trait Reading {
    fn blocks(&self) -> Result<Vec<Block>, ReadRefusal>;
    fn pieces(&self) -> Result<Vec<Piece>, ReadRefusal>;
    fn heading(&self, index: usize, depth: Depth) -> Result<Piece, ReadRefusal>;
    fn fence(&self, index: usize) -> Result<(Block, usize), ReadRefusal>;
    fn paragraph(&self, index: usize) -> Result<(Block, usize), ReadRefusal>;
    fn list(&self, index: usize) -> Result<(Block, usize), ReadRefusal>;
    fn quote(&self, index: usize) -> Result<(Block, usize), ReadRefusal>;
    fn table(&self, index: usize) -> Result<(Block, usize), ReadRefusal>;
    fn line(&self, index: usize) -> Line;
    fn refused(&self, index: usize) -> ReadRefusal;
    fn unclosed(&self, index: usize) -> ReadRefusal;
}

impl Reading for Region<'_> {
    fn blocks(&self) -> Result<Vec<Block>, ReadRefusal> {
        let mut root = Vec::new();
        let mut open: Vec<Open> = Vec::new();
        // The first heading's level is the top; a deeper level is a child.
        let mut top: Option<Depth> = None;
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
        for piece in self.pieces()? {
            match piece {
                Piece::Block(block) => match open.last_mut() {
                    Some(section) => section.block_vector.push(block),
                    None => root.push(block),
                },
                Piece::Heading(written, title, line) => {
                    let top = *top.get_or_insert(written);
                    if written.0 < top.0 {
                        return Err(line.refusal(self.source));
                    }
                    let depth = Depth(written.0 - top.0 + 1);
                    while open.last().is_some_and(|section| section.depth >= depth) {
                        close(&mut open, &mut root);
                    }
                    let above = open.last().map_or(Depth(0), |section| section.depth);
                    if depth.0 != above.0 + 1 {
                        return Err(line.refusal(self.source));
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

    fn pieces(&self) -> Result<Vec<Piece>, ReadRefusal> {
        let mut pieces = Vec::new();
        let mut index = 0;
        while index < self.lines.len() {
            let (block, next) = match self.lines[index].opening() {
                Opening::Blank => {
                    index += 1;
                    continue;
                }
                Opening::Heading(depth) => {
                    pieces.push(self.heading(index, depth)?);
                    index += 1;
                    continue;
                }
                Opening::Indented => {
                    return Err(self.refused(index));
                }
                Opening::Refused(_) => {
                    return Err(self.refused(index));
                }
                Opening::Fence => self.fence(index)?,
                Opening::Item => self.list(index)?,
                Opening::Quote => self.quote(index)?,
                Opening::Table => self.table(index)?,
                Opening::Underline | Opening::Paragraph => self.paragraph(index)?,
            };
            pieces.push(Piece::Block(block));
            index = next;
        }
        Ok(pieces)
    }

    fn heading(&self, index: usize, depth: Depth) -> Result<Piece, ReadRefusal> {
        let source = &self.lines[index][depth.0..];
        match source.strip_prefix(' ') {
            Some(title) if !title.is_empty() => {
                let text = Inline::new(title, self.line(index), self.source).text()?;
                Ok(Piece::Heading(depth, Title(text), self.line(index)))
            }
            _ => Err(self.refused(index)),
        }
    }

    fn fence(&self, index: usize) -> Result<(Block, usize), ReadRefusal> {
        let programming_language = ProgrammingLanguage::try_from(&self.lines[index][3..])
            .map_err(|_| self.refused(index))?;
        let close = (index + 1..self.lines.len())
            .find(|candidate| self.lines[*candidate] == "```")
            .ok_or_else(|| self.unclosed(index))?;
        let content = &self.lines[index + 1..close];
        let text = match content.is_empty() {
            true => Text(Vec::new()),
            false => Text(vec![Run::Words(content.join("\n"))]),
        };
        Ok((
            Block::Code(Code {
                programming_language,
                text,
            }),
            close + 1,
        ))
    }

    fn paragraph(&self, index: usize) -> Result<(Block, usize), ReadRefusal> {
        let mut end = index + 1;
        while end < self.lines.len() {
            match self.lines[end].opening() {
                Opening::Paragraph | Opening::Indented | Opening::Table => end += 1,
                Opening::Underline | Opening::Refused(Construct::ThematicBreak) => {
                    return Err(self.refused(end));
                }
                Opening::Refused(_) => {
                    return Err(self.refused(end));
                }
                _ => break,
            }
        }
        let source = self.lines[index..end].join("\n");
        let text = Inline::new(&source, self.line(index), self.source).text()?;
        Ok((Block::Statement(text), end))
    }

    fn list(&self, index: usize) -> Result<(Block, usize), ReadRefusal> {
        let mut items = Vec::new();
        let mut cursor = index;
        while cursor < self.lines.len() && self.lines[cursor].opening() == Opening::Item {
            let start = cursor;
            let mut content = vec![self.lines[cursor].get(2..).unwrap_or("").to_string()];
            cursor += 1;
            while cursor < self.lines.len() {
                let line = &self.lines[cursor];
                match line.opening() {
                    Opening::Blank | Opening::Item => break,
                    _ if line.starts_with("  ") => {
                        content.push(line[2..].to_string());
                        cursor += 1;
                    }
                    Opening::Paragraph | Opening::Indented | Opening::Underline => {
                        return Err(self.refused(cursor));
                    }
                    _ => break,
                }
            }
            let region = Region {
                lines: content,
                first: self.line(start),
                source: self.source,
            };
            let mut blocks = region.blocks()?;
            match blocks.len() {
                1 => items.extend(blocks.pop()),
                _ => return Err(self.refused(start)),
            }
        }
        Ok((Block::List(items), cursor))
    }

    fn quote(&self, index: usize) -> Result<(Block, usize), ReadRefusal> {
        let end = (index..self.lines.len())
            .find(|candidate| !self.lines[*candidate].starts_with('>'))
            .unwrap_or(self.lines.len());
        if end < self.lines.len()
            && matches!(
                self.lines[end].opening(),
                Opening::Paragraph | Opening::Indented
            )
        {
            return Err(self.refused(end));
        }
        let lines = self.lines[index..end]
            .iter()
            .map(|line| line.strip_prefix("> ").unwrap_or(&line[1..]).to_string())
            .collect();
        let region = Region {
            lines,
            first: self.line(index),
            source: self.source,
        };
        Ok((Block::Quote(region.blocks()?), end))
    }

    fn table(&self, index: usize) -> Result<(Block, usize), ReadRefusal> {
        let end = (index..self.lines.len())
            .find(|candidate| !self.lines[*candidate].starts_with('|'))
            .unwrap_or(self.lines.len());
        let cells: Vec<Vec<&str>> = self.lines[index..end]
            .iter()
            .map(|line| line.as_str().cells())
            .collect();
        let delimiter = cells.get(1).ok_or_else(|| self.refused(index))?;
        let dashes = |cell: &&str| !cell.is_empty() && cell.chars().all(|c| c == '-');
        if !delimiter.iter().all(dashes) {
            return Err(self.refused(index + 1));
        }
        let header = cells[0].len();
        let mut rows = Vec::new();
        for (offset, row) in cells.iter().enumerate().filter(|(offset, _)| *offset != 1) {
            if row.len() != header {
                return Err(self.refused(index + offset));
            }
            let texts = row
                .iter()
                .map(|cell| Inline::new(cell, self.line(index + offset), self.source).text())
                .collect::<Result<Vec<Text>, ReadRefusal>>()?;
            rows.push(Row(texts));
        }
        Ok((Block::Table(rows), end))
    }

    fn line(&self, index: usize) -> Line {
        Line(self.first.0 + index as i64)
    }

    fn refused(&self, index: usize) -> ReadRefusal {
        self.line(index).refusal(self.source)
    }

    fn unclosed(&self, index: usize) -> ReadRefusal {
        ReadRefusal {
            line: self.line(index),
            problem: Problem::UnclosedFence,
        }
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

/// A table line split into its cells at each unescaped bar, each cell trimmed.
trait Cells {
    fn cells(&self) -> Vec<&str>;
}

impl Cells for str {
    fn cells(&self) -> Vec<&str> {
        let inner = self.trim_end();
        let inner = inner.strip_prefix('|').unwrap_or(inner);
        let inner = inner
            .strip_suffix('|')
            .filter(|rest| !rest.ends_with('\\'))
            .unwrap_or(inner);
        let mut cells = Vec::new();
        let mut start = 0;
        let bytes = inner.as_bytes();
        for (position, byte) in bytes.iter().enumerate() {
            if *byte == b'|' && (position == 0 || bytes[position - 1] != b'\\') {
                cells.push(inner[start..position].trim());
                start = position + 1;
            }
        }
        cells.push(inner[start..].trim());
        cells
    }
}

/// The characters a backslash escapes inside a run of words.
pub(crate) const ESCAPABLE: [char; 5] = ['\\', '*', '`', '[', ']'];

/// What closes the runs being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Closer {
    Emphasis,
    Strong,
    Reference,
}

/// The inside of a paragraph, a title or a cell, read into runs.
struct Inline<'a> {
    characters: Vec<char>,
    position: usize,
    line: Line,
    source: &'a [String],
}

trait RunReading<'a> {
    fn new(text: &str, line: Line, source: &'a [String]) -> Self;
    fn text(self) -> Result<Text, ReadRefusal>;
    fn runs(&mut self, closer: Option<Closer>) -> Result<Text, ReadRefusal>;
    fn reference(&mut self) -> Option<Reference>;
    fn at(&self, offset: usize) -> Option<char>;
    fn refused(&self) -> ReadRefusal;
}

impl<'a> RunReading<'a> for Inline<'a> {
    fn new(text: &str, line: Line, source: &'a [String]) -> Self {
        Inline {
            characters: text.chars().collect(),
            position: 0,
            line,
            source,
        }
    }

    fn text(mut self) -> Result<Text, ReadRefusal> {
        self.runs(None)
    }

    fn runs(&mut self, closer: Option<Closer>) -> Result<Text, ReadRefusal> {
        let mut runs = Vec::new();
        let mut words = String::new();
        let flush = |words: &mut String, runs: &mut Vec<Run>| {
            if !words.is_empty() {
                runs.push(Run::Words(std::mem::take(words)));
            }
        };
        loop {
            let Some(character) = self.at(0) else {
                return match closer {
                    None => {
                        flush(&mut words, &mut runs);
                        Ok(Text(runs))
                    }
                    Some(Closer::Emphasis) => Err(self.refused()),
                    Some(Closer::Strong) => Err(self.refused()),
                    Some(Closer::Reference) => Err(self.refused()),
                };
            };
            match character {
                '\\' => match self.at(1) {
                    Some(escaped) if ESCAPABLE.contains(&escaped) => {
                        words.push(escaped);
                        self.position += 2;
                    }
                    _ => {
                        words.push('\\');
                        self.position += 1;
                    }
                },
                '\n' => {
                    words.push('\n');
                    self.line = Line(self.line.0 + 1);
                    self.position += 1;
                }
                '`' => {
                    if self.at(1) == Some('`') {
                        return Err(self.refused());
                    }
                    let close = (self.position + 1..self.characters.len())
                        .find(|candidate| self.characters[*candidate] == '`')
                        .ok_or_else(|| self.refused())?;
                    let code: String = self.characters[self.position + 1..close].iter().collect();
                    self.line = Line(self.line.0 + code.matches('\n').count() as i64);
                    flush(&mut words, &mut runs);
                    runs.push(Run::Code(code));
                    self.position = close + 1;
                }
                '*' => {
                    let strong = self.at(1) == Some('*');
                    let (own, width) = match strong {
                        true => (Closer::Strong, 2),
                        false => (Closer::Emphasis, 1),
                    };
                    self.position += width;
                    flush(&mut words, &mut runs);
                    if closer == Some(own) {
                        return Ok(Text(runs));
                    }
                    let inner = self.runs(Some(own))?;
                    runs.push(match strong {
                        true => Run::Strong(inner),
                        false => Run::Emphasis(inner),
                    });
                }
                ']' if closer == Some(Closer::Reference) => {
                    self.position += 1;
                    flush(&mut words, &mut runs);
                    return Ok(Text(runs));
                }
                '[' => match self.reference() {
                    Some(reference) => {
                        flush(&mut words, &mut runs);
                        runs.push(Run::Reference(reference));
                    }
                    None => {
                        words.push('[');
                        self.position += 1;
                    }
                },
                other => {
                    words.push(other);
                    self.position += 1;
                }
            }
        }
    }

    /// A link read from the bracket at the position; on any other shape the
    /// position is left where it was and the bracket is words.
    fn reference(&mut self) -> Option<Reference> {
        let (position, line) = (self.position, self.line);
        self.position += 1;
        let text = self.runs(Some(Closer::Reference)).ok();
        let close = (self.at(0) == Some('('))
            .then(|| {
                (self.position + 1..self.characters.len())
                    .find(|candidate| self.characters[*candidate] == ')')
            })
            .flatten();
        match (text, close) {
            (Some(text), Some(close)) => {
                let url: String = self.characters[self.position + 1..close].iter().collect();
                self.line = Line(self.line.0 + url.matches('\n').count() as i64);
                self.position = close + 1;
                Some(Reference {
                    text,
                    target: Target::Url(url),
                })
            }
            _ => {
                self.position = position;
                self.line = line;
                None
            }
        }
    }

    fn at(&self, offset: usize) -> Option<char> {
        self.characters.get(self.position + offset).copied()
    }

    fn refused(&self) -> ReadRefusal {
        self.line.refusal(self.source)
    }
}
