//! AST tree

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    ops::Range,
    rc::{Rc, Weak},
};

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
        Self {
            data: Rc::new(RefCell::new(data)),
        }
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
            children.push(Node {
                data: Rc::clone(&child),
            })
        }
        children
    }

    pub fn read(&self) -> Ref<NodeData> {
        self.data.borrow()
    }

    pub fn len(&self) -> usize {
        self.data.borrow().range.end - self.data.borrow().range.start
    }

    pub fn transform<F, E>(&self, hook: &F)
    where
        F: Fn(&Node) -> Result<(), E>,
    {
        _ = hook(&self);
        for child in self.children().iter() {
            _ = child.transform::<F, E>(hook);
        }
    }

    pub fn is_inlined<S: AsRef<str>>(&self, content: S) -> bool {
        let content = content.as_ref();
        let nodedata = self.data.borrow();
        match nodedata.tag.name {
            NodeTagName::Math => {
                if let Some(parent) = &nodedata.parent.upgrade() {
                    for child in
                        parent.borrow().children.iter().filter(|node| {
                            node.borrow().tag.name != NodeTagName::Math
                        })
                    {
                        let child_content = &content[child.borrow().range.start
                            ..child.borrow().range.end];
                        if !child_content.chars().all(|x| x.is_whitespace()) {
                            return false;
                        }
                    }
                }
                true
            }
            _ => return nodedata.tag.attrs.contains_key("inlined"),
        }
    }

    pub fn get_attr_or<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        name: S1,
        def: S2,
    ) -> String {
        let nodedata = self.data.borrow();
        if let Some(value) = nodedata.tag.attrs.get(name.as_ref()) {
            value.to_owned()
        } else {
            def.as_ref().to_owned()
        }
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

    pub fn with_attr<S1: AsRef<str>, S2: AsRef<str>>(
        mut self,
        key: S1,
        val: S2,
    ) -> Self {
        self.attrs
            .insert(key.as_ref().to_owned(), val.as_ref().to_owned());
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Emphasis {
    Italics,
    Bold,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeTagName {
    /// Emphasis Itaclics
    Emphasis(Emphasis),
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
