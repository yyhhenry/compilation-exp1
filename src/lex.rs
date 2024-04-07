use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::line_pos::OffsetError;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Token in PL/0 language.
/// Ignore case.
/// ```txt
/// var|integer|longint|bool|real|if|then|else|while|do
/// |begin|end|and|or|+|-|*|/|:=|<|>|<>|>=|<=|==
/// |:|(|)|,|[1-9][0-9]*|[a-zA-Z][a-zA-Z0-9]*
/// ```
/// ```txt
/// <var> ::= <id> {, <id>}: <type>
/// <vars> ::= var <var> {; <var>}
/// <stmt> ::= while <expr> do <stmt>
///        | begin <stmt> {; <stmt>} end
///        | if <expr> then <stmt>
///        | if <expr> then <stmt> else <stmt>
/// ```
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

    // operators
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
    /// ==
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
    Ident(String),
    Int(i64),
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
    offset: usize,
    token: Token,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Rc<Vec<char>>,
    pos: usize,
    errors: Vec<OffsetError>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: Rc::new(input.chars().collect()),
            pos: 0,
            errors: Vec::new(),
        }
    }
    pub fn errors(self) -> Vec<OffsetError> {
        self.errors
    }
    fn peek_char(&self) -> Option<char> {
        self.input.get(self.pos + 1).cloned()
    }
    fn consume_char(&mut self) -> Option<char> {
        match self.input.get(self.pos).cloned() {
            Some(c) => {
                self.pos += 1;
                Some(c)
            }
            None => None,
        }
    }
    pub fn push_error(&mut self, msg: &str) {
        self.errors.push(OffsetError {
            offset: self.pos,
            msg: msg.to_string(),
        });
    }
    pub fn peek_token(&self) -> Option<Token> {
        let mut lexer = self.clone();
        lexer.consume_token()
    }
    pub fn consume_token_offset(&mut self) -> Option<OffsetToken> {
        let offset = self.pos;
        self.consume_token()
            .map(|token| OffsetToken { offset, token })
    }
    pub fn consume_token(&mut self) -> Option<Token> {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
                continue;
            }
            if c.is_ascii_alphabetic() {
                let mut ident = String::new();
                while self
                    .peek_char()
                    .map_or(false, |c| c.is_ascii_alphanumeric())
                {
                    ident.push(c);
                    self.consume_char();
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
                    _ => Token::Ident(ident),
                };
                return Some(token);
            } else if c.is_numeric() {
                let mut num = String::new();
                while self.peek_char().map_or(false, |c| c.is_numeric()) {
                    num.push(c);
                    self.consume_char();
                }
                if self.peek_char().map_or(false, |c| c.is_ascii_alphabetic()) {
                    self.push_error("Unexpected character after number");
                }
                return Some(Token::Int(num.parse().unwrap()));
            } else {
                match c {
                    '+' => {
                        self.consume_char();
                        return Some(Token::Add);
                    }
                    '-' => {
                        self.consume_char();
                        return Some(Token::Sub);
                    }
                    '*' => {
                        self.consume_char();
                        return Some(Token::Mul);
                    }
                    '/' => {
                        self.consume_char();
                        return Some(Token::Div);
                    }
                    ':' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return Some(Token::Assign);
                        }
                        return Some(Token::Colon);
                    }
                    '<' => {
                        self.consume_char();
                        if self.peek_char() == Some('>') {
                            self.consume_char();
                            return Some(Token::Ne);
                        } else if self.peek_char() == Some('=') {
                            self.consume_char();
                            return Some(Token::Le);
                        }
                        return Some(Token::Lt);
                    }
                    '>' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return Some(Token::Ge);
                        }
                        return Some(Token::Gt);
                    }
                    '=' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return Some(Token::Eq);
                        }
                        self.push_error("Unexpected character after =");
                    }
                    '(' => {
                        self.consume_char();
                        return Some(Token::LParen);
                    }
                    ')' => {
                        self.consume_char();
                        return Some(Token::RParen);
                    }
                    ',' => {
                        self.consume_char();
                        return Some(Token::Comma);
                    }
                    ';' => {
                        self.consume_char();
                        return Some(Token::SemiColon);
                    }
                    c => {
                        self.push_error(&format!("Unexpected character `{}`", c));
                    }
                }
            }
        }
        None
    }
}
