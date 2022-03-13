use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;

#[derive(Clone, Debug)]
pub struct WhileLoop {
    pub condition: Expression,
    pub statement: Statement,
    pub loop_type: WhileType,
}

#[derive(Clone, Debug)]
pub enum WhileType {
    While,
    DoWhile,
}
