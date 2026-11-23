mod environment;
mod value;

use std::{cell::RefCell, io::Write, rc::Rc};

use crate::{
    error::{Error, ErrorKind},
    interpreter::environment::Environment,
    lexer::{Token, TokenType},
    parser::{Expr, LiteralValue, Statement},
};

pub use value::Value;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    output: Box<dyn Write>,
}

impl Interpreter {
    pub fn new(output: Box<dyn Write>) -> Self {
        Self {
            environment: Rc::default(),
            output,
        }
    }

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

                self.environment.borrow_mut().define(name, value);
            }
            Statement::Print { expression } => {
                let value = self.eval(expression)?;
                writeln!(self.output, "{value}").expect("failed to write program output");
            }
            Statement::Block { statements } => {
                let environment = Environment::with_enclosing(Rc::clone(&self.environment));
                self.execute_block(statements, environment)?;
            }
            Statement::If {
                expression,
                then_branch,
                else_branch,
            } => {
                let condition = self.eval(expression)?;
                if condition.is_truthy() {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
            }
        }

        Ok(())
    }

    fn execute_block(
        &mut self,
        statements: Vec<Statement>,
        environment: Environment,
    ) -> Result<(), Error> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

        let result = statements
            .into_iter()
            .try_for_each(|stmt| self.execute(stmt));

        // Restore on both the success and the error path (the book's `finally`).
        self.environment = previous;

        result
    }

    fn eval(&mut self, expr: Expr) -> Result<Value, Error> {
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
            Expr::Variable { identifier } => self.environment.borrow().get(&identifier),
            Expr::Assign { identifier, value } => {
                let value = self.eval(*value)?;
                self.environment
                    .borrow_mut()
                    .assign(&identifier, value.clone())?;
                Ok(value)
            }
        }
    }

    fn eval_conditional(
        &mut self,
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

    fn eval_binary_expr(
        &mut self,
        left: Expr,
        operator: Token,
        right: Expr,
    ) -> Result<Value, Error> {
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

    fn eval_unary_expr(&mut self, operator: Token, right: Expr) -> Result<Value, Error> {
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
    use std::{
        cell::RefCell,
        io::{self, Write},
        rc::Rc,
    };

    use crate::error::ErrorKind;
    use crate::runtime::Runtime;

    #[derive(Clone, Default)]
    struct Output(Rc<RefCell<Vec<u8>>>);

    impl Write for Output {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Output {
        fn take(&self) -> String {
            String::from_utf8(self.0.take()).unwrap()
        }
    }

    struct Session {
        runtime: Runtime,
        output: Output,
    }

    impl Session {
        fn new() -> Self {
            let output = Output::default();
            Self {
                runtime: Runtime::new(Box::new(output.clone())),
                output,
            }
        }

        fn run(&mut self, source: &str) -> String {
            let result = self.runtime.run(None, source);
            let mut lines: Vec<String> = self.output.take().lines().map(String::from).collect();
            if let Err(errors) = result {
                lines.extend(
                    errors
                        .iter()
                        .map(|error| error.render(&self.runtime.sources)),
                );
            }

            lines.join("\n")
        }
    }

    fn run(source: &str) -> String {
        Session::new().run(source)
    }

    fn eval(expression: &str) -> String {
        run(&format!("print {expression};"))
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
    fn test_plus_concatenates_string_and_number() {
        assert_eq!(eval("\"a\" + 1"), "a1");
        assert_eq!(eval("1 + \"a\""), "1a");
    }

    #[test]
    fn test_plus_with_invalid_operands_is_runtime_error() {
        assert_eq!(
            eval("\"a\" + nil"),
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
        let mut runtime = Runtime::new(Box::new(io::sink()));
        let errors = match runtime.run(None, "1 + nil;") {
            Ok(()) => panic!("expected a runtime error"),
            Err(errors) => errors,
        };

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), ErrorKind::Runtime);
    }

    #[test]
    fn test_print_statements_run_in_order() {
        assert_eq!(run("print 1; print \"two\"; print nil;"), "1\ntwo\nnil");
    }

    #[test]
    fn test_expression_statement_prints_nothing() {
        assert_eq!(run("1 + 2;"), "");
    }

    #[test]
    fn test_var_without_initializer_is_nil() {
        assert_eq!(run("var a; print a;"), "nil");
    }

    #[test]
    fn test_redeclaring_a_global_is_allowed() {
        assert_eq!(run("var a = 1; var a = 2; print a;"), "2");
    }

    #[test]
    fn test_assignment_is_an_expression() {
        assert_eq!(
            run("var a; var b; print a = b = 3; print a; print b;"),
            "3\n3\n3"
        );
    }

    #[test]
    fn test_reading_undefined_variable_is_runtime_error() {
        assert_eq!(
            run("print x;"),
            "[line 1] Error (CLI) at 'x': Undefined variable 'x'."
        );
    }

    #[test]
    fn test_assigning_undefined_variable_is_runtime_error() {
        assert_eq!(
            run("x = 1;"),
            "[line 1] Error (CLI) at 'x': Undefined variable 'x'."
        );
    }

    #[test]
    fn test_runtime_error_stops_later_statements() {
        assert_eq!(
            run("print 1; print -\"x\"; print 2;"),
            "1\n[line 1] Error (CLI) at '-': Operand must be a number."
        );
    }

    #[test]
    fn test_nested_blocks_shadow_and_restore() {
        let source = "\
            var a = \"global a\"; var b = \"global b\"; var c = \"global c\";
            {
              var a = \"outer a\"; var b = \"outer b\";
              { var a = \"inner a\"; print a; print b; print c; }
              print a; print b; print c;
            }
            print a; print b; print c;";

        assert_eq!(
            run(source),
            "inner a\nouter b\nglobal c\nouter a\nouter b\nglobal c\nglobal a\nglobal b\nglobal c"
        );
    }

    #[test]
    fn test_assignment_in_block_updates_enclosing_variable() {
        assert_eq!(run("var a = 1; { a = 2; } print a;"), "2");
    }

    #[test]
    fn test_block_variable_is_gone_after_block() {
        assert_eq!(
            run("{ var x = 1; } print x;"),
            "[line 1] Error (CLI) at 'x': Undefined variable 'x'."
        );
    }

    #[test]
    fn test_globals_persist_across_runs() {
        let mut session = Session::new();
        session.run("var a = 1;");

        assert_eq!(session.run("print a;"), "1");
    }

    #[test]
    fn test_runtime_error_in_block_restores_enclosing_scope() {
        let mut session = Session::new();
        assert_eq!(
            session.run("var a = 1; { var a = 2; print -nil; }"),
            "[line 1] Error (CLI) at '-': Operand must be a number."
        );

        assert_eq!(session.run("print a;"), "1");
    }
}
