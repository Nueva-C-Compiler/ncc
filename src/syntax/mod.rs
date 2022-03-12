use crate::syntax::declaration::Declaration;
use crate::syntax::statement::Statement;

pub mod declaration;
pub mod expression;
pub mod statement;

pub enum Node {
    Declaration(Declaration),
    Statement(Statement),
}

pub struct Identifier {
    name: String,
    declaration: Option<Box<Declaration>>,
}

pub enum Type {
    Void,
    Char(bool),
    Short(bool),
    Int(bool),
    Long(bool),
    Float,
    Double,
    LongDouble,
    Struct(Identifier),
    Enum(Identifier),
    Other(Identifier),
}
