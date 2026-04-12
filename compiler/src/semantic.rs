// =============================================================================
// semantic.rs — Semantic Analysis for the Pico compiler
// =============================================================================
//
// The SEMANTIC CHECKER is phase 3 of the Pico compiler.
//
// The lexer already checked that every character is valid.
// The parser already checked that the grammar (structure) is correct.
// But neither of them checks whether the code MAKES SENSE.
//
// Example — syntactically valid, semantically wrong:
//   `let x = z + 1;`   ← 'z' was never declared!
//   `let result = foo(1, 2);`  ← 'foo' was never defined!
//
// The semantic checker catches these logical errors BEFORE code generation,
// so we never emit TypeScript for programs that are broken.
//
// ── What we check ────────────────────────────────────────────────────────────
//   1. Undefined variables  — using a name that was never declared with `let`
//   2. Undefined functions  — calling a function that was never defined with `fn`
//   3. Duplicate names      — declaring the same variable twice in one scope
//
// ── Scope stack ──────────────────────────────────────────────────────────────
//   We maintain a STACK of SCOPES.  Each scope is a set of names.
//
//   When we enter a new block `{ … }`, we PUSH a new scope.
//   When we leave a block, we POP that scope (its names disappear).
//   When we look up a name, we search from the INNERMOST scope outward.
//
//   Visualisation for:
//     fn add(a, b) { let temp = a + b; return temp; }
//     let result = add(1, 2);
//
//   Global scope:  { add }
//   Inside 'add':  { add } ← outer  +  { a, b, temp } ← inner
//   Back at global: { add, result }
//
// ── Error collection ─────────────────────────────────────────────────────────
//   We do NOT panic on the first error.  Instead we collect ALL errors into
//   `self.errors` and report them all at once in `main.rs`.
//   This is friendlier: the user sees every mistake in one compilation pass.
// =============================================================================

use std::collections::HashSet;

use crate::ast::{Expr, Program, Stmt};

// =============================================================================
// SemanticChecker struct
// =============================================================================
pub struct SemanticChecker {
    /// A stack of scopes.  Each scope is a set of defined names (variables
    /// and functions are in the same namespace in Pico).
    ///
    /// - `scopes[0]`  is the global (outermost) scope.
    /// - `scopes.last()`  is the current (innermost) scope.
    ///
    /// `HashSet<String>` gives O(1) average-case lookup — ideal for a symbol table.
    scopes: Vec<HashSet<String>>,

    /// All error messages collected during the traversal.
    /// These are reported to the user after the full AST has been walked.
    pub errors: Vec<String>,
}

impl SemanticChecker {
    // -------------------------------------------------------------------------
    // Construction
    // -------------------------------------------------------------------------

    /// Create a new semantic checker with one empty global scope.
    pub fn new() -> Self {
        SemanticChecker {
            scopes: vec![HashSet::new()], // one empty scope to start
            errors: Vec::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Scope management helpers
    // -------------------------------------------------------------------------

    /// Push a new, empty scope onto the stack.
    /// Called when we enter a block `{`, an if-branch, or a function body.
    fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    /// Pop the innermost scope off the stack.
    /// Called when we leave a block.  All names declared inside disappear.
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Add `name` to the CURRENT (innermost) scope.
    ///
    /// If the name is already in the current scope, it is a duplicate
    /// declaration error (same variable declared twice in the same block).
    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            if !scope.insert(name.to_string()) {
                // `HashSet::insert` returns false when the value already existed
                self.error(format!(
                    "Error: '{}' is already declared in this scope.",
                    name
                ));
            }
        }
    }

    /// Return `true` if `name` is defined in ANY scope (inner to outer).
    ///
    /// We search from the innermost scope outward so that inner
    /// declarations shadow outer ones (though Pico currently disallows
    /// redeclaration, the search order is still correct for lookup).
    fn is_defined(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    /// Append an error message to the error list.
    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    // -------------------------------------------------------------------------
    // Program-level entry point
    // -------------------------------------------------------------------------

    /// Walk every statement in the program and check it.
    ///
    /// This is the public entry point called from `main.rs`.
    pub fn check_program(&mut self, program: &Program) {
        for stmt in program {
            self.check_stmt(stmt);
        }
    }

    // -------------------------------------------------------------------------
    // Statement checking
    // -------------------------------------------------------------------------

    /// Semantically check a single statement.
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            // -----------------------------------------------------------------
            // `let name = value;`
            //
            // 1. Check the RHS expression FIRST (it may reference names that
            //    must already be defined — e.g. `let y = x + 1;`).
            // 2. Then define the new name in the current scope.
            //
            // This ordering means  `let x = x;`  would be caught as an error
            // because when we check `x` on the RHS, the LHS `x` hasn't been
            // defined yet.
            // -----------------------------------------------------------------
            Stmt::Let { name, value } => {
                self.check_expr(value); // check RHS first
                self.define(name);      // then bind the name
            }

            // -----------------------------------------------------------------
            // `return expr;`
            // -----------------------------------------------------------------
            Stmt::Return(expr) => {
                self.check_expr(expr);
            }

            // -----------------------------------------------------------------
            // `if condition { then } else { else }`
            //
            // Each branch gets its OWN scope so that variables declared
            // inside a branch are not visible outside.
            // -----------------------------------------------------------------
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expr(condition);

                // 'then' branch
                self.enter_scope();
                for s in then_block {
                    self.check_stmt(s);
                }
                self.exit_scope();

                // 'else' branch (optional)
                if let Some(else_stmts) = else_block {
                    self.enter_scope();
                    for s in else_stmts {
                        self.check_stmt(s);
                    }
                    self.exit_scope();
                }
            }

            // -----------------------------------------------------------------
            // `fn name(params) { body }`
            //
            // 1. Define the function name in the CURRENT (outer) scope first,
            //    so that recursive calls work:
            //      fn factorial(n) { … return factorial(n-1); … }
            // 2. Open a NEW scope for the function body.
            // 3. Define each parameter in that inner scope.
            // 4. Check the body statements.
            // 5. Close the inner scope.
            // -----------------------------------------------------------------
            Stmt::Function { name, params, body } => {
                self.define(name); // define in outer scope for recursion

                self.enter_scope(); // new scope for params + body
                for param in params {
                    self.define(param);
                }
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
            }

            // -----------------------------------------------------------------
            // `print(expr);`
            // -----------------------------------------------------------------
            Stmt::Print(expr) => {
                self.check_expr(expr);
            }

            // -----------------------------------------------------------------
            // Expression statement  (e.g. standalone function call)
            // -----------------------------------------------------------------
            Stmt::ExprStmt(expr) => {
                self.check_expr(expr);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Expression checking
    // -------------------------------------------------------------------------

    /// Semantically check a single expression.
    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            // -----------------------------------------------------------------
            // Literal values — always valid, nothing to check
            // -----------------------------------------------------------------
            Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) => {}

            // -----------------------------------------------------------------
            // Variable reference — the name must have been declared
            // -----------------------------------------------------------------
            Expr::Ident(name) => {
                if !self.is_defined(name) {
                    self.error(format!(
                        "Error: '{}' is used but was never declared.",
                        name
                    ));
                }
            }

            // -----------------------------------------------------------------
            // Binary expression — check both operands recursively
            // -----------------------------------------------------------------
            Expr::Binary { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }

            // -----------------------------------------------------------------
            // Function call — the function must be defined, and all arguments
            // must be valid expressions too
            // -----------------------------------------------------------------
            Expr::Call { name, args } => {
                if !self.is_defined(name) {
                    self.error(format!(
                        "Error: function '{}' is called but was never defined.",
                        name
                    ));
                }
                for arg in args {
                    self.check_expr(arg);
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::Parser;

    /// Helper: compile src through lexer + parser + semantic checker,
    /// then return the error list.
    fn check_src(src: &str) -> Vec<String> {
        let tokens = tokenize(src);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();
        let mut checker = SemanticChecker::new();
        checker.check_program(&program);
        checker.errors
    }

    #[test]
    fn test_undefined_variable_caught() {
        // 'z' is never declared; the checker must report an error about 'z'
        let errors = check_src("let x = z + 1;");
        assert!(!errors.is_empty(), "expected at least one error");
        assert!(errors[0].contains('z'), "error should mention 'z'");
    }

    #[test]
    fn test_valid_let_chain() {
        // Each variable is defined before it is used → no errors
        let errors = check_src("let x = 10; let y = x + 1; let z = y * 2;");
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_function_defined_and_called() {
        let errors = check_src(
            "fn add(a, b) { return a + b; } let r = add(1, 2);",
        );
        assert!(errors.is_empty(), "got errors: {:?}", errors);
    }

    #[test]
    fn test_undefined_function_caught() {
        let errors = check_src(r#"greet("Alice");"#);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("greet"));
    }

    #[test]
    fn test_recursive_function_allowed() {
        // The function name is defined in the outer scope before entering the body,
        // so recursive calls should be valid.
        let errors =
            check_src("fn fact(n) { return fact(n); }");
        assert!(errors.is_empty(), "recursive call raised error: {:?}", errors);
    }

    #[test]
    fn test_variable_not_visible_outside_if() {
        // 'inner' is declared inside the if block.
        // Using it after the block should be an error.
        // (Pico's scoping rule: variables declared in a block die when the block ends.)
        let errors = check_src("if true { let inner = 1; } let y = inner + 1;");
        assert!(
            !errors.is_empty(),
            "expected scope error for 'inner', got none"
        );
    }
}
