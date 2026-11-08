use crate::error::Error;
use crate::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::SourceMap;

#[derive(Default)]
pub struct Runtime {
    pub sources: SourceMap,
}

impl Runtime {
    pub fn run(&mut self, path: Option<String>, text: &str) -> Result<Value, Vec<Error>> {
        let file = self.sources.add(path, text);
        let source = &self.sources.get(file).chars;
        let tokens = Lexer::new(file, source)
            .scan()
            .map_err(|error| vec![error])?;
        let ast = Parser::new(tokens).parse()?;

        match Interpreter::new().interpret(ast) {
            Ok(result) => Ok(result),
            Err(e) => Err(vec![e]),
        }
    }
}
