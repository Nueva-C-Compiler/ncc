use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct IfStatement {
    condition: Expression,
    statement: Statement,
    else_statement: Option<Statement>,
}
