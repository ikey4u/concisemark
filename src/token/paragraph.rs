use anyhow::Result;

use super::{Pair, Property};

#[derive(Debug)]
pub struct Paragraph {
    pub prop: Property,
}

impl Paragraph {
    pub fn new(lines: &[&str], indent: usize) -> Result<Self> {
        let text = lines
            .iter()
            .take_while(|x| !x.trim().is_empty())
            .copied()
            .collect::<String>();
        let chars = text.chars().collect::<Vec<char>>();
        let mut para = String::new();
        let mut pos = 0;
        while pos < chars.len() {
            if let Some(mark) = super::Mark::new(&chars[pos..]) {
                let t = chars[pos..]
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<String>();
                let t = &t[..mark.size].to_string();
                para += t;
                pos += t.chars().count();
            } else if let Some(pair) = Pair::new(&chars[pos..], '`') {
                para += &format!(
                    "{}{}{}",
                    pair.boundaries, pair.content, pair.boundaries
                );
                pos += pair.number_of_char;
            } else if chars[pos] == '\n' {
                para.push('\n');
                pos += 1;
                let nextline: String = chars[pos..]
                    .iter()
                    .take_while(|&&x| x != '\n')
                    .map(|c| c.to_string())
                    .collect();
                let line_indent = nextline.len() - nextline.trim_start().len();
                if line_indent == indent {
                    para.push_str(&nextline);
                    pos += nextline.len();
                } else {
                    break;
                }
            } else {
                para.push(chars[pos]);
                pos += 1;
            }
        }
        Ok(Self {
            prop: Property { val: para },
        })
    }
}
