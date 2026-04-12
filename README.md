# Compiler in Rust 🦀

> **A beginner-friendly book that teaches you how to build a real compiler from scratch using Rust.**
> *Book created by **Mr Binary Sniper***.

---

## What You Will Build

You will design **Pico** — a small programming language — and write a compiler for it in Rust.
The compiler reads Pico source code and outputs working **TypeScript**.

```pico
fn add(a, b) { return a + b; }
let result = add(10, 32);
print(result);   // → 42
```

---

## Compiler Pipeline

```
source.pico  →  Lexer  →  Parser  →  Semantic Check  →  Code Gen  →  output.ts
```

| Phase | File | What it does |
|---|---|---|
| Lexer | `lexer.rs` | Turns text into tokens |
| Parser | `parser.rs` | Turns tokens into an AST |
| Semantic | `semantic.rs` | Catches undefined variables / functions |
| Code Gen | `codegen.rs` | Turns the AST into TypeScript |

---

## Quick Start

```bash
git clone https://github.com/MrBns/compiler-book-rust
cd compiler-book-rust/compiler

cargo build          # build the compiler
cargo test           # run all 28 unit tests
cargo run -- hello.pico   # compile a Pico file → hello.ts
```

---

## Read the Book

The full step-by-step guide lives in the `src/content/docs/` folder and is
served as a web book.  Each chapter explains one compiler phase in plain English
with inline code examples.

```bash
npm install && npm run dev   # open http://localhost:4321
```

---

*Book creator: **Mr Binary Sniper** · GitHub: [@MrBns](https://github.com/MrBns)*
