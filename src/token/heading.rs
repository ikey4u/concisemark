use super::Property;

use anyhow::Result;

#[derive(Debug)]
pub struct Heading {
    pub prop: Property,
}

impl Heading {
    pub const MARK: &str = "#";

    pub fn new(lines: &[&str]) -> Result<Self> {
        let heading = lines.get(0).unwrap_or(&"");
        Ok(Self {
            prop: Property {
                val: format!("{heading}\n"),
            }
        })
    }
}
