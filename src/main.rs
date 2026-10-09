#![forbid(unsafe_code)]
#![allow(dead_code)]

mod ast;
mod frontend_error;
mod lexer;
mod parser;
mod scanner;
mod semantic;
#[cfg(test)]
mod tests;
mod token;
mod frontend;

fn main() {
    println!("Hello, world!");
}
