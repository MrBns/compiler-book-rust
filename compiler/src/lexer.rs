// =============================================================================
// lexer.rs — Lexer (Tokenizer) for the Pico language
// =============================================================================
//
// The lexer is the very first phase of the Pico compiler.
//
// Its job: read raw source code TEXT and break it into a list of TOKENS.
//
// Think of it like reading English sentences:
//   your brain doesn't see individual letters — it sees WORDS.
//   The lexer does the same thing for code.
//
// Input (raw text):
//   `let x = 42;`
//
// Output (token list):
//   [Let, Ident("x"), Equals, Number(42.0), Semicolon, Eof]
//
// The lexer does NOT care about grammar or meaning.
//   It just breaks text into pieces.  The parser handles grammar.
//   The semantic checker handles meaning.
//
// High-level algorithm:
//   1. Look at the current character.
//   2. Decide what kind of token starts here.
//   3. Consume more characters if needed  (e.g. `==` needs two chars).
//   4. Build a Token and push it to the output list.
//   5. Move to the next character.
//   6. Repeat until end of file.
// =============================================================================

use crate::token::{Span, Token, TokenKind};

// =============================================================================
// Lexer struct
// =============================================================================
//
// We store the source code as `Vec<char>` rather than a `&str` slice because:
//   - `Vec<char>` gives O(1) random access by index.
//   - Handling Unicode is simpler — each element is one Unicode scalar value.
//   - We never modify the source, so no allocation overhead after construction.
pub struct Lexer {
    /// The entire source program, split into individual Unicode characters.
    chars: Vec<char>,

    /// Current read position (index into `chars`).
    /// After `advance()` is called, `pos` points at the NEXT unread character.
    pos: usize,

    /// Current line number.  Starts at 1.  Incremented every time we see `\n`.
    line: usize,

    /// Current column number.  Starts at 1.  Resets to 1 after every newline.
    col: usize,
}

impl Lexer {
    // -------------------------------------------------------------------------
    // Construction
    // -------------------------------------------------------------------------

    /// Create a new `Lexer` from a source string.
    ///
    /// `source.chars().collect()` converts the string into a `Vec<char>`.
    /// For example,  `"hi"` becomes  `['h', 'i']`.
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1, // lines start at 1 (not 0) — that's what editors show
            col: 1,  // columns also start at 1
        }
    }

    // -------------------------------------------------------------------------
    // Low-level character helpers
    // -------------------------------------------------------------------------

    /// Return the character at the current position **without** moving forward.
    /// Returns `None` when we have consumed all characters (end of file).
    ///
    /// `self.chars.get(self.pos)` returns `Option<&char>`.
    /// `.copied()` converts `Option<&char>` → `Option<char>` so callers
    /// don't need to deal with the reference.
    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Look at the character ONE position AHEAD without moving forward.
    /// This is essential for two-character tokens like `==`, `!=`, `<=`, `>=`.
    ///
    /// Without peek, when we see `=` we couldn't know whether the NEXT char is
    /// another `=` (making it `==`) until we had already consumed the first `=`.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    /// Consume the current character and return it.
    /// Also updates the line and column counters.
    ///
    /// Called "advance" because we advance the read position by one step.
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                // A newline ends the current line → bump line, reset column
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    /// Capture the current source location as a `Span`.
    /// Called just before we start reading a new token so the token's span
    /// points at the FIRST character of that token, not the last.
    fn span(&self) -> Span {
        Span {
            line: self.line,
            col: self.col,
        }
    }

    // -------------------------------------------------------------------------
    // Whitespace and comment skipping
    // -------------------------------------------------------------------------

    /// Skip over any whitespace characters (space, tab, newline, carriage return).
    /// Whitespace is not meaningful in Pico — it only serves as a separator.
    fn skip_whitespace(&mut self) {
        // Keep advancing while the current character is whitespace.
        // `is_whitespace()` handles space, tab, `\n`, `\r`, and more.
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break; // found a non-whitespace char — stop skipping
            }
        }
    }

    /// Skip a single-line comment (`// …`) until the end of the line.
    ///
    /// Called AFTER the lexer has already consumed the leading `//`.
    /// We just eat characters until we hit `\n` or end-of-file.
    fn skip_comment(&mut self) {
        while let Some(c) = self.current() {
            self.advance(); // consume the character
            if c == '\n' {
                break; // newline ends the comment
            }
        }
    }

    // -------------------------------------------------------------------------
    // Main tokenising method
    // -------------------------------------------------------------------------

    /// Read and return the **next** token from the source.
    ///
    /// This is called in a loop by `tokenize()` until it returns `Eof`.
    pub fn next_token(&mut self) -> Token {
        // --- Step 1: skip whitespace and line comments ---
        //
        // We loop here because after skipping a comment we might immediately
        // hit more whitespace, which needs skipping too.
        loop {
            self.skip_whitespace();

            // A comment starts with `//`.  We use `peek()` so we can check
            // BOTH characters before consuming either one.
            if self.current() == Some('/') && self.peek() == Some('/') {
                self.advance(); // consume first  '/'
                self.advance(); // consume second '/'
                self.skip_comment();
                // loop again: there might be more whitespace after the comment
            } else {
                break; // no comment here — proceed to tokenise
            }
        }

        // Record the start position of this token BEFORE consuming any chars.
        let span = self.span();

        // --- Step 2: check for end of file ---
        let ch = match self.current() {
            Some(c) => c,
            None => {
                // No more characters → emit the sentinel Eof token.
                // The parser relies on this to know when to stop.
                return Token::new(TokenKind::Eof, span.line, span.col);
            }
        };

        // --- Step 3: match the first character to decide the token type ---
        let kind = match ch {
            // -----------------------------------------------------------------
            // Single-character tokens
            // These are unambiguous — one character maps to exactly one token.
            // -----------------------------------------------------------------
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                TokenKind::Minus
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            '.' => {
                self.advance();
                TokenKind::Dot
            }

            // -----------------------------------------------------------------
            // '/' — could be division OR the start of a comment.
            // We already handled `//` above in the skip-comment loop, so if
            // we reach here with '/' it MUST be division.
            // -----------------------------------------------------------------
            '/' => {
                self.advance();
                TokenKind::Slash
            }

            // -----------------------------------------------------------------
            // Two-character operators that start with '='
            //   '='  → Equals  (assignment)
            //   '==' → EqEq    (equality test)
            // -----------------------------------------------------------------
            '=' => {
                self.advance(); // consume first '='
                if self.current() == Some('=') {
                    self.advance(); // consume second '='
                    TokenKind::EqEq // it was '=='
                } else {
                    TokenKind::Equals // it was just '='
                }
            }

            // -----------------------------------------------------------------
            // '!' — only valid as part of '!='
            // Pico does not have a unary `!` operator.
            // -----------------------------------------------------------------
            '!' => {
                self.advance(); // consume '!'
                if self.current() == Some('=') {
                    self.advance(); // consume '='
                    TokenKind::BangEq
                } else {
                    panic!(
                        "Unexpected character '!' without '=' at line {}, col {}",
                        self.line, self.col
                    );
                }
            }

            // -----------------------------------------------------------------
            // '<' — less-than OR less-than-or-equal
            // -----------------------------------------------------------------
            '<' => {
                self.advance(); // consume '<'
                if self.current() == Some('=') {
                    self.advance(); // consume '='
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }

            // -----------------------------------------------------------------
            // '>' — greater-than OR greater-than-or-equal
            // -----------------------------------------------------------------
            '>' => {
                self.advance(); // consume '>'
                if self.current() == Some('=') {
                    self.advance(); // consume '='
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }

            // -----------------------------------------------------------------
            // Number literals — digits 0-9
            // `'0'..='9'` is a Rust RANGE PATTERN; it matches any digit.
            // -----------------------------------------------------------------
            '0'..='9' => self.read_number(),

            // -----------------------------------------------------------------
            // Identifiers and keywords — start with a letter or underscore
            // -----------------------------------------------------------------
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),

            // -----------------------------------------------------------------
            // String literals — start and end with double-quote
            // -----------------------------------------------------------------
            '"' => self.read_string(),

            // -----------------------------------------------------------------
            // Anything else is an error in Pico
            // -----------------------------------------------------------------
            other => {
                panic!(
                    "Unknown character '{}' at line {}, col {}",
                    other, self.line, self.col
                );
            }
        };

        Token::new(kind, span.line, span.col)
    }

    // -------------------------------------------------------------------------
    // Readers for multi-character tokens
    // -------------------------------------------------------------------------

    /// Read a number literal and return the corresponding `TokenKind`.
    ///
    /// Handles integers (`42`) and floats (`3.14`).
    /// All numbers are stored as `f64` internally — simple and sufficient for
    /// a teaching compiler.
    ///
    /// A decimal point is only recognised when:
    ///   1. the current char is '.'
    ///   2. AND the char AFTER it is a digit (so `point.x` is not confused
    ///      with the float `3.14` — the `.x` has no digit after the dot).
    fn read_number(&mut self) -> TokenKind {
        let mut text = String::new();

        // Consume all leading digits
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Optionally consume a decimal part  (.digits)
        // `map_or(false, |c| c.is_ascii_digit())` safely handles the case
        // where peek() returns None (end of file).
        if self.current() == Some('.')
            && self.peek().is_some_and(|c| c.is_ascii_digit())
        {
            text.push('.'); // include the dot
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

        // Parse the collected digit-string into an f64.
        // `expect` will panic if somehow the string isn't a valid number,
        // but that can't happen given we only pushed digits and a dot.
        let value: f64 = text.parse().expect("ICE: invalid number literal");
        TokenKind::Number(value)
    }

    /// Read an identifier or keyword and return the corresponding `TokenKind`.
    ///
    /// An identifier starts with a letter or `_` and continues with
    /// letters, digits, or `_`.  After collecting the full word, we check
    /// whether it is a reserved keyword.
    ///
    /// The order of checks matters:
    ///   keywords are checked first → unrecognised words become `Ident`.
    fn read_identifier(&mut self) -> TokenKind {
        let mut text = String::new();

        // Collect the full word
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Map known keywords to their dedicated TokenKind variants.
        // Everything else is a user-defined name → Ident.
        match text.as_str() {
            "let"    => TokenKind::Let,
            "fn"     => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if"     => TokenKind::If,
            "else"   => TokenKind::Else,
            "print"  => TokenKind::Print,
            "struct" => TokenKind::Struct,
            "true"   => TokenKind::True,
            "false"  => TokenKind::False,
            // Type-annotation keywords
            "int"    => TokenKind::TyInt,
            "float"  => TokenKind::TyFloat,
            "str"    => TokenKind::TyStr,
            "bool"   => TokenKind::TyBool,
            // User-defined identifier
            _        => TokenKind::Ident(text),
        }
    }

    /// Read a double-quoted string literal and return `TokenKind::StringLit`.
    ///
    /// Called when `current()` is `"`.  We consume the opening quote, then
    /// collect characters until we find the closing quote.
    ///
    /// Supported escape sequences:
    ///   `\n`  → newline
    ///   `\t`  → tab
    ///   `\"`  → literal double-quote inside a string
    ///   `\\`  → literal backslash
    ///   Any other `\x` → `x`  (lenient fallback)
    fn read_string(&mut self) -> TokenKind {
        self.advance(); // consume the opening `"`
        let mut text = String::new();

        loop {
            match self.current() {
                // Found the closing quote → done
                Some('"') => {
                    self.advance(); // consume `"`
                    break;
                }

                // Escape sequence (backslash followed by something)
                Some('\\') => {
                    self.advance(); // consume `\`
                    match self.advance() {
                        Some('n')  => text.push('\n'),  // newline
                        Some('t')  => text.push('\t'),  // tab
                        Some('"')  => text.push('"'),   // literal quote
                        Some('\\') => text.push('\\'),  // literal backslash
                        Some(c)    => text.push(c),     // anything else: keep it
                        None       => panic!("Unterminated string at line {}", self.line),
                    }
                }

                // Regular character — include it verbatim
                Some(c) => {
                    text.push(c);
                    self.advance();
                }

                // End of file before closing quote → error
                None => {
                    panic!("Unterminated string literal at line {}", self.line);
                }
            }
        }

        TokenKind::StringLit(text)
    }
}

// =============================================================================
// Public convenience function
// =============================================================================

/// Run the lexer on `source` and return **all** tokens as a `Vec<Token>`.
///
/// The `Eof` token is included at the end so the parser always has something
/// to look at when it reaches the end of the program.
///
/// Usage:
/// ```rust
/// let tokens = tokenize("let x = 5;");
/// ```
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof {
            break; // Eof is the last token — stop looping
        }
    }

    tokens
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    /// Helper: tokenize and return just the kinds (without position info).
    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_basic_let() {
        assert_eq!(
            kinds("let x = 10;"),
            vec![
                TokenKind::Let,
                TokenKind::Ident("x".to_string()),
                TokenKind::Equals,
                TokenKind::Number(10.0),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comment_is_skipped() {
        // The comment should produce no tokens; `let` should be first
        let k = kinds("// this is a comment\nlet x = 1;");
        assert_eq!(k[0], TokenKind::Let);
    }

    #[test]
    fn test_string_literal() {
        let k = kinds(r#"let s = "hello";"#);
        // tokens: Let Ident("s") Equals StringLit("hello") Semicolon Eof
        assert_eq!(k[3], TokenKind::StringLit("hello".to_string()));
    }

    #[test]
    fn test_two_char_operators() {
        let k = kinds("x == y != z <= w >= v");
        assert!(k.contains(&TokenKind::EqEq));
        assert!(k.contains(&TokenKind::BangEq));
        assert!(k.contains(&TokenKind::LtEq));
        assert!(k.contains(&TokenKind::GtEq));
    }

    #[test]
    fn test_float_number() {
        let k = kinds("let pi = 3.14;");
        assert!(k.contains(&TokenKind::Number(3.14)));
    }

    #[test]
    fn test_keywords() {
        let k = kinds("fn if else return print struct true false");
        assert!(k.contains(&TokenKind::Fn));
        assert!(k.contains(&TokenKind::If));
        assert!(k.contains(&TokenKind::Else));
        assert!(k.contains(&TokenKind::Return));
        assert!(k.contains(&TokenKind::Print));
        assert!(k.contains(&TokenKind::Struct));
        assert!(k.contains(&TokenKind::True));
        assert!(k.contains(&TokenKind::False));
    }

    #[test]
    fn test_type_keywords() {
        let k = kinds("int float str bool");
        assert!(k.contains(&TokenKind::TyInt));
        assert!(k.contains(&TokenKind::TyFloat));
        assert!(k.contains(&TokenKind::TyStr));
        assert!(k.contains(&TokenKind::TyBool));
    }
}
