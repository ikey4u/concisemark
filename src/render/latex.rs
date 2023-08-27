use std::path::Path;

use indoc::formatdoc;

use super::{mark, prettier, RenderType};
use crate::node::{Emphasis, Node, NodeTagName};

#[derive(Debug)]
pub struct Cmd {
    pub name: String,
    pub posargs: Vec<String>,
    pub optargs: Vec<String>,
    pub body: String,
    pub is_enclosed: bool,
}

impl Cmd {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().trim().to_owned(),
            posargs: vec![],
            optargs: vec![],
            is_enclosed: false,
            body: "".to_owned(),
        }
    }

    pub fn enclosed(mut self) -> Self {
        self.is_enclosed = true;
        self
    }

    pub fn with_posarg<S: AsRef<str>>(mut self, posarg: S) -> Self {
        self.posargs.push(posarg.as_ref().trim().to_owned());
        self
    }

    pub fn with_optarg<S: AsRef<str>>(mut self, optarg: S) -> Self {
        self.optargs.push(optarg.as_ref().trim().to_owned());
        self
    }

    pub fn append<S: AsRef<str>>(&mut self, content: S) {
        self.body.push_str(content.as_ref())
    }

    pub fn append_cmd(&mut self, cmd: &Cmd) {
        self.body.push_str(&cmd.to_string())
    }

    pub fn to_string(&self) -> String {
        if self.name == "" {
            return self.body.clone();
        }

        let mut content = if self.is_enclosed {
            format!(r#"\begin{{{}}}"#, self.name)
        } else {
            format!(r#"\{}"#, self.name)
        };
        for optarg in self.optargs.iter() {
            content.push_str(&format!("[{}]", optarg));
        }
        for posarg in self.posargs.iter() {
            content.push_str(&format!("{{{}}}", posarg));
        }
        content.push('\n');
        if self.is_enclosed {
            content.push_str(&self.body);
            content.push_str(&format!(r#"\end{{{}}}"#, self.name));
            content.push('\n');
        }
        content
    }
}

pub fn generate<S: AsRef<str>>(node: &Node, content: S) -> String {
    let content = content.as_ref();
    let nodedata = node.data.borrow();
    let bodystr = &content[nodedata.range.start..nodedata.range.end];
    match nodedata.tag.name {
        NodeTagName::Emphasis(typ) => {
            // TODO: add unit test
            let bodystr = bodystr.trim_matches('*');
            match typ {
                Emphasis::Italics => {
                    format!(r#"\textit{{ {} }}"#, bodystr)
                }
                Emphasis::Bold => {
                    format!(r#"\textbf{{ {} }}"#, bodystr)
                }
            }
        }
        NodeTagName::Text => {
            return bodystr.to_owned();
        }
        NodeTagName::BlankLine => {
            return "".to_owned();
        }
        NodeTagName::Math => {
            let bodystr = bodystr.trim_matches(|x| x == '$').trim();
            if node.is_inlined(content) {
                return format!("$${bodystr}$$");
            } else {
                return format!("${bodystr}$");
            }
        }
        NodeTagName::Code => {
            if node.is_inlined(content) {
                let bodystr = bodystr.trim_matches(|c| c == '`').trim();
                return format!("\\verb|{bodystr}|");
            } else {
                let mut texenv =
                    Cmd::new("lstlisting").with_optarg("style=verb").enclosed();
                texenv.append(prettier::remove_indent(bodystr));
                texenv.to_string()
            }
        }
        NodeTagName::Link => {
            let url = node.get_attr_or("href", "");
            let mut name = node.get_attr_or("name", url.as_str());
            if name.len() == 0 {
                name = url.clone();
            }
            return Cmd::new("href")
                .with_posarg(url)
                .with_posarg(name)
                .to_string();
        }
        NodeTagName::List | NodeTagName::Section | NodeTagName::ListBody => {
            let mut texenv = match nodedata.tag.name {
                NodeTagName::Section | NodeTagName::ListBody => Cmd::new(""),
                _ => Cmd::new("itemize").enclosed(),
            };
            for child in node.children().iter() {
                texenv.append(generate(child, content).as_str());
            }
            return texenv.to_string();
        }
        NodeTagName::ListItem => {
            let mut text = Cmd::new("item").to_string();
            for child in node.children().iter() {
                text.push_str(generate(child, content).as_str());
            }
            return text;
        }
        NodeTagName::ListHead | NodeTagName::Para => {
            let mut text = String::new();
            if nodedata.tag.name == NodeTagName::Para {
                text.push('\n');
            }
            for child in node.children().iter() {
                text.push_str(generate(child, content).as_str());
            }
            return text.to_owned();
        }
        NodeTagName::Image => {
            let alt = node.get_attr_or("name", "image link is broken");
            let src = node.get_attr_or("src", "");
            let imgpath = Path::new(&src);
            if imgpath.exists() {
                let mut cmd = Cmd::new("figure").enclosed().with_optarg("H");
                cmd.append(formatdoc!("
                    \\centerline{{\\includegraphics[width=0.7\\textwidth]{{{src}}}}}
                    \\caption{{{alt}}}
                "));
                cmd.to_string()
            } else {
                log::warn!(
                    "image path [{}] does not exist, ignored.",
                    imgpath.display()
                );
                "\n\n\\textbf{could not find image}\n\n".to_owned()
            }
        }
        NodeTagName::Heading => {
            let level = match nodedata
                .tag
                .attrs
                .get("level")
                .map(|s| s.as_str().parse::<usize>())
            {
                Some(Ok(level)) => level,
                _ => {
                    log::warn!(
                        "heading level parse failed: {:?}, set it to level 1",
                        nodedata.tag.attrs.get("level")
                    );
                    1
                }
            };
            let name = match level {
                1 => "section",
                2 => "subsection",
                _ => "subsubsection",
            };
            let mut text = String::new();
            for child in node.children().iter() {
                text.push_str(generate(child, content).as_str());
            }
            Cmd::new(name).with_posarg(text).to_string()
        }
        NodeTagName::Extension => {
            if let Some(value) = mark::generate(bodystr, RenderType::Latex) {
                return value;
            } else {
                log::warn!("unsupported mark element: {}", bodystr);
                return format!("{bodystr}");
            }
        }
    }
}
