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
        if let Some(token) = self.lexer.consume_token() {
            self.tokens.push(token);
        }
    }
    fn push_error(&mut self, msg: &str) {
        self.lexer.push_error_peek(msg);
    }
    fn parse_one_var(&mut self) {
        match self.lexer.peek_token() {
            Some(Token::Ident(s)) => {
                self.push_token();
                if !self.identifiers.insert(s.clone()) {
                    self.push_error(&format!("Duplicate identifier: {}", s));
                }
            }
            _ => {
                self.push_error("Expected identifier");
            }
        }
    }
    /// Parse the var block.
    /// Check for unexpected type, missing comma, duplicate identifier.
    fn parse_var_block(&mut self) {
        if self.lexer.peek_token() != Some(Token::Var) {
            if self.lexer.peek_token() != Some(Token::Begin) {
                self.push_error("Expected var");
            }
            return;
        }
        self.push_token();
        while let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Begin => {
                    break;
                }
                Token::Ident(_) => {
                    self.parse_one_var();
                    while self.lexer.peek_token() == Some(Token::Comma) {
                        self.push_token();
                        match self.lexer.peek_token() {
                            Some(Token::Ident(_)) => {
                                self.parse_one_var();
                            }
                            _ => {
                                self.push_error("Expected identifier");
                                continue;
                            }
                        }
                    }

                    if self.lexer.peek_token() != Some(Token::Colon) {
                        self.push_error("Expected colon");
                        continue;
                    }
                    self.push_token();

                    match self.lexer.peek_token() {
                        Some(token) if token.is_type() => {
                            self.push_token();
                        }
                        _ => {
                            self.push_error("Expected type");
                            continue;
                        }
                    }

                    if self.lexer.peek_token() != Some(Token::SemiColon) {
                        self.push_error("Expected semicolon");
                        continue;
                    }
                    self.push_token();
                }
                _ => {
                    self.lexer.consume_token();
                    self.push_error("Expected identifier");
                }
            }
        }
    }
    /// Parse the program block.
    /// Check if `begin` and `end` are present.
    /// Check if the identifiers have been declared.
    fn parse_program_block(&mut self) {
        if self.lexer.peek_token() != Some(Token::Begin) {
            self.push_error("Expected begin");
        }
        let mut layer = 0;
        let mut should_finish = false;
        while let Some(token) = self.lexer.peek_token() {
            if should_finish {
                self.push_error("Unexpected token after end");
                break;
            }
            match token {
                Token::Begin => {
                    layer += 1;
                }
                Token::End => {
                    if layer == 0 {
                        self.push_error("Unexpected end");
                        continue;
                    }
                    if layer == 1 {
                        should_finish = true;
                    }
                    layer -= 1;
                }
                Token::Ident(s) => {
                    if !self.identifiers.contains(&s) {
                        self.push_error(&format!("Undeclared identifier: {}", s));
                    }
                }
                _ => {}
            }
            self.push_token();
        }
        if layer > 0 {
            self.push_error("Missing end");
        }
    }
    /// Parse the input.
    pub fn parse(&mut self) {
        self.parse_var_block();
        self.parse_program_block();
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
