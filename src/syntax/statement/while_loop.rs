use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct WhileLoop {
    condition: Expression,
    statement: Statement,
    loop_type: WhileType,
}

#[derive(Clone, Debug)]
pub enum WhileType {
    While,
    DoWhile,
}
