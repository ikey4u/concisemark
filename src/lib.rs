//! # ConciseMark - a simplified markdown parsing library
//!
//! ## Usage
//!
//!     use concisemark::Page;
//!
//!     let content = "# Title";
//!     let page = Page::new(content);
//!     let html = page.render();
//!
//! The output html will be
//!
//! ```text
//! <div><h1> Title\n</h1></div>
//! ```
//!
//! The outermost `div` is the root of the rendered html page.
//!
//! ## Hook
//!
//! `page` maintains a AST structure which you can use to hook the node you are
//! interested, please see [`Page`] for more information.
//!
pub mod meta;
pub mod node;
pub mod token;
mod parser;

use meta::Meta;
use node::Node;
use parser::Parser;

pub struct Page {
    pub meta: Option<Meta>,
    pub ast: Node,
    pub content: String,
}

impl Page {
    /// Create a new markdown page from `content`
    pub fn new<S: AsRef<str>>(content: S) -> Self {
        Parser::new(content).parse()
    }

    /// Render markdown into HTML page
    pub fn render(&self) -> String {
        self.render_with_hook(&|_| {
            None
        })
    }

    /// Render markdown into HTML page with hook
    ///
    /// If the hook returns None, then the default rendering function will be used or else
    /// use the returned value as render result.
    pub fn render_with_hook<F>(&self, hook: &F) -> String
    where
        F: Fn(&Node) -> Option<String>
    {
        self.ast.render(self.content.as_str(), Some(hook))
    }

    /// Modify markdown AST node with hook
    ///
    /// The following is an exmaple to change image url
    ///
    ///     use concisemark::node::{Node, NodeTagName};
    ///     use concisemark::Page;
    ///
    ///     let content = "![imgs](/path/to/image.jpg)";
    ///     let page = Page::new(content);
    ///     let hook = |node: &Node| {
    ///         let mut nodedata = node.data.borrow_mut();
    ///         if nodedata.tag.name == NodeTagName::Image {
    ///             let src = nodedata.tag.attrs.get("src").unwrap().to_owned();
    ///             let src = if src.starts_with("/") {
    ///                 format!("https://example.com{src}")
    ///             } else {
    ///                 format!("https://example.com/{src}")
    ///             };
    ///             nodedata.tag.attrs.insert("src".to_owned(), src);
    ///         }
    ///     };
    ///     let img = &page.ast.children()[0].children()[0];
    ///     assert_eq!(img.data.borrow().tag.attrs.get("src").map(|s| s.as_str()), Some("/path/to/image.jpg"));
    ///     page.transform(hook);
    ///     assert_eq!(img.data.borrow().tag.attrs.get("src").map(|s| s.as_str()), Some("https://example.com/path/to/image.jpg"));
    ///
    pub fn transform<F>(&self, hook: F)
    where
        F: Fn(&Node)
    {
        self.ast.transform(&hook);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use node::NodeTagName;

    #[test]
    fn test_heading() {
        let tcases = [
            ("# title", "1"),
            ("## title", "2"),
            ("### title", "3"),
            ("#### title", "4"),
            ("##### title", "5"),
            ("###### title", "6"),
            ("####### title", "6"),
        ];
        for (content, level) in tcases {
            let page = Page::new(content);
            let ast = page.ast.data.borrow();
            assert_eq!(ast.tag.name, NodeTagName::Section);
            assert_eq!(ast.children[0].borrow().tag.name, NodeTagName::Heading);
            assert_eq!(ast.children[0].borrow().tag.attrs.get("level").map(|s| s.as_str()), Some(level));
        }
    }
}
