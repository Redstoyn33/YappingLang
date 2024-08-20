use crate::token::Token;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>)-> Self {
        Self {
            tokens,
            current: 0,
        }
    }
}