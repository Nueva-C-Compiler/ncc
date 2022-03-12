use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

pub struct SwitchStatement {
    match_expression: Expression,
    statement: Statement,
}
