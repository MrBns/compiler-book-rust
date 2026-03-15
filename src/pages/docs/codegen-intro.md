---
layout: ../../layouts/DocLayout.astro
title: "Code Generation Intro"
description: "Learn the strategy for generating TypeScript from an AST."
---

# Code Generation Intro

We are at the last phase of our compiler: **code generation**.

This is the most exciting step! We take the AST (our tree of meaning) and write it out as TypeScript code.

## What is Code Generation?

Code generation means: walk every node in the AST and produce the equivalent code in the target language.

For us, the target language is **TypeScript**.

Here is the idea in pseudocode:

```
For each AST node:
    If it is a LetStatement → write "const name = value;"
    If it is a Number      → write the number
    If it is an Ident      → write the variable name
    If it is a BinaryExpr  → write "left op right"
    If it is a FnStatement → write "function name(params) { body }"
    ... and so on
```

This is called **tree traversal** or **tree walking**.

## The Strategy: Walking the AST

We will write two main functions:

1. `gen_stmt(stmt)` — generate code for a statement
2. `gen_expr(expr)` — generate code for an expression

These functions call each other recursively, just like the parser did.

```rust
fn gen_stmt(stmt) {
    match stmt {
        Let { name, value } => format!("const {} = {};", name, gen_expr(value))
        If { cond, then, else_block } => {
            // generate the condition, then the body
        }
        // ... etc
    }
}

fn gen_expr(expr) {
    match expr {
        Number(n) => n.to_string()
        Ident(name) => name
        Binary { left, op, right } => format!("({} {} {})", gen_expr(left), op, gen_expr(right))
        // ... etc
    }
}
```

Each function returns a `String` containing TypeScript code.

## Pico to TypeScript Mapping

Here is the translation table from Pico to TypeScript:

| Pico | TypeScript |
|---|---|
| `let x = 5;` | `const x = 5;` |
| `let s = "hi";` | `const s = "hi";` |
| `print(x);` | `console.log(x);` |
| `if x > 0 { ... }` | `if (x > 0) { ... }` |
| `fn add(a, b) { return a + b; }` | `function add(a: any, b: any): any { return a + b; }` |
| `add(1, 2)` | `add(1, 2)` |
| `x + y` | `x + y` |
| `x == y` | `x === y` |
| `x != y` | `x !== y` |

Notice:
- `let` becomes `const` (TypeScript prefers `const` for values that don't change)
- `print()` becomes `console.log()`
- `==` becomes `===` (strict equality in TypeScript)
- `!=` becomes `!==`
- Functions get `: any` types because Pico has no type system yet

## Indentation

Good code generation produces nicely **indented** output. Nobody wants to read minified code.

We will track an **indentation level** — a number that increases when we enter a block and decreases when we leave:

```rust
// indent_level = 0  → no indentation (top level)
// indent_level = 1  → 4 spaces (inside a function or if)
// indent_level = 2  → 8 spaces (nested block)

fn indent(&self) -> String {
    "    ".repeat(self.indent_level) // 4 spaces per level
}
```

Example output with proper indentation:

```typescript
function add(a: any, b: any): any {
    return (a + b);
}
const r = add(1, 2);
console.log(r);
```

In the next chapter, we will write the complete code generator!
