# My Calculator

A small calculator for simple calculations. I mainly did this because I can never remember how to use decimal numbers with bc...  
Also I wanted to try some handwritten lexing and parsing. The parser is an [Operator-precedence parser](https://en.wikipedia.org/wiki/Operator-precedence_parser) and the evaluation is done with arbitrary precision decimals, so `0.1 + 0.2 == 0.3`.

## Installing
Clone the repo and issue `cargo install --path .`.

## Usage
Either `mc "5 + 8 * 2"` or just `mc` for interactive mode.