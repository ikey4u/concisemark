mod meta;
mod token;
mod node;

use node::Node;
use meta::Meta;

use token::{Token, Tokenizer, Paragraph, Subtitle, List, Codeblock, Mark, Pair, Link};
use anyhow::Result;

pub trait Plugin {
    fn run(&self) -> Result<String>;
    fn name(&self) -> &'static str;
    fn marks(&self) -> Vec<&'static str>;
}

#[derive(Debug)]
pub struct Page {
    pub meta: Option<Meta>,
    pub ast: Node<String>,
    pub content: String,
}

impl Page {
    pub fn to_html(&self) -> String {
        self.ast.to_html(self.content.as_str())
    }
}

pub struct Parser {
    content: String,
    plugins: Vec<Box<dyn Plugin>>,
    meta: Option<Meta>,
}

impl Parser {
    pub fn new<B: AsRef<str>>(content: B) -> Self {
        Self::new_with_plugins(content, vec![])
    }

    pub fn new_with_plugins<B: AsRef<str>>(content: B, plugins: Vec<Box<dyn Plugin>>) -> Self {
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
            plugins,
            meta,
        }
    }

    pub fn parse(&self) -> Page {
        let ast = self.parse_document("div", 0, self.content.len(), 0);
        Page { meta: self.meta.clone(), ast, content: self.content.to_string() }
    }

    pub fn parse_document<S: AsRef<str>>(&self, root: S, pbase: usize, length: usize, indent: usize) -> Node<String> {
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

    pub fn parse_statements<S: AsRef<str>>(&self, pbase: usize, text: S) -> Vec<Node<String>> {
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

    pub fn parse_statement<S: AsRef<str>>(&self, pbase: usize, text: S) -> Node<String> {
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

    pub fn parse_paragraph(&self, pbase: usize, paragaph: Paragraph) -> Node<String> {
        let node = Node::new("p".to_owned(), pbase..(pbase + paragaph.prop.val.len()));
        let text = paragaph.prop.val.as_str();
        for subnode in self.parse_statements(pbase, text) {
            node.add(&subnode);
        }
        node
    }

    pub fn parse_subtitle(&self, pbase: usize, subtitle: Subtitle) -> Node<String> {
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

    pub fn parse_list(&self, pbase: usize, list: List) -> Node<String> {
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

    pub fn parse_codeblock(&self, pbase: usize, codeblock: Codeblock) -> Node<String> {
        Node::new("codeblock".to_owned(), pbase..(pbase + codeblock.prop.val.len()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_render() {
        tracing_subscriber::fmt().event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true)
        ).init();

        let content = "# Title";
        let parser = Parser::new(content);
        let page = parser.parse();
        let ast = page.ast.data.borrow();
        assert_eq!(ast.tag, "div");
        assert_eq!(ast.children[0].borrow().tag, "h1");
    }
}
