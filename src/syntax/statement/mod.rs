use crate::syntax::declaration::Declaration;
use crate::syntax::expression::Expression;
pub use crate::syntax::statement::for_loop::ForLoop;
pub use crate::syntax::statement::if_statement::IfStatement;
pub use crate::syntax::statement::switch_statement::SwitchStatement;
pub use crate::syntax::statement::while_loop::WhileLoop;
use crate::syntax::Identifier;

mod for_loop;
mod if_statement;
mod switch_statement;
mod while_loop;

pub struct Statement {
    label: Option<Label>,
    statement_contents: Box<StatementContents>,
}

pub enum Label {
    Case,
    Default,
    Custom(Identifier),
}

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

pub enum Node {
    Declaration(Declaration),
    Statement(Statement),
}
