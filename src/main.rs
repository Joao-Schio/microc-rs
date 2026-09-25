#![forbid(unsafe_code)]

mod ast;
mod lexer;
mod parser;
mod scanner;
mod token;
mod semantic;
#[cfg(test)]
mod tests;

fn main() {
    println!("Hello, world!");
}
