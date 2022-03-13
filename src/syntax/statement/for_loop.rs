use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;
use crate::syntax::Node;

#[derive(Clone, Debug)]
pub struct ForLoop {
    initializer: Node,
    condition: Expression,
    incrementer: Expression,
    statement: Statement,
}
