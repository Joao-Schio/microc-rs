# MicroC-RS

<p align="center">
  <img src="https://www.rust-lang.org/logos/rust-logo-512x512.png" alt="Rust logo" width="120" />
</p>

[![CI](https://github.com/Joao-Schio/microc-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Joao-Schio/microc-rs/actions/workflows/ci.yml)
[![Coverage](https://raw.githubusercontent.com/Joao-Schio/microc-rs/coverage-badge/.github/badges/coverage.svg)](https://github.com/Joao-Schio/microc-rs/actions/workflows/coverage.yml)

A from-scratch implementation of the **Micro C** compiler written in Rust.

This project started as an implementation of the lexical analyzer described in the Micro C teaching material, but has since grown into a personal compiler-learning project.

The goal is not to mechanically translate the reference C implementation into Rust. Instead, MicroC-RS aims to preserve the language semantics while designing the compiler around Rust's type system, ownership model, and modern software-engineering practices.

> **Status:** Work in progress. Scanner and lexer support are in place, and development is currently focused on the parser and typed AST. Expression parsing, function calls, assignments, and `return` statements are implemented.

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

## Project Constraints

MicroC-RS is intentionally developed under a small set of restrictions intended to keep the project focused on learning and implementing the compiler itself.

### Safe Rust only

All MicroC-RS code must be written in **safe Rust**.

The project does not permit `unsafe` blocks, `unsafe fn`, or other first-party unsafe Rust. If an implementation appears to require `unsafe`, the preferred solution is to redesign it around Rust's safe ownership and type system rather than bypassing those guarantees.

This restriction applies to MicroC-RS itself; third-party dependencies are not required to be internally free of `unsafe`.

### Compiler phases are implemented by MicroC-RS

Core compiler functionality must not be delegated to compiler generators, compiler frameworks, or libraries that implement compiler phases on behalf of the project.

This includes tools and libraries such as:

- Flex, Lex, or equivalent lexer generators
- parser generators
- Rust equivalents of lexer/parser generators
- libraries that provide a ready-made compiler frontend
- libraries that perform semantic analysis or AST construction for MicroC-RS

The scanner, lexer, parser, AST construction, semantic analysis, and lowering to LLVM IR are implemented directly by MicroC-RS.

External libraries are allowed when they support peripheral concerns rather than replacing compiler implementation work. For example, a crate used for colored diagnostics, command-line parsing, testing, or similar infrastructure is acceptable.

LLVM is the intentional boundary for V1: MicroC-RS generates textual LLVM IR itself and may then invoke the LLVM toolchain to perform native code generation.

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
Typed AST
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

Representations such as token dumps, AST output, semantic information, and generated LLVM IR may eventually be exposed through the CLI for debugging and inspection, but they are not used as file-based communication mechanisms between compiler stages.

The initial execution target remains **Linux x86-64**, while the compiler itself is intended to remain runnable on macOS/Apple Silicon. Using LLVM IR keeps the frontend independent from the details of register allocation, calling-convention lowering, stack management, and final machine-code generation.

V1 will favor generating **textual LLVM IR** and delegating native lowering to the LLVM toolchain rather than immediately introducing a Rust LLVM binding. A handwritten x86-64 backend remains a useful future learning milestone, but it is deliberately deferred until the frontend and LLVM-backed compiler are complete.

## Current Structure

```text
src/
├── ast/
│   ├── expression.rs
│   ├── mod.rs
│   └── statement.rs
├── parser/
│   ├── expression.rs
│   └── mod.rs
├── tests/
│   ├── parser/
│   │   ├── error.rs
│   │   ├── expression.rs
│   │   ├── mod.rs
│   │   └── statement.rs
│   ├── comments.rs
│   ├── helpers.rs
│   ├── lexer.rs
│   ├── numeric.rs
│   ├── reserved_words.rs
│   └── tokenize.rs
├── lexer.rs
├── main.rs
├── scanner.rs
└── token.rs
```

### Scanner

The scanner owns the underlying input reader and provides byte-level source traversal.

It tracks source position and supports non-consuming lookahead, allowing the lexer to recognize multi-character operators without consuming unrelated input.

### Lexer

The lexer consumes bytes from a scanner and produces typed `Token` values.

The scanner is injected into the lexer through a generic type:

```rust
pub struct Lexer<S: TScanner> {
    scanner: S,
}
```

This keeps the lexer independent from a particular input source and makes the scanner/lexer boundary straightforward to test.

Lexical failures are represented as typed `LexerError` values rather than sentinel tokens. The compiler follows a fail-fast approach for V1.

### Tokens

Tokens contain their token type, source line, and original lexeme.

`TokenType` is represented as a Rust enum, allowing literal tokens to carry typed values where appropriate.

### AST

The AST is represented using typed Rust enums rather than the generic first-child / next-sibling representation used by the reference implementation.

The expression AST currently represents:

- integer and character literals
- identifiers
- unary operations
- binary operations
- array access
- function calls

The statement AST currently represents:

- assignments to identifiers
- assignments to array elements
- `return` statements with optional expressions

### Parser

The parser currently supports the complete expression layer needed by the implemented statements, including operator precedence, unary expressions, array access, and function calls.

Statement parsing currently supports:

```text
x = expression;
array[index] = expression;
return;
return expression;
```

`ParserContext` owns token traversal, while expression parsing is injected through `TExprParser`. Statement parsing reuses that expression parser instead of duplicating expression grammar.

Malformed syntax produces typed `ParserError` values. Parser routines are responsible for consuming their own delimiters, which is enforced by tests that parse consecutive constructs.

The next parser work is centered on the remaining statement forms and larger grammar structures such as `print`, blocks, control flow, declarations, functions, and complete programs.

## Testing

TDD is a first-class part of the project.

The testing strategy favors:

- **contract tests** at architectural boundaries such as scanners, lexers, and expression parsers
- focused unit tests for individual compiler transformations
- regression tests for parser cursor and delimiter ownership
- a smaller number of end-to-end compiler tests as later phases are implemented

The parser tests intentionally verify not only AST output but also token consumption. For example, consecutive statement tests ensure that parsing one statement consumes exactly one statement and leaves the parser positioned at the next one.

Run the test suite with:

```bash
cargo test
```

Line coverage is measured in CI with `cargo-llvm-cov`. Coverage is calculated from `main`, while the generated badge is stored on the dedicated `coverage-badge` branch so badge updates do not add commits to the development history.

To generate a coverage report locally:

```bash
cargo llvm-cov --workspace --all-features
```

## Building

MicroC-RS uses Rust edition 2024.

Clone the repository:

```bash
git clone https://github.com/Joao-Schio/microc-rs.git
cd microc-rs
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

The repository's CI runs formatting checks and the test suite, while the coverage workflow tracks line coverage separately.

> The compiler executable itself is not functional yet. `main.rs` remains a placeholder while the compiler frontend is developed.

## Backend

MicroC-RS V1 will generate **LLVM IR** and rely on the LLVM toolchain for lowering to native code.

The frontend remains responsible for lexical analysis, parsing, AST construction, semantic analysis, type checking, control-flow representation, and lowering Micro C semantics into LLVM IR.

The first backend implementation will intentionally emit straightforward textual LLVM IR. Mutable variables can initially be represented using operations such as `alloca`, `load`, and `store`, allowing LLVM to perform later optimization and SSA promotion instead of requiring MicroC-RS to implement those transformations immediately.

This keeps V1 focused on the compiler concepts specific to Micro C while delegating register allocation, ABI lowering, stack layout, instruction selection, assembly generation, and machine-code emission to LLVM.

A handwritten Linux x86-64 backend using the System V ABI and GNU/AT&T assembly remains planned as future work after the LLVM-backed compiler is complete.

## Roadmap

### V1 — LLVM-backed Micro C compiler

- [x] Scanner foundation
- [x] Lexical analyzer and typed lexical errors
- [x] Expression AST
- [x] Expression parser and operator precedence
- [x] Function-call parsing
- [x] Assignment statements
- [x] `return` statements
- [ ] Remaining statements (`print`, empty statement, blocks)
- [ ] Control flow (`if` / `else`, `for`)
- [ ] Variable declarations
- [ ] Function definitions and complete program parsing
- [ ] Complete typed AST
- [ ] Semantic analysis and symbol tables
- [ ] LLVM IR generation
- [ ] LLVM toolchain integration
- [ ] Compiler CLI
- [ ] End-to-end Micro C programs

### Future handwritten backend

After V1, a second backend may be implemented as a deeper backend/code-generation exercise:

- [ ] Linux x86-64 / AMD64 target
- [ ] System V ABI lowering
- [ ] Stack-frame layout
- [ ] Register and temporary-value strategy
- [ ] GNU/AT&T assembly generation

Additional targets, language features, and optimization experiments are deliberately deferred until the original Micro C implementation is complete.

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