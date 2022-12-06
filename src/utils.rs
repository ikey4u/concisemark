pub fn escape_to_html(text: &str) -> String {
    let mut html = String::new();
    for ch in text.chars() {
        match ch {
            '&' => {
                html.push_str("&amp;");
            },
            '>' => {
                html.push_str("&gt;");
            },
            '<' => {
                html.push_str("&lt;");
            },
            _ => {
                html.push(ch);
            }
        }
    }
    html
}
