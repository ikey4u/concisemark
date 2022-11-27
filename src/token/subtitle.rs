use super::Property;

use anyhow::Result;

#[derive(Debug)]
pub struct Subtitle {
    pub prop: Property,
}

impl Subtitle {
    pub const MARK: &str = "#";

    pub fn new(lines: &[&str]) -> Result<Self> {
        let subtitle = lines.get(0).unwrap_or(&"");
        Ok(Self {
            prop: Property {
                val: format!("{subtitle}\n"),
            }
        })
    }
}
