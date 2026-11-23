mod expr;

pub use expr::{Expr, LiteralValue, Statement};

use crate::error::{Error, ErrorKind};
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Result<Vec<Statement>, Vec<Error>> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors)
        }
    }

    fn declaration(&mut self) -> Result<Statement, Error> {
        if self.match_tokens(&[TokenType::Var]) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Statement, Error> {
        let identifier = self.consume_identifier("Expect variable name.")?;
        let expression = if self.match_tokens(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Statement::Var {
            identifier,
            expression,
        })
    }

    fn statement(&mut self) -> Result<Statement, Error> {
        if self.match_tokens(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_tokens(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_tokens(&[TokenType::LeftBrace]) {
            return Ok(Statement::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Statement, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Statement::Print { expression: expr })
    }

    fn expression_statement(&mut self) -> Result<Statement, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Statement::Expression { expression: expr })
    }

    fn if_statement(&mut self) -> Result<Statement, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_tokens(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            expression: condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Statement>, Error> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        let mut expr = self.assignment()?;
        while self.match_tokens(&[TokenType::Comma]) {
            let operator = self.previous().clone();
            let right = self.assignment()?;
            expr = Expr::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;
        if self.match_tokens(&[TokenType::Question]) {
            let left = self.expression()?;
            self.consume(TokenType::Colon, "Missing `:` for ternary operator.")?;
            let right = self.ternary()?;
            expr = Expr::Conditional {
                condition: Box::new(expr),
                then_branch: Box::new(left),
                else_branch: Box::new(right),
            }
        };

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.ternary()?;
        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            if let Expr::Variable { identifier } = expr {
                return Ok(Expr::Assign {
                    identifier,
                    value: Box::new(value),
                });
            } else {
                return Err(self.error(&equals, "Invalid assignment target."));
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
            TokenType::EqualEqual | TokenType::BangEqual => {
                self.errors
                    .push(self.error(self.peek(), "Missing left-hand operand."));
                self.advance();
                return self.equality();
            }
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                self.errors
                    .push(self.error(self.peek(), "Missing left-hand operand."));
                self.advance();
                return self.comparison();
            }
            TokenType::Plus => {
                self.errors
                    .push(self.error(self.peek(), "Missing left-hand operand."));
                self.advance();
                return self.term();
            }
            TokenType::Star | TokenType::Slash => {
                self.errors
                    .push(self.error(self.peek(), "Missing left-hand operand."));
                self.advance();
                return self.factor();
            }
            TokenType::Comma => {
                self.errors
                    .push(self.error(self.peek(), "Missing left-hand operand."));
                self.advance();
                return self.expression();
            }
            TokenType::Question => {
                self.errors
                    .push(self.error(self.peek(), "Missing conditional for ternary operator."));
                self.advance();
                let _ = self.expression()?;
                self.consume(TokenType::Colon, "Missing `:` for ternary operator.")?;
                return self.ternary();
            }
            TokenType::Identifier(_) => {
                let identifier = self.advance().clone();
                return Ok(Expr::Variable { identifier });
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

    fn consume_identifier(&mut self, message: &str) -> Result<Token, Error> {
        if matches!(self.peek().token, TokenType::Identifier(_)) {
            return Ok(self.advance().clone());
        }

        Err(self.error(self.peek(), message))
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
                _ => {}
            }

            self.advance();
        }
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
    fn test_validate_ternary_nested_in_then_branch() {
        let source = String::from("1 ? 1 ? 2 : 2 : 3");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(?: 1 (?: 1 2 2) 3)")
    }

    #[test]
    fn test_validate_ternary_right_associative() {
        let source = String::from("1 ? 2 : 3 ? 4 : 5");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(?: 1 2 (?: 3 4 5))")
    }

    #[test]
    fn test_validate_ternary_binds_tighter_than_comma() {
        let source = String::from("1 ? 2 : 3, 4");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(, (?: 1 2 3) 4)")
    }

    #[test]
    fn test_validate_ternary_allows_comma_in_then_branch() {
        let source = String::from("1 ? 2, 3 : 4");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(?: 1 (, 2 3) 4)")
    }

    #[test]
    fn test_validate_ternary_looser_than_equality() {
        let source = String::from("1 == 2 ? 3 : 4");
        let ast = parse_source(&source).unwrap();

        assert_eq!(ast.to_string(), "(?: (== 1 2) 3 4)")
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

    #[test]
    fn test_missing_colon_in_ternary_at_end() {
        assert_eq!(
            parse_error("1 ? 2"),
            "[line 1] Error (CLI) at end: Missing `:` for ternary operator."
        );
    }

    #[test]
    fn test_missing_colon_in_ternary_at_unexpected_token() {
        assert_eq!(
            parse_error("1 ? 2 3"),
            "[line 1] Error (CLI) at '3': Missing `:` for ternary operator."
        );
    }

    #[test]
    fn test_missing_left_operand_for_equality() {
        assert_eq!(
            parse_error("== 1;"),
            "[line 1] Error (CLI) at '==': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_for_comparison() {
        assert_eq!(
            parse_error("> 1;"),
            "[line 1] Error (CLI) at '>': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_for_term() {
        assert_eq!(
            parse_error("+ 1;"),
            "[line 1] Error (CLI) at '+': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_for_factor() {
        assert_eq!(
            parse_error("* 1;"),
            "[line 1] Error (CLI) at '*': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_for_comma() {
        assert_eq!(
            parse_error(", 1;"),
            "[line 1] Error (CLI) at ',': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_condition_for_ternary() {
        assert_eq!(
            parse_error("? 1 : 2;"),
            "[line 1] Error (CLI) at '?': Missing conditional for ternary operator."
        );
    }

    #[test]
    fn test_missing_left_operand_discards_whole_right_operand() {
        assert_eq!(
            parse_error("+ 1 * 2 + 3;"),
            "[line 1] Error (CLI) at '+': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_keeps_parsing() {
        assert_eq!(
            parse_error("== 1, == 2;"),
            "[line 1] Error (CLI) at '==': Missing left-hand operand.\n\
             [line 1] Error (CLI) at '==': Missing left-hand operand."
        );
    }

    #[test]
    fn test_missing_left_operand_followed_by_hard_error() {
        assert_eq!(
            parse_error("== 1 + (2"),
            "[line 1] Error (CLI) at '==': Missing left-hand operand.\n\
             [line 1] Error (CLI) at end: Expect ')' after expression."
        );
    }

    #[test]
    fn test_unary_minus_is_not_an_error_production() {
        let ast = parse_source("-1").unwrap();

        assert_eq!(ast.to_string(), "(- 1)")
    }

    #[test]
    fn test_assignment_is_right_associative() {
        let ast = parse_source("a = b = 1").unwrap();

        assert_eq!(ast.to_string(), "(assign a (assign b 1))")
    }

    #[test]
    fn test_assignment_is_looser_than_ternary() {
        let ast = parse_source("a = 1 ? 2 : 3").unwrap();

        assert_eq!(ast.to_string(), "(assign a (?: 1 2 3))")
    }

    #[test]
    fn test_comma_is_looser_than_assignment() {
        let ast = parse_source("a = 1, b = 2").unwrap();

        assert_eq!(ast.to_string(), "(, (assign a 1) (assign b 2))")
    }

    #[test]
    fn test_invalid_assignment_target() {
        assert_eq!(
            parse_error("1 = 2;"),
            "[line 1] Error (CLI) at '=': Invalid assignment target."
        );
    }

    #[test]
    fn test_missing_semicolon_after_expression() {
        assert_eq!(
            parse_error("1 + 2"),
            "[line 1] Error (CLI) at end: Expect ';' after expression."
        );
    }

    #[test]
    fn test_missing_variable_name() {
        assert_eq!(
            parse_error("var = 1;"),
            "[line 1] Error (CLI) at '=': Expect variable name."
        );
    }

    #[test]
    fn test_unclosed_block() {
        assert_eq!(
            parse_error("{ print 1;"),
            "[line 1] Error (CLI) at end: Expect '}' after block."
        );
    }

    #[test]
    fn test_synchronize_reports_errors_in_later_statements() {
        assert_eq!(
            parse_error("var = 1; print 2 print 3;"),
            "[line 1] Error (CLI) at '=': Expect variable name.\n\
             [line 1] Error (CLI) at 'print': Expect ';' after value."
        );
    }

    fn parse_source(source: &str) -> Result<Expr, Vec<Error>> {
        let mut sources = SourceMap::default();
        let file = sources.add(None, &format!("{source};"));
        let tokens = Lexer::new(file, &sources.get(file).chars)
            .scan()
            .map_err(|error| vec![error])?;

        let mut statements = Parser::new(tokens).parse()?;
        assert_eq!(statements.len(), 1, "expected exactly one statement");
        match statements.remove(0) {
            Statement::Expression { expression } => Ok(expression),
            other => panic!("expected an expression statement, got {other:?}"),
        }
    }

    fn parse_error(source: &str) -> String {
        let mut sources = SourceMap::default();
        let file = sources.add(None, source);
        let tokens = Lexer::new(file, &sources.get(file).chars).scan().unwrap();

        Parser::new(tokens)
            .parse()
            .unwrap_err()
            .iter()
            .map(|error| error.render(&sources))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
