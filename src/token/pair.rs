#[derive(Debug, PartialEq)]
pub struct Pair {
    pub content: String,
    pub number_of_char: usize,
    pub boundaries: String,
}

impl Pair {
    pub fn new(chars: &[char], boundary: char) -> Option<Self> {
        if chars.is_empty() || chars[0] != boundary {
            return None;
        }

        let boundaries: String = chars
            .iter()
            .take_while(|&&c| c == boundary)
            .map(|_c| boundary)
            .collect();
        let content = chars[boundaries.len()..]
            .windows(boundaries.len())
            .take_while(|chunk| chunk.iter().collect::<String>() != boundaries)
            .map(|chunk| chunk[0])
            .collect::<Vec<char>>();

        let consuemd_char_count = boundaries.len() + content.len();
        let suffix = chars[consuemd_char_count..]
            .iter()
            .take(boundaries.len())
            .collect::<String>();
        if suffix != boundaries {
            return None;
        }

        let number_of_char = boundaries.len() * 2 + content.len();
        Some(Self {
            content: content.iter().collect::<String>(),
            number_of_char,
            boundaries,
        })
    }

    pub fn from_str<S: AsRef<str>>(content: S, boundary: char) -> Option<Self> {
        let chars = content.as_ref().chars().collect::<Vec<char>>();
        Pair::new(&chars[..], boundary)
    }
}

#[cfg(test)]
mod tests {
    use super::Pair;

    #[test]
    fn test_pair() {
        let pair = Pair::from_str("$\n", '$');
        assert!(pair.is_none());

        let pair = Pair::from_str("`", '`');
        assert!(pair.is_none());

        let pair = Pair::from_str("`inline code`", '`');
        assert_eq!(
            pair,
            Some(Pair {
                content: "inline code".to_owned(),
                number_of_char: 13,
                boundaries: "`".to_owned()
            })
        );

        let pair = Pair::from_str("``$``", '`');
        assert_eq!(
            pair,
            Some(Pair {
                content: "$".to_owned(),
                number_of_char: 5,
                boundaries: "``".to_owned()
            })
        );
    }
}
