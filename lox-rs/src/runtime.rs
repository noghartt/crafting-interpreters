use crate::error::Error;
use crate::lexer::Lexer;
use crate::parser::{Expr, Parser};
use crate::source::SourceMap;

#[derive(Default)]
pub struct Runtime {
    pub sources: SourceMap,
}

impl Runtime {
    pub fn run(&mut self, path: Option<String>, text: &str) -> Result<Expr, Error> {
        let file = self.sources.add(path, text);
        let source = &self.sources.get(file).chars;
        let tokens = Lexer::new(file, source).scan()?;
        Parser::new(tokens).parse()
    }
}
