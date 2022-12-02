pub mod meta;
pub mod node;
pub mod token;

use meta::Meta;
use node::{AstNode, Node};
use token::{Token, Tokenizer, Paragraph, Subtitle, List, Codeblock, Mark, Pair, Link};

// type HookFn = dyn Fn(&AstNode) -> Option<String>;

pub struct Page {
    pub meta: Option<Meta>,
    pub ast: Node<String>,
    pub content: String,
}

impl Page {
    pub fn to_html<F>(&self, plugin: Option<F>) -> String
    where
        F: Fn(&AstNode) -> Option<String>
    {
        self.ast.to_html(self.content.as_str(), plugin)
    }
}

pub struct Parser {
    content: String,
    meta: Option<Meta>,
}

impl Parser {
    /// Create a ConciseMarkdown parser from content
    pub fn new<B: AsRef<str>>(content: B) -> Self {
        Self::new_with_plugins(content)
    }

    /// Consume current paser and generate a parsed page
    pub fn parse(self) -> Page {
        let ast = self.parse_document("div", 0, self.content.len(), 0);
        Page { meta: self.meta, ast, content: self.content }
    }

    fn new_with_plugins<B: AsRef<str>>(content: B) -> Self {
        let meta = Meta::new(content.as_ref());
        let mut content = if let Some(ref meta) = meta {
            content.as_ref()[meta.size..].to_owned()
        } else {
            content.as_ref().to_owned()
        };

        // parser requires the content ends with newline
        if !content.ends_with("\n") {
            content.push('\n');
        }

        Parser {
            content,
            meta,
        }
    }

    fn parse_document<S: AsRef<str>>(&self, root: S, pbase: usize, length: usize, indent: usize) -> Node<String> {
        let root = Node::new(root.as_ref().to_owned(), pbase..(pbase + length));
        let tokenizer = Tokenizer::new(&self.content[pbase..(pbase + length)], indent);
        let mut pbase = pbase;
        for token in tokenizer {
            let node = match token {
                Token::Paragraph(paragraph) => {
                    self.parse_paragraph(pbase, paragraph)
                }
                Token::Subtitle(subtitle) => {
                    self.parse_subtitle(pbase, subtitle)
                }
                Token::List(list) => {
                    self.parse_list(pbase, list)
                }
                Token::Codeblock(codelock) => {
                    self.parse_codeblock(pbase, codelock)
                }
                Token::BlankLine => {
                    Node::new("blankline".to_owned(), pbase..(pbase + 1))
                }
                Token::Empty => {
                    Node::new("empty".to_owned(), pbase..pbase)
                }
            };
            pbase += node.len();
            root.add(&node);
        }
        root
    }

    fn parse_statements<S: AsRef<str>>(&self, pbase: usize, text: S) -> Vec<Node<String>> {
        let mut cursor = pbase;
        let text = text.as_ref();
        let mut nodes = vec![];
        while cursor < pbase + text.len() {
            let node = self.parse_statement(cursor, &text[(cursor - pbase)..]);
            cursor += node.len();
            nodes.push(node);
        }
        nodes
    }

    fn parse_statement<S: AsRef<str>>(&self, pbase: usize, text: S) -> Node<String> {
        let text = text.as_ref();

        let chars: Vec<char> = text.chars().collect();
        let mut pos = 0;
        let mut peeked_text = String::new();
        while pos < chars.len() {
            match chars[pos] {
                '@' => {
                    if let Some(mark) = Mark::new(&chars[pos..]) {
                        if pos == 0 {
                            return Node::new("@".to_owned(), pbase..(pbase + mark.size));
                        } else {
                            break;
                        }
                    } else {
                        peeked_text.push(chars[pos]);
                    }
                }
                ch @ '$' | ch @ '`' => {
                    if let Some(pair) = Pair::new(&chars[pos..], ch) {
                        if pos == 0 {
                            let sz = pair.content.len() + pair.boundaries.len() * 2;
                            let name = if ch == '$' {
                                "math"
                            } else {
                                "inlinecode"
                            };
                            return Node::new(name.to_owned(), pbase..(pbase + sz));
                        } else {
                            // None zero position `pos` indicates that we have to stop collect
                            // `peeked_text` since we encounter another valid statement element.
                            break;
                        }
                    } else {
                        peeked_text.push(chars[pos]);
                    }
                }
                '!' | '[' => {
                    // FIXME: any better way to convert char array to string with efficiency in mind?
                    let content = chars[pos..].iter().collect::<String>();
                    if let Some(link) = Link::new(content) {
                        if pos == 0 {
                            return Node::new("link".to_owned(), pbase..(pbase + link.size));
                        } else {
                            break;
                        }
                    } else {
                        peeked_text.push(chars[pos]);
                    }
                }
                _ => {
                    peeked_text.push(chars[pos]);
                }
            }
            pos += 1;
        }

        return Node::new("text".to_owned(), pbase..(pbase + peeked_text.len()));
    }

    fn parse_paragraph(&self, pbase: usize, paragaph: Paragraph) -> Node<String> {
        let node = Node::new("p".to_owned(), pbase..(pbase + paragaph.prop.val.len()));
        let text = paragaph.prop.val.as_str();
        for subnode in self.parse_statements(pbase, text) {
            node.add(&subnode);
        }
        node
    }

    fn parse_subtitle(&self, pbase: usize, subtitle: Subtitle) -> Node<String> {
        let value = subtitle.prop.val.as_str();
        let headdeep = value.chars().take_while(|&c| c == '#').count();
        let headtag = match headdeep {
            1 => "h1",
            2 => "h2",
            _ => "h3",
        };
        let subtitle_stmt = value.trim_start_matches(|c| c == '#');
        let node = Node::new(headtag.to_owned(), pbase..(pbase + value.len()));
        for subnode in self.parse_statements(pbase + (value.len() - subtitle_stmt.len()), subtitle_stmt) {
            node.add(&subnode);
        }
        node
    }

    fn parse_list(&self, pbase: usize, list: List) -> Node<String> {
        let node = Node::new("ul".to_owned(), pbase..(pbase + list.prop.val.len()));
        for item in list.iter() {
            let list_node = Node::new("li".to_owned(), (pbase + item.head.start)..(pbase + item.body.end));
            let head_node = Node::new("listhead".to_owned(), (pbase + item.head.start)..(pbase + item.head.end));
            let head_node_content = &self.content[(pbase + item.head.start)..(pbase + item.head.end)];
            let list_indent = head_node_content.len() - head_node_content.trim_start().len();
            for head_title_node in self.parse_statements(pbase + item.head.start + list_indent + 2, &head_node_content[(list_indent + 2)..]) {
                head_node.add(&head_title_node);
            }
            list_node.add(&head_node);

            let list_body_base = pbase + item.body.start;
            let list_body_size = item.body.end - item.body.start;
            let list_body_indent = item.indent;
            let body_node = self.parse_document("listbody", list_body_base, list_body_size, list_body_indent);
            list_node.add(&body_node);

            node.add(&list_node);
        }
        node
    }

    fn parse_codeblock(&self, pbase: usize, codeblock: Codeblock) -> Node<String> {
        Node::new("codeblock".to_owned(), pbase..(pbase + codeblock.prop.val.len()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn init_logger() {
        tracing_subscriber::fmt().event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true)
        ).init();
    }

    #[test]
    fn test_subtitle() {
        init_logger();
        let tcases = [
            ("# title", "h1"),
            ("## title", "h2"),
            ("### title", "h3"),
            ("#### title", "h3"),
            ("##### title", "h3"),
            ("###### title", "h3"),
            ("####### title", "h3"),
        ];
        for (content, tag) in tcases {
            let page = Parser::new(content).parse();
            let ast = page.ast.data.borrow();
            assert_eq!(ast.tag, "div");
            assert_eq!(ast.children[0].borrow().tag, tag);
        }
    }

    #[test]
    fn test_hook() {
        let content = "![imgs][/path/to/image.jpg]";
        let page = Parser::new(content).parse();

        let imgsrc = "/images/test.png";
        let hook = |astnode: &AstNode| {
            if let AstNode::Image(name, url) = astnode {
                return Some(format!(r#"<img src="{imgsrc}" />"#))
            }
            return None
        };
        let html = page.to_html(Some(hook));
        println!("{html}")
    }
}
