---
layout: ../../layouts/DocLayout.astro
title: "What is Parsing?"
description: "Understand how a parser turns tokens into an Abstract Syntax Tree (AST)."
---

# What is Parsing?

The **parser** is the second phase of our compiler. It takes the flat list of tokens from the lexer and builds a **tree** structure that shows the meaning and hierarchy of the code.

## What is Parsing?

When you read a sentence like: **"The cat sat on the mat."** — your brain does not just see a list of words. It builds a tree of meaning:

```
Sentence
├── Subject:   "The cat"
├── Verb:      "sat"
└── Location:  "on the mat"
```

A parser does the same thing for code. It takes a flat list of tokens and builds a tree of meaning.

**Input to parser** (from lexer):

```
[Let] [Ident("x")] [Equals] [Number(10)] [Plus] [Number(20)] [Semicolon]
```

**Output from parser** (the AST):

```
LetStatement {
    name: "x",
    value: BinaryExpr {
        left:  Number(10),
        op:    Plus,
        right: Number(20),
    }
}
```

Now we can *see* that `10 + 20` is the value. The tree captures the meaning.

## What is an AST?

**AST** stands for **Abstract Syntax Tree**.

- **Abstract** — it does not include every detail (like semicolons or parentheses). Only the meaning.
- **Syntax** — it represents the structure of the code.
- **Tree** — it is organized as a tree with parent nodes and child nodes.

For example, this Pico code:

```
if x > 0 {
    print(x);
}
```

Becomes this AST:

```
IfStatement {
    condition: BinaryExpr {
        left:  Ident("x"),
        op:    Gt,
        right: Number(0),
    },
    then_block: [
        PrintStatement {
            value: Ident("x"),
        }
    ],
    else_block: None,
}
```

## Types of Statements

In Pico, a **statement** is a complete instruction. Here are the statements we will support:

| Statement | Example |
|---|---|
| Let statement | `let x = 5;` |
| Return statement | `return x + 1;` |
| If statement | `if x > 0 { ... }` |
| Function definition | `fn add(a, b) { return a + b; }` |
| Print statement | `print(x);` |
| Expression statement | `add(1, 2);` |

## Types of Expressions

An **expression** is a piece of code that produces a value. Statements can *contain* expressions.

| Expression | Example |
|---|---|
| Number literal | `42`, `3.14` |
| String literal | `"hello"` |
| Boolean literal | `true`, `false` |
| Identifier | `x`, `myVar` |
| Binary operation | `x + y`, `a > b` |
| Function call | `add(1, 2)` |
| Grouped | `(x + y) * z` |

## Recursive Descent Parsing

The parsing technique we will use is called **Recursive Descent Parsing**. It is the easiest technique to understand and code by hand.

The idea: we write one function for each grammar rule. Functions can call each other recursively.

```
parse_program()
  └─ calls parse_statement()
        ├─ calls parse_let_statement()
        ├─ calls parse_if_statement()
        └─ calls parse_expression()
              ├─ calls parse_addition()
              │     └─ calls parse_multiplication()
              │           └─ calls parse_primary()
              └─ ...
```

The recursion handles nesting naturally. For example, `(1 + (2 * 3))` — the outer `+` calls the inner `*` recursively.

## Operator Precedence

**Operator precedence** means: which operation happens first?

In math: `2 + 3 * 4` = `2 + (3 * 4)` = `14` — NOT `(2 + 3) * 4 = 20`.

We say `*` has **higher precedence** than `+`.

In recursive descent, we handle this by having separate functions for each precedence level:

```
parse_expression()        → lowest precedence
  → parse_comparison()    → <, >, <=, >=, ==, !=
    → parse_addition()    → +, -
      → parse_multiplication() → *, /
        → parse_primary() → highest: literals, identifiers, (...)
```

When we call `parse_expression()`, it eventually calls `parse_multiplication()` first, which means multiplication binds tighter than addition. This automatically gives us correct math order!

We will code all this in the next chapter.
