use crate::lexer::Token;

#[derive(Debug)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
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
    UnaryExpr -> operator: Token;
    Grouping -> expression: Expr;
    Literal -> value: LiteralValue;
);
