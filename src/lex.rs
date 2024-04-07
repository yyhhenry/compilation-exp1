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
    Int(i32),
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
        self.input.get(self.pos).cloned()
    }
    fn consume_char(&mut self) -> Option<char> {
        match self.peek_char() {
            Some(c) => {
                self.pos += 1;
                Some(c)
            }
            None => None,
        }
    }
    pub fn peek_token(&self) -> Option<Token> {
        let mut lexer = Self {
            input: self.input.clone(),
            pos: self.pos,
            errors: Vec::new(),
        };
        lexer.consume_token().map(|t| t.token)
    }
    pub fn peek_pos(&self) -> usize {
        let mut lexer = self.clone();
        lexer.consume_token().map(|t| t.offset).unwrap_or(self.pos)
    }
    fn push_error_pos(&mut self, offset: usize, msg: &str) {
        self.errors.push(OffsetError {
            offset,
            msg: msg.to_string(),
        });
    }
    fn push_error(&mut self, msg: &str) {
        self.push_error_pos(self.pos, msg);
    }
    pub fn push_error_peek(&mut self, msg: &str) {
        self.push_error_pos(self.peek_pos(), msg);
    }
    pub fn consume_token(&mut self) -> Option<OffsetToken> {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
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
                while self
                    .peek_char()
                    .map_or(false, |c| c.is_ascii_alphanumeric())
                {
                    ident.push(self.consume_char().unwrap());
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
                return result(token);
            } else if c.is_numeric() {
                let mut num = String::new();
                while self.peek_char().map_or(false, |c| c.is_numeric()) {
                    num.push(self.consume_char().unwrap());
                }
                if self.peek_char().map_or(false, |c| c.is_ascii_alphabetic()) {
                    self.push_error("Unexpected character after number");
                }
                if num.starts_with('0') && num.len() > 1 {
                    self.push_error_pos(start, "Number cannot start with 0");
                }
                match num.parse() {
                    Ok(num) => return result(Token::Int(num)),
                    Err(_) => self.push_error_pos(start, "Invalid number or out of range"),
                };
            } else {
                match c {
                    '+' => {
                        self.consume_char();
                        return result(Token::Add);
                    }
                    '-' => {
                        self.consume_char();
                        return result(Token::Sub);
                    }
                    '*' => {
                        self.consume_char();
                        return result(Token::Mul);
                    }
                    '/' => {
                        if self.peek_char() == Some('/') {
                            while self.peek_char() != Some('\n') {
                                self.consume_char();
                            }
                            continue;
                        } else {
                            self.consume_char();
                            return result(Token::Div);
                        }
                    }
                    ':' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return result(Token::Assign);
                        }
                        return result(Token::Colon);
                    }
                    '<' => {
                        self.consume_char();
                        if self.peek_char() == Some('>') {
                            self.consume_char();
                            return result(Token::Ne);
                        } else if self.peek_char() == Some('=') {
                            self.consume_char();
                            return result(Token::Le);
                        }
                        return result(Token::Lt);
                    }
                    '>' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return result(Token::Ge);
                        }
                        return result(Token::Gt);
                    }
                    '=' => {
                        self.consume_char();
                        if self.peek_char() == Some('=') {
                            self.consume_char();
                            return result(Token::Eq);
                        }
                        self.push_error("Unexpected character after =");
                    }
                    '(' => {
                        self.consume_char();
                        return result(Token::LParen);
                    }
                    ')' => {
                        self.consume_char();
                        return result(Token::RParen);
                    }
                    ',' => {
                        self.consume_char();
                        return result(Token::Comma);
                    }
                    ';' => {
                        self.consume_char();
                        return result(Token::SemiColon);
                    }
                    c => {
                        self.push_error(&format!("Unexpected character `{}`", c));
                        self.consume_char();
                    }
                }
            }
        }
        None
    }
}
