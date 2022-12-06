//! AST tree
use crate::token::Mark;
use crate::utils;

use std::collections::HashMap;
use std::rc::{Weak, Rc};
use std::ops::Range;
use std::cell::{Ref, RefCell};

/// An AST node
///
/// It seems impossible to use only one structure to implement feature that
///
/// 1. add a child `C` to current node
/// 2. and change the parent of `C` to current node (this is where the impossible comes from ...)
///
/// As a result, we use a compromise way to finish that task.
///
/// We separate node and its data into two structures [`Node`] and [`NodeData`],
/// [`Node`] is the main interface when interact with a tree, and [`NodeData`] should be used internal
/// only.
///
#[derive(Debug)]
pub struct Node {
    // Node data is shared by multiple nodes and can be changed around, that's why we wrap it in Rc
    // and RefCell
    pub data: Rc<RefCell<NodeData>>,
}

impl Node {
    pub fn new(tag: NodeTag, range: Range<usize>) -> Self {
        let data = NodeData {
            tag,
            range: range.start..range.end,
            // TODO: a temporary hack...
            content_range: range.start..range.end,
            parent: Weak::new(),
            children: Vec::new(),
        };
        Self { data: Rc::new(RefCell::new(data)) }
    }

    pub fn rc(&self) -> Rc<RefCell<NodeData>> {
        Rc::clone(&self.data)
    }

    pub fn add(&self, node: &Node) {
        self.data.borrow_mut().children.push(node.rc());
        node.data.borrow_mut().parent = Rc::downgrade(&self.rc());
    }

    pub fn children(&self) -> Vec<Node> {
        let mut children = vec![];
        for child in self.data.borrow().children.iter() {
            children.push(Node { data: Rc::clone(&child) })
        }
        children
    }

    pub fn read(&self) -> Ref<NodeData> {
        self.data.borrow()
    }

    pub fn len(&self) -> usize {
        self.data.borrow().range.end - self.data.borrow().range.start
    }

    pub fn transform<F>(&self, hook: &F)
    where
        F: Fn(&Node)
    {
        hook(&self);
        for child in self.children().iter() {
            child.transform(hook);
        }
    }

    pub fn get_content<S: AsRef<str>>(&self, content: S) -> String {
        let content = content.as_ref();
        content[self.data.borrow().range.start..self.data.borrow().range.end].to_owned()
    }

    pub fn render<S: AsRef<str>, F>(&self, content: S, hook: Option<&F>) -> String
    where
        F: Fn(&Node) -> Option<String>
    {
        if let Some(hook) = hook {
            if let Some(html) = hook(&self) {
                return html;
            }
        }

        let content = content.as_ref();
        let nodedata = self.data.borrow();
        let bodystr = &content[nodedata.range.start..nodedata.range.end];

        // Render all void tag.
        //
        // Void tag contains no content, but only name and optional attrs see [4.3. Elements](https://www.w3.org/TR/2011/WD-html-markup-20110113/syntax.html#syntax-elements).
        //
        // Note that ConciseMark extends this concept to denote a node that contains optional
        // characters body as its value.
        //
        match nodedata.tag.name {
            NodeTagName::Text => return bodystr.to_owned(),
            NodeTagName::Code => {
                if nodedata.tag.attrs.contains_key("inlined") {
                    return format!("<code>{}</code>", utils::escape_to_html(bodystr.trim_matches(|c| c == '`')));
                } else {
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
                    return format!("<pre><code>{}</pre></code>", utils::escape_to_html(bodystr.join("\n").trim()));
                }
            }
            NodeTagName::Math => {
                // if math node is leaf node, then we render it in display mode
                let is_leaf = if let Some(parent) = &nodedata.parent.upgrade() {
                    // tailing newline character always counts one AST node
                    parent.borrow().children.len() == 2
                } else {
                    false
                };

                let opts = match katex::Opts::builder().display_mode(is_leaf).build() {
                    Ok(opts) => opts,
                    Err(e) => {
                        log::warn!("failed to create katex options: {e:?}");
                        return bodystr.to_owned();
                    }
                };
                if let Ok(math) = katex::render_with_opts(bodystr.trim_matches(|x| x == '$'), &opts) {
                    return format!("{}", math);
                } else {
                    log::warn!("failed to render math equation: {}", bodystr);
                    return bodystr.to_owned();
                }
            }
            NodeTagName::Link => {
                let url = if let Some(url) = nodedata.tag.attrs.get("href") {
                    url.to_owned()
                } else {
                    "".to_owned()
                };
                let mut name = if let Some(name) = nodedata.tag.attrs.get("name") {
                    name.to_owned()
                } else {
                    "".to_owned()
                };
                if name.len() == 0 {
                    name = url.clone();
                }
                return format!(r#"<a href="{}">{}</a>"#, url, name);
            }
            NodeTagName::Image => {
                let alt = if let Some(name) = nodedata.tag.attrs.get("name") {
                    name.to_owned()
                } else {
                    format!("image link is broken")
                };
                let src = if let Some(src) = nodedata.tag.attrs.get("src") {
                    src.to_owned()
                } else {
                    format!("")
                };
                return format!(r#"<img alt="{alt}" src="{src}"/>"#);
            }
            NodeTagName::Extension => {
                let chars = bodystr.chars().collect::<Vec<char>>();
                if let Some(mark) = Mark::new(&chars[..]) {
                    return mark.parse();
                } else {
                    log::warn!("unsupported mark element: {}", bodystr);
                    return format!("<pre><code>{}</pre></code>", bodystr);
                }
            }
            _ => {}
        }

        // Render all non-void element
        let mut html = String::new();
        let (start_tag, end_tag) = if let Some(mark) = nodedata.tag.get_markup() {
            (format!("<{mark}>"), format!("</{mark}>"))
        } else {
            ("".to_owned(), "".to_owned())
        };
        html += &start_tag;
        for child in self.children().iter() {
            html.push_str(child.render(content, hook).as_str());
        }
        html += &end_tag;

        html
    }
}

/// Data contained in a [`Node`]
#[derive(Debug)]
pub struct NodeData {
    pub tag: NodeTag,
    // The full range of this node. Note that we do not store node text directly but rather a cheap
    // range which can be used to index into markdown text
    pub range: Range<usize>,
    // The content range of this node
    pub content_range: Range<usize>,
    // The parent of this node. Use `Weak` to avoid recycle references.
    pub parent: Weak<RefCell<NodeData>>,
    pub children: Vec<Rc<RefCell<NodeData>>>,
}

/// Meta information for [`Node`]
#[derive(Debug, PartialEq)]
pub struct NodeTag {
    /// Node name
    pub name: NodeTagName,
    /// Node attributes
    pub attrs: HashMap<String, String>,
}

impl NodeTag {
    pub fn new(name: NodeTagName) -> Self {
        Self {
            name,
            attrs: HashMap::new(),
        }
    }

    pub fn with_attr<S1: AsRef<str>, S2: AsRef<str>>(mut self, key: S1, val: S2) -> Self {
        self.attrs.insert(key.as_ref().to_owned(), val.as_ref().to_owned());
        self
    }

    pub fn get_markup(&self) -> Option<String> {
        match self.name {
            NodeTagName::Heading => {
                let level = match self.attrs.get("level").map(|s| s.as_str().parse::<usize>()) {
                    Some(Ok(level)) => {
                        level
                    }
                    _ => {
                        log::warn!("heading level parse failed: {:?}, set it to level 1", self.attrs.get("level"));
                        1
                    }
                };
                Some(format!("h{level}"))
            },
            NodeTagName::Section => Some("div".to_owned()),
            NodeTagName::Para => Some("p".to_owned()),
            NodeTagName::Code => Some("code".to_owned()),
            NodeTagName::Link => Some("a".to_owned()),
            NodeTagName::Image => Some("img".to_owned()),
            NodeTagName::List => Some("ul".to_owned()),
            NodeTagName::ListItem => Some("li".to_owned()),
            _ => None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeTagName {
    /// A title
    Heading,
    /// A seciton
    Section,
    /// A paragraph
    Para,
    /// Codeblock or inlined code
    Code,
    /// A math symbol or equation
    Math,
    /// A URL link
    Link,
    /// An image link
    Image,
    /// Charaters data
    Text,
    /// To parse a list, ConciseMark split list into several segments, take the following as an
    /// example
    ///
    /// ```text
    /// - This is a item
    ///
    ///     Some content
    ///
    /// - This is another list item
    ///
    ///     Some other content
    /// ```
    /// After parsing above list, we got a [`NodeTagName::List`]
    ///
    /// ```text
    /// +----------------------------+
    /// |- This is a item            |
    /// |                            |
    /// |    Some content            |
    /// |                            |
    /// |- This is another list item |
    /// |                            |
    /// |    Some other content      |
    /// +----------------------------+
    /// ```
    ///
    /// And two [`NodeTagName::ListItem`]s
    ///
    /// ```text
    /// +----------------------------+
    /// |- This is a item            |
    /// |                            |
    /// |    Some content            |
    /// +----------------------------+
    /// |- This is another list item |
    /// |                            |
    /// |    Some other content      |
    /// +----------------------------+
    /// ```
    ///
    /// For each [`NodeTagName::ListItem`], we have a [`NodeTagName::ListHead`]
    ///
    /// ```text
    ///  +--------------------------+
    /// -|This is a item            |
    ///  +--------------------------+
    ///
    ///     Some content
    ///  +--------------------------+
    /// -|This is another list item |
    ///  +--------------------------+
    ///
    ///      Some other content
    /// ```
    ///
    /// and a [`NodeTagName::ListBody`]
    ///
    /// ```text
    /// - This is a item
    ///
    ///     +-----------------------+
    ///     |Some content           |
    ///     +-----------------------+
    /// - This is another list item
    ///
    ///     +-----------------------+
    ///     |Some other content     |
    ///     +-----------------------+
    /// ```
    List,
    /// See [`NodeTagName::List`]
    ListItem,
    /// See [`NodeTagName::List`]
    ListHead,
    /// See [`NodeTagName::List`]
    ListBody,
    /// ConciseMark extension
    Extension,
    // Just a blank line
    BlankLine,
}
