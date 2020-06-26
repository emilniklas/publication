use crate::*;

pub trait Extension {
    fn parse_element(&self, _parser: &mut Parser) -> ParseResult<Option<Element>> {
        Ok(None)
    }
}

pub const BOLD: &str = "builtin:BOLD";

pub struct Bold;

impl Extension for Bold {
    fn parse_element(&self, parser: &mut Parser) -> ParseResult<Option<Element>> {
        if parser.peek() != '*' {
            return Ok(None);
        }

        parser.take(); // *

        let mut text = String::new();
        loop {
            if parser.sees_end_of_paragraph() {
                return Ok(None);
            }
            match parser.take() {
                '*' => return Ok(Some(Element::Decorated(BOLD, text))),

                c => text.push(c),
            }
        }
    }
}

pub const ITALICS: &str = "builtin:ITALICS";

pub struct Italics;

impl Extension for Italics {
    fn parse_element(&self, parser: &mut Parser) -> ParseResult<Option<Element>> {
        if parser.peek() != '/' {
            return Ok(None);
        }

        parser.take(); // /

        let mut text = String::new();
        loop {
            if parser.sees_end_of_paragraph() {
                return Ok(None);
            }
            match parser.take() {
                '/' => return Ok(Some(Element::Decorated(ITALICS, text))),

                c => text.push(c),
            }
        }
    }
}
