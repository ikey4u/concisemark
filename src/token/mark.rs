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

    pub fn new_from_str<S: AsRef<str>>(content: S) -> Option<Mark> {
        let content = content.as_ref().chars().collect::<Vec<char>>();
        Self::new(&content[..])
    }

    // format: @<name>[attrs]{value}
    pub fn new(chars: &[char]) -> Option<Mark> {
        if chars.is_empty() || chars[0] != '@' {
            return None;
        }

        let mut has_syntax_error = true;
        let head: Vec<char> = chars
            .iter()
            .take_while(|&&c| -> bool {
                match c {
                    // allowed start mark char (I use hex to represent left brace since neovim cannot pair brace correctly)
                    '\x7b' | '\x28' | '\x3c' => {
                        has_syntax_error = false;
                        false
                    }
                    '\n' => {
                        has_syntax_error = true;
                        false
                    }
                    _ => true,
                }
            })
            .copied()
            .collect();
        if has_syntax_error {
            return None;
        }

        let attrbeg = head.iter().position(|&c| c == '[');
        let attrend = head.iter().position(|&c| c == ']');
        let (tag, attrs) = match (attrbeg, attrend) {
            (Some(beg), Some(end)) => {
                let tag = head[1..beg].iter().collect::<String>();
                let attrs = head[beg + 1..end].iter().collect::<String>();
                (tag, attrs)
            }
            (None, None) => {
                (head[1..].iter().collect::<String>(), "".to_string())
            }
            _ => {
                return None;
            }
        };
        if !Self::MARK_TAG_LIST.contains(&tag.as_str()) {
            return None;
        }

        let (start_mark_char, end_mark_char) =
            if let Some(&c) = chars.get(head.len()) {
                match c {
                    '\x7b' => ('{', '}'),
                    '\x28' => ('(', ')'),
                    '\x3c' => ('<', '>'),
                    _ => return None,
                }
            } else {
                return None;
            };

        let end_mark: String = chars[head.len()..]
            .iter()
            .take_while(|&&c| c == start_mark_char)
            .map(|_c| end_mark_char)
            .collect();
        // no start mark
        if end_mark.is_empty() {
            return None;
        }

        has_syntax_error = true;
        // collect mark body (end_mark is always ASCII character, as a result, the char count
        // equlas byte count)
        let body: Vec<char> = chars[head.len() + end_mark.len()..]
            .windows(end_mark.len())
            .take_while(|chunk| -> bool {
                if chunk.iter().collect::<String>() != end_mark {
                    true
                } else {
                    has_syntax_error = false;
                    false
                }
            })
            .map(|chunk| chunk[0])
            .collect();
        if has_syntax_error {
            return None;
        }
        let body = body.iter().collect::<String>();

        Some(Mark {
            name: tag.trim().to_string(),
            attrs: attrs.trim().to_string(),
            value: body.trim().to_string(),
            size: head.iter().collect::<String>().len()
                + end_mark.len()
                + body.len()
                + end_mark.len(),
        })
    }
}
