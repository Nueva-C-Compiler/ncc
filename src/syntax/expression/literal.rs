use crate::syntax::Identifier;

pub enum Literal {
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    EnumInstance(Identifier),
    Character(CharacterLiteral),
}

pub enum IntegerLiteralType {
    Decimal,
    Octal,
    Hexadecimal,
}

pub enum IntegerLiteralLength {
    Normal,
    Long,
    LongLong,
}

pub struct IntegerLiteral {
    literal_type: IntegerLiteralType,
    value: i64,
    is_unsigned: bool,
    length: IntegerLiteralLength,
}

pub enum FloatLiteralType {
    Decimal,
    Hexadecimal,
}

pub enum FloatLength {
    Float,
    Normal,
    Long,
}

pub struct FloatLiteral {
    literal_type: FloatLiteralType,
    value: f64,
    length: FloatLength,
}

pub struct CharacterLiteral {
    value: String,
    is_wide: bool,
}
