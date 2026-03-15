---
title: "Semantic Analysis"
description: "Check user code for logical errors like using undefined variables."
---

# Semantic Analysis

The lexer checks if tokens are valid. The parser checks if the syntax is correct. But neither of them checks if the code *makes sense*.

That is the job of **semantic analysis**.

## What Does Semantic Analysis Do?

Semantic analysis catches errors that the parser cannot catch. For example:

```
-- ERROR: 'z' is used but never declared
let x = z + 1;
```

The parser is happy with this — the syntax is valid. But semantically, it is wrong because `z` does not exist.

Here are the checks we will implement:

1. **Undefined variables** — Using a variable that was never declared with `let`
2. **Undefined functions** — Calling a function that does not exist
3. **Duplicate variable names** — Declaring the same variable twice in the same scope

## Symbol Table

To track which variables and functions exist, we use a **symbol table** — basically a list of names that are currently in scope.

When we enter a new block (`{...}`), we create a new scope. When we leave the block, we destroy that scope.

```
Program scope:
  [add, x, y]       ← functions and variables at the top level

Inside 'if' block:
  [add, x, y, temp] ← new scope added on top, can see outer scope too
```

This is called a **scope stack**.

## The Semantic Checker Struct

Open `src/semantic.rs`:

```rust
// semantic.rs — Checks the AST for semantic errors
//
// We walk the entire AST and keep a "scope stack" — a stack of
// HashSets, where each HashSet holds the names defined in that scope.

use std::collections::HashSet;
use crate::ast::{Stmt, Expr, Program};

pub struct SemanticChecker {
    // A stack of scopes.
    // Each scope is a set of variable/function names in that scope.
    // The LAST item in the Vec is the current (innermost) scope.
    scopes: Vec<HashSet<String>>,

    // Collect all errors instead of panicking right away.
    // This way we can show ALL errors to the user at once.
    pub errors: Vec<String>,
}

impl SemanticChecker {
    // new() — Create a checker with one empty global scope
    pub fn new() -> Self {
        SemanticChecker {
            scopes: vec![HashSet::new()], // start with one scope
            errors: Vec::new(),
        }
    }

    // enter_scope() — Push a new (empty) scope onto the stack
    fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    // exit_scope() — Pop the innermost scope (leave the block)
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    // define() — Add a name to the current (innermost) scope
    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    // is_defined() — Check if a name exists in ANY scope (inner to outer)
    fn is_defined(&self, name: &str) -> bool {
        // Search from the innermost scope outward
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    // error() — Record an error message
    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }
}
```

> **Rust note:** `HashSet` is from `std::collections`. It stores unique values and has fast lookup — perfect for a symbol table.

## Checking Statements

```rust
impl SemanticChecker {
    // check_program() — Check the entire program
    pub fn check_program(&mut self, program: &Program) {
        for stmt in program {
            self.check_stmt(stmt);
        }
    }

    // check_stmt() — Check one statement
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                // Check the value expression first
                self.check_expr(value);
                // Then define the variable name in the current scope
                self.define(name);
            }

            Stmt::Return(expr) => {
                self.check_expr(expr);
            }

            Stmt::If { condition, then_block, else_block } => {
                self.check_expr(condition);

                // Check the 'then' block in a new scope
                self.enter_scope();
                for s in then_block {
                    self.check_stmt(s);
                }
                self.exit_scope();

                // Check the 'else' block in a new scope (if it exists)
                if let Some(else_stmts) = else_block {
                    self.enter_scope();
                    for s in else_stmts {
                        self.check_stmt(s);
                    }
                    self.exit_scope();
                }
            }

            Stmt::Function { name, params, body } => {
                // Define the function name in the CURRENT scope
                self.define(name);

                // Check the function body in a NEW scope
                // (parameters are local to the function)
                self.enter_scope();
                for param in params {
                    self.define(param);
                }
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
            }

            Stmt::Print(expr) => {
                self.check_expr(expr);
            }

            Stmt::ExprStmt(expr) => {
                self.check_expr(expr);
            }
        }
    }
}
```

## Checking Expressions

```rust
impl SemanticChecker {
    // check_expr() — Check one expression
    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            // Literals are always fine — nothing to check
            Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) => {}

            // Check if the variable name is defined
            Expr::Ident(name) => {
                if !self.is_defined(name) {
                    self.error(format!("Error: '{}' is used but never declared.", name));
                }
            }

            // Check both sides of a binary expression
            Expr::Binary { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }

            // Check a function call
            Expr::Call { name, args } => {
                if !self.is_defined(name) {
                    self.error(format!("Error: function '{}' is not defined.", name));
                }
                for arg in args {
                    self.check_expr(arg);
                }
            }
        }
    }
}
```

## Testing the Checker

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::Parser;

    fn check_source(src: &str) -> Vec<String> {
        let tokens = tokenize(src);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();
        let mut checker = SemanticChecker::new();
        checker.check_program(&program);
        checker.errors
    }

    #[test]
    fn test_undefined_variable() {
        // 'z' is used but never declared
        let errors = check_source("let x = z + 1;");
        assert!(!errors.is_empty(), "Should have caught an error");
        assert!(errors[0].contains("z"), "Error should mention 'z'");
    }

    #[test]
    fn test_valid_code() {
        let errors = check_source("let x = 10; let y = x + 1;");
        assert!(errors.is_empty(), "Should have no errors");
    }

    #[test]
    fn test_function_call_ok() {
        let errors = check_source(
            "fn add(a, b) { return a + b; } let r = add(1, 2);"
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_undefined_function() {
        let errors = check_source("greet(\"Alice\");");
        assert!(!errors.is_empty());
        assert!(errors[0].contains("greet"));
    }
}
```

Run:

```bash
cargo test
```

Now our compiler can catch real mistakes in user code! In the next chapter, we will generate TypeScript output.
