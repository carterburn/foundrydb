use crate::token::{Token, TokenKind};

/// Errors when parsing. This type will hold the actual Token that caused the error as well as a
/// helpful and contextual message.
pub struct ParseError<'a> {
    token: Token<'a>,

    msg: String,
}

impl std::fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // the token holds where in the input we have, we'll give an error showing the Token's
        // lexeme like "Parsing error near 'token.lexeme': msg"
        write!(
            f,
            "Parsing error near '{}' ({}): {}",
            self.token.lexeme, self.token.position, self.msg
        )
    }
}

/// The top level Statement AST node. All nodes derive from here.
pub enum Statement<'a> {
    Create {
        table: &'a str,
        columns: Vec<ColumnDef<'a>>,
    },
    Insert {
        table: &'a str,
        columns: Option<Vec<&'a str>>,
        values: Vec<Literal<'a>>,
    },
    Select {
        table: &'a str,
        columns: Vec<&'a str>,
        where_clause: Option<Comparison<'a>>,
    },
}

fn write_comma_separated<T: std::fmt::Display>(
    f: &mut std::fmt::Formatter,
    items: &[T],
) -> std::fmt::Result {
    for (idx, item) in items.iter().enumerate() {
        write!(f, "{item}")?;
        if idx != items.len() - 1 {
            write!(f, ", ")?;
        }
    }
    Ok(())
}

impl std::fmt::Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            Create { table, columns } => {
                write!(f, "CREATE TABLE {table} (")?;
                write_comma_separated(f, columns)?;
                write!(f, ")")
            }
            Insert {
                table,
                columns,
                values,
            } => {
                write!(f, "INSERT INTO {table} ")?;
                if let Some(cols) = columns {
                    write!(f, "(")?;
                    write_comma_separated(f, cols)?;
                    write!(f, ") ")?;
                }
                write!(f, "VALUES (")?;
                write_comma_separated(f, values)?;
                write!(f, ")")
            }
            Select {
                table,
                columns,
                where_clause,
            } => {
                write!(f, "SELECT ")?;
                write_comma_separated(f, columns)?;
                if let Some(clause) = where_clause {
                    write!(f, " FROM {table} WHERE {clause}")
                } else {
                    write!(f, " FROM {table}")
                }
            }
        }
    }
}

/// A column definition with name and type as used in CREATE TABLE statements.
pub struct ColumnDef<'a> {
    pub name: &'a str,
    pub data_type: DataType,
}

impl std::fmt::Display for ColumnDef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)
    }
}

/// The data types supported in CREATE TABLE statements.
pub enum DataType {
    Int,
    Text,
    VarChar(usize),
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DataType::*;
        match self {
            Int => write!(f, "INT"),
            Text => write!(f, "TEXT"),
            VarChar(size) => write!(f, "VARCHAR({size})"),
        }
    }
}

/// Literal values found in INSERT INTO statements and WHERE clauses.
pub enum Literal<'a> {
    Number(i64),
    String(&'a str),
}

impl std::fmt::Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;
        match self {
            Number(n) => write!(f, "{n}"),
            String(s) => write!(f, "'{s}'"),
        }
    }
}

/// As opposed to full Expressions, we start with simple comparison operations for our where
/// clauses. Future expansions would likely make this an Expression type that would be recursive.
pub struct Comparison<'a> {
    pub column: &'a str,
    pub op: ComparisonOp,
    pub value: Literal<'a>,
}

impl std::fmt::Display for Comparison<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.column, self.op, self.value)
    }
}

/// Comparison operators.
pub enum ComparisonOp {
    Eq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl std::fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ComparisonOp::*;
        write!(
            f,
            "{}",
            match self {
                Eq => "=",
                Lt => "<",
                LtEq => "<=",
                Gt => ">",
                GtEq => ">=",
            }
        )
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,

    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse the AST.
    pub fn parse(mut self) -> Result<Vec<Statement<'a>>, ParseError<'a>> {
        let mut statements = Vec::new();
        while self.peek().kind != TokenKind::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    /// Peek at the next token. We don't return Option<T> because we know that we will stop
    /// parsing at the Eof token.
    fn peek(&self) -> Token<'a> {
        self.tokens[self.current]
    }

    /// Advance the cursor, returning a reference to the Token<'a> at the current position.
    fn advance(&mut self) -> Token<'a> {
        let tok = self.peek();
        self.current += 1;
        tok
    }

    /// Consume the next token if it matches the provided TokenKind and return the Token
    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>, ParseError<'a>> {
        let next = self.peek();
        if next.kind == kind {
            Ok(self.advance())
        } else {
            Err(ParseError {
                token: next,
                msg: format!("Expected {:?}; got {:?}", kind, next.kind),
            })
        }
    }

    /// Consume the next token if it matches and return whether or not the Token matched but don't
    /// return the Token itself.
    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Parses a literal (Number or String) and returns the token.
    fn parse_literal(&mut self) -> Result<Literal<'a>, ParseError<'a>> {
        let val_tok = self.peek();
        let literal = match val_tok.kind {
            TokenKind::String => Literal::String(val_tok.lexeme),
            TokenKind::Number => {
                Literal::Number(val_tok.lexeme.parse().map_err(|e| ParseError {
                    token: val_tok,
                    msg: format!("Error parsing number: {e}"),
                })?)
            }
            _ => {
                return Err(ParseError {
                    token: val_tok,
                    msg: format!(
                        "Expected a literal (number or string); got {:?}",
                        val_tok.kind
                    ),
                });
            }
        };
        // successfully parsed a literal so advance
        self.advance();
        Ok(literal)
    }

    /// Parses a Data Type. The peek()'d Token will be an Identifier first and then transformed to
    /// a DataType. This method, if successful, advances past the Token's that make up the
    /// DataType.
    fn parse_datatype(&mut self) -> Result<DataType, ParseError<'a>> {
        let data_tok = self.peek();
        if data_tok.kind != TokenKind::Identifier {
            return Err(ParseError {
                token: data_tok,
                msg: format!("Expected a data type identifier; got {:?}", data_tok.kind),
            });
        }
        // attempt to parse the lexeme as either 'int', 'text', or 'varchar'
        let normalized = data_tok.lexeme.to_lowercase();
        match normalized.as_str() {
            "int" => {
                self.advance();
                Ok(DataType::Int)
            }
            "text" => {
                self.advance();
                Ok(DataType::Text)
            }
            "varchar" => {
                // consume tokens to get through the varchar(NUM)
                self.advance(); // advance past the data type
                self.expect(TokenKind::LeftParen)?; // left paren needs to be there
                let size_tok = self.expect(TokenKind::Number)?; // need a number

                let data_type =
                    DataType::VarChar(size_tok.lexeme.parse().map_err(|e| ParseError {
                        token: size_tok,
                        msg: format!("Error parsing varchar size: {e}"),
                    })?);
                self.expect(TokenKind::RightParen)?; // final right paren
                Ok(data_type)
            }
            _ => {
                // if it's not, we've exhausted datatypes, so this is an error
                Err(ParseError {
                    token: data_tok,
                    msg: format!("Unknown data type: {}", data_tok.lexeme),
                })
            }
        }
    }

    /// Parse a comma separated list with the provided smaller parser. The closure should be able
    /// to parse the "item" that the comma separated list represents. This method just handles the
    /// parsing of the comma.
    fn parse_comma_separated<T>(
        &mut self,
        mut parse_item: impl FnMut(&mut Self) -> Result<T, ParseError<'a>>,
    ) -> Result<Vec<T>, ParseError<'a>> {
        let mut items = Vec::new();
        loop {
            items.push(parse_item(self)?);
            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    /// Parse a top-level statement.
    fn parse_statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        // peek at the next token and match on the kind
        let nxt = self.peek();
        match nxt.kind {
            TokenKind::Select => self.parse_select(),
            TokenKind::Insert => self.parse_insert(),
            TokenKind::Create => self.parse_create(),
            _ => Err(ParseError {
                token: nxt,
                msg: format!(
                    "Expected the start of a statement (SELECT, CREATE, INSERT), found {:?}",
                    nxt.kind
                ),
            }),
        }
    }

    /// Parse a SELECT statement.
    /// Ex: SELECT column[, column ..] FROM table [WHERE column =|<|<=|>|>= String|Number]
    fn parse_select(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        // advance past the SELECT keyword (it should be SELECT but error if somehow its not)
        self.expect(TokenKind::Select)?;
        // consume column names (identifiers) until we have no more
        let columns =
            self.parse_comma_separated(|p| Ok(p.expect(TokenKind::Identifier)?.lexeme))?;

        // get the FROM keyword and table name
        self.expect(TokenKind::From)?;
        // now we expect an IDENTIFIER. NOTE: If a user has a table named 'table' (or any other
        // keyword we defined in the lexer) this will fail. We don't support quoted identifiers
        // yet (or likely ever?).
        let table_token = self.expect(TokenKind::Identifier)?;

        // now check if we have a WHERE clause
        let where_clause = if self.match_kind(TokenKind::Where) {
            // we only support one where clause and it is just a Comparison
            // we get a column name, operation, and a literal (number or string)
            let column = self.expect(TokenKind::Identifier)?;
            let op_tok = self.peek();
            let op = match op_tok.kind {
                TokenKind::Equal => ComparisonOp::Eq,
                TokenKind::LessThan => ComparisonOp::Lt,
                TokenKind::LessThanEqual => ComparisonOp::LtEq,
                TokenKind::GreaterThan => ComparisonOp::Gt,
                TokenKind::GreaterThanEqual => ComparisonOp::GtEq,
                _ => {
                    return Err(ParseError {
                        token: op_tok,
                        msg: format!("Expected a comparison operator; got {:?}", op_tok.kind),
                    });
                }
            };
            // successfully parsed a comparison so advance
            self.advance();

            // next is either a String or Number (literal)
            let literal = self.parse_literal()?;

            Some(Comparison {
                column: column.lexeme,
                op,
                value: literal,
            })
        } else {
            None
        };

        Ok(Statement::Select {
            table: table_token.lexeme,
            columns,
            where_clause,
        })
    }

    /// Parse an INSERT statement.
    /// Ex: INSERT INTO table VALUES (Literal [, Literal ..])
    fn parse_insert(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        // parse the INSERT INTO
        self.expect(TokenKind::Insert)?;
        self.expect(TokenKind::Into)?;

        // get the table name
        let table_token = self.expect(TokenKind::Identifier)?;

        let columns = if self.peek().kind == TokenKind::LeftParen {
            // user chose to define column names so we parse it
            self.expect(TokenKind::LeftParen)?;
            let cols =
                self.parse_comma_separated(|p| Ok(p.expect(TokenKind::Identifier)?.lexeme))?;
            self.expect(TokenKind::RightParen)?;
            Some(cols)
        } else {
            None
        };

        self.expect(TokenKind::Values)?;
        self.expect(TokenKind::LeftParen)?;
        let values = self.parse_comma_separated(|p| p.parse_literal())?;

        self.expect(TokenKind::RightParen)?;

        Ok(Statement::Insert {
            table: table_token.lexeme,
            columns,
            values,
        })
    }

    /// Parse a CREATE TABLE statement.
    /// Ex: CREATE TABLE table (column_name data_type [, column_name data_type ... ])
    fn parse_create(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        // parse CREATE TABLE
        self.expect(TokenKind::Create)?;
        self.expect(TokenKind::Table)?;

        // get the table name
        let table_token = self.expect(TokenKind::Identifier)?;

        // parse the columns
        self.expect(TokenKind::LeftParen)?;
        let columns = self.parse_comma_separated(|p| {
            let col_tok = p.expect(TokenKind::Identifier)?;
            let data_type = p.parse_datatype()?;
            Ok(ColumnDef {
                name: col_tok.lexeme,
                data_type,
            })
        })?;
        self.expect(TokenKind::RightParen)?;

        Ok(Statement::Create {
            table: table_token.lexeme,
            columns,
        })
    }
}
