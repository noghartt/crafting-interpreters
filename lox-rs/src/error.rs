#[derive(Debug)]
pub struct Error {
    path: String,
    line: usize,
    message: String,
}

impl Error {
    pub fn new(line: usize, path: String, message: String) -> Self {
        Self {
            path,
            line,
            message,
        }
    }

    pub fn report(self, path: Option<String>) {
        // TODO: Fix path here in the future.
        println!(
            "[line: {}] Error ({}): {}",
            self.line, self.path, self.message
        );
    }
}
