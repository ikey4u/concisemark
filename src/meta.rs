use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// You can put an optional html comment (whose body is in toml format) in the front of your markdown file
/// ```text
/// <!---
/// title = "Your title"
/// subtitle = "Your subtitle"
/// date = "2021-10-13 00:00:00"
/// authors = ["name <example@gmail.com>"]
/// tags = ["demo", "example"]
/// -->
/// ```
/// This content will be parsed as your page meta [`Meta`].
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Meta {
    #[serde(with = "serde_meta_date")]
    pub date: DateTime<Utc>,
    pub title: String,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    #[doc(hidden)]
    #[serde(skip)]
    pub size: usize,
}

impl Meta {
    const META_START_MARK: &'static str = "<!---\n";
    const META_END_MARK: &'static str = "-->\n";

    pub fn new<S: AsRef<str>>(content: S) -> Option<Self> {
        let content = content.as_ref();
        let text = content.trim_start();
        if !text.starts_with(Self::META_START_MARK) {
            return None;
        }

        let start_index = content.len() - text.len();
        let end_index = if let Some(pos) = text.find(Self::META_END_MARK) {
            start_index + pos + Self::META_END_MARK.len()
        } else {
            return None;
        };

        let meta_start = start_index + Self::META_START_MARK.len();
        let meta_end = end_index - Self::META_END_MARK.len();
        let meta_text = &content[meta_start..meta_end];
        if let Ok(mut meta) = toml::from_str::<Meta>(meta_text) {
            meta.size = end_index;
            return Some(meta);
        } else {
            log::error!("failed to parse meta text: {meta_text}");
        }
        None
    }
}

mod serde_meta_date {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
