use anyhow::{anyhow, Error, Result};
use std::collections::BTreeMap;

use crate::{
    error::ErrorRecorder,
    lex::{lex, Token, TokenEnum},
};
#[derive(Debug, Clone, PartialEq)]
pub enum TypeEnum {
    Integer,
    Longint,
    Bool,
    Real,
}
impl TryFrom<TokenEnum> for TypeEnum {
    type Error = Error;
    fn try_from(token: TokenEnum) -> Result<Self> {
        match token {
            TokenEnum::Integer => Ok(Self::Integer),
            TokenEnum::Longint => Ok(Self::Longint),
            TokenEnum::Bool => Ok(Self::Bool),
            TokenEnum::Real => Ok(Self::Real),
            _ => Err(anyhow!("Expected type, found {:?}", token)),
        }
    }
}
#[derive(Debug, Clone)]
pub struct TokenStream {
    tokens: Vec<Token>,
    index: usize,
}
impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }
    pub fn peek(&self) -> Option<&TokenEnum> {
        self.tokens.get(self.index).map(|t| &t.token)
    }
    pub fn peek_pos(&self) -> usize {
        self.tokens
            .get(self.index)
            .map(|t| t.offset)
            .unwrap_or(usize::MAX)
    }
    pub fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index);
        if token.is_some() {
            self.index += 1;
        }
        token
    }
    pub fn expect(&mut self, token: TokenEnum, errors: &mut ErrorRecorder) -> Result<&Token> {
        let pos = self.peek_pos();
        match self.next() {
            Some(t) if t.token == token => Ok(t),
            Some(t) => errors.hard(pos, format!("Expected {:?}, found {:?}", token, t.token)),
            None => errors.hard(pos, format!("Expected {:?}, found EOF", token)),
        }
    }
    /// Match identifier
    /// Returns (identifier_lowercase, offset)
    pub fn identifier(&mut self, errors: &mut ErrorRecorder) -> Result<(String, usize)> {
        let token = self.expect(TokenEnum::Identifier, errors)?;
        Ok((token.content.to_lowercase(), token.offset))
    }
    /// Match: i0, i1, i2
    /// Return vec of (identifier, offset)
    pub fn identifier_list(&mut self, errors: &mut ErrorRecorder) -> Result<Vec<(String, usize)>> {
        let mut identifiers = Vec::new();
        identifiers.push(self.identifier(errors)?);
        loop {
            match self.peek() {
                Some(TokenEnum::Colon) => return Ok(identifiers),
                Some(TokenEnum::Comma) => {
                    self.next();
                    identifiers.push(self.identifier(errors)?);
                }
                Some(TokenEnum::Identifier) => {
                    errors.error(self.peek_pos(), "Missing comma");
                    identifiers.push(self.identifier(errors)?);
                }
                _ => {
                    return errors.hard(self.peek_pos(), "Expected comma or colon");
                }
            }
        }
    }
    /// Match: i0, i1, i2: Type;
    pub fn def_line(
        &mut self,
        vars: &mut BTreeMap<String, TypeEnum>,
        errors: &mut ErrorRecorder,
    ) -> Result<()> {
        let identifiers = self.identifier_list(errors)?;
        self.expect(TokenEnum::Colon, errors)?;
        let pos = self.peek_pos();
        let type_enum = match self.next().and_then(|t| TypeEnum::try_from(t.token).ok()) {
            Some(t) => t,
            None => {
                return errors.hard(pos, "Expected type");
            }
        };
        match self.peek() {
            Some(TokenEnum::SemiColon) => {
                self.next();
            }
            _ => {
                errors.error(self.peek_pos(), "Missing semicolon");
            }
        }
        for (identifier, offset) in identifiers {
            if vars.contains_key(&identifier) {
                errors.error(offset, format!("Duplicate identifier: {}", identifier));
            } else {
                vars.insert(identifier.clone(), type_enum.clone());
            }
        }
        Ok(())
    }
    /// Match: var i0, i1, i2: Type; ... ;
    pub fn var_block(&mut self, errors: &mut ErrorRecorder) -> Result<BTreeMap<String, TypeEnum>> {
        let mut identifiers = BTreeMap::new();
        match self.peek() {
            Some(TokenEnum::Var) => {
                self.next();
            }
            Some(TokenEnum::Begin) => {
                return Ok(identifiers);
            }
            _ => {
                return errors.hard(self.peek_pos(), "Expected var");
            }
        }
        while self.peek().map_or(false, |t| t != &TokenEnum::Begin) {
            self.def_line(&mut identifiers, errors)?;
        }
        Ok(identifiers)
    }
    /// Match: begin ... end
    /// Check if the identifiers have been declared.
    pub fn program_block(
        &mut self,
        vars: &BTreeMap<String, TypeEnum>,
        errors: &mut ErrorRecorder,
    ) -> Result<()> {
        if self.peek() != Some(&TokenEnum::Begin) {
            return errors.hard(self.peek_pos(), "Expected begin");
        }
        while let Some(token) = self.peek() {
            match token {
                TokenEnum::Identifier => {
                    let token = self.next().unwrap();
                    let s = token.content.to_lowercase();
                    if !vars.contains_key(&s) {
                        errors.error(token.offset, format!("Undeclared identifier: {}", s));
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
        Ok(())
    }
    pub fn code(&mut self, errors: &mut ErrorRecorder) -> Result<()> {
        let vars = self.var_block(errors)?;
        self.program_block(&vars, errors)
    }
}
pub fn parse(content: &str, errors: &mut ErrorRecorder) -> Vec<Token> {
    let tokens = lex(content, errors);
    let mut stream = TokenStream::new(tokens.clone());
    match stream.code(errors) {
        Ok(()) => {}
        Err(_) => {
            eprintln!("Hard error detected, aborting");
        }
    }
    tokens
}
