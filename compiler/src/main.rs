// =============================================================================
// main.rs — Entry point of the Pico compiler
// =============================================================================
//
//  ____  _
// |  _ \(_) ___ ___
// | |_) | |/ __/ _ \
// |  __/| | (_| (_) |
// |_|   |_|\___\___/
//   Compiler  ·  written in Rust  ·  by Mr Binary Sniper
//
// This binary is the Pico compiler.  It reads a `.pico` source file and
// produces an equivalent `.ts` (TypeScript) file.
//
// ── Compilation pipeline ─────────────────────────────────────────────────────
//
//   .pico source file
//       │
//       ▼
//   [1] Lexer  (lexer.rs)
//       Turns raw text into a flat list of tokens.
//       e.g. "let x = 5;" → [Let, Ident("x"), Equals, Number(5.0), Semicolon, Eof]
//       │
//       ▼
//   [2] Parser  (parser.rs)
//       Turns the token list into an Abstract Syntax Tree (AST).
//       e.g. Stmt::Let { name: "x", value: Expr::Number(5.0) }
//       │
//       ▼
//   [3] Semantic Checker  (semantic.rs)
//       Walks the AST and reports logical errors:
//         - Undefined variables / functions
//         - Duplicate declarations in the same scope
//       If errors are found → print them all and exit with code 1.
//       │
//       ▼
//   [4] Code Generator  (codegen.rs)
//       Walks the AST and produces TypeScript source code as a String.
//       │
//       ▼
//   .ts output file
//
// ── Usage ─────────────────────────────────────────────────────────────────────
//   cargo run -- <source.pico>          compile a file
//   cargo run -- hello.pico             produces hello.ts
//   cargo test                          run all unit tests
//
// ── Example ──────────────────────────────────────────────────────────────────
//   $ echo 'let x = 42; print(x);' > hello.pico
//   $ cargo run -- hello.pico
//   Compiling: hello.pico
//     [1] Lexed 6 tokens
//     [2] Parsed 2 statements
//     [3] Semantic check passed ✓
//     [4] Generated TypeScript
//   Output: hello.ts
//   Compilation successful! 🎉
// =============================================================================

// Declare all modules — Rust requires us to name every source file we use.
mod ast;      // AST node type definitions
mod codegen;  // Phase 4: TypeScript code generator
mod lexer;    // Phase 1: Lexer / tokenizer
mod parser;   // Phase 2: Recursive-descent parser
mod semantic; // Phase 3: Semantic / scope checker
mod token;    // Token type definitions (used by lexer + parser)

use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    // ── Argument parsing ──────────────────────────────────────────────────────
    // We collect the command-line arguments into a Vec<String>.
    // `args[0]` is always the program name (the binary path), so we need
    // at least 2 elements: the binary name + the source file path.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: pico <source.pico>");
        eprintln!("Example: pico hello.pico");
        process::exit(1);
    }

    let source_path = &args[1];

    // ── Step 1: Read the source file ──────────────────────────────────────────
    // `fs::read_to_string` reads the entire file into a String.
    // We handle the error case explicitly with a clear message instead of
    // using `.unwrap()`, which would give a cryptic panic message.
    let source = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: cannot read '{}': {}", source_path, e);
            process::exit(1);
        }
    };

    println!("Compiling: {}", source_path);

    // ── Step 2: Lex ───────────────────────────────────────────────────────────
    // Turn the source text into a flat list of tokens.
    // `tokenize` is a convenience function in lexer.rs that runs the lexer
    // until Eof and returns all tokens as a Vec.
    let tokens = lexer::tokenize(&source);
    println!("  [1] Lexed {} tokens", tokens.len());

    // ── Step 3: Parse ─────────────────────────────────────────────────────────
    // Turn the token list into an AST.
    // `Parser::new(tokens)` creates the parser.
    // `parser.parse()` consumes all tokens and returns a Vec<Stmt> (= Program).
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse();
    println!("  [2] Parsed {} top-level statements", program.len());

    // ── Step 4: Semantic check ────────────────────────────────────────────────
    // Walk the AST and collect any logical errors.
    // We do NOT panic on the first error — all errors are collected so the
    // user can see and fix all of them in one compilation round.
    let mut checker = semantic::SemanticChecker::new();
    checker.check_program(&program);

    if !checker.errors.is_empty() {
        eprintln!("\nSemantic errors found ({} total):\n", checker.errors.len());
        for error in &checker.errors {
            eprintln!("  • {}", error);
        }
        process::exit(1);
    }
    println!("  [3] Semantic check passed ✓");

    // ── Step 5: Generate TypeScript ───────────────────────────────────────────
    // Walk the AST and produce TypeScript source code as a String.
    let typescript_code = codegen::generate(&program);
    println!("  [4] Generated TypeScript");

    // ── Step 6: Write the output file ─────────────────────────────────────────
    // We replace the source file's extension with ".ts".
    // e.g. "hello.pico" → "hello.ts"
    // `Path::with_extension` does this cleanly without string manipulation.
    let output_path = Path::new(source_path).with_extension("ts");

    match fs::write(&output_path, &typescript_code) {
        Ok(_) => {
            println!("  Output: {}", output_path.display());
            println!("\nCompilation successful! 🎉");
        }
        Err(e) => {
            eprintln!("Error: cannot write output file '{}': {}", output_path.display(), e);
            process::exit(1);
        }
    }
}
