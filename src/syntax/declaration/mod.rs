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
    pub var_type: VariableType,
    pub identifier: Identifier,
    pub initializer: Option<Expression>,
}

#[derive(Clone, Debug)]
pub struct FunctionParameter {
    pub param_type: VariableType,
    pub identifier: Identifier,
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub return_type: ReturnType,
    pub identifier: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub body: Option<Vec<Node>>,
}
