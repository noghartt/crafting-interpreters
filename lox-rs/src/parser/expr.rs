use std::fmt;

use crate::lexer::Token;

#[derive(Debug)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Bool(b) => write!(f, "{b}"),
            LiteralValue::String(s) => write!(f, "{s}"),
            LiteralValue::Number(n) => write!(f, "{n}"),
            LiteralValue::Nil => write!(f, "nil"),
        }
    }
}

macro_rules! syntax {
    (@field_ty Expr) => { Box<Expr> };
    (@field_ty $ty:ident) => { $ty };

    ($ ( $variant:ident -> $( $field:ident : $ty:ident ),* ; )*) => {
        #[derive(Debug)]
        pub enum Expr {
            $( $variant { $( $field: syntax!(@field_ty $ty) ),* } ),*
        }
    }
}

syntax!(
    Conditional -> condition: Expr, then_branch: Expr, else_branch: Expr;
    BinaryExpr -> left: Expr, operator: Token, right: Expr;
    UnaryExpr -> operator: Token, right: Expr;
    Grouping -> expression: Expr;
    Literal -> value: LiteralValue;
    Variable -> identifier: Token;
);

#[derive(Debug)]
pub enum Statement {
    Var {
        identifier: Token,
        expression: Option<Expr>,
    },
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryExpr {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.token),
            Self::Conditional {
                condition,
                then_branch,
                else_branch,
            } => write!(f, "(?: {condition} {then_branch} {else_branch})"),
            Self::UnaryExpr { operator, right } => write!(f, "({} {right})", operator.token),
            Self::Grouping { expression } => write!(f, "(group {expression})"),
            Self::Literal { value } => write!(f, "{value}"),
            Self::Variable { identifier } => write!(f, "{}", identifier.token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::TokenType;
    use crate::source::{SourceMap, Span};

    #[test]
    fn test_ast_printer() {
        let file = SourceMap::default().add(None, "");
        let span = Span {
            file,
            start: 0,
            end: 0,
        };

        let expression = Expr::BinaryExpr {
            left: Box::new(Expr::UnaryExpr {
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(123.0),
                }),
                operator: Token::new(TokenType::Minus, span),
            }),
            operator: Token::new(TokenType::Star, span),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Number(45.67),
                }),
            }),
        };

        assert_eq!(expression.to_string(), "(* (- 123) (group 45.67))");
    }
}
