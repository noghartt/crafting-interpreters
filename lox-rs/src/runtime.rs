use crate::error::Error;
use crate::lexer::Lexer;

pub struct Runtime {
    had_error: bool,
    start_file: String,
}

impl Runtime {
    pub fn new(path: String) -> Self {
        Self {
            had_error: false,
            start_file: path,
        }
    }

    pub fn run(self, source: String) -> Result<(), Error> {
        let lexer = Lexer::new(self.start_file, source);
        let tokens = lexer.scan()?;
        println!("Tokens: {tokens:?}");

        Ok(())
    }
}
