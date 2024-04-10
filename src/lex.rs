use serde::{Deserialize, Serialize};

use crate::error::ErrorRecorder;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Token in PL/0 Like language.
/// Ignore case.
pub enum Token {
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
    Ident(String),
    /// Integer literal [1-9][0-9]*|0, no leading 0
    Int(String),
}
impl Token {
    pub fn is_type(&self) -> bool {
        match self {
            Token::Integer | Token::Longint | Token::Bool | Token::Real => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetToken {
    pub offset: usize,
    pub token: Token,
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
    pub fn next_token(
        &mut self,
        chars: &mut CharStream,
        errors: &mut ErrorRecorder,
    ) -> Option<OffsetToken> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
                continue;
            }
            let start = self.pos;
            let result = |token| {
                Some(OffsetToken {
                    offset: start,
                    token,
                })
            };
            if c.is_ascii_alphabetic() {
                let mut ident = String::new();
                while self.peek().map_or(false, |c| c.is_ascii_alphanumeric()) {
                    ident.push(self.next().unwrap());
                }
                let token = match ident.to_lowercase().as_str() {
                    "var" => Token::Var,
                    "integer" => Token::Integer,
                    "longint" => Token::Longint,
                    "bool" => Token::Bool,
                    "real" => Token::Real,
                    "if" => Token::If,
                    "then" => Token::Then,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "do" => Token::Do,
                    "begin" => Token::Begin,
                    "end" => Token::End,
                    "and" => Token::And,
                    "or" => Token::Or,
                    ident => Token::Ident(ident.to_string()),
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
                return result(Token::Int(num));
            } else {
                self.next();
                match c {
                    '+' => {
                        return result(Token::Add);
                    }
                    '-' => {
                        return result(Token::Sub);
                    }
                    '*' => {
                        return result(Token::Mul);
                    }
                    '/' => {
                        if self.peek() == Some('/') {
                            while self.next() != Some('\n') {}
                            continue;
                        }
                        return result(Token::Div);
                    }
                    ':' => {
                        if self.peek() == Some('=') {
                            self.next();
                            return result(Token::Assign);
                        }
                        return result(Token::Colon);
                    }
                    '<' => {
                        if self.peek() == Some('>') {
                            self.next();
                            return result(Token::Ne);
                        } else if self.peek() == Some('=') {
                            self.next();
                            return result(Token::Le);
                        }
                        return result(Token::Lt);
                    }
                    '>' => {
                        if self.peek() == Some('=') {
                            self.next();
                            return result(Token::Ge);
                        }
                        return result(Token::Gt);
                    }
                    '=' => {
                        return result(Token::Eq);
                    }
                    '(' => {
                        return result(Token::LParen);
                    }
                    ')' => {
                        return result(Token::RParen);
                    }
                    ',' => {
                        return result(Token::Comma);
                    }
                    ';' => {
                        return result(Token::SemiColon);
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
}
pub fn lex(input: &str, errors: &mut ErrorRecorder) -> Vec<OffsetToken> {
    let mut stream = CharStream::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = stream.next_token(&mut stream, errors) {
        tokens.push(token);
    }
    tokens
}
