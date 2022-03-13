// Heavily simplified declaration definitions

use crate::syntax::expression::Expression;
use crate::syntax::Identifier;
use crate::syntax::Node;
use crate::syntax::Type;

#[derive(Clone, Debug)]
pub enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
}

#[derive(Clone, Debug)]
pub enum VariableType {
    Scalar(Type),
    Pointer(Box<VariableType>),
    Array(Type, u64),
}

#[derive(Clone, Debug)]
pub enum ReturnType {
    Scalar(Type),
    Pointer(Box<VariableType>),
}

#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    var_type: VariableType,
    identifier: Identifier,
    initializer: Option<Expression>,
}

#[derive(Clone, Debug)]
pub struct FunctionParameter {
    param_type: VariableType,
    identifier: Identifier,
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    return_type: ReturnType,
    identifier: Identifier,
    parameters: Vec<FunctionParameter>,
    body: Option<Vec<Node>>,
}
