use std::collections::HashMap;

use once_cell::sync::Lazy;
use indoc::formatdoc;

static SYMBOLS: Lazy<HashMap::<&'static str, char>> = Lazy::new(|| {
    let mut mp = HashMap::new();
    mp.insert("alpha", 'α');
    mp.insert("beta", 'β');
    mp.insert("gamma", 'γ');
    mp.insert("delta", 'δ');
    mp.insert("epsilon", 'ε');
    mp.insert("eta", 'η');
    mp.insert("theta", 'θ');
    mp.insert("kappa", 'κ');
    mp.insert("lambda", 'λ');
    mp.insert("mu", 'μ');
    mp.insert("pi", 'π');
    mp.insert("rho", 'ρ');
    mp.insert("sigma", 'σ');
    mp.insert("tau", 'τ');
    mp.insert("phi", 'φ');
    mp.insert("psi", 'ψ');
    mp.insert("omega", 'ω');
    mp.insert("ok", '✓');
    mp.insert("xx", 'x');
    mp.insert("l", '←');
    mp.insert("r", '→');
    mp.insert("u", '↑');
    mp.insert("d", '↓');
    mp.insert("ll", '⇐');
    mp.insert("rr", '⇒');
    mp.insert("lh", '☜');
    mp.insert("rh", '☞');
    mp.insert("*", '☆');
    mp.insert("**", '★');
    mp
});

#[derive(Debug, Default)]
pub struct Mark {
    pub name: String,
    pub attrs: String,
    pub value: String,
    pub size: usize,
}

impl Mark {
    const MARK_TAG_LIST: &'static [&'static str] = &[
        "math", "sym", "plot", "img", "video", "emoji", "a", "char", "kbd",
    ];

    // format: @<name>[attrs]{value}
    pub fn new(chars: &[char]) -> Option<Mark> {
        if chars.len() <= 0 || chars[0] != '@' {
            return None;
        }

        let mut has_syntax_error = true;
        let head: Vec<char> = chars.iter().take_while(|&&c| -> bool {
            match c {
                // allowed start mark char (I use hex to represent left brace since neovim cannot pair brace correctly)
                '\x7b' | '\x28' | '\x3c' => {
                    has_syntax_error = false;
                    false
                },
                '\n' => {
                    has_syntax_error = true;
                    false
                },
                _ => {
                    true
                }
            }
        }).map(|&c| c).collect();
        if has_syntax_error {
            return None
        }

        let attrbeg = head.iter().position(|&c| c == '[');
        let attrend = head.iter().position(|&c| c == ']');
        let (tag, attrs) = match (attrbeg, attrend) {
            (Some(beg), Some(end)) => {
                let tag = head[1..beg].iter().collect::<String>();
                let attrs = head[beg + 1..end].iter().collect::<String>();
                (tag, attrs)
            },
            (None, None) => {
                (head[1..].iter().collect::<String>(), "".to_string())
            },
            _ => {
                return None;
            }
        };
        if !Self::MARK_TAG_LIST.contains(&tag.as_str()) {
            return None;
        }

        let (start_mark_char, end_mark_char) = if let Some(&c) = chars.get(head.len()) {
            match c {
                '\x7b' => ('{', '}'),
                '\x28' => ('(', ')'),
                '\x3c' => ('<', '>'),
                _ => return None,
            }
        } else {
            return None;
        };

        let end_mark: String = chars[head.len()..].iter().take_while(|&&c| c == start_mark_char).map(|_c| end_mark_char).collect();
        // no start mark
        if end_mark.len() <= 0 {
            return None
        }

        has_syntax_error = true;
        // collect mark body (end_mark is always ASCII character, as a result, the char count
        // equlas byte count)
        let body: Vec<char> = chars[head.len() + end_mark.len()..].windows(end_mark.len()).take_while(|chunk| -> bool {
            if chunk.iter().collect::<String>() != end_mark {
                true
            } else {
                has_syntax_error = false;
                false
            }
        }).map(|chunk| chunk[0]).collect();
        if has_syntax_error {
            return None;
        }
        let body = body.iter().collect::<String>();

        Some(Mark {
            name: tag.trim().to_string(),
            attrs: attrs.trim().to_string(),
            value: body.trim().to_string(),
            size: head.iter().collect::<String>().len() + end_mark.len() + body.len() + end_mark.len(),
        })
    }

    pub fn parse(&self) -> String {
        match self.name.as_str() {
            "img" => {
                formatdoc!(r#"<img class="img-center" src="{}"/>"#, self.value).to_string()
            },
            "math" => {
                if let Ok(math) = katex::render(self.value.as_str()) {
                    format!("{}", math)
                } else {
                    format!(r#"<p class="error">failed to render math equation: {}</p>"#, self.value)
                }
            },
            "char" => {
                if let Some(c) = self.value.chars().nth(0) {
                    c.to_string()
                } else {
                    "".to_string()
                }
            },
            "emoji" => {
                let value = self.value.trim();
                let mut emojis = String::new();
                for name in value.split(";") {
                    let name = name.trim();
                    if let Some(emoji) = gh_emoji::get(name) {
                        emojis.push_str(&emoji.to_string());
                    } else {
                        emojis.push_str(&format!(" {} ", name));
                    }
                }
                emojis
            },
            "plot" => {
                format!(r#"<div id="{}"></div>"#, self.value.len())
            },
            "sym" => {
                let value = self.value.trim();
                if let Some(c) = SYMBOLS.get(value.clone()) {
                    c.to_string()
                } else {
                    format!(" {} ", value)
                }
            },
            "a" => {
                let value = self.value.trim();
                format!(r#"<a href="{}">{}</a>"#, value, value)
            },
            "kbd" => {
                let value = self.value.trim().split("+")
                    .map(|key| {
                        let key = if key.trim() == "cmd" {
                            "⌘"
                        } else {
                            key.trim()
                        };
                        format!("<kbd>{}</kbd>", key)
                    }).collect::<Vec<String>>();
                format!(r#"{}"#, value.join("+"))
            }
            _ => {
                self.value.clone()
            }
        }
    }
}
