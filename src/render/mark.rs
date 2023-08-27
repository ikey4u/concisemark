use super::RenderType;
use crate::{token::Mark, utils};

pub fn generate<S: AsRef<str>>(content: S, typ: RenderType) -> Option<String> {
    let content = content.as_ref();
    if let Some(mark) = Mark::new_from_str(content) {
        match mark.name.as_str() {
            "char" => {
                if let Some(c) = mark.value.chars().nth(0) {
                    if typ == RenderType::Html {
                        return Some(utils::escape_to_html(&c.to_string()));
                    } else {
                        return Some(utils::escape_to_tex(&c.to_string()));
                    }
                }
                return Some("".to_string());
            }
            "emoji" => {
                let value = mark.value.trim();
                let mut emojis = String::new();
                for name in value.split(";") {
                    let name = name.trim();
                    if let Some(emoji) = gh_emoji::get(name) {
                        emojis.push_str(&emoji.to_string());
                    } else {
                        emojis.push_str(&format!(" {} ", name));
                    }
                }
                return Some(emojis);
            }
            "kbd" => {
                let value = mark
                    .value
                    .trim()
                    .split("+")
                    .map(|key| {
                        let key = if key.trim() == "cmd" {
                            "⌘"
                        } else {
                            key.trim()
                        };
                        if typ == RenderType::Html {
                            format!("<kbd>{}</kbd>", key)
                        } else {
                            key.to_owned()
                        }
                    })
                    .collect::<Vec<String>>();
                return Some(format!(r#"{}"#, value.join("+")));
            }
            _ => {
                return Some(mark.value);
            }
        }
    }
    None
}
