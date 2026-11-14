use std::collections::HashMap;

use crate::{
    error::{Error, ErrorKind},
    lexer::{Token, TokenType},
};

use super::Value;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn assign(&mut self, identifier: &Token, value: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &identifier.token else {
            unreachable!("parser only assigns to identifier tokens");
        };

        match self.values.get_mut(name) {
            Some(slot) => {
                *slot = value;
                Ok(())
            }
            None => Err(Error::new(
                ErrorKind::Runtime,
                identifier.metadata.span,
                format!("Undefined variable '{name}'."),
            )),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, identifier: &Token) -> Result<Value, Error> {
        let TokenType::Identifier(name) = &identifier.token else {
            unreachable!("parser only builds variables from identifier token");
        };

        self.values.get(name).cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::Runtime,
                identifier.metadata.span,
                format!("Undefined variable '{name}'"),
            )
        })
    }
}
