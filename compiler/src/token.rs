// =============================================================================
// token.rs — Token definitions for the Pico language
// =============================================================================
//
// A "token" is the smallest meaningful unit of source code.
// The lexer reads raw text and produces a stream of tokens.
//
// Example: the source text  `let x = 42;`  becomes:
//
//   Token { kind: Let,          span: line 1, col 1 }
//   Token { kind: Ident("x"),   span: line 1, col 5 }
//   Token { kind: Equals,       span: line 1, col 7 }
//   Token { kind: Number(42.0), span: line 1, col 9 }
//   Token { kind: Semicolon,    span: line 1, col 11 }
//   Token { kind: Eof,          span: line 1, col 12 }
//
// Each token has:
//   1. A `TokenKind`  — what *type* of token it is (keyword, number, operator…)
//   2. A `Span`       — where in the source file it appeared (line + column)
//
// We store position info so error messages can say "Error at line 3, col 5"
// instead of just "Error somewhere".
// =============================================================================

// -----------------------------------------------------------------------------
// TokenKind — an enum that covers every possible token in Pico
// -----------------------------------------------------------------------------
//
// `#[derive(Debug)]`    lets us print it with `{:?}` for debugging.
// `#[derive(Clone)]`    lets us copy a TokenKind (needed because the parser
//                       sometimes needs to keep a copy while advancing forward).
// `#[derive(PartialEq)]` lets us compare with `==` (used in `expect()`).
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // -------------------------------------------------------------------------
    // Literals — actual values written in source code
    // -------------------------------------------------------------------------

    /// A floating-point number literal.  e.g. `42`, `3.14`, `0.5`
    /// We store all numbers as f64 even when they look like integers.
    /// The code generator later strips the ".0" for whole numbers.
    Number(f64),

    /// A string literal.  e.g. `"hello"`, `"Alice"`
    /// The surrounding double-quotes are stripped; escape sequences
    /// like `\n` and `\"` are resolved during lexing.
    StringLit(String),

    /// The keyword `true`
    True,

    /// The keyword `false`
    False,

    // -------------------------------------------------------------------------
    // Identifiers
    // -------------------------------------------------------------------------

    /// Any name that is not a keyword.  e.g. `x`, `myVariable`, `add`
    /// The lexer checks keywords first; anything that doesn't match becomes
    /// an `Ident`.
    Ident(String),

    // -------------------------------------------------------------------------
    // Keywords — reserved words with special meaning in Pico
    // -------------------------------------------------------------------------

    /// `let`    — starts a variable declaration:  `let x = 5;`
    Let,
    /// `fn`     — starts a function definition:   `fn add(a, b) { ... }`
    Fn,
    /// `return` — returns a value from a function: `return a + b;`
    Return,
    /// `if`     — starts a conditional block:      `if x > 0 { ... }`
    If,
    /// `else`   — the alternative branch of an if: `else { ... }`
    Else,
    /// `print`  — built-in print statement:        `print(x);`
    Print,
    /// `struct` — starts a struct definition:      `struct Point { x: float, y: float }`
    Struct,

    // -------------------------------------------------------------------------
    // Type keywords — used in type annotations
    //   e.g.  `let age: int = 25;`
    //         `fn add(a: int, b: int): int { ... }`
    // -------------------------------------------------------------------------

    /// `int`   — integer type
    TyInt,
    /// `float` — floating-point type
    TyFloat,
    /// `str`   — string type
    TyStr,
    /// `bool`  — boolean type
    TyBool,

    // -------------------------------------------------------------------------
    // Operators
    // -------------------------------------------------------------------------

    /// `+`  addition
    Plus,
    /// `-`  subtraction
    Minus,
    /// `*`  multiplication
    Star,
    /// `/`  division
    Slash,
    /// `=`  assignment  (single equals)
    Equals,
    /// `==` equality check  (double equals)
    EqEq,
    /// `!=` not-equal check
    BangEq,
    /// `<`  less-than
    Lt,
    /// `>`  greater-than
    Gt,
    /// `<=` less-than-or-equal
    LtEq,
    /// `>=` greater-than-or-equal
    GtEq,

    // -------------------------------------------------------------------------
    // Punctuation
    // -------------------------------------------------------------------------

    /// `(`  left parenthesis  — function calls, grouped expressions
    LParen,
    /// `)`  right parenthesis
    RParen,
    /// `{`  left brace  — opens a block of code
    LBrace,
    /// `}`  right brace — closes a block of code
    RBrace,
    /// `,`  comma  — separates items in a list
    Comma,
    /// `;`  semicolon — ends a statement
    Semicolon,
    /// `:`  colon  — used for type annotations:  `let x: int = 5`
    Colon,
    /// `.`  dot    — used for property access:   `p.name`
    Dot,

    // -------------------------------------------------------------------------
    // Special tokens
    // -------------------------------------------------------------------------

    /// End-of-file.  The lexer emits this as the very last token so the
    /// parser always has a token to look at even after the source runs out.
    /// Without this, every `current()` call in the parser would need an
    /// `Option` check — `Eof` simplifies the parser significantly.
    Eof,
}

// -----------------------------------------------------------------------------
// Span — source-location information for a token
// -----------------------------------------------------------------------------
//
// Storing line/column lets us produce precise error messages like:
//
//   Error: 'z' is not defined.  (line 4, col 9)
//
// instead of the unhelpful "Error: 'z' is not defined."
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// Line number in the source file.  Starts at 1 (not 0).
    pub line: usize,
    /// Column number (character offset) on that line.  Starts at 1.
    pub col: usize,
}

// -----------------------------------------------------------------------------
// Token — combines a kind with its source location
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// What type of token this is.
    pub kind: TokenKind,
    /// Where in the source file this token appeared.
    pub span: Span,
}

impl Token {
    /// Convenience constructor so callers can write
    /// `Token::new(TokenKind::Let, 1, 5)` instead of building the structs
    /// manually every time.
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token {
            kind,
            span: Span { line, col },
        }
    }
}
