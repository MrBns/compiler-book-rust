// =============================================================================
// parser.rs — Recursive-descent parser for the Pico language
// =============================================================================
//
// The PARSER is phase 2 of the Pico compiler.
//
// Input:  a flat list of Tokens produced by the lexer.
// Output: a nested AST (Abstract Syntax Tree) representing the program.
//
// We use the technique called RECURSIVE DESCENT PARSING.
// The idea is simple: each grammar rule becomes a Rust function.
//
//   Grammar rule (pseudo-BNF)          Rust function
//   ─────────────────────────────────  ─────────────────────
//   program  → stmt*                   parse()
//   stmt     → let | return | if | fn  parse_statement()
//   let      → 'let' IDENT '=' expr ';'  parse_let()
//   expr     → comparison              parse_expression()
//   comparison → addition (cmp addition)*  parse_comparison()
//   addition   → multiplication ((+|-) multiplication)*  parse_addition()
//   multiplication → primary ((*|/) primary)*  parse_multiplication()
//   primary  → NUMBER | STRING | IDENT | call | '(' expr ')'  parse_primary()
//
// OPERATOR PRECEDENCE is handled by the call chain:
//
//   parse_expression()
//     calls parse_comparison()        ← lowest precedence: == != < > <= >=
//       calls parse_addition()        ← medium:            + -
//         calls parse_multiplication() ← higher:           * /
//           calls parse_primary()     ← highest:           literals, idents, calls
//
// Each level only handles operators at ITS level; it delegates
// everything with higher precedence to the function below it.
// This naturally makes  `1 + 2 * 3`  parse as  `1 + (2 * 3)`.
// =============================================================================

use crate::ast::{BinOp, Expr, Program, Stmt};
use crate::token::{Token, TokenKind};

// =============================================================================
// Parser struct
// =============================================================================
pub struct Parser {
    /// The complete flat token list from the lexer (including the final Eof).
    tokens: Vec<Token>,

    /// Index of the token we are currently looking at.
    /// We always move forward — never backward.
    pos: usize,
}

impl Parser {
    // -------------------------------------------------------------------------
    // Construction
    // -------------------------------------------------------------------------

    /// Create a new parser from a token list.
    ///
    /// Normally called as:
    /// ```rust
    /// let tokens = lexer::tokenize(source);
    /// let mut parser = Parser::new(tokens);
    /// let program = parser.parse();
    /// ```
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // -------------------------------------------------------------------------
    // Low-level token navigation helpers
    // -------------------------------------------------------------------------

    /// Return a reference to the **current** token without consuming it.
    ///
    /// If `pos` somehow goes past the end of `tokens` (shouldn't happen in
    /// correct usage because we always have a final `Eof`), we fall back to
    /// the last token so we never panic with an out-of-bounds access.
    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Look at the token ONE position ahead without consuming it.
    ///
    /// Used in `parse_primary()` to determine whether an identifier is
    /// a plain variable name or the start of a function call:
    ///   `foo`    → Ident  (peek is NOT `(`)
    ///   `foo()`  → Call   (peek IS `(`)
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos + 1)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Move forward by one position and return a reference to the token
    /// we just passed.
    ///
    /// This is the primary way to "consume" a token.
    /// After calling `advance()`, `current()` returns the NEXT token.
    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        // Guard against advancing past the end of the list
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    /// Consume the current token if its kind matches `expected`, or panic.
    ///
    /// This is the "assert and advance" helper.  The parser uses it whenever
    /// a grammar rule says "there MUST be a specific token here".
    ///
    /// Example: after parsing `fn name`, we know `(` MUST come next.
    ///   `self.expect(&TokenKind::LParen);`
    ///
    /// If the token is wrong, a clear panic message tells us exactly what
    /// was expected vs. what was found, and where.
    fn expect(&mut self, expected: &TokenKind) -> &Token {
        if &self.current().kind == expected {
            self.advance()
        } else {
            panic!(
                "Parse error: expected {:?} but found {:?} at line {}, col {}",
                expected,
                self.current().kind,
                self.current().span.line,
                self.current().span.col,
            );
        }
    }

    /// Return `true` if the current token's kind equals `kind`.
    /// Does NOT consume the token.
    ///
    /// Used for optional grammar elements like `else { … }`.
    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    /// Return `true` if the current token is `Eof` (end of the program).
    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }
}

// =============================================================================
// Statement parsing
// =============================================================================
impl Parser {
    // -------------------------------------------------------------------------
    // Top-level entry point
    // -------------------------------------------------------------------------

    /// Parse the entire program and return a list of top-level statements.
    ///
    /// We keep calling `parse_statement()` until we reach `Eof`.
    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }
        statements
    }

    // -------------------------------------------------------------------------
    // Statement dispatcher
    // -------------------------------------------------------------------------

    /// Decide which specific statement parser to call based on the current token.
    ///
    /// This is the "dispatch" function — it peeks at the current token and
    /// hands off to the right sub-parser.
    fn parse_statement(&mut self) -> Stmt {
        match self.current().kind.clone() {
            TokenKind::Let    => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If     => self.parse_if(),
            TokenKind::Fn     => self.parse_function(),
            TokenKind::Print  => self.parse_print(),
            // Anything else is treated as an expression statement
            // e.g. a standalone function call:  `greet("Alice");`
            _                 => self.parse_expr_stmt(),
        }
    }

    // -------------------------------------------------------------------------
    // Individual statement parsers
    // -------------------------------------------------------------------------

    /// Parse a `let` statement:  `let name = expr;`
    ///
    /// Grammar:  'let' IDENT '=' expression ';'
    fn parse_let(&mut self) -> Stmt {
        self.advance(); // consume 'let'

        // The next token must be an identifier (the variable name)
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            other => panic!("Expected identifier after 'let', got {:?}", other),
        };

        // Optional type annotation:  `let x: int = …`
        // We parse and discard it for now (the base code-gen uses `any`).
        if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'
            self.advance(); // consume the type name (e.g. 'int', 'str', …)
        }

        self.expect(&TokenKind::Equals); // '='
        let value = self.parse_expression();
        self.expect(&TokenKind::Semicolon); // ';'

        Stmt::Let { name, value }
    }

    /// Parse a `return` statement:  `return expr;`
    ///
    /// Grammar:  'return' expression ';'
    fn parse_return(&mut self) -> Stmt {
        self.advance(); // consume 'return'
        let value = self.parse_expression();
        self.expect(&TokenKind::Semicolon);
        Stmt::Return(value)
    }

    /// Parse an `if` statement:  `if condition { … } else { … }`
    ///
    /// Grammar:
    ///   'if' expression '{' stmt* '}' ('else' '{' stmt* '}')?
    ///
    /// The `else` branch is optional — `else_block` is `None` when absent.
    fn parse_if(&mut self) -> Stmt {
        self.advance(); // consume 'if'
        let condition = self.parse_expression();

        self.expect(&TokenKind::LBrace); // '{'
        let then_block = self.parse_block(); // parses until '}'

        // Check for optional `else` branch
        let else_block = if self.check(&TokenKind::Else) {
            self.advance(); // consume 'else'
            self.expect(&TokenKind::LBrace); // '{'
            Some(self.parse_block()) // parse else body until '}'
        } else {
            None
        };

        Stmt::If {
            condition,
            then_block,
            else_block,
        }
    }

    /// Parse a block of statements: everything between `{` (already consumed)
    /// and the matching `}`.
    ///
    /// Called by `parse_if`, `parse_function`, etc.
    /// The opening `{` must have been consumed BEFORE calling this.
    /// This method consumes the closing `}`.
    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        // Keep parsing until we hit '}' or run out of tokens
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement());
        }
        self.expect(&TokenKind::RBrace); // consume '}'
        stmts
    }

    /// Parse a function definition:  `fn name(params) { body }`
    ///
    /// Grammar:
    ///   'fn' IDENT '(' (IDENT (',' IDENT)*)? ')' '{' stmt* '}'
    ///
    /// Parameter types and return type annotations are parsed and discarded.
    fn parse_function(&mut self) -> Stmt {
        self.advance(); // consume 'fn'

        // The function name
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            other => panic!("Expected function name after 'fn', got {:?}", other),
        };

        self.expect(&TokenKind::LParen); // '('

        // Parse parameter list:  `a, b, c`
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            match self.advance().kind.clone() {
                TokenKind::Ident(p) => params.push(p),
                other => panic!("Expected parameter name, got {:?}", other),
            }
            // Each parameter may have an optional type annotation:  `a: int`
            if self.check(&TokenKind::Colon) {
                self.advance(); // consume ':'
                self.advance(); // consume type name
            }
            // Consume the comma between parameters (if present)
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen); // ')'

        // Optional return type annotation:  `: int`
        if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'
            self.advance(); // consume return type
        }

        self.expect(&TokenKind::LBrace); // '{'
        let body = self.parse_block(); // parses until '}'

        Stmt::Function { name, params, body }
    }

    /// Parse a `print` statement:  `print(expr);`
    ///
    /// Grammar:  'print' '(' expression ')' ';'
    fn parse_print(&mut self) -> Stmt {
        self.advance(); // consume 'print'
        self.expect(&TokenKind::LParen); // '('
        let value = self.parse_expression();
        self.expect(&TokenKind::RParen); // ')'
        self.expect(&TokenKind::Semicolon); // ';'
        Stmt::Print(value)
    }

    /// Parse an expression statement: an expression followed by `;`.
    ///
    /// The most common case is a standalone function call:
    ///   `greet("Bob");`
    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expression();
        self.expect(&TokenKind::Semicolon);
        Stmt::ExprStmt(expr)
    }
}

// =============================================================================
// Expression parsing — the precedence climbing chain
// =============================================================================
//
// Operators with LOWER precedence are handled at HIGHER levels in the call
// stack (closer to `parse_expression`).  The chain is:
//
//   parse_expression           (entry point, lowest precedence overall)
//     → parse_comparison       (== != < > <= >=)
//       → parse_addition       (+ -)
//         → parse_multiplication  (* /)
//           → parse_primary    (literals, idents, calls, grouped exprs)
//
// Each level calls the next level to get its left operand, then loops to
// consume any same-level operators.
//
impl Parser {
    /// Top-level expression entry point.
    /// Currently just delegates to `parse_comparison`.
    pub fn parse_expression(&mut self) -> Expr {
        self.parse_comparison()
    }

    /// Handle comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`.
    ///
    /// These have LOWER precedence than addition/subtraction, so they are
    /// handled at a higher level in the call chain.
    fn parse_comparison(&mut self) -> Expr {
        // Get the left operand from the next-higher-precedence level
        let mut left = self.parse_addition();

        // Keep consuming comparison operators (left-to-right associativity)
        loop {
            let op = match self.current().kind {
                TokenKind::EqEq   => BinOp::Eq,
                TokenKind::BangEq => BinOp::Ne,
                TokenKind::Lt     => BinOp::Lt,
                TokenKind::Gt     => BinOp::Gt,
                TokenKind::LtEq   => BinOp::Le,
                TokenKind::GtEq   => BinOp::Ge,
                _ => break, // not a comparison operator — stop looping
            };
            self.advance(); // consume the operator token
            let right = self.parse_addition(); // right side is at higher precedence
            left = Expr::Binary {
                left:  Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    /// Handle `+` and `-` operators.
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

    /// Handle `*` and `/` operators.
    ///
    /// Because this level is called from `parse_addition`, multiplication and
    /// division naturally bind TIGHTER than addition and subtraction.
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

    /// Parse a "primary" expression — the most tightly-bound things:
    ///   - Number literals
    ///   - String literals
    ///   - Boolean literals (`true` / `false`)
    ///   - Variable names (`Ident`)
    ///   - Function calls  (`name(arg, arg, …)`)
    ///   - Parenthesised sub-expressions  `(expr)`
    fn parse_primary(&mut self) -> Expr {
        match self.current().kind.clone() {
            // Number literal
            TokenKind::Number(n) => {
                self.advance();
                Expr::Number(n)
            }

            // String literal
            TokenKind::StringLit(s) => {
                self.advance();
                Expr::Str(s)
            }

            // Boolean true
            TokenKind::True => {
                self.advance();
                Expr::Bool(true)
            }

            // Boolean false
            TokenKind::False => {
                self.advance();
                Expr::Bool(false)
            }

            // Identifier — could be a plain variable OR a function call.
            //
            // We use `peek()` to look ONE token AHEAD *before* consuming
            // the identifier.  If the token after the name is '(', then this
            // is a function call; otherwise it is a plain variable reference.
            //
            // This is the classic use-case for the peek helper:
            //   `foo`    → peek is not '('  → plain Ident
            //   `foo()`  → peek IS '('      → Call
            TokenKind::Ident(name) => {
                if self.peek().kind == TokenKind::LParen {
                    self.advance(); // consume the identifier name
                    self.parse_call(name) // then parse `(args…)`
                } else {
                    self.advance(); // consume the identifier name
                    Expr::Ident(name)
                }
            }

            // Parenthesised expression: `( expr )`
            // Used to override precedence, e.g.  `(1 + 2) * 3`
            TokenKind::LParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression(); // parse inner expression
                self.expect(&TokenKind::RParen);    // consume ')'
                expr // the parentheses are "abstract" — not kept in the AST
            }

            other => panic!(
                "Unexpected token in expression: {:?} at line {}, col {}",
                other,
                self.current().span.line,
                self.current().span.col,
            ),
        }
    }

    /// Parse a function call argument list after the name was already consumed.
    ///
    /// Grammar:  '(' (expression (',' expression)*)? ')'
    ///
    /// Called from `parse_primary` when we see `IDENT '('`.
    /// By the time we enter here, `name` has been consumed and we are
    /// looking at `(`.
    fn parse_call(&mut self, name: String) -> Expr {
        self.advance(); // consume '('

        let mut args = Vec::new();
        // Parse arguments until we see ')' or hit end of file
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            args.push(self.parse_expression()); // each argument is a full expression
            // Consume the comma between arguments (if present)
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RParen); // consume ')'
        Expr::Call { name, args }
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, Expr, Stmt};
    use crate::lexer::tokenize;

    /// Helper: parse source and return the program (list of statements).
    fn parse_src(src: &str) -> Program {
        let tokens = tokenize(src);
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_let_number() {
        let prog = parse_src("let x = 42;");
        assert_eq!(prog.len(), 1);
        match &prog[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Number(n) if *n == 42.0));
            }
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn test_precedence_mul_before_add() {
        // `1 + 2 * 3` should be parsed as `1 + (2 * 3)`
        let prog = parse_src("let z = 1 + 2 * 3;");
        match &prog[0] {
            Stmt::Let { value, .. } => match value {
                Expr::Binary { op: BinOp::Add, right, .. } => match right.as_ref() {
                    Expr::Binary { op: BinOp::Mul, .. } => {} // correct
                    _ => panic!("expected Mul on the right"),
                },
                _ => panic!("expected Add at top level"),
            },
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn test_function_call() {
        let prog = parse_src("let r = add(1, 2);");
        match &prog[0] {
            Stmt::Let { value, .. } => match value {
                Expr::Call { name, args } => {
                    assert_eq!(name, "add");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("expected Call"),
            },
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn test_if_without_else() {
        let prog = parse_src("if x > 0 { print(x); }");
        match &prog[0] {
            Stmt::If { else_block, .. } => assert!(else_block.is_none()),
            _ => panic!("expected If"),
        }
    }

    #[test]
    fn test_if_with_else() {
        let prog = parse_src("if x > 0 { print(x); } else { print(x); }");
        match &prog[0] {
            Stmt::If { else_block, .. } => assert!(else_block.is_some()),
            _ => panic!("expected If"),
        }
    }

    #[test]
    fn test_function_definition() {
        let prog = parse_src("fn add(a, b) { return a + b; }");
        match &prog[0] {
            Stmt::Function { name, params, body } => {
                assert_eq!(name, "add");
                assert_eq!(params, &["a", "b"]);
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected Function"),
        }
    }
}
