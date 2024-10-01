use std::ops::Range;

use anyhow::Result;

use super::Property;

#[derive(Debug)]
pub struct List {
    pub prop: Property,
}

#[derive(Debug)]
pub struct ListIterator<'a> {
    list: &'a List,
    pos: usize,
}

#[derive(Debug)]
pub struct ListItem {
    pub indent: usize,
    pub head: Range<usize>,
    pub body: Range<usize>,
}

impl List {
    pub const LIST_MARK: &'static str = "- ";
    pub const INDENT_MARK: &'static str = "    ";
    pub fn new(lines: &[&str], indent: usize) -> Result<Self> {
        let head_prefix = " ".repeat(indent) + Self::LIST_MARK;
        let head_multiple_line_prefix = " ".repeat(indent + 2);
        let body_prefix = " ".repeat(indent + 4);
        let list = lines
            .iter()
            .take_while(|line| {
                line.starts_with(&head_prefix)
                    || line.starts_with(&head_multiple_line_prefix)
                    || line.trim().is_empty()
                    || line.starts_with(&body_prefix)
            })
            .copied()
            .collect::<String>();
        Ok(Self {
            prop: Property { val: list },
        })
    }

    pub fn iter(&self) -> ListIterator {
        ListIterator { list: self, pos: 0 }
    }
}

impl<'a> Iterator for ListIterator<'a> {
    type Item = ListItem;
    fn next(&mut self) -> Option<Self::Item> {
        let content = self.list.prop.val.as_str();
        if self.pos >= content.len() {
            return None;
        }

        let remained_content = &content[self.pos..];
        let indent = if let Some(headline) =
            remained_content.split_inclusive("\n").nth(0)
        {
            let trimed_head_line = headline.trim_start();
            if !trimed_head_line.starts_with(List::LIST_MARK) {
                log::warn!(
                    "list does not start with list mark: [{}]",
                    headline
                );
                return None;
            }
            let indent = headline.len() - trimed_head_line.len();
            if indent % List::INDENT_MARK.len() != 0 {
                log::warn!("incorrect list indent: {}", headline);
                return None;
            }
            indent
        } else {
            return None;
        };

        let headline_indentstr = if indent == 0 {
            " ".repeat(2)
        } else {
            " ".repeat(indent + 2)
        };
        let headsz = remained_content
            .split_inclusive("\n")
            .enumerate()
            .take_while(|(i, line)| {
                if *i == 0 {
                    return true;
                }
                if !line.starts_with(headline_indentstr.as_str()) {
                    return false;
                }
                // This is valid mutiple head lines
                //
                //     - list head
                //       head line continued1...
                //       head line continued2...
                //
                // but this is not
                //
                //     - list head
                //        this continued head line does not align with the first line
                //       ^
                //
                // this malformed line will stop the list.
                //
                if line.len() > headline_indentstr.len()
                    && !line[headline_indentstr.len()..].starts_with(" ")
                {
                    return true;
                }
                false
            })
            // the `+ 1` means the newline character size
            .map(|(_, line)| line.len())
            .fold(0, |mut sum, val| {
                sum += val;
                sum
            });

        let body_indentstr = if indent == 0 {
            " ".repeat(4)
        } else {
            " ".repeat(indent + 4)
        };
        let bodysz = remained_content[headsz..]
            .split_inclusive("\n")
            .take_while(|&line| {
                line.trim().is_empty() || line.starts_with(&body_indentstr)
            })
            .map(|line| line.len())
            .fold(0, |mut sum, val| {
                sum += val;
                sum
            });

        let item = ListItem {
            indent: body_indentstr.len(),
            head: self.pos..(self.pos + headsz),
            body: (self.pos + headsz)..(self.pos + headsz + bodysz),
        };

        self.pos += headsz + bodysz;

        Some(item)
    }
}
