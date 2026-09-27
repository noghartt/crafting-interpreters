use std::env;

mod error;
mod lexer;
mod runtime;

use crate::runtime::Runtime;

fn main() {
    let args: Vec<String> = env::args().collect();
    let args = &args[1..];

    if args.is_empty() {
        return run_prompt();
    }

    if args.len() > 1 {
        println!("Usage: lox-rs [script]");
        std::process::exit(64);
    }

    run_file(args.first().unwrap());
}

fn run_prompt() {
    use std::io::{self, Write};

    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        input.clear();

        // TODO: Improve how to manage path, see some way to approach it better.
        let runtime = Runtime::new("".to_string());
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nExiting...");
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed == "exit" {
                    println!("\nExiting...");
                }

                match runtime.run(trimmed.to_owned()) {
                    Ok(result) => println!("{result:?}"),
                    Err(e) => println!("Error: {e:?}"),
                }
            }
            Err(err) => {
                eprintln!("Error reading input: {err}");
                break;
            }
        }
    }
}

fn run_file(path: &String) {
    use std::fs;

    let content = fs::read_to_string(path).expect("Should read the right file path");

    let runtime = Runtime::new(path.to_owned());
    let _ = runtime.run(content);
}
