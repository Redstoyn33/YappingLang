use crate::token::TokenData::{Decimal, Def, DefFn, Identifier, Integer, LeftParen, RightParen, Semicolon};
use crate::token::{Token, TokenData};
use crate::utils::ResultToString;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    line: u64,
    start: usize,
    current: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            line: 1,
            start: 0,
            current: 0,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;

            self.scan_token()?;
        }
        return Ok(self.tokens);
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            ';' => self.add_token(Semicolon),
            '"' => self.string()?,
            '\n' => self.line += 1,
            _ => {
                if c.is_whitespace() {
                    return Ok(());
                }
                if c.is_ascii_digit() {
                    if c == '0' && self.peek() == 'x' {
                        return self.hex_integer();
                    } else {
                        return self.number();
                    }
                }
                if c.is_uppercase() || !c.is_alphabetic() {
                    self.identifier();
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        return c;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source[self.current];
    }
    fn add_token(&mut self, typ: TokenData) {
        self.tokens.push(Token {
            typ,
            line: self.line,
        })
    }
    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error("Unterminated string");
        }

        // The closing ".
        self.advance();

        self.add_token(TokenData::String(String::from_iter(
            self.source[self.start + 1..self.current - 1].iter(),
        )));
        Ok(())
    }

    fn error<T>(&self, err: &str) -> Result<T, String> {
        Err(format!("[line {}] Error: {}", self.line, err))
    }
    fn hex_integer(&self) -> Result<(),String> {
        todo!()
    }
    fn number(&mut self) -> Result<(), String> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }

            let dec: f64 = String::from_iter(
                self.source[self.start..self.current].iter(),
            ).parse().str_res()?;
            self.add_token(Decimal(dec));
        } else {
            let int: i64 = String::from_iter(
                self.source[self.start..self.current].iter(),
            ).parse().str_res()?;
            self.add_token(Integer(int));
        }
        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source[self.current + 1];
    }
    fn identifier(&mut self) {
        while !self.peek().is_whitespace() && !"();".contains(self.peek()) {
            self.advance();
        }
        let str = String::from_iter(
            self.source[self.start..self.current].iter(),
        );
        if str == "Def-fn" {
            self.add_token(DefFn);
        } else if str == "Def" {
            self.add_token(Def);
        } else {
            self.add_token(Identifier(str));
        }
    }
}
