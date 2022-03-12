use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

pub struct IfStatement {
    condition: Expression,
    statement: Statement,
    else_statement: Option<Statement>,
}
