use my_calculator::eval;
use std::{env, io};

fn handle_input(input: &str) {
    match eval(input) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            let err: anyhow::Error = err.into();
            eprintln!("{:#}", err);
        }
    }
}

fn main() {
    let mut args = env::args();
    if let Some(input) = args.nth(1) {
        handle_input(&input)
    } else {
        println!("Type in an expression and hit enter");
        let mut buffer = String::new();
        let stdin = io::stdin();
        loop {
            stdin.read_line(&mut buffer).expect("Stdin error");
            handle_input(&buffer);
            buffer.clear();
        }
    }
}
