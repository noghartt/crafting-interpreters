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
    BinaryExpr -> left: Expr, operator: Token, right: Expr;
    UnaryExpr -> operator: Token, right: Expr;
    Grouping -> expression: Expr;
    Literal -> value: LiteralValue;
);

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryExpr {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.token),
            Self::UnaryExpr { operator, right } => write!(f, "({} {right})", operator.token),
            Expr::Grouping { expression } => write!(f, "(group {expression})"),
            Expr::Literal { value } => write!(f, "{value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::TokenType;

    #[test]
    fn test_ast_printer() {
        let expression = Expr::BinaryExpr {
            left: Box::new(Expr::UnaryExpr {
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(123.0),
                }),
                operator: Token::new(TokenType::Minus, (0, 0)),
            }),
            operator: Token::new(TokenType::Star, (0, 0)),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Number(45.67),
                }),
            }),
        };

        assert_eq!(expression.to_string(), "(* (- 123) (group 45.67))");
    }
}
