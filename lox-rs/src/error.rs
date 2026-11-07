use crate::source::{SourceFile, SourceMap, Span};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    Lex,
    Parse,
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    span: Span,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span, message: String) -> Self {
        Self {
            kind,
            span,
            message,
        }
    }

    pub fn render(&self, sources: &SourceMap) -> String {
        let file = sources.get(self.span.file);
        let path = file.path.as_deref().unwrap_or("CLI");
        let location = self.render_location(file);
        let line = self.get_line_from_offset(&file.chars, self.span.start);
        format!("[line {line}] Error ({path}){location}: {}", self.message)
    }

    fn get_line_from_offset(&self, source: &[char], offset: usize) -> usize {
        source[..offset].iter().filter(|&&c| c == '\n').count() + 1
    }

    fn render_location(&self, file: &SourceFile) -> String {
        if self.kind == ErrorKind::Lex {
            return String::new();
        }

        if self.span.start == file.chars.len() {
            return String::from(" at end");
        }

        let span = &file.chars[self.span.start..self.span.end];
        let lexeme = String::from_iter(span);
        format!(" at '{lexeme}'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECT_PAREN: &str = "Expect ')' after expression.";

    fn render(
        source: &str,
        path: Option<&str>,
        kind: ErrorKind,
        start: usize,
        end: usize,
    ) -> String {
        let mut sources = SourceMap::default();
        let file = sources.add(path.map(String::from), source);
        let message = match kind {
            ErrorKind::Lex => "Unterminated string.",
            ErrorKind::Parse => EXPECT_PAREN,
        };

        Error::new(kind, Span { file, start, end }, message.to_string()).render(&sources)
    }

    #[test]
    fn test_parse_error_at_lexeme() {
        assert_eq!(
            render("(1 + 2 3", None, ErrorKind::Parse, 7, 8),
            "[line 1] Error (CLI) at '3': Expect ')' after expression."
        );
    }

    #[test]
    fn test_parse_error_at_end() {
        assert_eq!(
            render("(1 + 2", None, ErrorKind::Parse, 6, 6),
            "[line 1] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_parse_error_on_later_line() {
        assert_eq!(
            render("(1 +\n2 3", None, ErrorKind::Parse, 7, 8),
            "[line 2] Error (CLI) at '3': Expect ')' after expression."
        );
    }

    #[test]
    fn test_parse_error_at_end_after_trailing_newline() {
        assert_eq!(
            render("1 +\n(2\n", None, ErrorKind::Parse, 7, 7),
            "[line 3] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_parse_error_uses_char_offsets() {
        assert_eq!(
            render("\"olá\" == (1 2", None, ErrorKind::Parse, 12, 13),
            "[line 1] Error (CLI) at '2': Expect ')' after expression."
        );
    }

    #[test]
    fn test_parse_error_at_non_ascii_lexeme() {
        assert_eq!(
            render("\"olá\" 1", None, ErrorKind::Parse, 0, 5),
            "[line 1] Error (CLI) at '\"olá\"': Expect ')' after expression."
        );
    }

    #[test]
    fn test_lex_error_has_no_location() {
        assert_eq!(
            render("\"abc", Some("main.lox"), ErrorKind::Lex, 0, 4),
            "[line 1] Error (main.lox): Unterminated string."
        );
    }
}
