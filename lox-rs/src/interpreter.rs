mod environment;
mod value;

use crate::{
    error::{Error, ErrorKind},
    interpreter::environment::Environment,
    lexer::{Token, TokenType},
    parser::{Expr, LiteralValue, Statement},
};

pub use value::Value;

#[derive(Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), Error> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn execute(&mut self, stmt: Statement) -> Result<(), Error> {
        match stmt {
            Statement::Expression { expression } => {
                self.eval(expression)?;
            }
            Statement::Var {
                identifier,
                expression,
            } => {
                let value = match expression {
                    Some(expr) => self.eval(expr)?,
                    None => Value::Nil,
                };

                let TokenType::Identifier(name) = identifier.token else {
                    unreachable!("parser only declares identifier token");
                };

                self.environment.define(name, value);
            }
            Statement::Print { expression } => {
                let value = self.eval(expression)?;
                println!("{value}");
            }
        }

        Ok(())
    }

    fn eval(&self, expr: Expr) -> Result<Value, Error> {
        match expr {
            Expr::Grouping { expression } => self.eval(*expression),
            Expr::BinaryExpr {
                left,
                operator,
                right,
            } => self.eval_binary_expr(*left, operator, *right),
            Expr::Conditional {
                condition,
                then_branch,
                else_branch,
            } => self.eval_conditional(*condition, *then_branch, *else_branch),
            Expr::UnaryExpr { operator, right } => self.eval_unary_expr(operator, *right),
            Expr::Literal { value } => self.eval_literal(value),
            Expr::Variable { identifier } => self.environment.get(&identifier),
        }
    }

    fn eval_conditional(
        &self,
        condition: Expr,
        then_branch: Expr,
        else_branch: Expr,
    ) -> Result<Value, Error> {
        let condition = self.eval(condition)?;
        if condition.is_truthy() {
            self.eval(then_branch)
        } else {
            self.eval(else_branch)
        }
    }

    fn eval_binary_expr(&self, left: Expr, operator: Token, right: Expr) -> Result<Value, Error> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;

        match operator.token {
            TokenType::Plus => match (left, right) {
                (Value::Number(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
                (Value::String(a), Value::Number(b)) => Ok(Value::String(format!("{a}{b}"))),
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                _ => Err(Error::new(
                    ErrorKind::Runtime,
                    operator.metadata.span,
                    String::from("Operands must be two numbers or two strings."),
                )),
            },
            TokenType::Minus => numeric(&operator, left, right, |a, b| Value::Number(a - b)),
            TokenType::Slash => {
                if right == Value::Number(0.0) {
                    return Err(Error::new(
                        ErrorKind::Runtime,
                        operator.metadata.span,
                        String::from("You can't divide by 0."),
                    ));
                }

                numeric(&operator, left, right, |a, b| Value::Number(a / b))
            }
            TokenType::Star => numeric(&operator, left, right, |a, b| Value::Number(a * b)),
            TokenType::Greater => numeric(&operator, left, right, |a, b| Value::Bool(a > b)),
            TokenType::GreaterEqual => numeric(&operator, left, right, |a, b| Value::Bool(a >= b)),
            TokenType::Less => numeric(&operator, left, right, |a, b| Value::Bool(a < b)),
            TokenType::Comma => Ok(right),
            TokenType::LessEqual => numeric(&operator, left, right, |a, b| Value::Bool(a <= b)),
            TokenType::BangEqual => Ok(Value::Bool(!is_equal(left, right))),
            TokenType::EqualEqual => Ok(Value::Bool(is_equal(left, right))),
            _ => Err(Error::new(
                ErrorKind::Runtime,
                operator.metadata.span,
                String::from("Unreachable code."),
            )),
        }
    }

    fn eval_unary_expr(&self, operator: Token, right: Expr) -> Result<Value, Error> {
        let right = self.eval(right)?;
        match (&operator.token, right) {
            (TokenType::Bang, value) => Ok(Value::Bool(!value.is_truthy())),
            (TokenType::Minus, Value::Number(n)) => Ok(Value::Number(-n)),
            (TokenType::Minus, _) => Err(Error::new(
                ErrorKind::Runtime,
                operator.metadata.span,
                String::from("Operand must be a number."),
            )),
            _ => unreachable!("parser only builds unary expression with `-` or `!`"),
        }
    }

    fn eval_literal(&self, value: LiteralValue) -> Result<Value, Error> {
        let value = match value {
            LiteralValue::Nil => Value::Nil,
            LiteralValue::Number(n) => Value::Number(n),
            LiteralValue::String(s) => Value::String(s),
            LiteralValue::Bool(b) => Value::Bool(b),
        };

        Ok(value)
    }
}

fn is_equal(left: Value, right: Value) -> bool {
    left == right
}

fn numeric(
    operator: &Token,
    left: Value,
    right: Value,
    op: impl Fn(f64, f64) -> Value,
) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(op(a, b)),
        _ => Err(Error::new(
            ErrorKind::Runtime,
            operator.metadata.span,
            String::from("Operands must be numbers."),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorKind;
    use crate::runtime::Runtime;

    fn eval(source: &str) -> String {
        let mut runtime = Runtime::default();
        match runtime.run(None, source) {
            Ok(value) => value.to_string(),
            Err(errors) => errors
                .iter()
                .map(|error| error.render(&runtime.sources))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    #[test]
    fn test_arithmetic_follows_precedence() {
        assert_eq!(eval("(1 + 2) * 3 - 4 / 2"), "7");
        assert_eq!(eval("10 / 4"), "2.5");
    }

    #[test]
    fn test_whole_number_prints_without_decimal() {
        assert_eq!(eval("1.5 + 0.5"), "2");
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval("-(3)"), "-3");
        assert_eq!(eval("--3"), "3");
    }

    #[test]
    fn test_string_concatenation() {
        assert_eq!(eval("\"foo\" + \"bar\""), "foobar");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("1 < 2"), "true");
        assert_eq!(eval("2 <= 2"), "true");
        assert_eq!(eval("3 > 4"), "false");
        assert_eq!(eval("3 >= 4"), "false");
    }

    #[test]
    fn test_only_nil_and_false_are_falsey() {
        assert_eq!(eval("!nil"), "true");
        assert_eq!(eval("!false"), "true");
        assert_eq!(eval("!0"), "false");
        assert_eq!(eval("!\"\""), "false");
    }

    #[test]
    fn test_equality_across_types() {
        assert_eq!(eval("nil == nil"), "true");
        assert_eq!(eval("nil == false"), "false");
        assert_eq!(eval("1 == \"1\""), "false");
        assert_eq!(eval("\"a\" == \"a\""), "true");
        assert_eq!(eval("1 != 2"), "true");
    }

    #[test]
    fn test_comma_returns_right_operand() {
        assert_eq!(eval("1, 2"), "2");
        assert_eq!(eval("\"x\" + (1, \"y\")"), "xy");
    }

    #[test]
    fn test_comma_still_evaluates_left_operand() {
        assert_eq!(
            eval("-\"x\", 1"),
            "[line 1] Error (CLI) at '-': Operand must be a number."
        );
    }

    #[test]
    fn test_ternary_picks_branch_by_truthiness() {
        assert_eq!(eval("true ? 1 : 2"), "1");
        assert_eq!(eval("nil ? 1 : 2"), "2");
        assert_eq!(eval("0 ? \"y\" : \"n\""), "y");
    }

    #[test]
    fn test_ternary_only_evaluates_chosen_branch() {
        assert_eq!(eval("true ? 1 : -\"x\""), "1");
        assert_eq!(eval("false ? -\"x\" : 2"), "2");
    }

    #[test]
    fn test_plus_with_mixed_operands_is_runtime_error() {
        assert_eq!(
            eval("\"a\" + 1"),
            "[line 1] Error (CLI) at '+': Operands must be two numbers or two strings."
        );
    }

    #[test]
    fn test_numeric_operator_with_string_is_runtime_error() {
        assert_eq!(
            eval("1 - \"a\""),
            "[line 1] Error (CLI) at '-': Operands must be numbers."
        );
        assert_eq!(
            eval("3 > \"x\""),
            "[line 1] Error (CLI) at '>': Operands must be numbers."
        );
    }

    #[test]
    fn test_negating_string_is_runtime_error() {
        assert_eq!(
            eval("-\"x\""),
            "[line 1] Error (CLI) at '-': Operand must be a number."
        );
    }

    #[test]
    fn test_runtime_error_in_nested_operand_propagates() {
        assert_eq!(
            eval("(1 + 2) * (3 - \"x\")"),
            "[line 1] Error (CLI) at '-': Operands must be numbers."
        );
    }

    #[test]
    fn test_runtime_error_has_runtime_kind() {
        let mut runtime = Runtime::default();
        let errors = match runtime.run(None, "1 + nil;") {
            Ok(()) => panic!("expected a runtime error"),
            Err(errors) => errors,
        };

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), ErrorKind::Runtime);
    }
}
