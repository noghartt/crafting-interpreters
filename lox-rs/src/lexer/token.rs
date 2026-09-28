#[derive(Debug)]
pub struct TokenMetadata {
    /// Start and end position of a token.
    offset: (usize, usize),
}

#[derive(Debug)]
pub struct Token {
    pub token: TokenType,
    pub metadata: TokenMetadata,
}

impl Token {
    pub fn new(token: TokenType, offset: (usize, usize)) -> Self {
        Token {
            token,
            metadata: TokenMetadata { offset },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(usize),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
