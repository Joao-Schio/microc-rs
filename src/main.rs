#![forbid(unsafe_code)]

mod ast;
mod lexer;
mod parser;
mod scanner;
mod semantic;
#[cfg(test)]
mod tests;
mod token;

fn main() {
    println!("Hello, world!");
}
