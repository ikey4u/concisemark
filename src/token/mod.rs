//! Markdown tokens
mod codeblock;
mod heading;
mod link;
mod list;
mod mark;
mod pair;
mod paragraph;
mod property;

use anyhow::{ensure, Result};
pub use codeblock::Codeblock;
pub use heading::Heading;
pub use link::Link;
pub use list::List;
pub use mark::Mark;
pub use pair::Pair;
pub use paragraph::Paragraph;
pub use property::Property;

#[derive(Debug)]
pub enum Token {
    Heading(Heading),
    Codeblock(Codeblock),
    List(List),
    Paragraph(Paragraph),
    BlankLine(usize),
}

impl Token {
    pub fn new(textlines: &[&str], indent: usize) -> Result<Self> {
        ensure!(!textlines.is_empty(), "textlines are empty");

        let peekline = textlines[0];
        if peekline.trim().is_empty() {
            return Ok(Self::BlankLine(textlines[0].len()));
        }

        let indentstr = " ".repeat(indent);
        if peekline.starts_with(Heading::MARK) {
            Ok(Token::Heading(Heading::new(textlines[0])?))
        } else if peekline
            .starts_with(&format!("{indentstr}{}", List::INDENT_MARK))
        {
            Ok(Token::Codeblock(Codeblock::new(
                textlines,
                indentstr.len() + List::INDENT_MARK.len(),
            )?))
        } else if peekline
            .starts_with(&format!("{indentstr}{}", List::LIST_MARK))
        {
            Ok(Token::List(List::new(textlines, indentstr.len())?))
        } else {
            Ok(Token::Paragraph(Paragraph::new(textlines, indent)?))
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Self::Heading(heading) => heading.prop.val.len(),
            Self::Codeblock(codeblock) => codeblock.prop.val.len(),
            Self::List(list) => list.prop.val.len(),
            Self::Paragraph(paragraph) => paragraph.prop.val.len(),
            Self::BlankLine(sz) => *sz,
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
        // Windows uses CRLF (\r\n) as line separator, but Unix and Linux use LF (\n), and old Mac
        // OS X ues CR (\r). For simplicity, we will use `\n` as the universal line separator, this
        // implmentation is also suitable for Windows since the `\r` has no significant impact when
        // we parse line by line.
        let textlines: Vec<&str> =
            self.text[self.pos..].split_inclusive("\n").collect();
        if let Ok(token) = Token::new(&textlines, self.indent) {
            self.pos += token.len();
            Some(token)
        } else {
            None
        }
    }
}
