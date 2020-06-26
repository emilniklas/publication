mod emitter;
pub use self::emitter::*;

use std::convert::TryInto;
use std::fmt;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Node {
    Paragraph(String),
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
}

impl Parser {
    pub fn new(raw: String) -> Parser {
        Parser {
            raw: raw.chars().collect(),
            offset: 0,
        }
    }

    pub fn emit_with(mut self, emitter: &dyn Emitter) -> ParseResult<String> {
        let mut out = String::new();
        self.move_past_whitespace();
        while !self.is_at_end() {
            emitter.emit(self.parse_block()?, &mut out);
            self.move_past_whitespace();
        }
        Ok(out)
    }

    pub fn parse(mut self) -> ParseResult<Vec<Node>> {
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

    #[inline]
    fn take(&mut self) -> char {
        let c = self.peek();
        self.offset += 1;
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

    fn parse_block(&mut self) -> ParseResult<Node> {
        match self.peek() {
            _ => self.parse_paragraph_block(),
        }
    }

    fn parse_paragraph_block(&mut self) -> ParseResult<Node> {
        if self.is_at_end() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        let mut paragraph = String::new();
        let mut whitespace = false;
        while !self.sees_end_of_paragraph() {
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
        Ok(Node::Paragraph(paragraph))
    }

    fn sees_end_of_paragraph(&self) -> bool {
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
            vec![Node::Paragraph("Hello! This is a sentence!".into())]
        );
    }

    #[test]
    fn comments_and_extraneous_whitespace_is_removed() {
        let parser = Parser::new(r#"
          # This is a comment
          
          This is a paragraph! # Which happens to include a comment
          And it spans multiple
          lines!
        "#.into());

        assert_eq!(
            parser.parse().unwrap(),
            vec![Node::Paragraph("This is a paragraph! And it spans multiple lines!".into())]
        );
    }
    
    #[test]
    fn html_emitter_escapes() {
        let parser = Parser::new(r#"
          This isn't Markdown!
        "#.into());

        assert_eq!(
            parser.emit_with(&HtmlEmitter).unwrap(),
            "<p>\n  This isn&apos;t Markdown!\n</p>\n"
        );
    }
}
