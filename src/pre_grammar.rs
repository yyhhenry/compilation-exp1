use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    lex::{Lexer, OffsetToken, Token},
    line_pos::OffsetError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensOutput {
    pub tokens: Vec<OffsetToken>,
}
#[derive(Debug, Clone)]
/// A pre-grammar parser.
/// Checking for some syntax errors before the grammar.
/// for:
/// the basic structure of the program ( var ... begin ... end ),
/// syntax errors in var block ( unexpected type, missing comma, duplicate identifier ).
pub struct PreGrammar {
    lexer: Lexer,
    identifiers: BTreeSet<String>,
    tokens: Vec<OffsetToken>,
}

impl PreGrammar {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            identifiers: BTreeSet::new(),
            tokens: Vec::new(),
        }
    }
    fn push_token(&mut self) {
        if let Some(token) = self.lexer.consume_token_offset() {
            self.tokens.push(token);
        }
    }
    pub fn parse_one_var(&mut self) {
        match self.lexer.peek_token() {
            Some(Token::Ident(s)) => {
                self.push_token();
                if !self.identifiers.insert(s.clone()) {
                    self.lexer
                        .push_error(&format!("Duplicate identifier: {}", s));
                }
            }
            _ => {
                self.lexer.push_error("Expected identifier");
            }
        }
    }
    /// Peek the var block.
    /// Check for unexpected type, missing comma, duplicate identifier.
    pub fn parse_var(&mut self) {
        if self.lexer.peek_token() != Some(Token::Var) {
            self.lexer.push_error("Expected var");
        }
        self.push_token();
        while let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Ident(_) => {
                    self.parse_one_var();

                    while self.lexer.peek_token() == Some(Token::Comma) {
                        self.push_token();
                        match self.lexer.peek_token() {
                            Some(Token::Ident(_)) => {
                                self.parse_one_var();
                            }
                            _ => {
                                self.lexer.push_error("Expected identifier");
                            }
                        }
                    }

                    if self.lexer.peek_token() != Some(Token::Colon) {
                        self.lexer.push_error("Expected colon");
                    }
                    self.push_token();

                    match self.lexer.peek_token() {
                        Some(token) if token.is_type() => {
                            self.push_token();
                        }
                        _ => {
                            self.lexer.push_error("Expected type");
                        }
                    }

                    if self.lexer.peek_token() != Some(Token::SemiColon) {
                        self.lexer.push_error("Expected semicolon");
                    }
                    self.push_token();
                }
                _ => {
                    self.lexer.push_error("Expected identifier");
                }
            }
        }
    }
    /// Parse the program.
    /// Check if `begin` and `end` are present.
    /// Check if the identifiers have been declared.
    pub fn parse_program(&mut self) {
        if self.lexer.peek_token() != Some(Token::Begin) {
            self.lexer.push_error("Expected begin");
        }
        let mut layer = 0;
        while let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Begin => {
                    layer += 1;
                }
                Token::End => {
                    layer -= 1;
                    if layer < 0 {
                        self.lexer.push_error("Unexpected end");
                    }
                }
                Token::Ident(s) => {
                    if !self.identifiers.contains(&s) {
                        self.lexer
                            .push_error(&format!("Undeclared identifier: {}", s));
                    }
                }
                _ => {
                    self.push_token();
                }
            }
            self.push_token();
        }
        if layer > 0 {
            self.lexer.push_error("Missing end");
        }
    }
    /// Parse the input.
    pub fn parse(&mut self) {
        self.parse_var();
        self.parse_program();
    }
    pub fn output(self) -> Result<TokensOutput, Vec<OffsetError>> {
        let errors = self.lexer.errors();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(TokensOutput {
            tokens: self.tokens,
        })
    }
}
