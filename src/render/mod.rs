pub mod latex;
pub mod html;
pub mod prettier;
pub mod mark;

#[derive(Debug, PartialEq)]
pub enum RenderType {
    Html,
    Latex,
}
