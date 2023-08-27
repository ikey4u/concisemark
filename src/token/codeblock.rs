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
                if indent >= minindent || line.trim().len() == 0 {
                    true
                } else {
                    false
                }
            })
            .map(|&x| x)
            .collect::<Vec<&str>>()
            .join("\n");
        Ok(Self {
            prop: Property { val: code + "\n" },
        })
    }
}
