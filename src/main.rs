use clap::Clap;
use my_calculator::eval;
use std::{env, io};

/// A small calculator intended as a bc alternative.
#[derive(Clap)]
struct Opts {
    /// Directly compute result of expression, if omitted, interactive mode is used
    expression: Option<String>,
    /// Print the parse tree for the provided expression
    #[clap(long)]
    print_parse_tree: bool,
}

fn handle_input(input: &str, opts: &Opts) {
    match eval(input, opts.print_parse_tree) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            let err: anyhow::Error = err.into();
            eprintln!("{:#}", err);
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut args = env::args();
    if let Some(input) = &opts.expression {
        handle_input(&input, &opts)
    } else {
        println!("Type in an expression and hit enter");
        let mut buffer = String::new();
        let stdin = io::stdin();
        loop {
            stdin.read_line(&mut buffer).expect("Stdin error");
            handle_input(&buffer, &opts);
            buffer.clear();
        }
    }
}
