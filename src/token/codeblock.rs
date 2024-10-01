use anyhow::Result;

use super::Property;

#[derive(Debug)]
pub struct Codeblock {
    pub prop: Property,
}

impl Codeblock {
    pub fn new(lines: &[&str], minindent: usize) -> Result<Self> {
        let code = lines
            .iter()
            .take_while(|line| {
                let indent = line.len() - line.trim_start().len();
                indent >= minindent || line.trim().is_empty()
            })
            .copied()
            .collect::<String>();
        Ok(Self {
            prop: Property { val: code },
        })
    }
}
