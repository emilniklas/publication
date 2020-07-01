use crate::*;

pub trait Extension {
    fn parse_block(&self, _parser: &mut Parser) -> ParseResult<Option<Block>> {
        Ok(None)
    }

    fn parse_element(&self, _parser: &mut Parser) -> ParseResult<Option<Element>> {
        Ok(None)
    }

    fn sees_end_of_block(&self, _parser: &Parser) -> bool {
        false
    }
}

pub const BOLD: ExtensionTag = ExtensionTag("builtin:BOLD");

pub struct Bold;

impl Extension for Bold {
    fn parse_element(&self, parser: &mut Parser) -> ParseResult<Option<Element>> {
        if parser.peek() != '*' {
            return Ok(None);
        }

        parser.take(); // *

        let mut text = String::new();
        loop {
            if parser.sees_end_of_block() {
                return Ok(None);
            }
            match parser.take() {
                '*' => return Ok(Some(Element::ExtensionElement(BOLD, Box::new(Element::Text(text))))),

                c => text.push(c),
            }
        }
    }
}

pub const ITALICS: ExtensionTag = ExtensionTag("builtin:ITALICS");

pub struct Italics;

impl Extension for Italics {
    fn parse_element(&self, parser: &mut Parser) -> ParseResult<Option<Element>> {
        if parser.peek() != '/' {
            return Ok(None);
        }

        parser.take(); // /

        let mut text = String::new();
        loop {
            if parser.sees_end_of_block() {
                return Ok(None);
            }
            match parser.take() {
                '/' => return Ok(Some(Element::ExtensionElement(ITALICS, Box::new(Element::Text(text))))),

                c => text.push(c),
            }
        }
    }
}

pub const LIST: ExtensionTag = ExtensionTag("builtin:LIST");
pub const LIST_ITEM: ExtensionTag = ExtensionTag("builtin:LIST_ITEM");

pub struct Lists(Vec<char>);

impl Lists {
    pub fn new<B: AsRef<str>>(bullet: B) -> Lists {
        Lists(bullet.as_ref().chars().collect())
    }

    fn sees_bullet(&self, parser: &Parser) -> bool {
        let bullet_chars = self.0.as_slice();
        parser.peek_many(bullet_chars.len()) == bullet_chars
    }

    fn parse_list(&self, parser: &mut Parser) -> ParseResult<Block> {
        let mut items = vec![];
        while self.sees_bullet(parser) {
            items.push(self.parse_list_item(parser)?);
        }
        Ok(Block::ExtensionBlocks(LIST, items))
    }

    fn parse_list_item(&self, parser: &mut Parser) -> ParseResult<Block> {
        parser.take_many(self.0.len()); // take bullet
        parser.move_past_whitespace();
        Ok(Block::ExtensionBlock(LIST_ITEM, parser.parse_elements()?))
    }
}

impl Extension for Lists {
    fn parse_block(&self, parser: &mut Parser) -> ParseResult<Option<Block>> {
        Ok(if self.sees_bullet(parser) {
            Some(self.parse_list(parser)?)
        } else {
            None
        })
    }

    fn sees_end_of_block(&self, parser: &Parser) -> bool {
        self.sees_bullet(parser)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_external_extension() {
        const MY_TAG: ExtensionTag = ExtensionTag("MY_TAG");

        struct MyExtension;

        impl Extension for MyExtension {
            fn parse_element(&self, parser: &mut Parser) -> ParseResult<Option<Element>> {
                let sign = ['$', '$'];
                if parser.peek_many(2) != &sign {
                    return Ok(None);
                }
                parser.take_many(2);
                let mut content = String::new();
                while !parser.is_at_end() && parser.peek_many(2) != &sign {
                    content.push(parser.take());
                }
                parser.take_many(2);
                Ok(Some(Element::ExtensionElement(MY_TAG, Box::new(Element::Text(content)))))
            }
        }

        let mut parser = Parser::new("This is $$some syntax$$".into());
        parser.add_extension(MyExtension);

        let mut emitter = HtmlEmitter::new();
        emitter.tagged_element(MY_TAG, |_| ("span".into(), vec![]));

        let output = parser.emit_with(&emitter).unwrap();

        assert_eq!(output, "<p>\n  This is <span>some syntax</span>\n</p>\n");
    }
}
