use crate::error::Error;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::SourceMap;

#[derive(Default)]
pub struct Runtime {
    sources: SourceMap,
    had_error: bool,
}

impl Runtime {
    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn run(&mut self, path: Option<String>, text: &str) {
        let file = self.sources.add(path, text);
        let source = &self.sources.get(file).chars;
        let tokens = match Lexer::new(file, source).scan() {
            Ok(tokens) => tokens,
            Err(error) => return self.report(error),
        };

        match Parser::new(tokens).parse() {
            Ok(ast) => println!("{ast}"),
            Err(error) => self.report(error),
        }
    }

    fn report(&mut self, error: Error) {
        eprintln!("{}", error.render(&self.sources));
        self.had_error = true;
    }
}
