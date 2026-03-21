---
title: "Building the Parser"
description: "Build the parser that turns tokens into an AST using recursive descent."
---

Now let's build the parser! We will first define the AST types, then write the parser itself.

## Define the AST Types

Open `src/ast.rs` and write all the AST node types:

```rust
// ast.rs — Defines all the AST (Abstract Syntax Tree) node types
//
// There are two main kinds of nodes:
//   1. Stmt  — a Statement (does something)
//   2. Expr  — an Expression (produces a value)

// ================================================================
//  Expressions
// ================================================================

// BinOp — binary operators (operations with two sides)
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,  // +
    Sub,  // -
    Mul,  // *
    Div,  // /
    Eq,   // ==
    Ne,   // !=
    Lt,   // <
    Gt,   // >
    Le,   // <=
    Ge,   // >=
}

// Expr — all the different kinds of expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),        // 42, 3.14
    Str(String),        // "hello"
    Bool(bool),         // true, false
    Ident(String),      // variable name

    // A binary operation like x + y
    Binary {
        left:  Box<Expr>, // Box because the size is unknown at compile time
        op:    BinOp,
        right: Box<Expr>,
    },

    // A function call like add(1, 2)
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

// ================================================================
//  Statements
// ================================================================

#[derive(Debug, Clone)]
pub enum Stmt {
    // let x = expr;
    Let {
        name:  String,
        value: Expr,
    },

    // return expr;
    Return(Expr),

    // if condition { then } else { else_block }
    If {
        condition:  Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>, // else is optional
    },

    // fn name(params) { body }
    Function {
        name:   String,
        params: Vec<String>,
        body:   Vec<Stmt>,
    },

    // print(expr);
    Print(Expr),

    // An expression used as a statement
    ExprStmt(Expr),
}

// A Program is just a list of top-level statements
pub type Program = Vec<Stmt>;
```

> **Rust note:** We use `Box<Expr>` for recursive types. If `Expr` contained `Expr` directly (without a Box), Rust could not calculate the size at compile time. `Box` stores the value on the heap, which solves this.

## The Parser Struct

Open `src/parser.rs`:

```rust
// parser.rs — Turns a list of Tokens into an AST

use crate::token::{Token, TokenKind};
use crate::ast::{BinOp, Expr, Stmt, Program};

pub struct Parser {
    tokens: Vec<Token>, // all tokens from the lexer
    pos: usize,         // current position in the tokens list
}

impl Parser {
    // new() — Create a parser from a token list
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // current() — Return reference to current token
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(self.tokens.last().unwrap())
    }

    // peek() — Look ahead at the next token
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(self.tokens.last().unwrap())
    }

    // advance() — Move forward and return the token we just passed
    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    // expect() — Consume the current token if it matches, or panic
    fn expect(&mut self, expected: &TokenKind) -> &Token {
        if &self.current().kind == expected {
            self.advance()
        } else {
            panic!(
                "Expected {:?} but found {:?} at line {}",
                expected,
                self.current().kind,
                self.current().span.line
            );
        }
    }

    // check() — Return true if current token matches
    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    // is_at_end() — Return true if we reached end of file
    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }
}
```

## Parser Helper Methods

The `peek()` method above is important. It lets us look ahead **one token** without consuming it. We need this when we see an identifier — is it just a variable name, or is it followed by `(` to make a function call?

## Parsing Statements

```rust
impl Parser {
    // parse() — Parse the entire program
    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }
        statements
    }

    // parse_statement() — Parse one statement
    fn parse_statement(&mut self) -> Stmt {
        match self.current().kind.clone() {
            TokenKind::Let    => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If     => self.parse_if(),
            TokenKind::Fn     => self.parse_function(),
            TokenKind::Print  => self.parse_print(),
            _                 => self.parse_expr_stmt(),
        }
    }

    // parse_let() — Parse:  let name = expr;
    fn parse_let(&mut self) -> Stmt {
        self.advance(); // consume 'let'
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => panic!("Expected identifier after 'let'"),
        };
        self.expect(&TokenKind::Equals);
        let value = self.parse_expression();
        self.expect(&TokenKind::Semicolon);
        Stmt::Let { name, value }
    }

    // parse_return() — Parse:  return expr;
    fn parse_return(&mut self) -> Stmt {
        self.advance(); // consume 'return'
        let value = self.parse_expression();
        self.expect(&TokenKind::Semicolon);
        Stmt::Return(value)
    }

    // parse_if() — Parse:  if condition { ... } else { ... }
    fn parse_if(&mut self) -> Stmt {
        self.advance(); // consume 'if'
        let condition = self.parse_expression();

        self.expect(&TokenKind::LBrace);
        let then_block = self.parse_block();

        let else_block = if self.check(&TokenKind::Else) {
            self.advance(); // consume 'else'
            self.expect(&TokenKind::LBrace);
            Some(self.parse_block())
        } else {
            None
        };

        Stmt::If { condition, then_block, else_block }
    }

    // parse_block() — Parse a list of statements until we see '}'
    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement());
        }
        self.expect(&TokenKind::RBrace); // consume '}'
        stmts
    }

    // parse_function() — Parse:  fn name(params) { body }
    fn parse_function(&mut self) -> Stmt {
        self.advance(); // consume 'fn'
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => panic!("Expected function name"),
        };

        self.expect(&TokenKind::LParen);
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            match self.advance().kind.clone() {
                TokenKind::Ident(p) => params.push(p),
                _ => panic!("Expected parameter name"),
            }
            if self.check(&TokenKind::Comma) {
                self.advance(); // skip comma between params
            }
        }
        self.expect(&TokenKind::RParen);
        self.expect(&TokenKind::LBrace);
        let body = self.parse_block();

        Stmt::Function { name, params, body }
    }

    // parse_print() — Parse:  print(expr);
    fn parse_print(&mut self) -> Stmt {
        self.advance(); // consume 'print'
        self.expect(&TokenKind::LParen);
        let value = self.parse_expression();
        self.expect(&TokenKind::RParen);
        self.expect(&TokenKind::Semicolon);
        Stmt::Print(value)
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expression();
        self.expect(&TokenKind::Semicolon);
        Stmt::ExprStmt(expr)
    }
}
```

## Parsing Expressions

Now the expression parser. We use the precedence chain we talked about in the previous chapter:

```rust
impl Parser {
    // parse_expression() — Top-level expression parser
    pub fn parse_expression(&mut self) -> Expr {
        self.parse_comparison()
    }

    // parse_comparison() — Handles ==, !=, <, >, <=, >=
    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_addition();

        loop {
            let op = match self.current().kind {
                TokenKind::EqEq   => BinOp::Eq,
                TokenKind::BangEq => BinOp::Ne,
                TokenKind::Lt     => BinOp::Lt,
                TokenKind::Gt     => BinOp::Gt,
                TokenKind::LtEq   => BinOp::Le,
                TokenKind::GtEq   => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition();
            left = Expr::Binary {
                left:  Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    // parse_addition() — Handles + and -
    fn parse_addition(&mut self) -> Expr {
        let mut left = self.parse_multiplication();

        loop {
            let op = match self.current().kind {
                TokenKind::Plus  => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication();
            left = Expr::Binary {
                left:  Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    // parse_multiplication() — Handles * and /
    fn parse_multiplication(&mut self) -> Expr {
        let mut left = self.parse_primary();

        loop {
            let op = match self.current().kind {
                TokenKind::Star  => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_primary();
            left = Expr::Binary {
                left:  Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    // parse_primary() — Literals, identifiers, grouped expressions
    fn parse_primary(&mut self) -> Expr {
        match self.current().kind.clone() {
            TokenKind::Number(n) => {
                self.advance();
                Expr::Number(n)
            }
            TokenKind::StringLit(s) => {
                self.advance();
                Expr::Str(s)
            }
            TokenKind::True  => { self.advance(); Expr::Bool(true) }
            TokenKind::False => { self.advance(); Expr::Bool(false) }

            TokenKind::Ident(name) => {
                self.advance();
                // Is the next token '('? Then this is a function call!
                if self.check(&TokenKind::LParen) {
                    self.parse_call(name)
                } else {
                    Expr::Ident(name)
                }
            }

            // Grouped expression: (expr)
            TokenKind::LParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression();
                self.expect(&TokenKind::RParen);
                expr
            }

            other => panic!("Unexpected token in expression: {:?}", other),
        }
    }
}
```

## Parsing Function Calls

```rust
impl Parser {
    // parse_call() — Parse:  name(arg1, arg2, ...)
    // We already consumed 'name', now we handle '(args)'
    fn parse_call(&mut self, name: String) -> Expr {
        self.advance(); // consume '('

        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            args.push(self.parse_expression());
            if self.check(&TokenKind::Comma) {
                self.advance(); // skip comma between arguments
            }
        }

        self.expect(&TokenKind::RParen); // consume ')'
        Expr::Call { name, args }
    }
}
```

## Testing the Parser

Add tests at the bottom of `parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::ast::{Stmt, Expr, BinOp};

    fn parse_source(src: &str) -> Program {
        let tokens = tokenize(src);
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_let_statement() {
        let program = parse_source("let x = 42;");
        assert_eq!(program.len(), 1);

        match &program[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");
                match value {
                    Expr::Number(n) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected number"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_binary_expression_precedence() {
        // '1 + 2 * 3' should parse as '1 + (2 * 3)' because * has higher precedence
        let program = parse_source("let z = 1 + 2 * 3;");
        match &program[0] {
            Stmt::Let { value, .. } => {
                match value {
                    Expr::Binary { op: BinOp::Add, right, .. } => {
                        match right.as_ref() {
                            Expr::Binary { op: BinOp::Mul, .. } => {} // correct!
                            _ => panic!("Expected multiplication on right side"),
                        }
                    }
                    _ => panic!("Expected addition"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }
}
```

Run tests:

```bash
cargo test
```

The parser is done! In the next chapter, we will add semantic analysis to catch errors in user code.
