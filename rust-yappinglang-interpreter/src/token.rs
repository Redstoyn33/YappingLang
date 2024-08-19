
pub struct Token {
    typ: TokenData,
    line: i32,
}

pub enum TokenData {
    // Literals.
    Identifier(String),
    String(String),
    Number(String),

    // Single-character tokens.
    Semicolon,
    LeftBrace,
    RightBrace,
    Minus,
    Plus,
    Slash,
    Star,
}
