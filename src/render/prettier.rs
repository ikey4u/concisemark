/// split content into lines, find the common indent and remove them
pub fn remove_indent<S: AsRef<str>>(content: S) -> String {
    let content = content.as_ref();
    let mut indent = content.len();
    for line in content.lines().filter(|line| line.len() > 0) {
        let current_indent = line.len() - line.trim().len();
        if current_indent < indent {
            indent = current_indent;
        }
    }
    let content = content.lines().map(|line| {
        if line.len() > 0 {
            &line[indent..]
        } else {
            line
        }
    }).collect::<Vec<&str>>();
    content.join("\n").trim().to_owned()
}
