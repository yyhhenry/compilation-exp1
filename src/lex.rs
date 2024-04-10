use serde::{Deserialize, Serialize};

use crate::error::ErrorRecorder;
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
/// Token in PL/0 Like language.
/// Ignore case.
pub enum TokenEnum {
    // struct keywords
    Var,
    If,
    Then,
    Else,
    While,
    Do,
    Begin,
    End,

    // operator keywords
    And,
    Or,

    // type keywords
    Integer,
    Longint,
    Bool,
    Real,

    // operators +|-|*|/|:=|<|>|<>|>=|<=|=|:|(|)
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// :=
    Assign,
    /// <
    Lt,
    /// >
    Gt,
    /// <>
    Ne,
    /// >=
    Ge,
    /// <=
    Le,
    /// =
    Eq,
    /// :
    Colon,
    /// (
    LParen,
    /// )
    RParen,

    // struct symbols
    /// ,
    Comma,
    /// ;
    SemiColon,

    // Literals and Identifiers
    /// Identifier [a-zA-Z][a-zA-Z0-9]*, case insensitive
    Identifier,
    /// Integer literal [1-9][0-9]*|0, no leading 0
    IntLiteral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub offset: usize,
    pub content: String,
    pub token: TokenEnum,
}

#[derive(Debug, Clone)]
pub struct CharStream {
    input: Vec<char>,
    pos: usize,
}
impl CharStream {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    pub fn peek(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }
    pub fn next(&mut self) -> Option<char> {
        match self.peek() {
            Some(c) => {
                self.pos += 1;
                Some(c)
            }
            None => None,
        }
    }
    /// Returns (start, token)
    fn next_token_base(&mut self, errors: &mut ErrorRecorder) -> Option<(usize, TokenEnum)> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
                continue;
            }
            let start = self.pos;
            let result = |token| Some((start, token));
            if c.is_ascii_alphabetic() {
                let mut ident = String::new();
                while self.peek().map_or(false, |c| c.is_ascii_alphanumeric()) {
                    ident.push(self.next().unwrap());
                }
                let token = match ident.to_lowercase().as_str() {
                    "var" => TokenEnum::Var,
                    "integer" => TokenEnum::Integer,
                    "longint" => TokenEnum::Longint,
                    "bool" => TokenEnum::Bool,
                    "real" => TokenEnum::Real,
                    "if" => TokenEnum::If,
                    "then" => TokenEnum::Then,
                    "else" => TokenEnum::Else,
                    "while" => TokenEnum::While,
                    "do" => TokenEnum::Do,
                    "begin" => TokenEnum::Begin,
                    "end" => TokenEnum::End,
                    "and" => TokenEnum::And,
                    "or" => TokenEnum::Or,
                    _ => TokenEnum::Identifier,
                };
                return result(token);
            } else if c.is_numeric() {
                let mut num = String::new();
                while self.peek().map_or(false, |c| c.is_numeric()) {
                    num.push(self.next().unwrap());
                }
                if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                    errors.error(self.pos, "Unexpected character after number");
                    // Automatically add a space after the number
                }
                if num.starts_with('0') && num.len() > 1 {
                    errors.warning(start, "Number cannot start with 0");
                    // Remove leading zeros
                    num = num.trim_start_matches('0').to_string();
                    if num.is_empty() {
                        num.push('0');
                    }
                }
                let start = self.pos - num.len();
                return Some((start, TokenEnum::IntLiteral));
            } else {
                self.next();
                match c {
                    '+' => {
                        return result(TokenEnum::Add);
                    }
                    '-' => {
                        return result(TokenEnum::Sub);
                    }
                    '*' => {
                        return result(TokenEnum::Mul);
                    }
                    '/' => {
                        if self.peek() == Some('/') {
                            while self.next() != Some('\n') {}
                            continue;
                        }
                        return result(TokenEnum::Div);
                    }
                    ':' => {
                        if self.peek() == Some('=') {
                            self.next();
                            return result(TokenEnum::Assign);
                        }
                        return result(TokenEnum::Colon);
                    }
                    '<' => {
                        if self.peek() == Some('>') {
                            self.next();
                            return result(TokenEnum::Ne);
                        } else if self.peek() == Some('=') {
                            self.next();
                            return result(TokenEnum::Le);
                        }
                        return result(TokenEnum::Lt);
                    }
                    '>' => {
                        if self.peek() == Some('=') {
                            self.next();
                            return result(TokenEnum::Ge);
                        }
                        return result(TokenEnum::Gt);
                    }
                    '=' => {
                        return result(TokenEnum::Eq);
                    }
                    '(' => {
                        return result(TokenEnum::LParen);
                    }
                    ')' => {
                        return result(TokenEnum::RParen);
                    }
                    ',' => {
                        return result(TokenEnum::Comma);
                    }
                    ';' => {
                        return result(TokenEnum::SemiColon);
                    }
                    c => {
                        errors.error(start, &format!("Unexpected character `{}`", c));
                        // Skip the character
                    }
                }
            }
        }
        None
    }

    pub fn next_token(&mut self, errors: &mut ErrorRecorder) -> Option<Token> {
        let (offset, token) = self.next_token_base(errors)?;
        let content = self.input[offset..self.pos].iter().collect();
        Some(Token {
            offset,
            content,
            token,
        })
    }
}
pub fn lex(input: &str, errors: &mut ErrorRecorder) -> Vec<Token> {
    let mut stream = CharStream::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = stream.next_token(errors) {
        tokens.push(token);
    }
    tokens
}
