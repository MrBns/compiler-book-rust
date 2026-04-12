// =============================================================================
// ast.rs — Abstract Syntax Tree (AST) node definitions for Pico
// =============================================================================
//
// After the lexer turns raw text into a flat list of tokens, the PARSER
// takes those tokens and builds a TREE that captures the program's structure.
// This tree is called the Abstract Syntax Tree, or AST.
//
// "Abstract" means we throw away things that don't carry meaning: parentheses
// used just for grouping, semicolons, whitespace.  What remains is the pure
// logical structure of the program.
//
// Example: the source  `let z = 1 + 2 * 3;`
// becomes the AST:
//
//   Stmt::Let {
//     name:  "z",
//     value: Expr::Binary {
//       left:  Expr::Number(1.0),
//       op:    BinOp::Add,
//       right: Expr::Binary {          ← multiplication has higher precedence
//         left:  Expr::Number(2.0),
//         op:    BinOp::Mul,
//         right: Expr::Number(3.0),
//       },
//     },
//   }
//
// Two fundamental node types:
//   • Expr  — an expression that PRODUCES a value  (e.g. `x + 1`, `"hi"`, `add(1, 2)`)
//   • Stmt  — a statement that DOES something      (e.g. `let x = 5;`, `if …`, `return …`)
//
// A complete program is just a list of top-level Stmts.
// =============================================================================

// =============================================================================
// BinOp — binary (two-operand) operators
// =============================================================================
//
// We separate operators into their own enum so:
//   1. Pattern matching on operators is clean and exhaustive.
//   2. We can add operators later without touching the `Expr` enum.
//
// derive macros:
//   Debug    — enables `{:?}` formatting for printing / debugging
//   Clone    — lets us copy a BinOp (needed when we move it into an Expr node)
//   PartialEq — lets us compare two BinOps with `==` (used in tests)
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    /// `+`  numeric addition      or string concatenation (future)
    Add,
    /// `-`  numeric subtraction
    Sub,
    /// `*`  numeric multiplication
    Mul,
    /// `/`  numeric division
    Div,
    /// `==` equality test     →  produces `true` or `false`
    Eq,
    /// `!=` not-equal test
    Ne,
    /// `<`  less-than test
    Lt,
    /// `>`  greater-than test
    Gt,
    /// `<=` less-than-or-equal test
    Le,
    /// `>=` greater-than-or-equal test
    Ge,
}

// =============================================================================
// Expr — an expression node in the AST
// =============================================================================
//
// An expression is anything that EVALUATES to a value.
//   `42`            → a number literal
//   `x`             → look up variable `x` in the environment
//   `1 + 2`         → compute and return the sum
//   `add(1, 2)`     → call the function `add` and return its result
//
// Why `Box<Expr>` for recursive fields?
//   Rust needs to know the SIZE of every type at compile time.
//   If `Expr::Binary` contained `Expr` directly, the size would be infinite:
//     size(Expr) = … + size(Expr) + …   (impossible)
//   `Box<T>` is a heap-allocated pointer — it is always the size of one pointer
//   (8 bytes on 64-bit), no matter what `T` is.  That breaks the cycle.
#[derive(Debug, Clone)]
pub enum Expr {
    // -------------------------------------------------------------------------
    // Literal values — the "leaf" nodes of the expression tree
    // -------------------------------------------------------------------------

    /// A numeric literal.  Stored as `f64` for simplicity.
    /// Examples: `42`, `3.14`, `0`
    Number(f64),

    /// A string literal.  The quotes have been stripped by the lexer.
    /// Example: `"hello"` → `Str("hello".to_string())`
    Str(String),

    /// A boolean literal.
    /// Examples: `true`, `false`
    Bool(bool),

    // -------------------------------------------------------------------------
    // Variable reference
    // -------------------------------------------------------------------------

    /// A reference to a named variable.
    /// Example: in  `let z = x + 1;`  the  `x`  is  `Expr::Ident("x")`
    Ident(String),

    // -------------------------------------------------------------------------
    // Composite expressions
    // -------------------------------------------------------------------------

    /// A binary operation with a left operand, operator, and right operand.
    ///
    /// Example: `a + b * c` is parsed as:
    ///   Binary {
    ///     left:  Ident("a"),
    ///     op:    BinOp::Add,
    ///     right: Binary { left: Ident("b"), op: BinOp::Mul, right: Ident("c") }
    ///   }
    ///
    /// The parser's precedence chain ensures `*` binds tighter than `+`.
    Binary {
        /// Left-hand side of the operator (e.g. `a` in `a + b`)
        left: Box<Expr>,
        /// The operator  (+, -, ==, <, …)
        op: BinOp,
        /// Right-hand side (e.g. `b` in `a + b`)
        right: Box<Expr>,
    },

    /// A function call.
    ///
    /// Example: `add(1, x + 2)` →
    ///   Call {
    ///     name: "add",
    ///     args: [Number(1.0), Binary { left: Ident("x"), op: Add, right: Number(2.0) }]
    ///   }
    Call {
        /// Name of the function being called (e.g. `"add"`)
        name: String,
        /// The argument expressions passed to the function
        args: Vec<Expr>,
    },
}

// =============================================================================
// Stmt — a statement node in the AST
// =============================================================================
//
// A statement DOES something — it either binds a value to a name, runs
// conditional code, defines a function, or asks for output.
//
// Statements do NOT produce a value (unlike expressions).
// Suppress the "variant name ends with enum name" lint for ExprStmt —
// the Stmt suffix is intentional in AST code, it aids readability.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Stmt {
    // -------------------------------------------------------------------------
    // Variable declaration
    // -------------------------------------------------------------------------

    /// `let name = value;`
    ///
    /// Binds the result of `value` to `name` in the current scope.
    /// Example: `let x = 42;`
    Let {
        /// The variable name being declared (e.g. `"x"`)
        name: String,
        /// The expression whose value is assigned to `name`
        value: Expr,
    },

    // -------------------------------------------------------------------------
    // Return statement  (only valid inside a function body)
    // -------------------------------------------------------------------------

    /// `return expr;`
    ///
    /// Exits the enclosing function and hands `expr`'s value back to the caller.
    Return(Expr),

    // -------------------------------------------------------------------------
    // Conditional branching
    // -------------------------------------------------------------------------

    /// `if condition { then_block } else { else_block }`
    ///
    /// The `else` branch is optional; if absent, `else_block` is `None`.
    If {
        /// The boolean expression to test
        condition: Expr,
        /// Statements to execute when condition is true
        then_block: Vec<Stmt>,
        /// Optional statements to execute when condition is false
        else_block: Option<Vec<Stmt>>,
    },

    // -------------------------------------------------------------------------
    // Function definition
    // -------------------------------------------------------------------------

    /// `fn name(param1, param2, …) { body }`
    ///
    /// Defines a function in the current scope.
    /// In the base version of Pico, parameters have no type annotations
    /// (they are implicitly `any` in the TypeScript output).
    Function {
        /// The function name (e.g. `"add"`)
        name: String,
        /// Parameter names in declaration order (e.g. `["a", "b"]`)
        params: Vec<String>,
        /// The function body — a list of statements
        body: Vec<Stmt>,
    },

    // -------------------------------------------------------------------------
    // Built-in print
    // -------------------------------------------------------------------------

    /// `print(expr);`
    ///
    /// A simple built-in statement that outputs a value to stdout.
    /// It maps to `console.log(expr)` in the TypeScript output.
    Print(Expr),

    // -------------------------------------------------------------------------
    // Expression statement
    // -------------------------------------------------------------------------

    /// An expression used as a statement, discarding the result.
    ///
    /// The most common use is a standalone function call:
    ///   `greet("Alice");`   → ExprStmt(Call { name: "greet", args: [...] })
    ExprStmt(Expr),
}

// =============================================================================
// Program type alias
// =============================================================================

/// A Pico program is simply a list of top-level statements.
///
/// Using a type alias keeps function signatures readable:
///   `fn parse() -> Program`  is nicer than  `fn parse() -> Vec<Stmt>`
pub type Program = Vec<Stmt>;

// Suppress Clippy's `enum_variant_names` lint for `Stmt`.
// In AST definitions it is conventional and clear to suffix statement
// variants with `Stmt` (e.g. `ExprStmt`) so readers know at a glance
// which enum they are looking at.  Renaming would hurt clarity here.
