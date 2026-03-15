---
title: "Structs & Properties"
description: "Add structs with typed properties to Pico and compile them to TypeScript interfaces."
---

# Structs & Properties

A **struct** is a way to group related data together under one name. It is like a template that says: "a `Person` always has a `name` and an `age`."

## What is a Struct?

Without structs, if you want to represent a person you need separate variables:

```
let person_name: str = "Alice";
let person_age:  int = 25;
```

This gets messy fast — especially when you have many people. A struct bundles these fields together:

```
struct Person {
    name: str,
    age:  int,
}
```

Now `Person` is a **type** just like `int` or `str`. You can create instances of it and pass them around.

## Defining a Struct

Use the `struct` keyword, then list each field with its name and type:

```
struct Point {
    x: float,
    y: float,
}

struct Person {
    name: str,
    age:  int,
}

struct Rectangle {
    width:  float,
    height: float,
}
```

Fields are separated by commas. Types are any valid Pico type (`int`, `float`, `str`, `bool`, or even another struct name).

## Creating a Struct Instance

To create a value of a struct type, write the struct name followed by `{ field: value, ... }`:

```
let origin: Point = Point { x: 0.0, y: 0.0 };
let p:      Point = Point { x: 3.0, y: 4.0 };

let alice: Person = Person { name: "Alice", age: 25 };
let bob:   Person = Person { name: "Bob",   age: 30 };
```

The left side declares the variable name and its type. The right side constructs the struct value. The compiler checks that every field is provided with the correct type.

## Accessing Properties

Use a dot `.` to read a field from a struct instance:

```
let alice: Person = Person { name: "Alice", age: 25 };

print(alice.name);   // prints: Alice
print(alice.age);    // prints: 25
```

Property access can appear in any expression:

```
fn greet(p: Person): str {
    return "Hello, " + p.name;
}

let msg: str = greet(alice);
print(msg);   // prints: Hello, Alice
```

## Structs as Function Parameters

You can pass a struct to a function just like any other value:

```
struct Rectangle {
    width:  float,
    height: float,
}

fn area(r: Rectangle): float {
    return r.width * r.height;
}

fn perimeter(r: Rectangle): float {
    return 2.0 * (r.width + r.height);
}

let rect: Rectangle = Rectangle { width: 5.0, height: 3.0 };
let a: float = area(rect);          // 15.0
let p: float = perimeter(rect);     // 16.0

print(a);
print(p);
```

## How Structs Compile to TypeScript

Pico structs map to TypeScript **interfaces**:

| Pico | TypeScript |
|------|-----------|
| `struct Person { name: str, age: int }` | `interface Person { name: string; age: number; }` |
| `Person { name: "Alice", age: 25 }` | `{ name: "Alice", age: 25 }` |
| `alice.name` | `alice.name` |

So this Pico program:

```
struct Point {
    x: float,
    y: float,
}

fn distance(a: Point, b: Point): float {
    let dx: float = a.x - b.x;
    let dy: float = a.y - b.y;
    return dx * dx + dy * dy;
}

let p1: Point = Point { x: 0.0, y: 0.0 };
let p2: Point = Point { x: 3.0, y: 4.0 };
let d:  float = distance(p1, p2);
print(d);
```

Compiles to this TypeScript:

```typescript
interface Point {
    x: number;
    y: number;
}

function distance(a: Point, b: Point): number {
    const dx: number = (a.x - b.x);
    const dy: number = (a.y - b.y);
    return ((dx * dx) + (dy * dy));
}

const p1: Point = { x: 0.0, y: 0.0 };
const p2: Point = { x: 3.0, y: 4.0 };
const d: number = distance(p1, p2);
console.log(d);
```

Clean and readable TypeScript!

## New Tokens Needed

Supporting structs requires a few more tokens in the lexer:

```rust
// New keyword
Struct, // 'struct'

// Already added for types
Dot,    // '.'  — property access: p.name
Colon,  // ':'  — field type annotation: name: str
```

## New AST Nodes

We need two new AST nodes:

```rust
// StructDef — the definition of a struct type
pub struct StructDef {
    pub name:   String,
    pub fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: String,
    pub ty:   TypeNode,
}

// StructExpr — creating a struct value
pub struct StructExpr {
    pub struct_name: String,
    pub fields:      Vec<(String, Box<Expr>)>, // field name → value
}

// FieldAccess — reading a field: expr.field
pub struct FieldAccess {
    pub object: Box<Expr>,
    pub field:  String,
}
```

## Type Checking Structs

When we see a struct definition, we register it in the **type environment** just like variables:

```rust
// Register the struct type so other code can use it
fn check_struct_def(&mut self, def: &StructDef, env: &mut TypeEnv) {
    env.register_struct(def.name.clone(), def.fields.clone());
}
```

When we see a struct expression (creating an instance):

```rust
fn check_struct_expr(&mut self, expr: &StructExpr, env: &TypeEnv) -> TypeNode {
    // Look up the struct definition
    let struct_def = env.get_struct(&expr.struct_name)
        .expect("unknown struct type");

    // Check every provided field
    for (field_name, field_value) in &expr.fields {
        let expected = struct_def.field_type(field_name)
            .expect("unknown field");
        let actual = self.infer_type(field_value, env);
        if expected != actual {
            self.error(format!(
                "field `{}`: expected {:?}, found {:?}",
                field_name, expected, actual
            ));
        }
    }

    // The type of the whole expression is the struct type
    TypeNode::Named(expr.struct_name.clone())
}
```

When we see a field access (`p.name`):

```rust
fn check_field_access(&mut self, expr: &FieldAccess, env: &TypeEnv) -> TypeNode {
    let obj_type = self.infer_type(&expr.object, env);

    // The object must be a named struct type
    if let TypeNode::Named(struct_name) = obj_type {
        let struct_def = env.get_struct(&struct_name)
            .expect("unknown struct");
        struct_def.field_type(&expr.field)
            .expect("struct has no such field")
            .clone()
    } else {
        self.error(format!(
            "cannot access field `{}` on non-struct type",
            expr.field
        ));
        TypeNode::Int // placeholder
    }
}
```

## Summary

| Feature | Syntax |
|---------|--------|
| Define a struct | `struct Name { field: Type, ... }` |
| Create an instance | `Name { field: value, ... }` |
| Access a field | `instance.field` |
| Pass to a function | `fn f(x: Name): RetType { ... }` |
| Compiles to | TypeScript `interface` + object literal |

Structs make Pico much more expressive. You can model real-world data cleanly, pass it around with full type safety, and the compiler will catch any mistakes for you.

In the next chapters we will implement all of this — first in the lexer, then the parser, then the semantic checker, and finally the code generator.
