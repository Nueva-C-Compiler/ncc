use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub statement: Statement,
    pub else_statement: Option<Statement>,
}
