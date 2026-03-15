---
title: "Types & Type Safety"
description: "Add type annotations to Pico and learn how the compiler enforces them."
---

# Types & Type Safety

So far the Pico language looks like a dynamic language — variables have no declared types. In this chapter we make Pico **type-safe**: every variable and function must declare its type, and the compiler will catch type mismatches before the program ever runs.

## Why Types Matter

Consider this program:

```
let age = "twenty-five";   // a string, but it looks like a number name
let next = age + 1;        // BUG! you can't add 1 to a string
```

Without types, a compiler cannot detect this mistake. With types, the compiler sees that `age` is a `str` and `1` is an `int`, and **rejects the program immediately** with a helpful error:

```
Error: type mismatch — cannot add `str` and `int`
  --> line 2, col 12
```

This is what type safety gives you: **bugs caught at compile time, not at runtime**.

## Pico's Four Basic Types

Pico has four built-in types:

| Type | What it holds | Example |
|------|--------------|---------|
| `int` | Whole numbers | `0`, `42`, `-7` |
| `float` | Decimal numbers | `3.14`, `0.5`, `-1.0` |
| `str` | Text (a string) | `"hello"`, `"Alice"` |
| `bool` | True or false | `true`, `false` |

## Type Annotations on Variables

You write the type after a colon, before the `=`:

```
let name:  str   = "Alice";
let age:   int   = 25;
let pi:    float = 3.14;
let adult: bool  = true;
```

The compiler reads the declared type and the actual value. If they do not match, it is an error:

```
let age: int = "old";   // ERROR: expected int, found str
```

## Type Annotations on Functions

Functions must declare the type of each parameter and the return type. The return type comes after a colon at the end of the signature:

```
fn add(a: int, b: int): int {
    return a + b;
}

fn greet(name: str): str {
    return "Hello, " + name;
}

fn is_adult(age: int): bool {
    return age >= 18;
}
```

The compiler checks:
- That every `return` expression matches the declared return type.
- That every call site passes arguments of the correct types.

```
let x: int = add(10, 20);    // OK
let y: int = add(10, "20");  // ERROR: argument 2 — expected int, found str
```

## Type Checking Rules

Here is how the compiler checks types:

| Situation | Rule |
|-----------|------|
| Variable declaration | `let x: T = expr` — `expr` must have type `T` |
| Assignment | not allowed in Pico (variables are immutable) |
| Binary `+`, `-`, `*`, `/` | both sides must be `int` or both `float` |
| Binary `>`, `<`, `==`, `!=` | both sides must have the same type |
| Function call | each argument must match the declared parameter type |
| `return` | the expression type must match the function return type |
| `print(x)` | any type is accepted |
| `if` condition | must be `bool` |

## What TypeScript Output Looks Like

Pico types map directly to TypeScript types:

| Pico type | TypeScript type |
|-----------|----------------|
| `int` | `number` |
| `float` | `number` |
| `str` | `string` |
| `bool` | `boolean` |

So this Pico code:

```
fn multiply(a: float, b: float): float {
    return a * b;
}

let area: float = multiply(3.0, 4.5);
```

Compiles to this TypeScript:

```typescript
function multiply(a: number, b: number): number {
    return (a * b);
}

const area: number = multiply(3.0, 4.5);
```

## New Tokens Needed

To support type annotations, the lexer needs three new things:

```rust
// New keywords for type names
TyInt,   // 'int'
TyFloat, // 'float'
TyStr,   // 'str'
TyBool,  // 'bool'

// New punctuation
Colon,  // ':'  — separates name from type: x: int
```

The parser needs new grammar rules too. A **typed declaration** looks like:

```
let <name> : <type> = <expression> ;
```

And a **typed function** looks like:

```
fn <name> ( <param> : <type> , ... ) : <return-type> { <body> }
```

## New AST Nodes Needed

The AST nodes for `let` and `fn` grow a type field:

```rust
// LetStatement now carries an optional declared type
pub struct LetStatement {
    pub name:      String,
    pub type_ann:  Option<TypeNode>, // e.g. TypeNode::Int
    pub value:     Box<Expr>,
}

// FunctionDef now carries typed parameters and a return type
pub struct FunctionDef {
    pub name:        String,
    pub params:      Vec<(String, TypeNode)>, // (param_name, param_type)
    pub return_type: TypeNode,
    pub body:        Vec<Stmt>,
}

// TypeNode — the possible types in Pico
pub enum TypeNode {
    Int,
    Float,
    Str,
    Bool,
    Named(String), // for structs, e.g. TypeNode::Named("Person")
}
```

## Type Checking in the Semantic Phase

The semantic analyzer now carries a **type environment** — a map from variable name to its type:

```rust
use std::collections::HashMap;

pub struct TypeEnv {
    vars: HashMap<String, TypeNode>,
}

impl TypeEnv {
    pub fn declare(&mut self, name: &str, ty: TypeNode) {
        self.vars.insert(name.to_string(), ty);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeNode> {
        self.vars.get(name)
    }
}
```

When we visit a `LetStatement`:

```rust
fn check_let(&mut self, stmt: &LetStatement, env: &mut TypeEnv) {
    // 1. Infer the type of the right-hand side expression
    let expr_type = self.infer_type(&stmt.value, env);

    // 2. If there is a type annotation, make sure it matches
    if let Some(ann) = &stmt.type_ann {
        if *ann != expr_type {
            self.error(format!(
                "type mismatch: declared {:?} but value has type {:?}",
                ann, expr_type
            ));
        }
    }

    // 3. Register the variable in the environment
    env.declare(&stmt.name, expr_type);
}
```

That is the core of type checking! It is not as hard as it sounds. We will build this fully in the Semantic Analysis chapter.
