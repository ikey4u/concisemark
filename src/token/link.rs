#[derive(Debug)]
pub struct Link {
    /// name with extensible attributes (the format has not been determined for now)
    pub namex: String,
    pub uri: String,
    pub is_image_link: bool,
    pub size: usize,
}

impl Link {
    pub fn new<S: AsRef<str>>(text: S) -> Option<Self> {
        let text = text.as_ref();
        if text.is_empty() {
            return None;
        }

        let is_image_link;
        let text = match &text[0..1] {
            "!" => {
                is_image_link = true;
                &text[1..]
            }
            "[" => {
                is_image_link = false;
                text
            }
            _ => {
                return None;
            }
        };

        let middle = text.find("](")?;
        let end = if let Some(end) = &text[middle..].find(")") {
            middle + end
        } else {
            return None;
        };
        let namex = text[1..middle].to_owned();
        let uri = text[(middle + 2)..end].to_owned();
        let mut size = 1 + namex.len() + 1 + 1 + uri.len() + 1;
        if is_image_link {
            size += 1;
        }

        Some(Self {
            namex,
            uri,
            size,
            is_image_link,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link() {
        let text = "[google](https://google.com)";
        let link = Link::new(text);
        assert!(link.is_some());
        let link = link.unwrap();
        assert_eq!(link.namex.as_str(), "google");
        assert_eq!(link.uri, "https://google.com");
        assert!(!link.is_image_link);

        let text = "![google](https://google.com)";
        let link = Link::new(text);
        assert!(link.is_some());
        let link = link.unwrap();
        assert!(link.is_image_link);

        let text = "[Google Home (google)](https://google.com)";
        let link = Link::new(text);
        assert!(link.is_some());
        let link = link.unwrap();
        assert_eq!(link.namex.as_str(), "Google Home (google)");
        assert_eq!(link.uri, "https://google.com");
        assert!(!link.is_image_link);

        let text = "[Google Home (google)](https://google.com";
        let link = Link::new(text);
        assert!(link.is_none());

        let text = "[Google Home (google)] (https://google.com)";
        let link = Link::new(text);
        assert!(link.is_none());

        let text = "[Google Home (google)(https://google.com)";
        let link = Link::new(text);
        assert!(link.is_none());
    }
}
