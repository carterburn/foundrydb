use std::io::{self, stdin, BufRead, Write};

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
                process_command(buffer.trim());
            }
            Err(e) => {
                eprintln!("Error while reading input: {e:?}");
                return Err(e);
            }
        }
    }
}

fn process_command(cmd: &str) {
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
