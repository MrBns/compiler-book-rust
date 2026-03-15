---
title: "Structs & Properties"
description: "Add structs with typed properties to Pico and compile them to TypeScript interfaces."
---

# Structs & Properties

So far, the only values Pico can work with are single pieces of data — one number, one string, one boolean. That is fine for small programs, but real programs deal with **things** that have multiple pieces of data attached to them. A user has a name and an age. A rectangle has a width and a height. A product has a name, a price, and a stock count.

Without a way to bundle related data together, you end up with a mess of separate variables that are hard to track and easy to mix up. In this chapter we introduce **structs** — a way to define your own custom data types that group named fields together under a single type name. Structs are one of the most fundamental building blocks in typed languages, and they make Pico dramatically more expressive.

## The Problem Structs Solve

Let us say you are writing a program that works with two-dimensional points. Without structs you might write:

```
let p1_x: float = 3.0;
let p1_y: float = 4.0;
let p2_x: float = 0.0;
let p2_y: float = 0.0;
```

This already looks a bit unwieldy. But as soon as you need to pass a "point" to a function, things fall apart:

```
// How do you write this function? You have to pass x and y separately.
fn distance(ax: float, ay: float, bx: float, by: float): float {
    let dx: float = ax - bx;
    let dy: float = ay - by;
    return dx * dx + dy * dy;
}

let d: float = distance(p1_x, p1_y, p2_x, p2_y);
```

Four parameters just to describe two points. And what if you mix them up? What if you accidentally write `distance(p1_x, p2_y, p1_y, p2_x)`? The compiler cannot help you — all four arguments are `float`, so the types match even though the values are wrong.

A struct solves this cleanly:

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

let p1: Point = Point { x: 3.0, y: 4.0 };
let p2: Point = Point { x: 0.0, y: 0.0 };
let d:  float = distance(p1, p2);
```

Now the function takes two `Point` values. The fields are named. You cannot mix them up. The compiler will reject `distance(p1, 42)` because `42` is not a `Point`. The code is shorter, safer, and far easier to read.

## What is a Struct?

A **struct** (short for *structure*) is a user-defined type that bundles together a fixed set of named, typed fields. Once you define a struct, its name becomes a **first-class type** in Pico — you can use it anywhere you would use `int`, `str`, `bool`, or `float`.

Think of a struct definition as a **blueprint**. The blueprint says: "every `Person` has exactly one `name` field of type `str` and one `age` field of type `int`." Every time you create a `Person` value, it must provide both of those fields, with the correct types. There is no way to accidentally create a `Person` that is missing a field, or that has an `age` stored as a string.

## Defining a Struct

You define a struct with the `struct` keyword, followed by the struct's name, an opening brace, the list of fields, and a closing brace. Each field is written as `fieldname: Type`, and fields are separated by commas.

```
struct Person {
    name: str,
    age:  int,
}
```

Here are several more examples. Notice how each struct captures exactly the data it needs and nothing more:

```
// A 2D point in space
struct Point {
    x: float,
    y: float,
}

// A rectangle defined by its dimensions
struct Rectangle {
    width:  float,
    height: float,
}

// A circle defined by its center and radius
struct Circle {
    center_x: float,
    center_y: float,
    radius:   float,
}

// A product in a store inventory
struct Product {
    name:     str,
    price:    float,
    in_stock: bool,
}

// A student record
struct Student {
    full_name:  str,
    student_id: int,
    gpa:        float,
    enrolled:   bool,
}
```

Fields can use any of Pico's four built-in types — `int`, `float`, `str`, `bool` — or the name of another struct you have already defined. For example, a `Circle` could be defined using a `Point` struct for its center:

```
struct Point {
    x: float,
    y: float,
}

struct Circle {
    center: Point,   // a field whose type is another struct
    radius: float,
}
```

This is called **struct composition** — building complex types out of simpler ones. We keep this simple in our first implementation, but it is good to know the design supports it.

## Creating a Struct Instance

Defining a struct just creates the blueprint. To actually use a struct, you need to create an **instance** — a concrete value with real data in its fields.

The syntax for creating an instance is: write the struct name, then `{ fieldname: value, ... }` inside braces. Every field defined in the struct must be provided, in any order.

```
let origin: Point = Point { x: 0.0, y: 0.0 };
let corner: Point = Point { x: 10.0, y: 5.0 };

let alice: Person = Person { name: "Alice", age: 25 };
let bob:   Person = Person { name: "Bob",   age: 30 };
let child: Person = Person { name: "Charlie", age: 8 };

let box1: Rectangle = Rectangle { width: 100.0, height: 50.0 };
let box2: Rectangle = Rectangle { width: 200.0, height: 75.0 };
```

The variable on the left is typed as the struct (e.g., `let alice: Person`). The value on the right constructs the struct (e.g., `Person { ... }`). The compiler checks that:

1. The struct name on the right matches the declared type on the left.
2. Every field listed in the struct definition is provided in the initializer.
3. Every field value has the correct type — `name` must be a `str`, `age` must be an `int`, and so on.

If any field is missing, has the wrong type, or if you try to assign a `Person` to a variable declared as `Point`, the compiler will tell you immediately:

```
let bad1: Person = Person { name: "Alice" };                 // ERROR: field `age` is missing
let bad2: Person = Person { name: "Alice", age: "twenty" };  // ERROR: field `age` — expected int, found str
let bad3: Point  = Person { name: "Alice", age: 25 };        // ERROR: expected Point, found Person
```

## Accessing Properties

Once you have a struct instance, you read its fields using the **dot operator** `.`. Write the variable name, then a dot, then the field name:

```
let alice: Person = Person { name: "Alice", age: 25 };

print(alice.name);   // output: Alice
print(alice.age);    // output: 25
```

Property access works inside any expression. The type of `alice.name` is `str` (because that is how `name` was declared in the `Person` struct), and the type of `alice.age` is `int`. The compiler knows both of these and will type-check any further operations on those values.

```
let alice: Person = Person { name: "Alice", age: 25 };
let bob:   Person = Person { name: "Bob",   age: 30 };

// We can use field values in expressions
let combined_age: int  = alice.age + bob.age;    // 55
let greeting:     str  = "Hello, " + alice.name; // "Hello, Alice"
let is_senior:    bool = alice.age >= 65;         // false
```

You can also access fields of a struct that is itself a field of another struct, using chained dots:

```
struct Point  { x: float, y: float }
struct Circle { center: Point, radius: float }

let c: Circle = Circle {
    center: Point { x: 3.0, y: 4.0 },
    radius: 5.0,
};

print(c.radius);     // 5.0
print(c.center.x);   // 3.0  — first access center (a Point), then access x on that Point
print(c.center.y);   // 4.0
```

## Passing Structs to Functions

One of the biggest benefits of structs is that they let you pass logically-related data to functions as a single, named argument. Compare the messy version with the struct version:

```
// WITHOUT structs — confusing parameter list
fn area_no_struct(width: float, height: float): float {
    return width * height;
}

// WITH structs — clear and self-documenting
fn area(rect: Rectangle): float {
    return rect.width * rect.height;
}
```

The struct version is not only shorter — it is also *safer*. If you have a `width` and a `height` as separate variables, you could accidentally swap them. With a `Rectangle` struct, the fields are named and the caller cannot mix them up.

Here is a more complete example that puts all the pieces together:

```
struct Rectangle {
    width:  float,
    height: float,
}

// Calculate the area of a rectangle
fn area(r: Rectangle): float {
    return r.width * r.height;
}

// Calculate the perimeter of a rectangle
fn perimeter(r: Rectangle): float {
    return 2.0 * (r.width + r.height);
}

// Check if one rectangle fits inside another (without rotation)
fn fits_inside(inner: Rectangle, outer: Rectangle): bool {
    return inner.width <= outer.width;
}

// Create some rectangles and do calculations with them
let small: Rectangle = Rectangle { width: 3.0,  height: 4.0 };
let large: Rectangle = Rectangle { width: 10.0, height: 8.0 };

let a1: float = area(small);        // 12.0
let a2: float = area(large);        // 80.0
let p1: float = perimeter(small);   // 14.0
let fits: bool = fits_inside(small, large);  // true

print(a1);
print(a2);
print(p1);
print(fits);
```

The code reads almost like plain English. The function names and field names together tell the story: "area of a rectangle", "perimeter of a rectangle", "small fits inside large". This is the power of named, typed data.

## Returning Structs from Functions

Functions can also *return* a struct value, just like they can return an `int` or a `str`. This is useful for "factory functions" that create a struct based on some inputs:

```
struct Point {
    x: float,
    y: float,
}

// Create a point at the midpoint between two other points
fn midpoint(a: Point, b: Point): Point {
    return Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    };
}

let p1: Point = Point { x: 0.0, y: 0.0 };
let p2: Point = Point { x: 4.0, y: 6.0 };
let mid: Point = midpoint(p1, p2);

print(mid.x);   // 2.0
print(mid.y);   // 3.0
```

The return type annotation `: Point` tells both the programmer and the compiler that this function produces a `Point`. If the `return` expression inside is not a `Point`, the compiler will catch it.

## How Structs Compile to TypeScript

In TypeScript, the natural equivalent of a Pico `struct` is an **interface**. A TypeScript interface defines the shape of an object — exactly the same idea as a Pico struct.

Here is the full translation table:

| Pico construct | TypeScript equivalent |
|---|---|
| `struct Name { field: Type, ... }` | `interface Name { field: TSType; ... }` |
| `let x: Name = Name { field: val }` | `const x: Name = { field: val }` |
| `x.field` | `x.field` |
| `fn f(p: Name): RetType` | `function f(p: Name): TSType` |
| `int` field | `number` |
| `float` field | `number` |
| `str` field | `string` |
| `bool` field | `boolean` |

Let us walk through a complete example. Here is a Pico program that defines a `Student` struct and operates on it:

**Pico source:**

```
struct Student {
    full_name:  str,
    student_id: int,
    gpa:        float,
    enrolled:   bool,
}

fn is_honor_roll(s: Student): bool {
    return s.gpa >= 3.7;
}

fn describe(s: Student): str {
    return "Student #" + s.student_id + ": " + s.full_name;
}

let alice: Student = Student {
    full_name:  "Alice Chen",
    student_id: 1042,
    gpa:        3.9,
    enrolled:   true,
};

print(describe(alice));
print(is_honor_roll(alice));
```

**Compiled TypeScript output:**

```typescript
interface Student {
    full_name: string;
    student_id: number;
    gpa: number;
    enrolled: boolean;
}

function is_honor_roll(s: Student): boolean {
    return (s.gpa >= 3.7);
}

function describe(s: Student): string {
    return ((("Student #" + String(s.student_id)) + ": ") + s.full_name);
}

const alice: Student = {
    full_name: "Alice Chen",
    student_id: 1042,
    gpa: 3.9,
    enrolled: true,
};

console.log(describe(alice));
console.log(is_honor_roll(alice));
```

Notice how clean and idiomatic the TypeScript output is. The Pico struct becomes a TypeScript interface, the function signatures stay the same, and the struct instance becomes a plain TypeScript object literal. Any TypeScript developer reading this output would immediately understand it.

## New Tokens the Lexer Needs

To support structs, the lexer needs to recognize one new keyword and two pieces of punctuation that were already planned for type annotations:

```rust
// New keyword
Struct,  // the keyword 'struct' — starts a struct definition

// Already needed for type annotations (from the previous chapter)
Colon,   // ':'  — separates field name from type:  name: str
Dot,     // '.'  — property access operator:         alice.name
```

The `Dot` token is a single character `.`. The lexer just emits it whenever it sees that character (while not inside a number literal — `3.14` is a `Number` token, not `Number(3)` + `Dot` + `Number(14)`). This distinction is easy to handle because a `.` after digits is always part of the number, while a `.` after an identifier is always property access.

## New AST Nodes for the Parser

The parser needs three new AST node types to represent the three new operations that involve structs:

```rust
// ---- StructDef ----
// Represents the definition of a struct type.
// Example:  struct Person { name: str, age: int }
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name:   String,        // the struct's type name, e.g. "Person"
    pub fields: Vec<FieldDef>, // the list of fields with their types
    pub span:   Span,
}

// ---- FieldDef ----
// One field inside a struct definition.
// Example: the `name: str` part of a struct
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,    // field name, e.g. "name"
    pub ty:   TypeNode,  // field type, e.g. TypeNode::Str
}

// ---- StructExpr ----
// Represents creating a new struct instance.
// Example:  Person { name: "Alice", age: 25 }
#[derive(Debug, Clone)]
pub struct StructExpr {
    pub struct_name: String,                   // "Person"
    pub fields:      Vec<(String, Box<Expr>)>, // [("name", StringLit("Alice")), ("age", Number(25))]
    pub span:        Span,
}

// ---- FieldAccess ----
// Represents reading a field from a struct instance.
// Example:  alice.name
#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub object: Box<Expr>,  // the expression to the left of the dot  (alice)
    pub field:  String,     // the field name to the right of the dot (name)
    pub span:   Span,
}
```

These four types cover everything: defining a struct, creating an instance, and reading a field.

## Grammar Rules for the Parser

The parser needs to recognise the new syntax. In terms of grammar rules, here is what changes:

```
// A program can now contain struct definitions at the top level
Statement → LetDecl | FnDef | StructDef | IfStmt | ReturnStmt | PrintStmt | ExprStmt

// Struct definition rule
StructDef → 'struct' IDENT '{' FieldList '}'
FieldList → FieldDef (',' FieldDef)* ','?
FieldDef  → IDENT ':' Type

// Creating a struct value (this is a primary expression)
Primary → ... | IDENT '{' FieldInitList '}'
FieldInitList → FieldInit (',' FieldInit)*
FieldInit     → IDENT ':' Expr

// Property access (this binds tightly, like function calls)
Postfix → Primary ('.' IDENT)*
```

The important point about the grammar is that `IDENT '{' ... }'` for struct creation and `IDENT` for a regular variable reference look similar at first. The parser tells them apart by **peeking** at the token after the identifier. If the next token is `{`, it is a struct expression; otherwise it is a plain variable reference.

## How the Semantic Analyzer Checks Structs

The semantic analyzer needs to handle three new situations: struct definitions, struct expressions, and field accesses. Each one builds on the type environment we described in the previous chapter.

### Checking a Struct Definition

When the analyzer encounters a `StructDef`, it registers the struct's name and fields in the type environment so that later code can use it as a type:

```rust
fn check_struct_def(
    &mut self,
    def: &StructDef,
    env: &mut TypeEnv,
) -> Result<(), SemanticError> {
    // Make sure no struct with this name was already defined
    if env.struct_exists(&def.name) {
        return Err(SemanticError::DuplicateStruct { name: def.name.clone() });
    }

    // Check that there are no duplicate field names within this struct
    let mut seen_fields = std::collections::HashSet::new();
    for field in &def.fields {
        if !seen_fields.insert(&field.name) {
            return Err(SemanticError::DuplicateField {
                struct_name: def.name.clone(),
                field_name:  field.name.clone(),
            });
        }
    }

    // Register the struct in the type environment
    env.register_struct(def.name.clone(), def.fields.clone());
    Ok(())
}
```

### Checking a Struct Expression

When the analyzer encounters code like `Person { name: "Alice", age: 25 }`, it needs to verify that `Person` is a known struct and that every field is present with the right type:

```rust
fn check_struct_expr(
    &mut self,
    expr: &StructExpr,
    env:  &TypeEnv,
) -> Result<TypeNode, SemanticError> {
    // Look up the struct definition by name
    let struct_def = env.get_struct(&expr.struct_name).ok_or_else(|| {
        SemanticError::UnknownType { name: expr.struct_name.clone() }
    })?;

    // Make sure all required fields are present
    for field_def in &struct_def.fields {
        let provided = expr.fields.iter().find(|(n, _)| n == &field_def.name);
        if provided.is_none() {
            return Err(SemanticError::MissingField {
                struct_name: expr.struct_name.clone(),
                field_name:  field_def.name.clone(),
            });
        }
    }

    // Type-check each field value against the declared field type
    for (field_name, field_value) in &expr.fields {
        let expected_type = struct_def
            .field_type(field_name)
            .ok_or_else(|| SemanticError::UnknownField {
                struct_name: expr.struct_name.clone(),
                field_name:  field_name.clone(),
            })?;

        let actual_type = self.infer_expr_type(field_value, env)?;

        if actual_type != *expected_type {
            return Err(SemanticError::TypeMismatch {
                expected: expected_type.clone(),
                found:    actual_type,
                span:     expr.span.clone(),
                hint:     format!(
                    "field `{}` of struct `{}` expects {:?}, but got {:?}",
                    field_name, expr.struct_name, expected_type, actual_type
                ),
            });
        }
    }

    // The whole struct expression has the type of the struct
    Ok(TypeNode::Named(expr.struct_name.clone()))
}
```

### Checking a Field Access

When the analyzer encounters `alice.name`, it needs to determine the type of `alice` (which must be a struct type), look up the field in that struct's definition, and return the field's type:

```rust
fn check_field_access(
    &mut self,
    expr: &FieldAccess,
    env:  &TypeEnv,
) -> Result<TypeNode, SemanticError> {
    // Find out the type of the expression to the left of the dot
    let object_type = self.infer_expr_type(&expr.object, env)?;

    // The object's type must be a named (struct) type, not a primitive
    let struct_name = match object_type {
        TypeNode::Named(ref name) => name.clone(),
        _ => {
            return Err(SemanticError::NotAStruct {
                found: object_type,
                field: expr.field.clone(),
                span:  expr.span.clone(),
            });
        }
    };

    // Look up the struct definition and find the field
    let struct_def = env.get_struct(&struct_name).ok_or_else(|| {
        SemanticError::UnknownType { name: struct_name.clone() }
    })?;

    let field_type = struct_def.field_type(&expr.field).ok_or_else(|| {
        SemanticError::UnknownField {
            struct_name: struct_name.clone(),
            field_name:  expr.field.clone(),
        }
    })?;

    // Return the type of the field — this is the type of the whole field-access expression
    Ok(field_type.clone())
}
```

This is the complete type-checking logic for structs. Together, these three functions ensure that every struct used in the program is well-formed, every instance creation is correct, and every property access is valid.

## Summary

Let us bring it all together. Here is everything you need to know about structs in Pico:

| Feature | Syntax | What the compiler checks |
|---------|--------|--------------------------|
| Define a struct | `struct Name { field: Type, ... }` | No duplicate names, all types are valid |
| Create an instance | `let x: Name = Name { field: val, ... }` | All fields present, types match |
| Access a field | `x.field` | `x` must be a struct type that has `field` |
| Pass to a function | `fn f(p: Name): RetType { ... }` | Call site must provide a `Name` value |
| Return from a function | `fn f(...): Name { return Name {...}; }` | Returned value must be a `Name` struct |
| Compiles to TypeScript | `struct` → `interface`, instance → object literal | — |

Structs are a cornerstone of well-organized, type-safe code. They let you model the real world in your programs — people, shapes, products, orders, messages — with clarity and safety. Once we have structs working, Pico starts to feel like a real programming language that you can use to write non-trivial programs.

In the next chapter we will begin implementing the lexer, and you will see how all of these new tokens — `struct`, `:`, `.`, `int`, `float`, `str`, `bool` — get recognized from raw source text.
