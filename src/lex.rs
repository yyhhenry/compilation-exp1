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

    // operators +|-|*|/|:=|<|>|<>|>=|<=|==|:|(|)
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
/// To simplify the parser.
enum NextToken {
    EOF,
    Blank,
    Type(TokenEnum),
    WithContent(TokenEnum, String),
}
impl From<TokenEnum> for NextToken {
    fn from(token: TokenEnum) -> Self {
        NextToken::Type(token)
    }
}
impl From<(TokenEnum, String)> for NextToken {
    fn from((token, content): (TokenEnum, String)) -> Self {
        NextToken::WithContent(token, content)
    }
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
    fn next_token_base(&mut self, errors: &mut ErrorRecorder) -> NextToken {
        let c = match self.peek() {
            Some(c) => c,
            None => return NextToken::EOF,
        };
        if c.is_whitespace() {
            self.next();
            return NextToken::Blank;
        }
        let start = self.pos;
        if c.is_ascii_alphabetic() {
            let mut ident = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_alphanumeric() {
                    ident.push(c);
                    self.next();
                } else {
                    break;
                }
            }
            match ident.to_lowercase().as_str() {
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
            }
            .into()
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
                errors.warning(start, "Number should not start with 0");
                // Remove leading zeros
                num = num.trim_start_matches('0').to_string();
                if num.is_empty() {
                    num.push('0');
                }
                return NextToken::WithContent(TokenEnum::IntLiteral, num);
            }
            TokenEnum::IntLiteral.into()
        } else {
            self.next();
            match c {
                '+' => TokenEnum::Add.into(),
                '-' => TokenEnum::Sub.into(),
                '*' => TokenEnum::Mul.into(),
                '/' => {
                    if self.peek() == Some('/') {
                        while self.next() != Some('\n') {}
                        return NextToken::Blank;
                    }
                    TokenEnum::Div.into()
                }
                ':' => {
                    if self.peek() == Some('=') {
                        self.next();
                        return TokenEnum::Assign.into();
                    }
                    TokenEnum::Colon.into()
                }
                '<' => {
                    if self.peek() == Some('>') {
                        self.next();
                        return TokenEnum::Ne.into();
                    } else if self.peek() == Some('=') {
                        self.next();
                        return TokenEnum::Le.into();
                    }
                    TokenEnum::Lt.into()
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.next();
                        return TokenEnum::Ge.into();
                    }
                    TokenEnum::Gt.into()
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.next();
                        return TokenEnum::Eq.into();
                    }
                    errors.error(start, "Unexpected operator `=`");
                    NextToken::Blank
                }
                '(' => TokenEnum::LParen.into(),
                ')' => TokenEnum::RParen.into(),
                ',' => TokenEnum::Comma.into(),
                ';' => TokenEnum::SemiColon.into(),
                c => {
                    errors.error(start, &format!("Unexpected character `{}`", c));
                    NextToken::Blank
                }
            }
        }
    }

    pub fn next_token(&mut self, errors: &mut ErrorRecorder) -> Option<Token> {
        loop {
            let start = self.pos;
            let next_token = self.next_token_base(errors);
            match next_token {
                NextToken::EOF => return None,
                NextToken::Blank => {}
                NextToken::Type(token) => {
                    let content = self.input[start..self.pos].iter().collect();
                    return Some(Token {
                        offset: start,
                        content,
                        token,
                    });
                }
                NextToken::WithContent(token, content) => {
                    return Some(Token {
                        offset: start,
                        content,
                        token,
                    });
                }
            }
        }
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
