use anyhow::Result;

use super::Property;

#[derive(Debug)]
pub struct Heading {
    pub prop: Property,
}

impl Heading {
    pub const MARK: &'static str = "#";

    pub fn new(heading: &str) -> Result<Self> {
        Ok(Self {
            prop: Property {
                val: heading.to_string(),
            },
        })
    }
}
