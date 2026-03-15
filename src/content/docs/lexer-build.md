---
title: "Building the Lexer"
description: "Write the lexer in Rust that turns source code text into a list of tokens."
---

# Building the Lexer

Now let's write the actual lexer! Open `src/lexer.rs`.

We will build this file step by step.

## The Lexer Struct

First, we define the `Lexer` struct:

```rust
// lexer.rs — Turns source code text into a list of Tokens
//
// The Lexer struct holds:
//   - the source code as a list of characters
//   - the current position in that list
//   - the current line and column numbers (for error messages)

use crate::token::{Token, TokenKind, Span};

pub struct Lexer {
    // We store source code as a Vec of chars.
    // Using chars (not bytes) lets us handle Unicode correctly.
    chars: Vec<char>,

    // pos = current index in the chars Vec
    pos: usize,

    // Track line and column for error messages
    line: usize,
    col: usize,
}
```

> **Rust note:** `Vec<char>` is a vector of characters. We use `collect()` to turn a `String` into a `Vec<char>`.

Now add an `impl` block with a constructor:

```rust
impl Lexer {
    // new() — Create a new Lexer from a source code string
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(), // "hello" → ['h','e','l','l','o']
            pos: 0,
            line: 1, // lines start at 1 (not 0)
            col: 1,  // columns start at 1 (not 0)
        }
    }
}
```

## Helper Methods

We need some small helper methods. Add these inside the `impl Lexer` block:

```rust
impl Lexer {
    // current() — Return the current character without moving forward
    // Returns None if we are at the end of the source
    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
        // .copied() converts Option<&char> to Option<char>
    }

    // peek() — Look at the NEXT character without moving forward
    // This is useful for two-character tokens like '==' or '!='
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    // advance() — Move forward one character and return it
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                // Newline → go to next line, reset column
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    // span() — Return the current Span (line + col)
    fn span(&self) -> Span {
        Span { line: self.line, col: self.col }
    }

    // skip_whitespace() — Skip spaces, tabs, and newlines
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    // skip_comment() — Skip a line comment starting with '//'
    fn skip_comment(&mut self) {
        // We already consumed '//', now skip until end of line
        while let Some(c) = self.current() {
            self.advance();
            if c == '\n' {
                break;
            }
        }
    }
}
```

## The Main next_token Method

This is the heart of the lexer. It reads one token at a time:

```rust
impl Lexer {
    // next_token() — Read and return the next token
    pub fn next_token(&mut self) -> Token {
        // First, skip any whitespace or comments
        loop {
            self.skip_whitespace();

            // Check if we have a comment (//)
            if self.current() == Some('/') && self.peek() == Some('/') {
                self.advance(); // skip first '/'
                self.advance(); // skip second '/'
                self.skip_comment();
            } else {
                break;
            }
        }

        let span = self.span();

        let ch = match self.current() {
            Some(c) => c,
            None => {
                // No more characters → return End-of-File token
                return Token::new(TokenKind::Eof, span.line, span.col);
            }
        };

        // Decide what kind of token to create based on the character
        let kind = match ch {
            // --- Single-character tokens ---
            '+' => { self.advance(); TokenKind::Plus }
            '-' => { self.advance(); TokenKind::Minus }
            '*' => { self.advance(); TokenKind::Star }
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            ',' => { self.advance(); TokenKind::Comma }
            ';' => { self.advance(); TokenKind::Semicolon }
            '/' => { self.advance(); TokenKind::Slash }

            // --- '=' can be '=' (assign) or '==' (equals check) ---
            '=' => {
                self.advance(); // consume '='
                if self.current() == Some('=') {
                    self.advance(); // consume second '='
                    TokenKind::EqEq  // it was '=='
                } else {
                    TokenKind::Equals  // it was just '='
                }
            }

            // --- '!' can only be '!=' ---
            '!' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenKind::BangEq
                } else {
                    panic!("Unexpected character '!' at line {}", self.line);
                }
            }

            // --- '<' can be '<' or '<=' ---
            '<' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }

            // --- '>' can be '>' or '>=' ---
            '>' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }

            // --- Numbers ---
            '0'..='9' => self.read_number(),

            // --- Identifiers and keywords ---
            // (letters and underscore can start an identifier)
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),

            // --- String literals ---
            '"' => self.read_string(),

            // --- Unknown character ---
            other => {
                panic!("Unknown character '{}' at line {}, col {}", other, self.line, self.col);
            }
        };

        Token::new(kind, span.line, span.col)
    }
}
```

> **Rust note:** `'0'..='9'` is a **range pattern**. It matches any character from '0' to '9'. Same as writing `'0' | '1' | ... | '9'` but much shorter!

## Reading Numbers

```rust
impl Lexer {
    // read_number() — Read a number like 42 or 3.14
    fn read_number(&mut self) -> TokenKind {
        let mut text = String::new();

        // Read all digits
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point (e.g., 3.14)
        if self.current() == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            text.push('.');
            self.advance(); // consume '.'
            while let Some(c) = self.current() {
                if c.is_ascii_digit() {
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse "42" or "3.14" into an f64 number
        let value: f64 = text.parse().expect("Invalid number");
        TokenKind::Number(value)
    }
}
```

## Reading Identifiers and Keywords

```rust
impl Lexer {
    // read_identifier() — Read a word like 'x', 'myFunc', or 'let'
    fn read_identifier(&mut self) -> TokenKind {
        let mut text = String::new();

        // Read letters, digits, and underscores
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check if this word is a keyword
        match text.as_str() {
            "let"    => TokenKind::Let,
            "fn"     => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if"     => TokenKind::If,
            "else"   => TokenKind::Else,
            "print"  => TokenKind::Print,
            "true"   => TokenKind::True,
            "false"  => TokenKind::False,
            _        => TokenKind::Ident(text), // not a keyword
        }
    }
}
```

## Reading Strings

```rust
impl Lexer {
    // read_string() — Read a string like "hello world"
    fn read_string(&mut self) -> TokenKind {
        self.advance(); // consume the opening '"'
        let mut text = String::new();

        loop {
            match self.current() {
                Some('"') => {
                    self.advance(); // consume the closing '"'
                    break;
                }
                Some('\\') => {
                    // Handle escape sequences like \n, \t, \"
                    self.advance(); // consume '\'
                    match self.advance() {
                        Some('n')  => text.push('\n'),
                        Some('t')  => text.push('\t'),
                        Some('"')  => text.push('"'),
                        Some('\\') => text.push('\\'),
                        Some(c)    => text.push(c),
                        None       => panic!("Unterminated string at line {}", self.line),
                    }
                }
                Some(c) => {
                    text.push(c);
                    self.advance();
                }
                None => {
                    panic!("Unterminated string at line {}", self.line);
                }
            }
        }

        TokenKind::StringLit(text)
    }
}
```

## The tokenize Function

Finally, let's add a helper function that tokenizes the whole source code at once:

```rust
// tokenize() — Run the lexer and return ALL tokens as a Vec
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);

        if is_eof {
            break; // stop when we reach end of file
        }
    }

    tokens
}
```

## Testing the Lexer

Add this to the bottom of `lexer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("let x = 10;");
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();

        assert_eq!(kinds, vec![
            TokenKind::Let,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equals,
            TokenKind::Number(10.0),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_comment_skipped() {
        let tokens = tokenize("// this is a comment\nlet x = 1;");
        assert_eq!(tokens[0].kind, TokenKind::Let);
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize("let s = \"hello\";");
        assert_eq!(tokens[3].kind, TokenKind::StringLit("hello".to_string()));
    }
}
```

Run the tests:

```bash
cargo test
```

If all tests pass — great job! 🎉 The lexer is done.

In the next chapter, we will build the **parser** that takes our tokens and builds an AST tree.
