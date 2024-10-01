use std::{path::Path};

use indoc::formatdoc;

use super::{mark, RenderType};
use crate::{
    node::{Emphasis, Node, NodeTagName},
    utils,
};

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
        if self.name.is_empty() {
            return self.body.to_owned();
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
            content.push_str(self.body.trim());
            content.push('\n');
            content.push_str(&format!(r#"\end{{{}}}"#, self.name));
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
        NodeTagName::Text => bodystr.to_owned(),
        NodeTagName::BlankLine => "".to_owned(),
        NodeTagName::Math => {
            let bodystr = bodystr.trim_matches(|x| x == '$').trim();
            if node.is_inlined(content) {
                format!("$${bodystr}$$")
            } else {
                format!("${bodystr}$")
            }
        }
        NodeTagName::Code => {
            if node.is_inlined(content) {
                let bodystr = bodystr.trim_matches(|c| c == '`').trim();
                format!("\\verb|{bodystr}|")
            } else {
                let mut texenv =
                    Cmd::new("lstlisting").with_optarg("style=verb").enclosed();
                texenv.append(utils::remove_indent(bodystr));
                texenv.to_string()
            }
        }
        NodeTagName::Link => {
            let url = node.get_attr_or("href", "");
            let mut name = node.get_attr_or("name", url.as_str());
            if name.is_empty() {
                name = url.clone();
            }
            Cmd::new("href")
                .with_posarg(url)
                .with_posarg(name)
                .to_string()
        }
        NodeTagName::List | NodeTagName::Section | NodeTagName::ListBody => {
            let mut texenv = match nodedata.tag.name {
                NodeTagName::Section | NodeTagName::ListBody => Cmd::new(""),
                _ => Cmd::new("itemize").enclosed(),
            };
            for child in node.children().iter() {
                texenv.append(generate(child, content).as_str());
            }
            texenv.to_string()
        }
        NodeTagName::ListItem => {
            let mut text = Cmd::new("item").to_string();
            for child in node.children().iter() {
                text.push_str(generate(child, content).as_str());
            }
            text
        }
        NodeTagName::ListHead | NodeTagName::Para => {
            let mut text = String::new();
            if nodedata.tag.name == NodeTagName::Para {
                text.push('\n');
            }
            for child in node.children().iter().filter(|x| {
                let (start, end) =
                    (x.data.borrow().range.start, x.data.borrow().range.end);
                !content[start..end].trim().is_empty()
            }) {
                text.push_str(generate(child, content).as_str());
            }
            text.to_owned()
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
                cmd.to_string().to_owned()
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
                value
            } else {
                log::warn!("unsupported mark element: {}", bodystr);
                bodystr.to_string()
            }
        }
    }
}
