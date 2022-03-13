use crate::syntax::expression::literal::Literal;
use crate::syntax::Identifier;
use crate::syntax::Type;

mod literal;

#[derive(Clone, Debug)]
pub enum Expression {
    Assignment(Box<AssignmentExpression>),
    ExpressionList(Box<Expression>, Box<AssignmentExpression>),
}

pub type ConstantExpression = ConditionalExpression;

#[derive(Clone, Debug)]
pub enum PrimaryExpression {
    Identifier(Identifier),
    Constant(Literal),
    StringLiteral(String),
    Parenthesized(Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum PostfixExpression {
    Primary(Box<PrimaryExpression>),
    Subscript(Box<PostfixExpression>, Box<Expression>),
    FunctionCall(Box<PostfixExpression>, Vec<Expression>),
    Member(Box<PostfixExpression>, Identifier),
    DereferencingMember(Box<PostfixExpression>, Identifier),
    Increment(Box<PostfixExpression>),
    Decrement(Box<PostfixExpression>),
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Reference,
    Dereference,
    Positive,
    Negative,
    BitwiseNegate,
    Negate,
}

#[derive(Clone, Debug)]
pub enum UnaryExpression {
    Postfix(Box<PostfixExpression>),
    Increment(Box<UnaryExpression>),
    Decrement(Box<UnaryExpression>),
    Operator(UnaryOperator, Box<CastExpression>),
    SizeOf(Box<UnaryExpression>),
    SizeOfType(Type),
}

#[derive(Clone, Debug)]
pub enum CastExpression {
    Unary(UnaryExpression),
    Cast(Type, Box<CastExpression>),
}

#[derive(Clone, Debug)]
pub enum MultiplicativeExpression {
    Cast(Box<CastExpression>),
    Multiply(Box<MultiplicativeExpression>, Box<CastExpression>),
    Divide(Box<MultiplicativeExpression>, Box<CastExpression>),
    Mod(Box<MultiplicativeExpression>, Box<CastExpression>),
}

#[derive(Clone, Debug)]
pub enum AdditiveExpression {
    Multiplicative(Box<MultiplicativeExpression>),
    Add(Box<AdditiveExpression>, Box<MultiplicativeExpression>),
    Subtract(Box<AdditiveExpression>, Box<MultiplicativeExpression>),
}

#[derive(Clone, Debug)]
pub enum ShiftExpression {
    Additive(Box<AdditiveExpression>),
    Left(Box<ShiftExpression>, Box<AdditiveExpression>),
    Right(Box<ShiftExpression>, Box<AdditiveExpression>),
}

#[derive(Clone, Debug)]
pub enum RelationalExpression {
    Shift(Box<ShiftExpression>),
    Less(Box<RelationalExpression>, Box<ShiftExpression>),
    Greater(Box<RelationalExpression>, Box<ShiftExpression>),
    LessEqual(Box<RelationalExpression>, Box<ShiftExpression>),
    GreaterEqual(Box<RelationalExpression>, Box<ShiftExpression>),
}

#[derive(Clone, Debug)]
pub enum EqualityExpression {
    Relational(Box<RelationalExpression>),
    Equal(Box<EqualityExpression>, Box<RelationalExpression>),
    NotEqual(Box<EqualityExpression>, Box<RelationalExpression>),
}

#[derive(Clone, Debug)]
pub enum AndExpression {
    Equality(Box<EqualityExpression>),
    And(Box<EqualityExpression>, Box<AndExpression>),
}

#[derive(Clone, Debug)]
pub enum XOrExpression {
    AndExpression(Box<AndExpression>),
    XOr(Box<AndExpression>, Box<XOrExpression>),
}

#[derive(Clone, Debug)]
pub enum OrExpression {
    XOrExpression(Box<XOrExpression>),
    Or(Box<XOrExpression>, Box<OrExpression>),
}

#[derive(Clone, Debug)]
pub enum LogicalAndExpression {
    OrExpression(Box<OrExpression>),
    LogicalAnd(Box<OrExpression>, Box<LogicalAndExpression>),
}

#[derive(Clone, Debug)]
pub enum LogicalOrExpression {
    LogicalAndExpression(Box<LogicalAndExpression>),
    LogicalOr(Box<LogicalAndExpression>, Box<LogicalOrExpression>),
}

#[derive(Clone, Debug)]
pub enum ConditionalExpression {
    LogicalOrExpression(Box<LogicalOrExpression>),
    Conditional(
        Box<LogicalOrExpression>,
        Box<Expression>,
        Box<ConditionalExpression>,
    ),
}

#[derive(Clone, Debug)]
pub enum AssignmentOperator {
    Equals,
    TimesEquals,
    DivideEquals,
    ModEquals,
    PlusEquals,
    MinusEquals,
    ShiftLeftEquals,
    ShiftRightEquals,
    AndEquals,
    XOrEquals,
    OrEquals,
}

#[derive(Clone, Debug)]
pub enum AssignmentExpression {
    ConditionalExpression(Box<ConditionalExpression>),
    Assignment(
        Box<UnaryExpression>,
        AssignmentOperator,
        Box<AssignmentExpression>,
    ),
}
