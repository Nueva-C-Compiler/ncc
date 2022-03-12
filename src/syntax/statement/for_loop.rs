use crate::syntax::expression::Expression;
use crate::syntax::statement::Statement;
use crate::syntax::Node;

pub struct ForLoop {
    initializer: Node,
    condition: Expression,
    incrementer: Expression,
    statement: Statement,
}
