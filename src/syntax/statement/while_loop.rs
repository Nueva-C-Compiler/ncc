use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

pub struct WhileLoop {
    condition: Expression,
    statement: Statement,
    loop_type: WhileType,
}

pub enum WhileType {
    While,
    DoWhile,
}
