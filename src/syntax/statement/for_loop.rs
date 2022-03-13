use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;
use crate::syntax::Node;

#[derive(Clone, Debug)]
pub struct ForLoop {
    pub initializer: Node,
    pub condition: Expression,
    pub incrementer: Expression,
    pub statement: Statement,
}
