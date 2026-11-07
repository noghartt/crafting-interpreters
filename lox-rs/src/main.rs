use std::env;

mod error;
mod lexer;
mod parser;
mod runtime;
mod source;

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

    // One session for the whole REPL, so later chapters can keep state
    // (e.g. global variables) across lines.
    let mut runtime = Runtime::default();
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        input.clear();

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

                match runtime.run(None, trimmed) {
                    Ok(result) => println!("{result}"),
                    Err(errors) => {
                        for error in &errors {
                            eprintln!("{}", error.render(&runtime.sources))
                        }
                    }
                };
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

    let mut runtime = Runtime::default();
    match runtime.run(Some(path.to_owned()), &content) {
        Ok(result) => println!("{result}"),
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error.render(&runtime.sources))
            }
            std::process::exit(65);
        }
    };
}
