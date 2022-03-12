use crate::syntax::expression::Expression;
use crate::syntax::statement::Node;
use crate::syntax::statement::Statement;

pub struct ForLoop {
    initializer: Node,
    condition: Expression,
    incrementer: Expression,
    statement: Statement,
}
