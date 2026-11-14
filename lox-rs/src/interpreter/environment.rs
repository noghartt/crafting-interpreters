use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{Error, ErrorKind},
    lexer::{Token, TokenType},
};

use super::Value;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, identifier: &Token) -> Result<Value, Error> {
        let name = identifier_name(identifier);

        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        match &self.enclosing {
            Some(enclosing) => enclosing.borrow().get(identifier),
            None => Err(undefined_variable(identifier, name)),
        }
    }

    pub fn assign(&mut self, identifier: &Token, value: Value) -> Result<(), Error> {
        let name = identifier_name(identifier);

        if let Some(slot) = self.values.get_mut(name) {
            *slot = value;
            return Ok(());
        }

        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(identifier, value),
            None => Err(undefined_variable(identifier, name)),
        }
    }
}

fn identifier_name(identifier: &Token) -> &str {
    let TokenType::Identifier(name) = &identifier.token else {
        unreachable!("parser only builds variables from identifier tokens");
    };

    name
}

fn undefined_variable(identifier: &Token, name: &str) -> Error {
    Error::new(
        ErrorKind::Runtime,
        identifier.metadata.span,
        format!("Undefined variable '{name}'."),
    )
}
