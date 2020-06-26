use crate::*;

pub trait Emitter {
    fn emit_block(&self, block: Block, out: &mut String);

    fn emit_elements(&self, elements: Vec<Element>, out: &mut String) {
        for element in elements {
            self.emit_element(element, out);
        }
    }

    fn emit_element(&self, element: Element, out: &mut String) {
        match element {
            Element::Text(t) => self.emit_text(t, out),
            Element::Decorated(d, t) => self.emit_decorated_text(d, t, out),
        }
    }

    fn emit_text(&self, text: String, out: &mut String) {
        out.push_str(text.as_ref());
    }

    fn emit_decorated_text(&self, _decoration: &'static str, text: String, out: &mut String) {
        self.emit_text(text, out);
    }
}

impl TryInto<Box<dyn Emitter>> for &Path {
    type Error = ();

    fn try_into(self) -> Result<Box<dyn Emitter>, ()> {
        match self.extension() {
            Some(html) if html == "html" => Ok(Box::new(HtmlEmitter)),
            Some(txt) if txt == "txt" => Ok(Box::new(TextEmitter)),
            _ => Err(()),
        }
    }
}

pub(crate) struct HtmlEmitter;

impl Emitter for HtmlEmitter {
    fn emit_block(&self, block: Block, out: &mut String) {
        match block {
            Block::Paragraph(e) => {
                out.push_str("<p>\n  ");
                self.emit_elements(e, out);
                out.push_str("\n</p>\n");
            }
        }
    }

    fn emit_text(&self, text: String, out: &mut String) {
        for c in text.chars() {
            match c {
                '\'' => out.push_str("&apos;"),
                '\"' => out.push_str("&quot;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                c => out.push(c),
            }
        }
    }

    fn emit_decorated_text(&self, decoration: &'static str, text: String, out: &mut String) {
        match decoration {
            extensions::BOLD => {
                out.push_str("<strong>");
                self.emit_text(text, out);
                out.push_str("</strong>");
            }
            extensions::ITALICS => {
                out.push_str("<em>");
                self.emit_text(text, out);
                out.push_str("</em>");
            }
            _ => self.emit_text(text, out),
        }
    }
}

pub(crate) struct TextEmitter;

impl Emitter for TextEmitter {
    fn emit_block(&self, block: Block, out: &mut String) {
        match block {
            Block::Paragraph(e) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                self.emit_elements(e, out);
                out.push('\n');
            }
        }
    }
}
