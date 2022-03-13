use crate::syntax::Identifier;

#[derive(Clone, Debug)]
pub enum Literal {
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    EnumInstance(Identifier),
    Character(CharacterLiteral),
}

#[derive(Clone, Debug)]
pub enum IntegerLiteralType {
    Decimal,
    Octal,
    Hexadecimal,
}

#[derive(Clone, Debug)]
pub enum IntegerLiteralLength {
    Normal,
    Long,
    LongLong,
}

#[derive(Clone, Debug)]
pub struct IntegerLiteral {
    literal_type: IntegerLiteralType,
    value: i64,
    is_unsigned: bool,
    length: IntegerLiteralLength,
}

#[derive(Clone, Debug)]
pub enum FloatLiteralType {
    Decimal,
    Hexadecimal,
}

#[derive(Clone, Debug)]
pub enum FloatLength {
    Float,
    Normal,
    Long,
}

#[derive(Clone, Debug)]
pub struct FloatLiteral {
    literal_type: FloatLiteralType,
    value: f64,
    length: FloatLength,
}

#[derive(Clone, Debug)]
pub struct CharacterLiteral {
    value: String,
    is_wide: bool,
}
