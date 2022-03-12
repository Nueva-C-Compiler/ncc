use crate::syntax::declaration::Declaration;
use crate::syntax::statement::Statement;

pub mod declaration;
pub mod expression;
pub mod statement;

pub enum Node {
    Declaration(Declaration),
    Statement(Statement),
}
pub type Identifier = String;
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
