#[derive(Debug)]
pub enum TokenKind {
    // Keywords
    Create,
    Table,
    Insert,
    Into,
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

#[derive(Debug)]
pub struct Token<'a> {
    /// The type of Token this is
    pub kind: TokenKind,

    /// The actual text this Token refers to (lexeme in compiler terms)
    pub lexeme: &'a str,

    /// The position in the broader input this Token starts at
    pub position: usize,
}
