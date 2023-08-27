//! # ConciseMark - a simplified markdown parsing library
//!
//! ConciseMark can render markdown into HTML or Latex page, for example
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
//! <div><h1>Title</h1></div>
//! ```
//!
//! The outermost `div` is the root of the rendered html page.
//!
//! If you want to render the markdown into a pretty PDF document, you may be interested in
//! [`Page::render_latex`], have it a look!
//!
//! ## Hook
//!
//! [`Page`] maintains an AST structure which you can use to hook the nodes you are
//! interested in, please see its document for more information.
//!
pub mod meta;
pub mod node;
mod parser;
mod render;
pub mod token;
pub mod utils;

use meta::Meta;
use node::Node;
use parser::Parser;

/// A placehodler for future usage
#[derive(Debug)]
pub struct PageOptions {}

/// A markdown page
pub struct Page {
    /// Meta information for the page, such as author, tags ...
    pub meta: Option<Meta>,
    /// Page AST (abstract syntax tree), see [`Page::transform`] to learn how to modify it
    pub ast: Node,
    /// The markdown file content (with `meta` stripped). `ast` does not store any text but only node range,
    /// and content is necessary to retrive node text with `ast` information.
    pub content: String,
    /// Page options, a placehodler for future usage
    pub options: Option<PageOptions>,
}

impl Page {
    /// Create a new markdown page from `content`
    pub fn new<S: AsRef<str>>(content: S) -> Self {
        let (meta, ast, content) = Parser::new(content).parse();
        Self {
            meta,
            ast,
            content,
            options: None,
        }
    }

    pub fn with_options(mut self, options: PageOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Render markdown into HTML page
    ///
    ///     use concisemark::Page;
    ///
    ///     let content = "# Title";
    ///     let page = Page::new(content);
    ///     let html = page.render();
    ///
    /// The output html will be
    ///
    /// ```text
    /// <div><h1>Title</h1></div>
    /// ```
    pub fn render(&self) -> String {
        self.render_with_hook(&|_| None)
    }

    /// Render markdown into XeLaTex source
    ///
    /// Note that latex can not embed image from url, you must download the image and fix the
    /// image path to generate a working tex file, the following is a dirty and quick example.
    ///
    ///     use concisemark::Page;
    ///     use concisemark::node::Node;
    ///     use concisemark::node::NodeTagName;
    ///     use concisemark::utils;
    ///
    ///     use std::fs::OpenOptions;
    ///     use std::process::Command;
    ///     use std::io::Write;
    ///
    ///     use indoc::indoc;
    ///
    ///     let content = indoc! {r#"
    ///         ![animal-online](https://cn.bing.com/th?id=OHR.NorwayMuskox_EN-CN7806818932_1920x1080.jpg&w=720)
    ///
    ///         ![animal-offlie](assets/th.jpg)
    ///     "#
    ///     };
    ///     let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    ///     let draft_dir = manifest_dir.join("draft");
    ///     std::fs::create_dir_all(draft_dir.as_path()).unwrap();
    ///
    ///     let page = Page::new(content);
    ///     let hook = |node: &Node| -> Result<(), ()> {
    ///         let mut nodedata = node.data.borrow_mut();
    ///         if nodedata.tag.name == NodeTagName::Image {
    ///             let src = nodedata.tag.attrs.get("src").unwrap().to_owned();
    ///             let name = nodedata.tag.attrs.get("name").unwrap().to_owned();
    ///             let output_path;
    ///             if src.starts_with("https://") || src.starts_with("http://") {
    ///                 output_path = utils::download_image_fs(src, draft_dir.as_path(), name).unwrap();
    ///             } else {
    ///                 output_path = manifest_dir.join(src);
    ///             }
    ///             nodedata.tag.attrs.insert("src".to_owned(), format!("{}", output_path.display()));
    ///         }
    ///         Ok(())
    ///     };
    ///     page.transform(hook);
    ///
    ///     let setup = include_str!("../assets/setup.tex");
    ///     let wanted = indoc! {r#"
    ///         \begin{document}
    ///
    ///         \begin{figure}[H]
    ///         \centerline{\includegraphics[width=0.7\textwidth]{PLACEHOLDER_ONLINE}}
    ///         \caption{animal-online}
    ///         \end{figure}
    ///
    ///
    ///         \begin{figure}[H]
    ///         \centerline{\includegraphics[width=0.7\textwidth]{PLACEHOLDER_OFFLINE}}
    ///         \caption{animal-offlie}
    ///         \end{figure}
    ///
    ///         \end{document}
    ///     "#};
    ///     let wanted = wanted.replace(
    ///         "PLACEHOLDER_ONLINE",
    ///         &format!("{}", manifest_dir.join("draft").join("animal-online.jpg").display())
    ///     ).replace(
    ///         "PLACEHOLDER_OFFLINE",
    ///         &format!("{}", manifest_dir.join("assets").join("th.jpg").display())
    ///     );
    ///     let pagesrc = &page.render_latex()[setup.len()..];
    ///     assert_eq!(wanted.trim(), pagesrc.trim());
    ///
    ///     let latex = page.render_latex();
    ///     let texfile = draft_dir.join("output.tex");
    ///     let mut f = OpenOptions::new().truncate(true).write(true).create(true).open(&texfile).unwrap();
    ///     f.write(latex.as_bytes()).unwrap();
    ///     let mut cmd = Command::new("xelatex");
    ///     cmd.current_dir(&draft_dir);
    ///     cmd.arg(&texfile);
    ///     _ = cmd.output();
    pub fn render_latex(&self) -> String {
        let mut page = include_str!("../assets/setup.tex").to_owned();
        let mut document = render::latex::Cmd::new("document").enclosed();
        if let Some(meta) = &self.meta {
            let title =
                render::latex::Cmd::new("title").with_posarg(&meta.title);
            document.append_cmd(&title);
            if let Some(authors) = &meta.authors {
                let authors = render::latex::Cmd::new("author")
                    .with_posarg(authors.join(", "));
                document.append_cmd(&authors);
            }
            let date = render::latex::Cmd::new("date")
                .with_posarg(&meta.date.to_string());
            document.append_cmd(&date);
            let maketitle = render::latex::Cmd::new("maketitle");
            document.append_cmd(&maketitle);
        }
        document
            .append(render::latex::generate(&self.ast, self.content.as_str()));
        page.push_str(&document.to_string());
        page
    }

    /// Render markdown into HTML page with hook
    ///
    /// If the hook returns None, then the default rendering function will be used or else
    /// use the returned value as render result.
    pub fn render_with_hook<F>(&self, hook: &F) -> String
    where
        F: Fn(&Node) -> Option<String>,
    {
        render::html::generate(&self.ast, self.content.as_str(), Some(hook))
    }

    /// Modify markdown AST node with hook.
    ///
    /// The error status of the hook function (when returns an Err) will not stop the transform
    /// process, instead it will print the error as a log message.
    ///
    /// The following is an exmaple to change image url
    ///
    ///     use concisemark::node::{Node, NodeTagName};
    ///     use concisemark::Page;
    ///
    ///     let content = "![imgs](/path/to/image.jpg)";
    ///     let page = Page::new(content);
    ///     let hook = |node: &Node| -> Result<(), ()> {
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
    ///         Ok(())
    ///     };
    ///     let img = &page.ast.children()[0].children()[0];
    ///     assert_eq!(img.data.borrow().tag.attrs.get("src").map(|s| s.as_str()), Some("/path/to/image.jpg"));
    ///     page.transform(hook);
    ///     assert_eq!(img.data.borrow().tag.attrs.get("src").map(|s| s.as_str()), Some("https://example.com/path/to/image.jpg"));
    ///
    pub fn transform<F, E>(&self, hook: F)
    where
        F: Fn(&Node) -> Result<(), E>,
    {
        self.ast.transform::<F, E>(&hook)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs::OpenOptions, io::Write, iter, process::Command};

    use html5ever::{
        driver::ParseOpts, local_name, namespace_url, ns, parse_fragment,
        tendril::TendrilSink, tree_builder::TreeSink, QualName,
    };
    use indoc::indoc;
    use markup5ever_rcdom::{Handle, NodeData, RcDom};
    use node::NodeTagName;

    use crate::*;

    fn is_self_closing_tag(tag: &str) -> bool {
        let self_closing_tag_list = vec![
            // svg tags
            "circle", "ellipse", "line", "path", "polygon", "polyline", "rect",
            "stop", "use", // void tags
            "area", "base", "br", "col", "command", "embed", "hr", "img",
            "input", "keygen", "link", "meta", "param", "source", "track",
            "wbr",
        ];
        if self_closing_tag_list.iter().any(|&i| i == tag) {
            true
        } else {
            false
        }
    }

    fn get_html_outline(dirty_html: &str) -> String {
        fn walker(indent: usize, node: &Handle) -> String {
            let indentstr = format!(
                "{}",
                iter::repeat(" ").take(indent).collect::<String>()
            );

            let mut outline = indentstr.to_string();
            match node.data {
                NodeData::Element { ref name, .. } => {
                    if is_self_closing_tag(&name.local) {
                        outline += &format!("<{}", name.local);
                    } else {
                        outline += &format!("<{}>\n", name.local);
                    }
                }
                _ => {}
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
            assert_eq!(
                ast.children[0]
                    .borrow()
                    .tag
                    .attrs
                    .get("level")
                    .map(|s| s.as_str()),
                Some(level)
            );
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
        assert_eq!(
            outline,
            indoc! {r#"
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
        "#}
        );
    }

    #[test]
    fn test_math() {
        let page = Page::new("inline math: $a^2 + b^2$");
        let html = page.render();
        let inlined_math_html = indoc! {r#"
            <div><p>inline math: <span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><msup><mi>a</mi><mn>2</mn></msup><mo>+</mo><msup><mi>b</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">a^2 + b^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.8974em;vertical-align:-0.0833em;"></span><span class="mord"><span class="mord mathnormal">a</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8141em;"><span style="top:-3.063em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span><span class="mspace" style="margin-right:0.2222em;"></span><span class="mbin">+</span><span class="mspace" style="margin-right:0.2222em;"></span></span><span class="base"><span class="strut" style="height:0.8141em;"></span><span class="mord"><span class="mord mathnormal">b</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8141em;"><span style="top:-3.063em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span>
            </p></div>
        "#};
        assert_eq!(inlined_math_html.trim(), html.trim());

        let page = Page::new("display math:\n\n$a^2 + b^2$");
        let displayed_math_html = indoc! {r#"
            <div><p>display math:
            </p><p><span class="katex-display"><span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><semantics><mrow><msup><mi>a</mi><mn>2</mn></msup><mo>+</mo><msup><mi>b</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">a^2 + b^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.9474em;vertical-align:-0.0833em;"></span><span class="mord"><span class="mord mathnormal">a</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8641em;"><span style="top:-3.113em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span><span class="mspace" style="margin-right:0.2222em;"></span><span class="mbin">+</span><span class="mspace" style="margin-right:0.2222em;"></span></span><span class="base"><span class="strut" style="height:0.8641em;"></span><span class="mord"><span class="mord mathnormal">b</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8641em;"><span style="top:-3.113em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span></span>
            </p></div>
        "#}.trim();
        let html = page.render();
        assert_eq!(displayed_math_html.trim(), html.trim());

        let page = Page::new(indoc! {r#"
            - display mode math equation in list

                The follwoing is a display mode math equation

                $a^2 + b^2 = c^2$
        "#});
        let html = page.render();
        let displyed_math_html_in_list = indoc! {r#"
            <div><ul><li>display mode math equation in list
            <p>    The follwoing is a display mode math equation
            </p><p>    <span class="katex-display"><span class="katex"><span class="katex-mathml"><math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><semantics><mrow><msup><mi>a</mi><mn>2</mn></msup><mo>+</mo><msup><mi>b</mi><mn>2</mn></msup><mo>=</mo><msup><mi>c</mi><mn>2</mn></msup></mrow><annotation encoding="application/x-tex">a^2 + b^2 = c^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true"><span class="base"><span class="strut" style="height:0.9474em;vertical-align:-0.0833em;"></span><span class="mord"><span class="mord mathnormal">a</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8641em;"><span style="top:-3.113em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span><span class="mspace" style="margin-right:0.2222em;"></span><span class="mbin">+</span><span class="mspace" style="margin-right:0.2222em;"></span></span><span class="base"><span class="strut" style="height:0.8641em;"></span><span class="mord"><span class="mord mathnormal">b</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8641em;"><span style="top:-3.113em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span><span class="mspace" style="margin-right:0.2778em;"></span><span class="mrel">=</span><span class="mspace" style="margin-right:0.2778em;"></span></span><span class="base"><span class="strut" style="height:0.8641em;"></span><span class="mord"><span class="mord mathnormal">c</span><span class="msupsub"><span class="vlist-t"><span class="vlist-r"><span class="vlist" style="height:0.8641em;"><span style="top:-3.113em;margin-right:0.05em;"><span class="pstrut" style="height:2.7em;"></span><span class="sizing reset-size6 size3 mtight"><span class="mord mtight">2</span></span></span></span></span></span></span></span></span></span></span></span>
            </p></li></ul></div>
        "#}.trim();
        assert_eq!(displyed_math_html_in_list.trim(), html.trim());
    }

    #[test]
    fn test_latex() {
        let content = indoc! {r#"
            ![animal-online](https://cn.bing.com/th?id=OHR.NorwayMuskox_EN-CN7806818932_1920x1080.jpg&w=720)

            ![animal-offlie](assets/th.jpg)
        "#
        };
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let draft_dir = manifest_dir.join("draft");
        std::fs::create_dir_all(draft_dir.as_path()).unwrap();

        let page = Page::new(content);
        let hook = |node: &Node| -> Result<(), ()> {
            let mut nodedata = node.data.borrow_mut();
            if nodedata.tag.name == NodeTagName::Image {
                let src = nodedata.tag.attrs.get("src").unwrap().to_owned();
                let name = nodedata.tag.attrs.get("name").unwrap().to_owned();
                let output_path;
                if src.starts_with("https://") || src.starts_with("http://") {
                    output_path = utils::download_image_fs(
                        src,
                        draft_dir.as_path(),
                        name,
                    )
                    .unwrap();
                } else {
                    output_path = manifest_dir.join(src);
                }
                nodedata.tag.attrs.insert(
                    "src".to_owned(),
                    format!("{}", output_path.display()),
                );
            }
            Ok(())
        };
        page.transform(hook);

        let setup = include_str!("../assets/setup.tex");
        let wanted = indoc! {r#"
            \begin{document}

            \begin{figure}[H]
            \centerline{\includegraphics[width=0.7\textwidth]{PLACEHOLDER_ONLINE}}
            \caption{animal-online}
            \end{figure}


            \begin{figure}[H]
            \centerline{\includegraphics[width=0.7\textwidth]{PLACEHOLDER_OFFLINE}}
            \caption{animal-offlie}
            \end{figure}

            \end{document}
        "#};
        let wanted = wanted
            .replace(
                "PLACEHOLDER_ONLINE",
                &format!(
                    "{}",
                    manifest_dir
                        .join("draft")
                        .join("animal-online.jpg")
                        .display()
                ),
            )
            .replace(
                "PLACEHOLDER_OFFLINE",
                &format!(
                    "{}",
                    manifest_dir.join("assets").join("th.jpg").display()
                ),
            );
        let pagesrc = &page.render_latex()[setup.len()..];
        assert_eq!(wanted.trim(), pagesrc.trim());

        let latex = page.render_latex();
        let texfile = draft_dir.join("output.tex");
        let mut f = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&texfile)
            .unwrap();
        f.write(latex.as_bytes()).unwrap();
        let mut cmd = Command::new("xelatex");
        cmd.current_dir(&draft_dir);
        cmd.arg(&texfile);
        _ = cmd.output();
    }
}
