pub mod html;
pub mod latex;
pub mod mark;

#[derive(Debug, PartialEq)]
pub enum RenderType {
    Html,
    Latex,
}
