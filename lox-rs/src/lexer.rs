mod token;

pub use token::{Token, TokenType};

use crate::error::Error;

pub struct Lexer {
    path: String,
    source: Vec<char>,
    start: usize,
    current: usize,
}

impl Lexer {
    pub fn new(path: String, source: String) -> Self {
        Self {
            path,
            source: source.chars().collect(),
            start: 0,
            current: 0,
        }
    }

    pub fn scan(mut self) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            if self.is_at_end() {
                break;
            }

            self.start = self.current;

            let token = self.get_token()?;
            if token.is_none() {
                continue;
            }

            tokens.push(token.unwrap());
        }

        tokens.push(Token::new(TokenType::Eof, (self.current, self.current)));

        Ok(tokens)
    }

    fn get_token(&mut self) -> Result<Option<Token>, Error> {
        let Some(c) = self.advance() else {
            return Ok(None);
        };

        let token_type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,
            '!' => {
                if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '=' => {
                if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '<' => {
                if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '/' => {
                if self.match_next('/') {
                    loop {
                        if self.peek() == Some('\n') || self.is_at_end() {
                            return Ok(None);
                        }

                        self.advance();
                    }
                }

                TokenType::Slash
            }
            '"' => return self.lex_string(),
            ' ' | '\r' | '\n' | '\t' => return Ok(None),
            c if is_digit(Some(c)) => return self.lex_number(),
            c if is_alpha(Some(c)) => return self.lex_identifier(),
            other => return Err(self.generate_error(format!("Unexpected token: {other}"))),
        };

        Ok(Some(Token::new(token_type, (self.start, self.current))))
    }

    fn lex_string(&mut self) -> Result<Option<Token>, Error> {
        while self.peek() != Some('"') && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.generate_error(String::from("Unterminated string.")));
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];
        let token_type = TokenType::String(String::from_iter(value));

        Ok(Some(Token::new(token_type, (self.start, self.current))))
    }

    fn lex_number(&mut self) -> Result<Option<Token>, Error> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == Some('.') && is_digit(self.peek_next()) {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = String::from_iter(&self.source[self.start..self.current]);
        let number = value.parse::<f64>().unwrap();
        let token_type = TokenType::Number(number);

        Ok(Some(Token::new(token_type, (self.start, self.current))))
    }

    fn lex_identifier(&mut self) -> Result<Option<Token>, Error> {
        while self.peek().is_some_and(|c| c.is_alphanumeric()) {
            self.advance();
        }

        let text = String::from_iter(&self.source[self.start..self.current]);
        let Some(token_type) = is_keyword(&text) else {
            let identifier = TokenType::Identifier(text);
            return Ok(Some(Token::new(identifier, (self.start, self.current))));
        };

        Ok(Some(Token::new(token_type, (self.start, self.current))))
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        Some(self.source[self.current])
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }

        self.source.get(self.current + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;

        self.current += 1;

        Some(c)
    }

    fn match_next(&mut self, expected_char: char) -> bool {
        if self.peek() != Some(expected_char) {
            return false;
        }

        self.current += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn get_line_from_offset(&self, offset: usize) -> usize {
        self.source[..offset].iter().filter(|&&c| c == '\n').count() + 1
    }

    fn generate_error(&self, message: String) -> Error {
        let line = self.get_line_from_offset(self.start);
        Error::new(line, self.path.clone(), message)
    }
}

fn is_digit(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_digit())
}

fn is_alpha(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphabetic())
}

fn is_keyword(text: &str) -> Option<TokenType> {
    match text {
        "and" => Some(TokenType::And),
        "class" => Some(TokenType::Class),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "for" => Some(TokenType::For),
        "fun" => Some(TokenType::Fun),
        "if" => Some(TokenType::If),
        "nil" => Some(TokenType::Nil),
        "or" => Some(TokenType::Or),
        "print" => Some(TokenType::Print),
        "return" => Some(TokenType::Return),
        "super" => Some(TokenType::Super),
        "this" => Some(TokenType::This),
        "true" => Some(TokenType::True),
        "var" => Some(TokenType::Var),
        "while" => Some(TokenType::While),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_parenthesis() {
        let source = "( )";

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 3);

        assert_eq!(tokens[0].token, TokenType::LeftParen);
        assert_eq!(tokens[1].token, TokenType::RightParen);
        assert_eq!(tokens[2].token, TokenType::Eof);
    }

    #[test]
    fn test_check_parenthesis_with_space() {
        let source = r#"
            ( 
            )
        "#;

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 3);

        assert_eq!(tokens[0].token, TokenType::LeftParen);
        assert_eq!(tokens[1].token, TokenType::RightParen);
        assert_eq!(tokens[2].token, TokenType::Eof);
    }

    #[test]
    fn test_string_lexing() {
        let source = r#""Hello, world!""#;

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 2);

        assert_eq!(
            tokens[0].token,
            TokenType::String("Hello, world!".to_string())
        );
        assert_eq!(tokens[1].token, TokenType::Eof);
    }

    #[test]
    fn test_comment() {
        let source = r#"// Hi ("#;

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 1);

        assert_eq!(tokens[0].token, TokenType::Eof)
    }

    #[test]
    fn test_comment_with_next_string() {
        let source = r#"
        // Hi :)
        "Hello, world!"
        "#;

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens[0].token, TokenType::String("Hello, world!".into()));
        assert_eq!(tokens[1].token, TokenType::Eof);
    }

    #[test]
    fn test_lex_digit_without_floating() {
        let source = "12345";

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens[0].token, TokenType::Number(12345_f64));
        assert_eq!(tokens[1].token, TokenType::Eof);
    }

    #[test]
    fn test_lex_digit_with_floating() {
        let source = "3.1415";

        let lexer = create_new_lexer(source);
        let tokens = lexer.scan().unwrap();

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens[0].token, TokenType::Number(3.1415));
        assert_eq!(tokens[1].token, TokenType::Eof);
    }

    fn create_new_lexer(source: &str) -> Lexer {
        Lexer::new("".to_string(), source.to_string())
    }
}
