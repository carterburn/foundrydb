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
    fn input_slice(&self, from: usize) -> &str {
        &self.input[from..self.current]
    }
}
