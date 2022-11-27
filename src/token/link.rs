#[derive(Debug)]
pub struct Link {
    pub namex: String,
    pub uri: String,
    pub is_image_link: bool,
    pub size: usize,
}

impl Link {
    pub fn new<S: AsRef<str>>(text: S) -> Option<Self> {
        let text = text.as_ref();
        if text.len() == 0 {
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

        let (middle, end) = match (text.find("]("), text.find(")")) {
            (Some(middle), Some(end)) => {
                if middle >= end {
                    return None;
                }
                (middle, end)
            }
            _ => {
                return None;
            }
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
