# MicroC-RS

<p align="center">
  <img src="https://www.rust-lang.org/logos/rust-logo-512x512.png" alt="Rust logo" width="120" />
</p>

[![CI](https://github.com/Joao-Schio/Lexer/actions/workflows/ci.yml/badge.svg)](https://github.com/Joao-Schio/Lexer/actions/workflows/ci.yml)
[![Coverage](.github/badges/coverage.svg)](https://github.com/Joao-Schio/microc-rs/actions/workflows/coverage.yml)

A from-scratch implementation of the **Micro C** compiler written in Rust.

This project started as an implementation of the lexical analyzer described in the Micro C teaching material, but has since grown into a personal compiler-learning project.

The goal is not to mechanically translate the reference C implementation into Rust. Instead, MicroC-RS aims to preserve the language semantics while designing the compiler around Rust's type system, ownership model, and modern software-engineering practices.

> **Status:** Work in progress. The project is currently focused on lexical analysis.

## Goals

MicroC-RS is primarily a learning project, but it is intentionally developed as a professional codebase.

The main engineering goals are:

- Separation of Concerns
- DRY where an abstraction genuinely improves the design
- Ownership and RAII for resource management
- Dependency Injection where it provides useful boundaries
- Test-Driven Development
- Strongly typed communication between compiler phases
- Minimal unnecessary abstraction

The project deliberately avoids reproducing implementation limitations from the teaching compiler when Rust provides a cleaner alternative.

## Language Scope

The first version aims to implement the original Micro C language, including:

- integers
- characters
- arrays
- functions
- recursion
- `if` / `else`
- `for`
- `return`
- `print`
- arithmetic operators
- relational operators
- logical operators
- identifiers
- integer literals
- character literals
- string literals

Features such as `bool`, floating-point values, `scan`, and additional target architectures are outside the V1 scope.

### Source encoding

MicroC-RS V1 treats source code as **ASCII**.

The lexer therefore works primarily with `u8`. Unicode identifiers and Unicode language semantics are intentionally outside the scope of V1.

### Negative numbers

`-` is always tokenized as the `MINUS` operator.

Negative values are represented syntactically through unary negation rather than being treated as negative numeric literals by the lexer.

Therefore:

```text
a-123
```

and:

```text
a - 123
```

must produce equivalent token sequences.

Whitespace must not affect tokenization.

## Architecture

The compiler is being developed as a sequence of explicit phases:

```text
Source
  |
  v
Scanner
  |
  v
Lexer
  |
  v
Vec<Token>
  |
  v
Parser
  |
  v
AST
  |
  v
Semantic Analysis
  |
  v
LLVM IR Generation
  |
  v
LLVM Toolchain
  |
  v
Native Executable
```

Compiler phases communicate through typed in-memory structures rather than intermediate files.

Representations such as token dumps, AST output, semantic information, and generated LLVM IR may be exposed through the CLI for debugging and inspection, but are not used as file-based communication mechanisms between compiler stages.

## Current Structure

```text
src/
├── lexer.rs
├── main.rs
├── scanner.rs
└── token.rs
```

### Scanner

The scanner owns the underlying input reader and provides byte-level source traversal.

It currently tracks:

- the next byte
- line number
- column number

The scanner supports non-consuming lookahead, allowing the lexer to recognize multi-character operators without moving through the source prematurely.

### Lexer

The lexer consumes bytes from a scanner and produces typed `Token` values.

The scanner is injected into the lexer through a generic type:

```rust
pub struct Lexer<S: TScanner> {
    scanner: S,
}
```

This keeps the lexer independent from a particular input source and makes the scanner/lexer boundary straightforward to test.

Lexical errors currently produce an `Undef` token. V1 follows a fail-fast approach, so higher-level tokenization can stop when the first invalid token is encountered.

### Tokens

Tokens contain their token type, source line, and original lexeme.

`TokenType` is represented as a Rust enum, allowing literal tokens to carry typed values where appropriate.

## Testing

TDD is a first-class part of the project.

The testing strategy favors:

- **contract tests** at architectural boundaries such as scanners and lexers
- focused unit tests for individual compiler transformations
- a smaller number of end-to-end compiler tests as later phases are implemented

For example, lexer tests verify not only that:

```text
&&
```

produces an `And` token, but also that:

```text
&&+
```

produces:

```text
And("&&")
Plus("+")
```

This ensures that lookahead recognizes **and consumes exactly the characters belonging to the token**.

Run the test suite with:

```bash
cargo test
```

Line coverage is measured in CI with `cargo-llvm-cov`, and the coverage badge at the top of this README is updated from the `main` branch.

To generate a coverage report locally:

```bash
cargo llvm-cov --workspace --all-features
```

## Building

MicroC-RS uses Rust edition 2024.

Clone the repository:

```bash
git clone https://github.com/Joao-Schio/Lexer.git
cd Lexer
```

Build it with:

```bash
cargo build
```

Run tests with:

```bash
cargo test
```

Check formatting with:

```bash
cargo fmt --all -- --check
```

The repository's CI runs both formatting checks and the test suite.

> The compiler executable itself is not functional yet. `main.rs` is currently only a placeholder while the compiler components are developed.

## Backend

MicroC-RS V1 will generate **LLVM IR**.

The compiler frontend remains responsible for:

- lexical analysis
- parsing
- AST construction
- semantic analysis
- type checking
- control-flow representation
- lowering Micro C semantics into LLVM IR

LLVM is then responsible for lowering the generated IR into native machine code.

The initial execution target is Linux x86-64, although using LLVM IR keeps the compiler frontend substantially less coupled to a specific machine architecture.

The compiler itself is intended to remain runnable on macOS/Apple Silicon while producing LLVM IR that can be compiled for the intended target.

### Future handwritten backend

A handwritten x86-64 backend is deliberately outside the V1 scope.

A future V2 may add an additional backend targeting:

- Linux x86-64 / AMD64
- System V ABI
- GNU/AT&T assembly syntax

That backend would exist alongside the LLVM backend rather than replacing it.

## Roadmap

### V1 — LLVM compiler

- [x] Scanner foundation
- [ ] Complete lexical analyzer
- [ ] Parser
- [ ] Abstract Syntax Tree
- [ ] Semantic analysis
- [ ] LLVM IR generation
- [ ] Compiler CLI
- [ ] End-to-end Micro C programs

### V2 — Native backend

- [ ] Handwritten x86-64 backend
- [ ] System V ABI lowering
- [ ] Stack-frame layout
- [ ] Register and temporary-value strategy
- [ ] GNU/AT&T assembly generation

## Design Philosophy

A recurring rule for this project is:

> Abstract when the code demonstrates a need for an abstraction, not merely because one could exist.

The project uses traits, generic types, and helper abstractions where they establish useful architectural boundaries or remove meaningful duplication.

At the same time, abstractions created solely for hypothetical future requirements are avoided.

The intention is to let the compiler architecture emerge alongside the language implementation and its tests.

## Acknowledgements

MicroC-RS is an independent Rust implementation of the **Micro C** language described in *Micro C — Aprendendo Compiladores*, by **Pedro Henrique** and **Brivaldo Junior**.

The teaching material is licensed under the [Creative Commons Attribution-NonCommercial 3.0 Unported License](https://creativecommons.org/licenses/by-nc/3.0/). The original Micro C compiler is available in the [WorksFacom/mcc](https://github.com/WorksFacom/mcc) repository, whose `LICENSE` file currently distributes the implementation under the [GNU Affero General Public License v3.0](https://www.gnu.org/licenses/agpl-3.0.html).

MicroC-RS is not a mechanical translation of the reference C implementation. The project does not copy or mechanically translate the original compiler source code; its Rust implementation is developed independently from the language specification, semantics, and compiler concepts presented in the teaching material.