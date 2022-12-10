use crate::node::{Node, NodeTagName};
use crate::utils;
use super::{prettier, mark, RenderType};

pub fn generate<S: AsRef<str>, F>(node: &Node, content: S, hook: Option<&F>) -> String
where
    F: Fn(&Node) -> Option<String>
{
    if let Some(hook) = hook {
        if let Some(html) = hook(&node) {
            return html;
        }
    }

    let content = content.as_ref();
    let nodedata = node.data.borrow();
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
            if node.is_inlined(content) {
                return format!("<code>{}</code>", utils::escape_to_html(bodystr.trim_matches(|c| c == '`').trim()));
            } else {
                return format!(
                    "<pre><code>{}</pre></code>",
                    utils::escape_to_html(prettier::remove_indent(bodystr).as_str())
                );
            }
        }
        NodeTagName::Math => {
            let opts = match katex::Opts::builder().display_mode(node.is_inlined(content)).build() {
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
            let url = node.get_attr_or("href", "");
            let mut name = node.get_attr_or("name", url.as_str());
            if name.len() == 0 {
                name = url.clone();
            }
            return format!(r#"<a href="{}">{}</a>"#, url, name);
        }
        NodeTagName::Image => {
            let alt = node.get_attr_or("name", "image link is broken");
            let src = node.get_attr_or("src", "");
            return format!(r#"<img alt="{alt}" src="{src}"/>"#);
        }
        NodeTagName::Extension => {
            if let Some(value) = mark::generate(bodystr, RenderType::Html) {
                return value;
            } else {
                log::warn!("unsupported mark element: {}", bodystr);
                return format!("<pre><code>{}</pre></code>", utils::escape_to_html(bodystr));
            }
        }
        _ => {}
    }

    // Render all non-void element
    let markup = match nodedata.tag.name {
        NodeTagName::Heading => {
            let level = match nodedata.tag.attrs.get("level").map(|s| s.as_str().parse::<usize>()) {
                Some(Ok(level)) => {
                    level
                }
                _ => {
                    log::warn!("heading level parse failed: {:?}, set it to level 1", nodedata.tag.attrs.get("level"));
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
    };
    let (start_tag, end_tag) = if let Some(mark) = markup {
        (format!("<{mark}>"), format!("</{mark}>"))
    } else {
        ("".to_owned(), "".to_owned())
    };

    let mut html = String::new();
    html += &start_tag;
    for child in node.children().iter() {
        html.push_str(generate(child, content, hook).as_str());
    }
    html += &end_tag;

    html
}
