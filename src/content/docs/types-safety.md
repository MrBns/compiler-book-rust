---
title: "Types & Type Safety"
description: "Add type annotations to Pico and learn how the compiler enforces them."
---

# Types & Type Safety

Up until now the Pico language we have been designing looks a lot like a **dynamic language** — one where variables simply hold values and nobody bothers to say what kind of value they are supposed to hold. Languages like Python and JavaScript work this way. They are very flexible, but that flexibility comes with a cost: mistakes that could have been caught instantly by the compiler instead crash your program at runtime, often in production, and sometimes in ways that are very hard to debug.

In this chapter we are going to fix that. We will make Pico a **statically-typed, type-safe language**. What that means is: every single variable, every function parameter, and every function return value must carry a type label. The compiler reads those labels and checks every operation in your program — before it ever runs — to make sure nothing is used the wrong way.

This is one of the most important features we will add to Pico. It brings us much closer to real production languages like Rust, TypeScript, and Go.

## Why Types Matter — A Story

Imagine you are building a website that calculates a user's age for a birthday greeting. A colleague writes this Pico code (without types):

```
let birth_year = "1998";    // oops — stored as text, not a number
let current_year = 2024;
let age = current_year - birth_year;   // BUG: subtracting a number from text
print("You are " + age + " years old!");
```

In a dynamic language, this code might compile and even run without an error — but silently produce garbage output, or crash with a confusing runtime error message like `"NaN"` or `"cannot subtract string"`. You only find out something is wrong *after* the program is running.

Now look at the same program with type annotations:

```
let birth_year:   str = "1998";      // clearly a string
let current_year: int = 2024;
let age: int = current_year - birth_year;  // ERROR: cannot subtract str from int
```

The compiler stops right there, before producing any output, and tells you:

```
Error: type mismatch on line 3
  The `-` operator requires both sides to be `int` or both `float`.
  Left side:  int   (current_year)
  Right side: str   (birth_year)
  Hint: did you mean to parse birth_year as an int first?
```

You fix it immediately. No runtime crash. No confused user. This is what **type safety** is all about — mistakes caught at compile time, where they are cheapest and easiest to fix.

## Pico's Four Basic Types

Pico has four built-in primitive types. We keep the type system small on purpose so it is easy to understand and implement. Here they are:

| Type | What it represents | Valid literal examples |
|------|--------------------|------------------------|
| `int` | A whole number, positive or negative | `0`, `42`, `-7`, `1000` |
| `float` | A number with a decimal point | `3.14`, `0.5`, `-1.0`, `2.718` |
| `str` | A piece of text, always in double quotes | `"hello"`, `"Alice"`, `"123"` |
| `bool` | A truth value — either `true` or `false` | `true`, `false` |

Notice that `"123"` is a `str`, not an `int`. Quotes always mean text, even if the text contains digits. This is an important distinction that the type system enforces for you.

### `int` — Whole Numbers

An `int` holds any whole number. You use it for things like ages, counts, indices, and scores.

```
let score:  int = 100;
let lives:  int = 3;
let damage: int = -5;
let zero:   int = 0;
```

You can use all the arithmetic operators on `int` values: `+`, `-`, `*`, `/`. You can also compare them with `>`, `<`, `>=`, `<=`, `==`, `!=`.

### `float` — Decimal Numbers

A `float` holds a number with a fractional part. You use it for measurements, scientific values, prices, and any time you need more precision than a whole number.

```
let pi:          float = 3.14159;
let temperature: float = 98.6;
let price:       float = 19.99;
let gravity:     float = 9.81;
```

Notice that float literals always have a decimal point — even if the fractional part is zero: write `1.0`, not `1`. This makes it clear to both you and the compiler that you are dealing with a float.

### `str` — Text

A `str` holds a sequence of characters. It is always surrounded by double quotes. You use it for names, messages, labels, and any text data.

```
let greeting: str = "Hello, world!";
let username: str = "alice_coder";
let empty:    str = "";
let sentence: str = "The quick brown fox.";
```

You can concatenate (join) two strings with the `+` operator:

```
let first: str = "Hello";
let last:  str = "World";
let full:  str = first + ", " + last + "!";   // "Hello, World!"
```

### `bool` — True or False

A `bool` holds exactly one of two values: `true` or `false`. You use it for flags, conditions, and the results of comparisons.

```
let is_logged_in:  bool = true;
let has_errors:    bool = false;
let is_adult:      bool = true;
let debug_enabled: bool = false;
```

The result of a comparison expression is always a `bool`:

```
let old_enough:  bool = age >= 18;    // true or false
let names_match: bool = name == "Alice";
```

And an `if` condition **must** be a `bool`. Writing `if age { ... }` is a type error — `age` is an `int`, not a `bool`. You must write `if age > 0 { ... }`.

## Type Annotations on Variables

The syntax for declaring a variable with a type is:

```
let <name> : <type> = <expression> ;
```

The colon `:` separates the variable name from its type. Here are all four types in use together:

```
let name:    str   = "Alice";
let age:     int   = 25;
let balance: float = 1024.50;
let active:  bool  = true;
```

Aligning the colons and equals signs (as shown above) is a style choice — it makes the declarations easier to read at a glance, especially when you have several variables of different types in a row.

### What Happens When Types Don't Match

If you write a type annotation that contradicts the value on the right-hand side, the compiler refuses to compile the program and tells you exactly what went wrong:

```
let age:   int  = "twenty-five";   // ERROR: expected int, found str
let price: int  = 9.99;            // ERROR: expected int, found float
let flag:  bool = 1;               // ERROR: expected bool, found int
```

Each of these is a different kind of mistake. The compiler will tell you which type it expected, which type it actually found, and on which line. This makes fixing the error very fast — you do not have to go hunting for where the bug is.

## Type Annotations on Functions

Functions are where types really pay off. Every parameter must declare its type, and the function itself must declare what type of value it returns. The return type is written with a `:` after the closing parenthesis of the parameter list:

```
fn <name>( <param1>: <type1>, <param2>: <type2>, ... ): <return-type> {
    <body>
}
```

Here are several complete examples with different parameter and return types:

```
// Takes two ints, returns their sum as int
fn add(a: int, b: int): int {
    return a + b;
}

// Takes a name as str, returns a greeting as str
fn greet(name: str): str {
    return "Hello, " + name + "!";
}

// Takes an age as int, returns whether the person is an adult
fn is_adult(age: int): bool {
    return age >= 18;
}

// Takes two floats, returns their average as float
fn average(x: float, y: float): float {
    return (x + y) / 2.0;
}

// Takes a count and a price, returns total cost
fn total_cost(count: int, unit_price: float): float {
    return count * unit_price;
}
```

Once you have written typed functions, the compiler can check every call site. This is extremely valuable. If someone calls `add` with the wrong type, the compiler catches it:

```
let result1: int = add(10, 20);         // OK — both arguments are int
let result2: int = add(10, 20.5);       // ERROR: argument 2 — expected int, found float
let result3: str = add(10, 20);         // ERROR: expected str result, but add returns int
let result4: int = add("ten", "twenty"); // ERROR: both arguments — expected int, found str
```

Every one of these errors is caught **before the program runs**. Without types, all four of those calls would be silently accepted, and you would only discover the problem when the output was wrong.

## What Happens Inside a Function — Return Type Checking

The compiler does not just check the call site. It also checks the *body* of the function to make sure every `return` statement produces a value that matches the declared return type.

```
fn double(n: int): int {
    return n * 2;     // OK: n*2 is int, matches return type int
}

fn bad_double(n: int): int {
    return n * 2.0;   // ERROR: n*2.0 is float, but return type is int
}
```

This ensures that the function always delivers what it promises. If you declare a function returns `bool`, it must always return a `bool` — not an `int` that happens to be `0` or `1`.

## The Full Type Checking Rules

Here is a complete summary of how the Pico compiler checks types:

| Situation | What the compiler checks |
|-----------|--------------------------|
| `let x: T = expr` | The type of `expr` must equal `T` |
| `let x: T = expr` (no annotation) | The type of `expr` is inferred and recorded for `x` |
| `a + b`, `a - b`, `a * b`, `a / b` | Both `a` and `b` must be `int`, or both must be `float` |
| `a + b` where `a` and `b` are `str` | Allowed — string concatenation |
| `a > b`, `a < b`, `a >= b`, `a <= b` | Both sides must have the same type (`int` or `float`) |
| `a == b`, `a != b` | Both sides must have the same type |
| Function call `f(e1, e2)` | Each `ei` must match the declared type of the `i`-th parameter |
| `return expr` inside a function | `expr` must match the function's declared return type |
| `if condition { ... }` | `condition` must be of type `bool` |
| `print(expr)` | Any type is accepted — `print` is polymorphic |

If any of these rules are violated, the compiler produces a clear error message and stops. It does not produce partial output. Either the whole program type-checks successfully, or it does not compile at all.

## How Pico Types Map to TypeScript

Since we compile Pico to TypeScript, we need to know how to translate each Pico type. The mapping is very straightforward:

| Pico type | TypeScript type | Notes |
|-----------|----------------|-------|
| `int` | `number` | TypeScript has only one numeric type |
| `float` | `number` | Both `int` and `float` become `number` in TypeScript |
| `str` | `string` | Direct match |
| `bool` | `boolean` | Direct match |

Both `int` and `float` become `number` because TypeScript (like JavaScript) does not distinguish between integer and floating-point numbers at the type level. Our Pico compiler does make that distinction — it gives you the precision of two separate types in Pico — but at the output stage, they both become `number`.

Here is a longer Pico program and its TypeScript output side by side:

**Pico source:**

```
let name:     str   = "Alice";
let age:      int   = 25;
let height:   float = 5.6;
let is_admin: bool  = false;

fn describe(who: str, years: int, tall: float): str {
    return who + " is " + years + " years old.";
}

let desc: str = describe(name, age, height);
print(desc);
```

**Compiled TypeScript output:**

```typescript
const name: string    = "Alice";
const age: number     = 25;
const height: number  = 5.6;
const is_admin: boolean = false;

function describe(who: string, years: number, tall: number): string {
    return ((who + " is ") + (String(years) + " years old."));
}

const desc: string = describe(name, age, height);
console.log(desc);
```

The structure is identical. Every Pico keyword and operator has a direct TypeScript equivalent. The types translate word-for-word (except `int`/`float` → `number`).

## New Tokens the Lexer Needs

To support type annotations, we need to add a few new token kinds to our `TokenKind` enum. Open `src/token.rs` and add these:

```rust
// ---- Added for type annotations ----

// Type keyword tokens — the names of Pico's four built-in types
TyInt,    // the keyword 'int'
TyFloat,  // the keyword 'float'
TyStr,    // the keyword 'str'
TyBool,   // the keyword 'bool'

// Colon — used to separate a name from its type annotation
//   e.g.  let age : int = 25;
//                 ↑ this is the Colon token
Colon,    // ':'
```

The lexer already knows how to read multi-character identifier tokens (like `let`, `fn`, `if`). Adding `int`, `float`, `str`, and `bool` as additional keyword cases is straightforward — we just add them to the same keyword-matching code. When the lexer reads the text `int`, instead of treating it as a plain `Ident("int")`, it now produces a `TyInt` token.

The `Colon` token is simpler still — it is a single character `:`, so the lexer emits it whenever it sees that character.

## Grammar Changes in the Parser

The parser needs to understand the new typed syntax. Before adding types, a `let` declaration looked like this in the grammar:

```
LetDecl → 'let' IDENT '=' Expr ';'
```

After adding types, the type annotation is written between the name and the `=`:

```
LetDecl → 'let' IDENT ':' Type '=' Expr ';'
```

And a function definition changes from:

```
FnDef → 'fn' IDENT '(' Params ')' Block
Params → IDENT (',' IDENT)*
```

To:

```
FnDef  → 'fn' IDENT '(' TypedParams ')' ':' Type Block
TypedParams → TypedParam (',' TypedParam)*
TypedParam  → IDENT ':' Type
Type        → 'int' | 'float' | 'str' | 'bool' | IDENT
```

The `Type` rule allows `IDENT` at the end to support struct types like `Person` or `Rectangle` — user-defined types that are not one of the four built-in keywords.

## New AST Nodes

The existing `LetStatement` and `FunctionDef` AST nodes need to be updated to carry type information. Here is how the updated Rust types look:

```rust
// ---- TypeNode ----
// Represents a type annotation in the source code.
// Every place in the AST that holds a type uses this enum.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    Int,           // the 'int' type
    Float,         // the 'float' type
    Str,           // the 'str' type
    Bool,          // the 'bool' type
    Named(String), // a user-defined type, like 'Person' or 'Rectangle'
}

// ---- LetStatement ----
// Updated to carry a mandatory type annotation.
#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name:     String,       // the variable name, e.g. "age"
    pub type_ann: TypeNode,     // the declared type, e.g. TypeNode::Int
    pub value:    Box<Expr>,    // the right-hand side expression
    pub span:     Span,         // source location for error messages
}

// ---- FunctionDef ----
// Updated to carry typed parameters and a return type.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name:        String,                // function name
    pub params:      Vec<(String, TypeNode)>, // [(param_name, param_type), ...]
    pub return_type: TypeNode,              // the declared return type
    pub body:        Vec<Stmt>,             // statements inside the function
    pub span:        Span,
}
```

Notice that `type_ann` in `LetStatement` is now a required `TypeNode`, not an `Option`. In our version of Pico, every variable must carry a type — we do not support leaving it out.

## How the Semantic Analyzer Does Type Checking

The semantic analyzer is the phase that actually enforces all the type rules. It does two things at once: it **infers** the type of every expression, and it **checks** that the inferred type matches the declared type wherever there is an annotation.

The key data structure is the **type environment** — a map from variable name to its type. Every time we declare a variable, we add it to the environment. Every time we use a variable, we look it up.

```rust
use std::collections::HashMap;

// TypeEnv — keeps track of what type every variable has
pub struct TypeEnv {
    // Maps variable name → its TypeNode
    vars: HashMap<String, TypeNode>,
    // Maps struct name → its field list (we will use this in the Structs chapter)
    structs: HashMap<String, Vec<FieldDef>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            vars: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    // Record that variable `name` has type `ty`
    pub fn declare_var(&mut self, name: &str, ty: TypeNode) {
        self.vars.insert(name.to_string(), ty);
    }

    // Look up the type of variable `name`
    // Returns None if the variable was never declared (undefined variable error)
    pub fn lookup_var(&self, name: &str) -> Option<&TypeNode> {
        self.vars.get(name)
    }
}
```

When the semantic analyzer visits a `LetStatement`, it follows three steps:

```rust
fn check_let_statement(
    &mut self,
    stmt: &LetStatement,
    env: &mut TypeEnv,
) -> Result<(), SemanticError> {
    // Step 1 — Figure out what type the right-hand-side expression has.
    //           For example, if the value is `25`, its type is TypeNode::Int.
    //           If the value is `add(10, 20)` where add returns int, the type is TypeNode::Int.
    let value_type = self.infer_expr_type(&stmt.value, env)?;

    // Step 2 — Compare the inferred type to the declared annotation.
    //           If they differ, it is a type error — report it and stop.
    if value_type != stmt.type_ann {
        return Err(SemanticError::TypeMismatch {
            expected: stmt.type_ann.clone(),
            found:    value_type,
            span:     stmt.span.clone(),
            hint:     format!(
                "variable `{}` is declared as {:?} but the value has type {:?}",
                stmt.name, stmt.type_ann, value_type
            ),
        });
    }

    // Step 3 — The types match. Record the variable so that later code can use it.
    env.declare_var(&stmt.name, stmt.type_ann.clone());

    Ok(())
}
```

And here is how the analyzer infers the type of an expression:

```rust
fn infer_expr_type(
    &self,
    expr: &Expr,
    env: &TypeEnv,
) -> Result<TypeNode, SemanticError> {
    match expr {
        // Literal values have obvious types
        Expr::Number(n) => {
            // If the number has a fractional part, it is a float; otherwise int
            if n.fract() == 0.0 {
                Ok(TypeNode::Int)
            } else {
                Ok(TypeNode::Float)
            }
        }
        Expr::StringLit(_) => Ok(TypeNode::Str),
        Expr::Bool(_)      => Ok(TypeNode::Bool),

        // An identifier's type comes from the environment
        Expr::Ident(name) => {
            env.lookup_var(name).cloned().ok_or_else(|| SemanticError::UndefinedVariable {
                name: name.clone(),
            })
        }

        // For binary operations, check both sides and make sure they are compatible
        Expr::BinaryOp { left, op, right } => {
            let left_type  = self.infer_expr_type(left,  env)?;
            let right_type = self.infer_expr_type(right, env)?;
            self.check_binary_op_types(op, left_type, right_type)
        }

        // A function call's type is the return type of the function
        Expr::Call { func_name, args } => {
            self.check_function_call(func_name, args, env)
        }
    }
}
```

The beauty of this design is that it is **recursive** — to find the type of `(a + b) * c`, the analyzer first finds the type of `a + b` (which requires finding the types of `a` and `b` separately), and then uses that result to check the `*` operation with `c`. This naturally handles any level of nesting in expressions.

That is the full picture of type checking. We will flesh out every detail — including error recovery and helpful messages — in the Semantic Analysis chapter. For now, you understand the core idea: a type environment, type inference on expressions, and comparison against declared annotations.
