pub enum TokenKind {
    // Keywords
    Create,
    Insert,
    Select,
    Values,
    From,
    Where,

    // Operators
    Plus,
    Equal,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    LeftParen,
    RightParen,
    Comma,

    // Literals
    Identifier,
    String,
    Number,

    // EOF marker
    Eof,
}

pub struct Token<'a> {
    /// The type of Token this is
    kind: TokenKind,

    /// The actual text this Token refers to (lexeme in compiler terms)
    lexeme: &'a str,

    /// The position in the broader input this Token starts at
    position: usize,
}
