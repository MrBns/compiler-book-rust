---
title: "What is a Compiler?"
description: "Understand what a compiler does, and the four main phases it goes through."
---

A **compiler** is a program that reads code written in one language and turns it into code in another language.

Think about it like a **translator**. A human translator reads a book written in Bengali and writes the same book in English. A compiler does the same thing — but for programming languages.

For example:

- The **Rust compiler** reads Rust code and turns it into machine code.
- The **TypeScript compiler** reads TypeScript code and turns it into JavaScript.
- Our **Pico compiler** will read Pico code and turn it into TypeScript.

## The Simple Idea

Here is the simple idea:

```txt
Your Code (text)  →  [Compiler]  →  New Code (text or machine code)
```

That is it! The compiler is just a program that transforms text.

But how? Let's look inside the compiler.

## Phases of a Compiler

A compiler does not do everything in one step. It has **phases** — like stages in a factory.

```txt
Source Code
    ↓
[Phase 1: Lexer]      → List of Tokens
    ↓
[Phase 2: Parser]     → Abstract Syntax Tree (AST)
    ↓
[Phase 3: Semantic]   → Checked AST
    ↓
[Phase 4: Codegen]    → Output Code (TypeScript)
```

Let's understand each phase.

### Phase 1: Lexing

The **lexer** (also called "tokenizer") reads the source code character by character and groups them into **tokens**.

A token is a small meaningful piece of code.

Example:

```txt
Source code:   let x = 10 + 20;
               ↓
Tokens:        [Let] [Ident("x")] [Equals] [Number(10)] [Plus] [Number(20)] [Semicolon]
```

The lexer does not care about meaning. It just cuts the text into pieces.

### Phase 2: Parsing

The **parser** takes the list of tokens and builds a **tree** — called the **Abstract Syntax Tree** (AST).

The tree shows the structure and relationships between tokens.

```txt
let x = 10 + 20;
        ↓
   LetStatement
   ├── name: "x"
   └── value: BinaryExpr
               ├── left:  Number(10)
               ├── op:    Plus
               └── right: Number(20)
```

Now we can see that `10 + 20` is the *value* being assigned to `x`.

### Phase 3: Semantic Analysis

**Semantic analysis** checks if the code *makes sense*.

For example:

```txt
let x = 10;
let y = x + z;  -- ERROR! 'z' is never defined!
```

The parser would accept this code (the syntax is fine). But the semantic phase will catch the error: `z` is used but never declared.

### Phase 4: Code Generation

The **code generator** walks the AST and writes the output code.

For our compiler, it will write TypeScript:

```rust
// For a LetStatement node, we will generate:  "const x = ..."
// For a BinaryExpr node, we will generate:    "left op right"
// For a PrintStatement node:                  "console.log(...)"
```

## Compiler vs Interpreter

You might hear the word **interpreter** too. What is the difference?

| | Compiler | Interpreter |
| --- | --- | --- |
| What it does | Translates all code first, then runs | Reads and runs code line by line |
| Speed | Faster at runtime | Slower at runtime |
| Examples | Rust compiler, GCC | Python, Ruby |
| Our project | ✅ We build a compiler | |

We are building a **compiler**. We translate all Pico code to TypeScript first, and then you run the TypeScript separately.

In the next chapter, we will plan our Pico language and decide what features it will have.
