use std::io::{self, stdin, BufRead, Write};

use foundrydb::{lexer::Lexer, parser::Parser};

fn main() -> Result<(), io::Error> {
    let mut stdin = stdin().lock();
    let mut buffer = String::new();

    loop {
        print!("fdb > ");
        io::stdout().flush()?;

        buffer.clear();

        match stdin.read_line(&mut buffer) {
            Ok(0) => {
                // EOF; just return (no error)
                return Ok(());
            }
            Ok(_n) => {
                if buffer.trim().starts_with(".") {
                    process_dot_command(buffer.trim());
                } else {
                    process_command(buffer.trim());
                }
            }
            Err(e) => {
                eprintln!("Error while reading input: {e:?}");
                return Err(e);
            }
        }
    }
}

fn process_dot_command(cmd: &str) {
    let cmd = cmd.to_lowercase();
    match cmd.as_str() {
        ".exit" => {
            // Exit with success
            eprintln!("Goodbye!");
            std::process::exit(0);
        }
        _ => {
            println!("Unrecognized command: {cmd}");
        }
    }
}

fn process_command(input: &str) {
    let mut lexer = Lexer::new(input);

    let tokens = match lexer.lex() {
        Ok(t) => t,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    println!("Tokens processed: {tokens:?}");
    let parser = Parser::new(tokens);
    match parser.parse() {
        Ok(statements) => {
            // we don't support multiple statements with semicolons or files (yet), so we'll just
            // print out the first statement
            println!(
                "Parsed into AST; recreated SQL from the AST: {}",
                statements[0]
            );
        }
        Err(e) => {
            println!("{e}");
        }
    }
}
