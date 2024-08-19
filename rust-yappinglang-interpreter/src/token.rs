
#[derive(Debug)]
pub struct Token {
    pub typ: TokenData,
    pub line: u64,
}

#[derive(Debug)]
pub enum TokenData {
    // Literals.
    Identifier(String),
    String(String),
    Integer(i64),
    Decimal(f64),

    // Single-character tokens.
    Semicolon,
    LeftParen,
    RightParen,

    // keywords
    DefFn,
    Def,
}
