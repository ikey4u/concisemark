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
    use std::iter;

    use crate::*;
    use node::NodeTagName;

    use indoc::indoc;
    use html5ever::tree_builder::TreeSink;
    use html5ever::QualName;
    use html5ever::driver::ParseOpts;
    use html5ever::{local_name, ns, namespace_url};
    use html5ever::parse_fragment;
    use markup5ever_rcdom::{Handle, NodeData, RcDom};
    use html5ever::tendril::TendrilSink;

    fn is_self_closing_tag(tag: &str) -> bool {
        let self_closing_tag_list = vec![
            // svg tags
            "circle", "ellipse", "line", "path", "polygon", "polyline", "rect", "stop", "use",
            // void tags
            "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
            "meta", "param", "source", "track", "wbr",
        ];
        if self_closing_tag_list.iter().any(|&i| i == tag) {
            true
        } else {
            false
        }
    }

    fn get_html_outline(dirty_html: &str) -> String {
        fn walker(indent: usize, node: &Handle) -> String {
            let indentstr = format!("{}", iter::repeat(" ").take(indent).collect::<String>());

            let mut outline = indentstr.to_string();
            match node.data {
                NodeData::Element {
                    ref name,
                    ..
                } => {
                    if is_self_closing_tag(&name.local) {
                        outline += &format!("<{}", name.local);
                    } else {
                        outline += &format!("<{}>\n", name.local);
                    }
                },
                _ => {},
            }

            for child in node.children.borrow().iter() {
                if let NodeData::Element { .. } = child.data {
                    outline += &walker(indent + 2, child);
                }
            }

            if let NodeData::Element { ref name, .. } = node.data {
                if is_self_closing_tag(&name.local) {
                    outline += &format!("/>\n");
                } else {
                    outline += &format!("{}</{}>\n", indentstr, name.local);
                }
            }

            outline
        }

        let parser = parse_fragment(
            RcDom::default(),
            ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("body")),
            vec![],
        );
        let mut dom = parser.one(dirty_html);
        let html = dom.get_document();
        let body = &html.children.borrow()[0];
        let mut outline = String::new();
        for child in body.children.borrow().iter() {
            outline += &walker(0, child);
        }
        outline
    }

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

    #[test]
    fn test_list() {
        let content = indoc! {r#"
        - [nvim](https://neovim.io/) >= 0.7.0

            nvim is great!

        - [rust](https://www.rust-lang.org/tools/install) >= 1.64
        "#};

        let page = Page::new(content);
        let html = page.render();
        let outline = get_html_outline(html.as_str());
        assert_eq!(outline, indoc! {r#"
            <div>
              <ul>
                <li>
                  <a>
                  </a>
                  <p>
                  </p>
                </li>
                <li>
                  <a>
                  </a>
                </li>
              </ul>
            </div>
        "#});
    }
}
