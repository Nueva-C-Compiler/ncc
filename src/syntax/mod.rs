use crate::syntax::declaration::Declaration;
use crate::syntax::statement::Statement;
use std::cell::RefCell;
use std::rc::Rc;

pub mod declaration;
pub mod expression;
pub mod statement;

#[derive(Clone, Debug)]
pub enum Node {
    Declaration(Rc<Declaration>),
    Statement(Rc<Statement>),
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub hash: u64,
    pub declaration: Option<Rc<Declaration>>,
}

#[derive(Clone, Debug)]
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
