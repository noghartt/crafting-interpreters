mod token;

pub use token::{Token, TokenType};

use crate::{error::Error, lexer::TokenType::Print};

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
                        if self.peek() == '\n' || self.is_at_end() {
                            break;
                        }

                        self.advance();
                    }
                }

                TokenType::Slash
            }
            ' ' | '\r' | '\n' | '\t' => return Ok(None),
            other => {
                let line = self.get_line_from_offset(self.start);
                return Err(Error::new(
                    line,
                    self.path.clone(),
                    format!("Unexepcted character: {other}"),
                ));
            }
        };

        Ok(Some(Token::new(token_type, (self.start, self.current))))
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        Some(self.source[self.current])
    }

    fn advance(&mut self) -> Option<char> {
        let Some(c) = self.peek() else {
            return None;
        };

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
}
