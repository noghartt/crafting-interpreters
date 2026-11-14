use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::SourceMap;

#[derive(Default)]
pub struct Runtime {
    pub sources: SourceMap,
    interpreter: Interpreter,
}

impl Runtime {
    pub fn run(&mut self, path: Option<String>, text: &str) -> Result<(), Vec<Error>> {
        let file = self.sources.add(path, text);
        let source = &self.sources.get(file).chars;
        let tokens = Lexer::new(file, source)
            .scan()
            .map_err(|error| vec![error])?;
        let ast = Parser::new(tokens).parse()?;

        self.interpreter.interpret(ast).map_err(|error| vec![error])
    }
}
