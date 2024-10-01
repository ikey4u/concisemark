use crate::{
    meta::Meta,
    node::{Emphasis, Node, NodeTag, NodeTagName},
    token::{
        Codeblock, Heading, Link, List, Mark, Pair, Paragraph, Token, Tokenizer,
    },
};

pub struct Parser {
    content: String,
    meta: Option<Meta>,
}

impl Parser {
    /// Create a ConciseMarkdown parser from content
    pub fn new<S: AsRef<str>>(content: S) -> Self {
        let content = content.as_ref().to_owned();
        Parser {
            meta: Meta::new(content.as_str()),
            content,
        }
    }

    /// Consume current paser and generate a parsed page
    pub fn parse(self) -> (Option<Meta>, Node, String) {
        let tag = NodeTag::new(NodeTagName::Section);
        let pbase = if let Some(meta) = &self.meta {
            meta.size
        } else {
            0
        };
        let psize = self.content.len() - pbase;
        let ast = self.parse_document(tag, pbase, psize, 0);
        (self.meta, ast, self.content)
    }

    fn parse_document(
        &self,
        root: NodeTag,
        pbase: usize,
        length: usize,
        indent: usize,
    ) -> Node {
        let root = Node::new(root, pbase..(pbase + length));
        let tokenizer =
            Tokenizer::new(&self.content[pbase..(pbase + length)], indent);
        let mut pbase = pbase;
        for token in tokenizer {
            let node = match token {
                Token::Paragraph(paragraph) => {
                    self.parse_paragraph(pbase, paragraph)
                }
                Token::Heading(heading) => self.parse_heading(pbase, heading),
                Token::List(list) => self.parse_list(pbase, list),
                Token::Codeblock(codelock) => {
                    self.parse_codeblock(pbase, codelock)
                }
                Token::BlankLine(sz) => {
                    let tag = NodeTag::new(NodeTagName::BlankLine);
                    Node::new(tag, pbase..(pbase + sz))
                }
            };
            pbase += node.len();
            root.add(&node);
        }
        root
    }

    fn parse_statements<S: AsRef<str>>(
        &self,
        pbase: usize,
        text: S,
    ) -> Vec<Node> {
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

    fn parse_statement<S: AsRef<str>>(&self, pbase: usize, text: S) -> Node {
        let text = text.as_ref();

        let chars: Vec<char> = text.chars().collect();
        let mut pos = 0;
        let mut peeked_text = String::new();
        while pos < chars.len() {
            match chars[pos] {
                '@' => {
                    if let Some(mark) = Mark::new(&chars[pos..]) {
                        if pos == 0 {
                            let tag = NodeTag::new(NodeTagName::Extension);
                            return Node::new(tag, pbase..(pbase + mark.size));
                        } else {
                            break;
                        }
                    } else {
                        peeked_text.push(chars[pos]);
                    }
                }
                ch @ '*' => {
                    if let Some(pair) = Pair::new(&chars[pos..], ch) {
                        if pos != 0 {
                            break;
                        }
                        let bsz = pair.boundaries.len();
                        if bsz <= 2 {
                            let emphasis = if bsz == 1 {
                                Emphasis::Italics
                            } else {
                                Emphasis::Bold
                            };
                            let tag =
                                NodeTag::new(NodeTagName::Emphasis(emphasis));
                            let sz = pair.content.len() + bsz * 2;
                            return Node::new(tag, pbase..(pbase + sz));
                        }
                    }
                    peeked_text.push(chars[pos]);
                }
                ch @ '$' | ch @ '`' => {
                    if let Some(pair) = Pair::new(&chars[pos..], ch) {
                        if pos == 0 {
                            let sz =
                                pair.content.len() + pair.boundaries.len() * 2;
                            let tag = if ch == '$' {
                                NodeTag::new(NodeTagName::Math)
                            } else {
                                NodeTag::new(NodeTagName::Code)
                                    .with_attr("inlined", "")
                            };
                            return Node::new(tag, pbase..(pbase + sz));
                        } else {
                            // Non-zero position `pos` indicates that we have to stop collect
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
                            let tag = if link.is_image_link {
                                NodeTag::new(NodeTagName::Image)
                                    .with_attr("src", link.uri)
                                    .with_attr("name", link.namex)
                            } else {
                                NodeTag::new(NodeTagName::Link)
                                    .with_attr("href", link.uri)
                                    .with_attr("name", link.namex)
                            };
                            return Node::new(tag, pbase..(pbase + link.size));
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

        let tag = NodeTag::new(NodeTagName::Text);
        Node::new(tag, pbase..(pbase + peeked_text.len()))
    }

    fn parse_paragraph(&self, pbase: usize, paragaph: Paragraph) -> Node {
        let tag = NodeTag::new(NodeTagName::Para);
        let node = Node::new(tag, pbase..(pbase + paragaph.prop.val.len()));
        let text = paragaph.prop.val.as_str();
        for subnode in self.parse_statements(pbase, text) {
            node.add(&subnode);
        }
        node
    }

    fn parse_heading(&self, pbase: usize, heading: Heading) -> Node {
        let value = heading.prop.val.as_str();
        let heading_size = value.len();
        let heading_stmt = value.trim_start_matches(|c| c == '#');
        // heading level should between h1 to h6, see (here)[https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements]
        let heading_level = match heading_size - heading_stmt.len() {
            0..=1 => 1,
            level @ 2..=6 => level,
            _ => 6,
        };
        let tag = NodeTag::new(NodeTagName::Heading)
            .with_attr("level", heading_level.to_string());
        let node = Node::new(tag, pbase..(pbase + value.len()));
        for subnode in self.parse_statements(
            pbase + (value.len() - heading_stmt.len()),
            heading_stmt,
        ) {
            node.add(&subnode);
        }
        node
    }

    fn parse_list(&self, pbase: usize, list: List) -> Node {
        let node = Node::new(
            NodeTag::new(NodeTagName::List),
            pbase..(pbase + list.prop.val.len()),
        );
        for item in list.iter() {
            let tag = NodeTag::new(NodeTagName::ListItem);
            let list_node = Node::new(
                tag,
                (pbase + item.head.start)..(pbase + item.body.end),
            );
            let tag = NodeTag::new(NodeTagName::ListHead);
            let head_node = Node::new(
                tag,
                (pbase + item.head.start)..(pbase + item.head.end),
            );
            let head_node_content = &self.content
                [(pbase + item.head.start)..(pbase + item.head.end)];
            let list_indent =
                head_node_content.len() - head_node_content.trim_start().len();
            for head_title_node in self.parse_statements(
                pbase + item.head.start + list_indent + 2,
                &head_node_content[(list_indent + 2)..],
            ) {
                head_node.add(&head_title_node);
            }
            list_node.add(&head_node);

            let list_body_base = pbase + item.body.start;
            let list_body_size = item.body.end - item.body.start;
            let list_body_indent = item.indent;
            let body_node = self.parse_document(
                NodeTag::new(NodeTagName::ListBody),
                list_body_base,
                list_body_size,
                list_body_indent,
            );
            list_node.add(&body_node);

            node.add(&list_node);
        }
        node
    }

    fn parse_codeblock(&self, pbase: usize, codeblock: Codeblock) -> Node {
        let tag = NodeTag::new(NodeTagName::Code);
        Node::new(tag, pbase..(pbase + codeblock.prop.val.len()))
    }
}
