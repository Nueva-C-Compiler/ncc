// Heavily simplified declaration definitions

use crate::syntax::expression::Expression;
use crate::syntax::Identifier;
use crate::syntax::Node;
use crate::syntax::Type;

pub enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
}

pub enum VariableType {
    Scalar(Type),
    Pointer(Box<VariableType>),
    Array(Type, u64),
}

pub enum ReturnType {
    Scalar(Type),
    Pointer(Box<VariableType>),
}

pub struct VariableDeclaration {
    var_type: VariableType,
    identifier: Identifier,
    initializer: Option<Expression>,
}

pub struct FunctionParameter {
    param_type: VariableType,
    identifier: Identifier,
}

pub struct FunctionDeclaration {
    return_type: ReturnType,
    identifier: Identifier,
    parameters: Vec<FunctionParameter>,
    body: Option<Vec<Node>>,
}
