---
title: "What is a Lexer?"
description: "Understand how a lexer works before we build one."
---

The **lexer** is the first part of our compiler. It is also called a **tokenizer** or **scanner**.

Its job is simple: read the source code text character by character, and group those characters into **tokens**.

## What Does a Lexer Do?

Think about reading English. When you read "Hello World", your brain does not see individual letters. It sees two **words**: "Hello" and "World".

The lexer does the same thing for code. It reads the source text and produces a list of words (tokens) that have meaning.

**Input to the lexer:**

```txt
let x = 10 + 20;
```

**Output from the lexer:**

```rust
Token { kind: Let,          span: (line=1, col=1) }
Token { kind: Ident("x"),   span: (line=1, col=5) }
Token { kind: Equals,       span: (line=1, col=7) }
Token { kind: Number(10.0), span: (line=1, col=9) }
Token { kind: Plus,         span: (line=1, col=12) }
Token { kind: Number(20.0), span: (line=1, col=14) }
Token { kind: Semicolon,    span: (line=1, col=16) }
Token { kind: Eof,          span: (line=1, col=17) }
```

The lexer does **not** care about meaning. It does not know that `10 + 20` is an addition. It just splits the text into pieces.

## How the Lexer Works

Here is the basic algorithm:

```txt
1. Look at the current character
2. Decide what kind of token it starts
3. Read more characters if needed (e.g., for "==")
4. Create a Token and add it to the list
5. Move to the next character
6. Repeat until end of file
```

Let's look at some examples:

**Single character tokens** — these are easy:

| Character | Token |
| --- | --- |
| `+` | `Plus` |
| `-` | `Minus` |
| `*` | `Star` |
| `/` | `Slash` |
| `(` | `LParen` |
| `)` | `RParen` |
| `{` | `LBrace` |
| `}` | `RBrace` |
| `,` | `Comma` |
| `;` | `Semicolon` |

**Two character tokens** — we need to peek at the next character:

| Characters | Token |
| --- | --- |
| `==` | `EqEq` |
| `!=` | `BangEq` |
| `<=` | `LtEq` |
| `>=` | `GtEq` |

**Multi-character tokens** — we keep reading until the token ends:

| Pattern | Token |
| --- | --- |
| Digits like `123` or `3.14` | `Number(123.0)` |
| Letters like `hello` | `Ident("hello")` |
| Keywords like `let`, `fn` | `Let`, `Fn` |
| Quoted text like `"hi"` | `StringLit("hi")` |

## Whitespace and Comments

Spaces, tabs, and newlines (`\n`) are **not** tokens. They are just separators. The lexer skips them.

Comments (lines starting with `//`) are also skipped. They exist only for the programmer, not for the compiler.

```rust
// When we see a space, tab, or newline → skip it
// When we see '//' → skip until end of line
```

## What is a Finite State Machine?

A lexer is often described as a **finite state machine** (FSM). Do not worry about this term!

It just means: the lexer can be in different **states**, and it changes state based on what character it reads.

For example:

```txt
State: Start
  → See digit  → go to State: ReadingNumber
  → See letter → go to State: ReadingIdentifier
  → See '"'    → go to State: ReadingString
  → See '+'    → emit Plus token, stay in State: Start

State: ReadingNumber
  → See digit  → keep reading
  → See '.'    → keep reading (decimal number)
  → See other  → emit Number token, go back to State: Start
```

This is what our Rust code will do. In the next chapter, we will actually **code** the lexer!
