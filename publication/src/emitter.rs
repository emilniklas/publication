use crate::*;
use std::collections::HashMap;

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
            Element::ExtensionElement(tag, e) => self.emit_extension_element(tag, *e, out),
        }
    }

    fn emit_text(&self, text: String, out: &mut String) {
        out.push_str(text.as_ref());
    }

    fn emit_extension_element(&self, _tag: ExtensionTag, element: Element, out: &mut String) {
        self.emit_element(element, out)
    }
}

impl TryInto<Box<dyn Emitter>> for &Path {
    type Error = ();

    fn try_into(self) -> Result<Box<dyn Emitter>, ()> {
        match self.extension() {
            Some(html) if html == "html" => Ok(Box::new(HtmlEmitter::new())),
            Some(txt) if txt == "txt" => Ok(Box::new(TextEmitter::new())),
            _ => Err(()),
        }
    }
}

pub(crate) struct HtmlEmitter {
    extension_element_map: HashMap<ExtensionTag, Box<dyn Fn(&Element) -> (String, Vec<(String, String)>)>>,
    extension_block_map: HashMap<ExtensionTag, Box<dyn Fn(&Vec<Element>) -> (String, Vec<(String, String)>)>>,
    extension_blocks_map: HashMap<ExtensionTag, Box<dyn Fn(&Vec<Block>) -> (String, Vec<(String, String)>)>>,
}

impl HtmlEmitter {
    pub fn new() -> HtmlEmitter {
        HtmlEmitter {
            extension_element_map: HashMap::new(),
            extension_block_map: HashMap::new(),
            extension_blocks_map: HashMap::new(),
        }
    }
}

impl HtmlEmitter {
    pub fn tagged_element<F: 'static + Fn(&Element) -> (String, Vec<(String, String)>)>(&mut self, tag: ExtensionTag, f: F) {
        self.extension_element_map.insert(tag, Box::new(f));
    }

    pub fn tagged_block<F: 'static + Fn(&Vec<Element>) -> (String, Vec<(String, String)>)>(&mut self, tag: ExtensionTag, f: F) {
        self.extension_block_map.insert(tag, Box::new(f));
    }

    pub fn tagged_blocks<F: 'static + Fn(&Vec<Block>) -> (String, Vec<(String, String)>)>(&mut self, tag: ExtensionTag, f: F) {
        self.extension_blocks_map.insert(tag, Box::new(f));
    }
}

impl Emitter for HtmlEmitter {
    fn emit_block(&self, block: Block, out: &mut String) {
        match block {
            Block::Paragraph(e) => {
                out.push_str("<p>\n  ");
                self.emit_elements(e, out);
                out.push_str("\n</p>\n");
            }
            Block::ExtensionBlock(extensions::LIST_ITEM, elements) => {
                out.push_str("  <li>\n    ");
                self.emit_elements(elements, out);
                out.push_str("\n  </li>\n");
            }
            Block::ExtensionBlock(tag, elements) => {
                let ext = self.extension_block_map.get(&tag).map(|f| f(&elements));
                if let Some((element, attrs)) = &ext {
                    out.push_str(format!("<{}", element).as_ref());
                    for (key, value) in attrs.iter() {
                        out.push_str(format!(" {}={:?}", key, value).as_ref());
                    }
                    out.push('>');
                } else {
                    out.push_str(format!("<div data-publ-tag={:?}>\n  ", tag).as_ref());
                }

                self.emit_elements(elements, out);

                if let Some((element, _)) = ext {
                    out.push_str(format!("</{}>", element).as_ref());
                } else {
                    out.push_str("\n</div>\n");
                }
            }
            Block::ExtensionBlocks(extensions::LIST, blocks) => {
                out.push_str("<ul>\n");
                for block in blocks {
                    self.emit_block(block, out);
                }
                out.push_str("</ul>\n");
            }
            Block::ExtensionBlocks(tag, blocks) => {
                let ext = self.extension_blocks_map.get(&tag).map(|f| f(&blocks));
                if let Some((element, attrs)) = &ext {
                    out.push_str(format!("<{}", element).as_ref());
                    for (key, value) in attrs.iter() {
                        out.push_str(format!(" {}={:?}", key, value).as_ref());
                    }
                    out.push('>');
                }
                for block in blocks {
                    self.emit_block(block, out);
                }
                if let Some((element, _)) = ext {
                    out.push_str(format!("</{}>", element).as_ref());
                }
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

    fn emit_extension_element(&self, tag: ExtensionTag, element: Element, out: &mut String) {
        match tag {
            extensions::BOLD => {
                out.push_str("<strong>");
                self.emit_element(element, out);
                out.push_str("</strong>");
            }
            extensions::ITALICS => {
                out.push_str("<em>");
                self.emit_element(element, out);
                out.push_str("</em>");
            }
            tag if self.extension_element_map.contains_key(&tag) => {
                let (el, attrs) = self.extension_element_map.get(&tag).unwrap()(&element);
                out.push_str(format!("<{}", el).as_ref());
                for (key, value) in attrs.iter() {
                    out.push_str(format!(" {}={:?}", key, value).as_ref());
                }
                out.push('>');
                self.emit_element(element, out);
                out.push_str(format!("</{}>", el).as_ref());
            }
            _ => self.emit_element(element, out),
        }
    }
}

pub(crate) struct TextEmitter {
}

impl TextEmitter {
    pub fn new() -> TextEmitter {
        TextEmitter {}
    }
}

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
            _ => {}
        }
    }
}
