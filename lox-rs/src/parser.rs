mod expr;

pub use expr::{Expr, LiteralValue};

use crate::error::{Error, ErrorKind};
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;
        while self.match_tokens(&[TokenType::Comma]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let value = match &self.peek().token {
            TokenType::False => LiteralValue::Bool(false),
            TokenType::True => LiteralValue::Bool(true),
            TokenType::Nil => LiteralValue::Nil,
            TokenType::Number(n) => LiteralValue::Number(*n),
            TokenType::String(s) => LiteralValue::String(s.clone()),
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

                return Ok(Expr::Grouping {
                    expression: Box::new(expr),
                });
            }
            _ => return Err(self.error(self.peek(), "Expect expression.")),
        };

        self.advance();
        Ok(Expr::Literal { value })
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        types.iter().any(|token| {
            if self.check(token.clone()) {
                self.advance();
                return true;
            }

            false
        })
    }

    fn check(&self, r#type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token == r#type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == TokenType::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn consume(&mut self, r#type: TokenType, message: &str) -> Result<&Token, Error> {
        if self.check(r#type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), message))
    }

    fn error(&self, token: &Token, message: &str) -> Error {
        Error::new(ErrorKind::Parse, token.metadata.span, String::from(message))
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token == TokenType::Semicolon {
                return;
            }

            match self.peek().token {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => continue,
            }
        }

        self.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::Lexer;
    use crate::source::SourceMap;

    #[test]
    fn test_validate_parser_flow() {
        let source = String::from("6 / 3 - 1");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(- (/ 6 3) 1)")
    }

    #[test]
    fn test_validate_parser_comma_with_equal_equal_on_first_precedence() {
        let source = String::from("1 == 1, 2");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(, (== 1 1) 2)")
    }

    #[test]
    fn test_validate_parser_comma_with_equal_equal_on_second_precedence() {
        let source = String::from("1, 2 == 2");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(, 1 (== 2 2))")
    }

    #[test]
    fn test_validate_comma_with_parenthesis() {
        let source = String::from("(1, 2)");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(group (, 1 2))")
    }

    #[test]
    fn test_validate_parser_comma() {
        let source = String::from("1, 2, 3");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(, (, 1 2) 3)")
    }

    #[test]
    fn test_missing_paren_at_end() {
        assert_eq!(
            parse_error("(1 + 2"),
            "[line 1] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_missing_paren_at_unexpected_token() {
        assert_eq!(
            parse_error("(1 + 2 3"),
            "[line 1] Error (CLI) at '3': Expect ')' after expression."
        );
    }

    #[test]
    fn test_missing_paren_in_nested_group() {
        assert_eq!(
            parse_error("((1)"),
            "[line 1] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_missing_paren_on_later_line() {
        assert_eq!(
            parse_error("1 +\n(2\n"),
            "[line 3] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_missing_paren_after_non_ascii() {
        assert_eq!(
            parse_error("\"olá\" == (1 2"),
            "[line 1] Error (CLI) at '2': Expect ')' after expression."
        );
    }

    fn parse_source(source: &str) -> Result<Expr, Error> {
        let mut sources = SourceMap::default();
        let file = sources.add(None, source);
        let tokens = Lexer::new(file, &sources.get(file).chars).scan()?;
        Parser::new(tokens).parse()
    }

    fn parse_error(source: &str) -> String {
        let mut sources = SourceMap::default();
        let file = sources.add(None, source);
        let tokens = Lexer::new(file, &sources.get(file).chars).scan().unwrap();

        Parser::new(tokens).parse().unwrap_err().render(&sources)
    }
}
