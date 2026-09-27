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
        let mut scanner = Lexer::new(self.start_file, source);
        scanner.scan()?;
        todo!("To be executed...");
    }
}
