//! Rough tokens for markdown
mod property;
mod heading;
mod codeblock;
mod list;
mod paragraph;
mod mark;
mod pair;
mod link;

pub use property::Property;
pub use heading::Heading;
pub use codeblock::Codeblock;
pub use list::List;
pub use paragraph::Paragraph;
pub use mark::Mark;
pub use pair::Pair;
pub use link::Link;

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub enum Token {
    Heading(Heading),
    Codeblock(Codeblock),
    List(List),
    Paragraph(Paragraph),
    BlankLine,
}

impl Token {
    pub fn new(textlines: &[&str], indent: usize) -> Result<Self> {
        if textlines.len() == 0 {
            return Err(anyhow!("textlines are empty"));
        }

        let peekline = textlines[0];
        if peekline.len() == 0 {
            return Ok(Self::BlankLine);
        }

        let indentstr = " ".repeat(indent);
        if peekline.starts_with(&format!("{indentstr}{}", Heading::MARK)) {
            Ok(Token::Heading(Heading::new(&textlines)?))
        } else if peekline.starts_with(&format!("{indentstr}{}", List::INDENT_MARK)) {
            Ok(Token::Codeblock(Codeblock::new(&textlines, indentstr.len() + List::INDENT_MARK.len())?))
        } else if peekline.starts_with(&format!("{indentstr}{}", List::LIST_MARK)) {
            Ok(Token::List(List::new(&textlines, indentstr.len())?))
        } else {
            Ok(Token::Paragraph(Paragraph::new(&textlines, indent)?))
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Self::Heading(heading) => heading.prop.val.len(),
            Self::Codeblock(codeblock) => codeblock.prop.val.len(),
            Self::List(list) => list.prop.val.len(),
            Self::Paragraph(paragraph) => paragraph.prop.val.len(),
            Self::BlankLine => 1,
        }
    }
}

pub struct Tokenizer<'a> {
    text: &'a str,
    pos: usize,
    indent: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str, indent: usize) -> Self {
        Tokenizer {
            text,
            pos: 0usize,
            indent,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.text.len() {
            return None;
        }
        let textlines: Vec<&str> = self.text[self.pos..].lines().collect();
        if let Ok(token) = Token::new(&textlines, self.indent) {
            self.pos += token.len();
            Some(token)
        } else {
            None
        }
    }
}
