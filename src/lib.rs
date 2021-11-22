mod syntax;
mod trace;

pub use syntax::*;
pub use trace::Trace;

pub enum SkeletonTree {
    Zeroary,
    Unary(Box<SyntaxTree>),
    Binary(Box<(SyntaxTree, SyntaxTree)>),
}
