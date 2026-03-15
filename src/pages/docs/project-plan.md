---
layout: ../../layouts/DocLayout.astro
title: "Our Project Plan"
description: "Design the Pico language we will compile and set up our Rust project files."
---

# Our Project Plan

Before we write any compiler code, we need a plan. What language will we compile? What features will it have? Let us decide all this now.

## The Pico Language

We will invent our own small language. We call it **Pico** (it means "small" in Spanish).

Here is a complete Pico program that shows all features:

```
// This is a comment in Pico

// Declare variables
let name = "Alice";
let age  = 25;
let pi   = 3.14;

// If / else
if age > 18 {
    print("Adult");
} else {
    print("Not adult");
}

// Function definition
fn add(a, b) {
    return a + b;
}

// Function call
let result = add(10, 20);
print(result);
```

Simple and clean. Very similar to JavaScript.

## Language Features

Here is a list of everything Pico can do:

| Feature | Example |
|---|---|
| Variable | `let x = 5;` |
| String | `let s = "hello";` |
| Boolean | `let ok = true;` |
| Arithmetic | `x + y`, `x * y`, `x - y`, `x / y` |
| Comparison | `x > y`, `x == y`, `x != y` |
| If/else | `if cond { ... } else { ... }` |
| Function | `fn name(a, b) { return a + b; }` |
| Print | `print(value);` |
| Comments | `// this is ignored` |

We keep it small on purpose. Once you understand a small compiler, you can add more features yourself!

## Why TypeScript as Target?

We chose TypeScript because:

1. **Easy to read** — TypeScript is very readable. You can check if our output looks correct.
2. **Runs anywhere** — You can run TypeScript in Node.js or the browser.
3. **Similar syntax** — Pico is inspired by TypeScript, so the translation is natural.

For example, `let x = 10;` in Pico becomes `const x = 10;` in TypeScript. Very easy!

## Create the File Structure

Now let's create all the files we need. In your `pico/src/` folder, create these files:

```bash
# Create the files (run this in your terminal)
touch src/token.rs
touch src/lexer.rs
touch src/ast.rs
touch src/parser.rs
touch src/semantic.rs
touch src/codegen.rs
```

Now open `src/main.rs` and add this:

```rust
// main.rs — The main entry point of our Pico compiler
//
// We declare the other modules here so Rust knows about them.

mod token;    // Token types
mod lexer;    // Phase 1: Lexer
mod ast;      // AST node types
mod parser;   // Phase 2: Parser
mod semantic; // Phase 3: Semantic checker
mod codegen;  // Phase 4: Code generator

fn main() {
    println!("Pico compiler ready!");
}
```

## Define the Token Types

A **token** is the smallest unit of meaning in our language. Before we build the lexer, let's define what tokens look like.

Open `src/token.rs` and write:

```rust
// token.rs — Defines all the token types in the Pico language
//
// A Token has two things:
//   1. TokenKind  — what type of token is it?
//   2. Span       — where in the source code is it? (line and column)

// ---- TokenKind ----
// This is an enum. Each variant represents a different type of token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // --- Literals ---
    Number(f64),       // e.g. 42, 3.14
    StringLit(String), // e.g. "hello"
    True,              // the keyword 'true'
    False,             // the keyword 'false'

    // --- Identifiers ---
    Ident(String), // e.g. variable names like 'x', 'myFunc'

    // --- Keywords ---
    Let,    // 'let'
    Fn,     // 'fn'
    Return, // 'return'
    If,     // 'if'
    Else,   // 'else'
    Print,  // 'print'

    // --- Operators ---
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Equals,  // =
    EqEq,    // ==
    BangEq,  // !=
    Lt,      // <
    Gt,      // >
    LtEq,    // <=
    GtEq,    // >=

    // --- Punctuation ---
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Comma,     // ,
    Semicolon, // ;

    // --- Special ---
    Eof, // End of file — the last token we produce
}

// ---- Span ----
// This tells us WHERE in the source code the token was found.
// Very useful for error messages like:  "Error at line 3, column 5"
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

// ---- Token ----
// A token combines TokenKind and Span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind, // what type of token
    pub span: Span,      // where in the source
}

impl Token {
    // Helper: create a new token
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token {
            kind,
            span: Span { line, col },
        }
    }
}
```

Great! Now we have our token types defined. In the next chapter, we will build the **lexer** that reads source code and produces these tokens.
