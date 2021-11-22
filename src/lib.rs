mod syntax;
mod trace;

pub use syntax::*;
pub use trace::*;

pub enum SkeletonTree {
    Zeroary,
    Unary(Box<SyntaxTree>),
    Binary(Box<(SyntaxTree, SyntaxTree)>),
}
