mod expr;

pub use expr::{Expr, LiteralValue};

use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            }
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::UnaryExpr {
                operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        let value = match &self.peek().token {
            TokenType::False => LiteralValue::Bool(false),
            TokenType::True => LiteralValue::Bool(true),
            TokenType::Nil => LiteralValue::Nil,
            TokenType::Number(n) => LiteralValue::Number(*n),
            TokenType::String(s) => LiteralValue::String(s.clone()),
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Expect ')' after expression.");

                return Expr::Grouping {
                    expression: Box::new(expr),
                };
            }
            _ => panic!("Expect expression."),
        };

        self.advance();
        Expr::Literal { value }
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

    fn consume(&mut self, r#type: TokenType, message: &str) -> &Token {
        if self.check(r#type) {
            return self.advance();
        }

        panic!("{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::Lexer;

    #[test]
    fn test_validate_parser_flow() {
        let source = String::from("6 / 3 - 1");
        let mut lexer = Lexer::new("test.rs".into(), source);
        let tokens = lexer.scan().unwrap();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        assert_eq!(ast.to_string(), "(- (/ 6 3) 1)")
    }

    #[test]
    fn test_foo() {
        let source = String::from("var foo = \"olá mundo\"");
        let mut lexer = Lexer::new("test.rs".into(), source);
        let tokens = lexer.scan().unwrap();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        println!("{ast}")
    }
}
