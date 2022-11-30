use crate::token::{Mark, Link};

use std::fmt::Display;
use std::rc::{Weak, Rc};
use std::ops::Range;
use std::cell::{Ref, RefCell};

#[derive(Debug)]
pub struct NodeData<T: Display + AsRef<str>> {
    pub tag: T,
    pub range: Range<usize>,
    pub parent: Weak<RefCell<NodeData<T>>>,
    pub children: Vec<Rc<RefCell<NodeData<T>>>>,
}

#[derive(Debug)]
pub struct Node<T: Display + AsRef<str>> {
    pub data: Rc<RefCell<NodeData<T>>>,
}

impl<T: Display + AsRef<str>> Node<T> {
    pub fn new(tag: T, range: Range<usize>) -> Self {
        let data = NodeData {
            tag,
            range,
            parent: Weak::new(),
            children: Vec::new(),
        };
        Self { data: Rc::new(RefCell::new(data)) }
    }

    pub fn rc(&self) -> Rc<RefCell<NodeData<T>>> {
        Rc::clone(&self.data)
    }

    pub fn add(&self, node: &Node<T>) {
        self.data.borrow_mut().children.push(node.rc());
        node.data.borrow_mut().parent = Rc::downgrade(&self.rc());
    }

    pub fn read(&self) -> Ref<NodeData<T>> {
        self.data.borrow()
    }

    pub fn len(&self) -> usize {
        self.data.borrow().range.end - self.data.borrow().range.start
    }

    pub fn to_html<S: AsRef<str>>(&self, content: S) -> String {
        self.data.borrow().to_html(content)
    }
}

impl<T: Display + AsRef<str>> NodeData<T> {
    fn to_html<S: AsRef<str>>(&self, content: S) -> String {
        let content = content.as_ref();
        let bodystr = &content[self.range.start..self.range.end];
        let (start_tag, end_tag) = match self.tag.as_ref() {
            "text" => {
                return bodystr.to_owned();
            }
            "div" | "p" | "li" | "ul" | "h1" | "h2" | "h3" => {
                (format!("<{}>", self.tag.as_ref()), format!("</{}>", self.tag.as_ref()))
            }
            "listhead" | "listbody" => {
                ("".to_owned(), "".to_owned())
            }
            "inlinecode" => {
                return format!("<code>{}</code>", bodystr.trim_matches(|c| c == '`'));
            }
            "codeblock" => {
                let mut indent = bodystr.len();
                for line in bodystr.lines().filter(|line| line.len() > 0) {
                    let current_indent = line.len() - line.trim().len();
                    if current_indent < indent {
                        indent = current_indent;
                    }
                }
                let bodystr = bodystr.lines().map(|line| {
                    if line.len() > 0 {
                        &line[indent..]
                    } else {
                        line
                    }
                }).collect::<Vec<&str>>();
                return format!("<pre><code>{}</pre></code>", bodystr.join("\n").trim());
            }
            "@" => {
                let chars = bodystr.chars().collect::<Vec<char>>();
                if let Some(mark) = Mark::new(&chars[..]) {
                    return mark.parse();
                } else {
                    log::warn!("unsupported mark element: {}", bodystr);
                    return format!("<pre><code>{}</pre></code>", bodystr);
                }
            }
            "math" => {
                if let Ok(math) = katex::render(bodystr.trim_matches(|x| x == '$')) {
                    return format!("{}", math);
                } else {
                    log::warn!("failed to render math equation: {}", bodystr);
                    return bodystr.to_owned();
                }
            }
            "link" => {
                if let Some(link) = Link::new(bodystr) {
                    if link.is_image_link {
                        return format!(r#"<img src="{}"/>"#, link.uri);
                    } else {
                        let name = if link.namex.len() == 0 {
                            link.uri.as_str()
                        } else {
                            link.namex.as_str()
                        };
                        return format!(r#"<a href="{}">{}</a>"#, link.uri, name);
                    }
                } else {
                    log::warn!("failed to parse link: {}", bodystr);
                    return bodystr.to_owned();
                }
            }
            _ => {
                ("".to_owned(), "".to_owned())
            }
        };

        let mut html = start_tag;
        for child in self.children.iter() {
            html += &child.borrow().to_html(content);
        }
        html += &end_tag;
        html
    }
}
