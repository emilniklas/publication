use crate::*;

pub trait Emitter {
    fn emit(&self, node: Node, out: &mut String);
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

impl HtmlEmitter {
    fn escape_text(&self, text: &mut String) {
        let mut i = 0;
        while i < text.len() {
            let range = i..i+1;
            match &text[range.clone()] {
                "'" => {
                    text.replace_range(range, "&apos;");
                    i += "&apos;".len();
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
}

impl Emitter for HtmlEmitter {
    fn emit(&self, node: Node, out: &mut String) {
        match node {
            Node::Paragraph(mut p) => {
                self.escape_text(&mut p);
                out.push_str(format!("<p>\n  {}\n</p>\n", p).as_ref());
            }
        }
    }
}

pub(crate) struct TextEmitter;

impl Emitter for TextEmitter {
    fn emit(&self, node: Node, out: &mut String) {
        match node {
            Node::Paragraph(p) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(p.as_ref());
                out.push('\n');
            }
        }
    }
}
