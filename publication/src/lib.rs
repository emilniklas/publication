mod emitter;
pub use self::emitter::*;

pub mod extensions;
use self::extensions::Extension;

use std::convert::TryInto;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

#[derive(PartialEq, Eq, Hash)]
pub struct ExtensionTag(&'static str);

impl fmt::Debug for ExtensionTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Debug, PartialEq)]
pub enum Block {
    Paragraph(Vec<Element>),
    ExtensionBlocks(ExtensionTag, Vec<Block>),
    ExtensionBlock(ExtensionTag, Vec<Element>),
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Text(String),
    ExtensionElement(ExtensionTag, Box<Element>),
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfFile,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfFile => write!(f, "Unexpected end of file."),
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    raw: Vec<char>,
    offset: usize,
    extensions: Vec<Rc<dyn Extension>>,
}

impl Parser {
    pub fn new(raw: String) -> Parser {
        Parser {
            raw: raw.chars().collect(),
            offset: 0,
            extensions: vec![],
        }
    }

    pub fn add_extension<E: Extension + 'static>(&mut self, extension: E) {
        self.extensions.push(Rc::new(extension));
    }

    pub fn emit_with(mut self, emitter: &dyn Emitter) -> ParseResult<String> {
        let mut out = String::new();
        self.move_past_whitespace();
        while !self.is_at_end() {
            emitter.emit_block(self.parse_block()?, &mut out);
            self.move_past_whitespace();
        }
        Ok(out)
    }

    pub fn parse(mut self) -> ParseResult<Vec<Block>> {
        let mut out = vec![];
        self.move_past_whitespace();
        while !self.is_at_end() {
            out.push(self.parse_block()?);
            self.move_past_whitespace();
        }
        Ok(out)
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.raw.len() == self.offset
    }

    #[inline]
    fn peek_at(&self, offset: usize) -> char {
        self.raw.get(offset).cloned().unwrap_or('\0')
    }

    #[inline]
    fn peek(&self) -> char {
        self.peek_at(self.offset)
    }

    fn peek_many(&self, len: usize) -> &[char] {
        let raw_len = self.raw.len();
        if raw_len > self.offset + len {
            &self.raw[self.offset..self.offset+len]
        } else if raw_len > self.offset {
            &self.raw[self.offset..]
        } else {
            &[]
        }
    }

    #[inline]
    fn take(&mut self) -> char {
        let c = self.peek();
        self.offset += 1;
        c
    }

    #[inline]
    fn take_many(&mut self, len: usize) -> Vec<char> {
        let c = self.peek_many(len).to_vec();
        self.offset += len;
        c
    }

    fn move_past_whitespace(&mut self) {
        loop {
            match self.peek() {
                '#' => self.move_past_comment(),
                w if w.is_whitespace() => self.offset += 1,
                _ => break,
            }
        }
    }

    fn move_past_comment(&mut self) {
        loop {
            match self.peek() {
                '\n' => {
                    self.offset += 1;
                    return;
                }
                '\0' => return,
                _ => self.offset += 1,
            }
        }
    }

    fn parse_block(&mut self) -> ParseResult<Block> {
        for ext in self.extensions.clone() {
            let offset_before_ext = self.offset;
            if let Some(block) = ext.parse_block(self)? {
                return Ok(block);
            }
            self.offset = offset_before_ext;
        }
        self.parse_paragraph_block()
    }

    fn parse_paragraph_block(&mut self) -> ParseResult<Block> {
        if self.is_at_end() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        Ok(Block::Paragraph(self.parse_elements()?))
    }

    fn parse_elements(&mut self) -> ParseResult<Vec<Element>> {
        let mut elements = vec![];

        let mut paragraph = String::new();
        let mut whitespace = false;
        'elements: while !self.sees_end_of_block() {
            for ext in self.extensions.clone() {
                let offset_before_ext = self.offset;
                if let Some(el) = ext.parse_element(self)? {
                    if whitespace {
                        paragraph.push(' ');
                        whitespace = false;
                    }
                    if !paragraph.is_empty() {
                        elements.push(Element::Text(std::mem::replace(
                            &mut paragraph,
                            String::new(),
                        )));
                    }
                    elements.push(el);
                    continue 'elements;
                }
                self.offset = offset_before_ext;
            }

            match self.take() {
                '#' => self.move_past_comment(),
                w if w.is_whitespace() => {
                    whitespace |= true;
                }
                c => {
                    if whitespace {
                        paragraph.push(' ');
                        whitespace = false;
                    }
                    paragraph.push(c)
                }
            }
        }
        if !paragraph.is_empty() {
            elements.push(Element::Text(paragraph));
        }
        Ok(elements)
    }

    fn sees_end_of_block(&self) -> bool {
        for ext in self.extensions.iter() {
            if ext.sees_end_of_block(self) {
                return true;
            }
        }

        match (self.peek(), self.peek_at(self.offset + 1)) {
            ('\n', '\n') | ('\n', '\0') | ('\0', '\0') => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        let parser = Parser::new("".into());

        assert_eq!(parser.parse().unwrap(), vec![]);
    }

    #[test]
    fn just_a_bit_of_text() {
        let parser = Parser::new("Hello! This is a sentence!".into());

        assert_eq!(
            parser.parse().unwrap(),
            vec![Block::Paragraph(vec![Element::Text(
                "Hello! This is a sentence!".into()
            )])]
        );
    }

    #[test]
    fn comments_and_extraneous_whitespace_is_removed() {
        let parser = Parser::new(
            r#"
              # This is a comment
              
              This is a paragraph! # Which happens to include a comment
              And it spans multiple
              lines!
            "#
            .into(),
        );

        assert_eq!(
            parser.parse().unwrap(),
            vec![Block::Paragraph(vec![Element::Text(
                "This is a paragraph! And it spans multiple lines!".into()
            )])]
        );
    }

    #[test]
    fn html_emitter_escapes() {
        let parser = Parser::new(
            r#"
              This isn't Markdown!
            "#
            .into(),
        );

        assert_eq!(
            parser.emit_with(&HtmlEmitter::new()).unwrap(),
            "<p>\n  This isn&apos;t Markdown!\n</p>\n"
        );
    }

    #[test]
    fn bold_extension() {
        let mut parser = Parser::new(
            r#"
              This *isn't* Markdown!
            "#
            .into(),
        );

        parser.add_extension(extensions::Bold);

        assert_eq!(
            parser.emit_with(&HtmlEmitter::new()).unwrap(),
            "<p>\n  This <strong>isn&apos;t</strong> Markdown!\n</p>\n"
        );
    }

    #[test]
    fn italics_extension() {
        let mut parser = Parser::new(
            r#"
              This /isn't/ Markdown!
            "#
            .into(),
        );

        parser.add_extension(extensions::Italics);

        assert_eq!(
            parser.emit_with(&HtmlEmitter::new()).unwrap(),
            "<p>\n  This <em>isn&apos;t</em> Markdown!\n</p>\n"
        );
    }

    #[test]
    fn lists_extension() {
        let mut parser = Parser::new(
            r#"
              This is a paragraph.
              ** This is a list item.

              ** This is a different list.
              ** With two items!
            "#
            .into(),
        );

        parser.add_extension(extensions::Lists::new("**"));

        assert_eq!(
            parser.parse().unwrap(),
            vec![
                Block::Paragraph(vec![Element::Text("This is a paragraph.".into())]),
                Block::ExtensionBlocks(
                    extensions::LIST,
                    vec![Block::ExtensionBlock(
                        extensions::LIST_ITEM,
                        vec![Element::Text("This is a list item.".into())]
                    )]
                ),
                Block::ExtensionBlocks(
                    extensions::LIST,
                    vec![
                        Block::ExtensionBlock(
                            extensions::LIST_ITEM,
                            vec![Element::Text("This is a different list.".into())]
                        ),
                        Block::ExtensionBlock(
                            extensions::LIST_ITEM,
                            vec![Element::Text("With two items!".into())]
                        ),
                    ]
                ),
            ],
        );
    }
}
