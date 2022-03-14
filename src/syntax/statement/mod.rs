use crate::syntax::expression::Expression;
pub use crate::syntax::statement::for_loop::ForLoop;
pub use crate::syntax::statement::if_statement::IfStatement;
pub use crate::syntax::statement::switch_statement::SwitchStatement;
pub use crate::syntax::statement::while_loop::WhileLoop;
use crate::syntax::Identifier;
use crate::syntax::Node;
use std::cell::RefCell;
use std::rc::Rc;

mod for_loop;
mod if_statement;
mod switch_statement;
mod while_loop;

#[derive(Clone, Debug)]
pub struct Statement {
    pub label: Option<Label>,
    pub statement_contents: Rc<RefCell<StatementContents>>,
}

#[derive(Clone, Debug)]
pub enum Label {
    Case,
    Default,
    Custom(Identifier),
}

#[derive(Clone, Debug)]
pub enum StatementContents {
    Compound(Vec<Node>),
    Expression(Expression),
    IfStatement(IfStatement),
    SwitchStatement(SwitchStatement),
    WhileLoop(WhileLoop),
    ForLoop(ForLoop),
    GotoStatement(Identifier),
    ContinueStatement,
    BreakStatement,
    ReturnStatement(Expression),
}