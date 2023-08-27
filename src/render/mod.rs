pub mod html;
pub mod latex;
pub mod mark;
pub mod prettier;

#[derive(Debug, PartialEq)]
pub enum RenderType {
    Html,
    Latex,
}
