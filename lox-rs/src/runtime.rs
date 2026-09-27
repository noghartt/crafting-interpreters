use crate::error::Error;
use crate::lexer::Lexer;

pub struct Runtime {
    had_error: bool,
}

impl Runtime {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn run(self, source: String) -> Result<(), Error> {
        println!("Input: {source}");
        let mut scanner = Lexer::new(source);
        todo!("To be executed...");
    }
}
