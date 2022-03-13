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
    pub literal_type: IntegerLiteralType,
    pub value: i64,
    pub is_unsigned: bool,
    pub length: IntegerLiteralLength,
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
    pub literal_type: FloatLiteralType,
    pub value: f64,
    pub length: FloatLength,
}

#[derive(Clone, Debug)]
pub struct CharacterLiteral {
    pub value: String,
    pub is_wide: bool,
}
