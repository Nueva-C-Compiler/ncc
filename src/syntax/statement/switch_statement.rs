use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct SwitchStatement {
    match_expression: Expression,
    statement: Statement,
}
