#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenData,
    pub line: u64,
}

#[derive(Debug, Clone)]
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
    Local,
}
