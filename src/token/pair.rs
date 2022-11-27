#[derive(Debug)]
pub struct Pair {
    pub content: String,
    pub number_of_char: usize,
    pub boundaries: String,
}

impl Pair {
    pub fn new(chars: &[char], boundary: char) -> Option<Self> {
        if chars.len() <= 0 || chars[0] != boundary {
            return None;
        }

        let boundaries: String = chars.iter()
            .take_while(|&&c| c == boundary)
            .map(|_c| boundary).collect();
        let content = chars[boundaries.len()..]
            .windows(boundaries.len())
            .take_while(|chunk| {
                chunk.iter().collect::<String>() != boundaries
            })
            .map(|chunk| chunk[0])
            .collect::<Vec<char>>();

        Some(Self {
            content: content.iter().collect::<String>(),
            number_of_char: boundaries.len() * 2 + content.len(),
            boundaries,
        })
    }
}
