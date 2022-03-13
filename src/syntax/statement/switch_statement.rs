use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct SwitchStatement {
    pub match_expression: Expression,
    pub statement: Statement,
}
