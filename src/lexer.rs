use std::fmt::Display;

use crate::token::{Token, TokenKind};

#[derive(Copy, Clone, Debug)]
pub struct LexerError<'a> {
    /// The original input
    input: &'a str,

    /// The position in the original input
    position: usize,

    /// The character that caused the error
    character: char,
}

impl<'a> Display for LexerError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let padding = " ".repeat(self.position);
        writeln!(f, "Syntax error: {}", self.input)?;
        writeln!(f, "              {}^", &padding)?;
        writeln!(f, "              {}|", &padding)?;
        writeln!(
            f,
            "              {} ------- Invalid character ({})",
            &padding,
            self.character
        )
    }
}

pub struct Lexer<'a> {
    /// The input string to lex
    input: &'a str,

    /// The current position in the input string we're processing
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, current: 0 }
    }

    /// Peeks at the next character to process but doesn't consume it
    fn peek(&self) -> Option<char> {
        self.input[self.current..].chars().next()
    }

    /// Advances the cursor and returns the character at the current position
    fn advance(&mut self) -> Option<char> {
        if let Some(next) = self.input[self.current..].chars().next() {
            // grabbed something from the input, advance current
            self.current += next.len_utf8();
            Some(next)
        } else {
            None
        }
    }

    /// Checks if we have exhausted the input
    fn finished(&self) -> bool {
        self.current >= self.input.len()
    }

    /// Provides a slice of the input from a given byte offset up to the current position
    /// (exclusive)
    fn input_slice(&self, from: usize) -> &'a str {
        &self.input[from..self.current]
    }

    /// A more comprehensive "alphabetic" check 
    #[inline(always)]
    fn is_alphabetic_with_underscore(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// A comprehensive "alphanumeric" check 
    #[inline(always)]
    fn is_alphanumeric_with_underscore(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Checks if a alphanumeric string is a SQL keyword 
    fn check_keyword(start: usize, lexeme: &'a str) -> Option<Token<'a>> {
        let normalized = lexeme.to_lowercase();
        let mut token = Token { kind: TokenKind::Create, lexeme, position: start};

        token.kind = match normalized.as_str() {
            "create" =>  TokenKind::Create,
            "table" => TokenKind::Table,
            "insert" => TokenKind::Insert,
            "into" => TokenKind::Into,
            "select" => TokenKind::Select,
            "values" => TokenKind::Values,
            "from" => TokenKind::From,
            "where" => TokenKind::Where,
            _ => {
                // anything else will be treated as an identifier
                return None;
            }
        };

        Some(token)
    }

    /// Lex a word after peeking that it is alphabetic. The first character of the word should have
    /// been 'peeked' but will be advanced on behalf of the caller.
    fn lex_word(&mut self) -> Token<'a> {
        let start = self.current;
        self.advance();
        while let Some(ch) = self.peek() && Self::is_alphanumeric_with_underscore(ch) {
            self.advance();
        }
        let lexeme = self.input_slice(start);
        // if the word is a keyword, a fully formed Token is provided otherwise we construct one as
        // an identifier
        Self::check_keyword(start, lexeme).unwrap_or(Token {
            kind: TokenKind::Identifier, lexeme, position: start,
        })
    }

    /// Lex a number. The first chracter should have been 'peeked' but will be advanced on behalf
    /// of the caller.
    fn lex_number(&mut self) -> Token<'a> {
        // lex a number until we can't 
        let start = self.current;
        self.advance();
        while let Some(ch) = self.peek() && ch.is_numeric() {
            self.advance();
        }
        let lexeme = self.input_slice(start);
        Token { kind: TokenKind::Number, lexeme, position: start }
    }

    /// Lex a string. The first quote should have been 'peeked' but will be advanced on behalf of
    /// the caller.
    fn lex_string(&mut self) -> Token<'a> {
        // lex a string until another '
        let start = self.current;
        self.advance();
        while let Some(ch) = self.peek() && ch != '\'' {
            self.advance();
        }
        // now we are at the ', so slice from one past the quote then advance 
        let lexeme = self.input_slice(start + 1);
        self.advance();
        Token { kind: TokenKind::String, lexeme, position: start + 1 }
    }

    /// Try to lex an operator with the 'peeked' character, 'c'. This could return an Error if an
    /// invalid character that isn't an operator is encountered.
    fn lex_operator(&mut self, c: char) -> Result<Token<'a>, LexerError<'a>> {
        Ok(match c {
            '+' => {
                let start = self.current;
                self.advance();
                Token {kind: TokenKind::Plus, lexeme: self.input_slice(start), position: start }
            },
            '=' => {
                let start = self.current;
                self.advance();
                Token {kind: TokenKind::Equal, lexeme: self.input_slice(start), position: start }
            },
            '<' => {
                let start = self.current;
                self.advance();
                let kind = if let Some(ch) = self.peek() && ch == '=' {
                    self.advance();
                    TokenKind::LessThanEqual
                } else {
                    TokenKind::LessThan
                };
                Token {kind, lexeme: self.input_slice(start), position: start}
            },
            '>' => {
                let start = self.current;
                self.advance();
                let kind = if let Some(ch) = self.peek() && ch == '=' {
                    self.advance();
                    TokenKind::GreaterThanEqual
                } else {
                    TokenKind::GreaterThan
                };
                Token {kind, lexeme: self.input_slice(start), position: start}
            },
            '(' => {
                let start = self.current;
                self.advance();
                Token {kind: TokenKind::LeftParen, lexeme: self.input_slice(start), position: start }
            },
            ')' => {
                let start = self.current;
                self.advance();
                Token {kind: TokenKind::RightParen, lexeme: self.input_slice(start), position: start }
            },
            ',' => {
                let start = self.current;
                self.advance();
                Token {kind: TokenKind::Comma, lexeme: self.input_slice(start), position: start }
            },
            _ => {
                // anything else, we have hit an error at this spot 
                return Err(LexerError { input: self.input, position: self.current, character: c });
            }
        })
    }

    /// The brains of the lexer to actually lex out a sequence of Token's
    pub fn lex(&mut self) -> Result<Vec<Token<'a>>, LexerError<'a>> {
        let mut tokens = Vec::new();

        while !self.finished() {
            // eat whitespace 
            while let Some(c) = self.peek() && c.is_whitespace() {
                self.advance();
            };

            // peek at the next character to choose our path 
            let Some(c) = self.peek() else {
                // break if there is somehow no more
                break;
            };

            if Self::is_alphabetic_with_underscore(c) {
                tokens.push(self.lex_word());
            } else if c.is_numeric() {
                tokens.push(self.lex_number());
            } else if c == '\'' {
                tokens.push(self.lex_string());
            } else {
                // anything else is either an operator or an invalid character 
                tokens.push(self.lex_operator(c)?);
            }
        }

        // push final Eof token 
        tokens.push(Token { kind: TokenKind::Eof, lexeme: "", position: self.input.len() });

        Ok(tokens)
    }
}
