//! `expander` — M4 macro expansion engine for LaTeX
//!
//! This module implements TeX-style macro expansion as a preprocessing stage
//! before the parser. It processes `\def`, `\newcommand`, `\renewcommand`,
//! `\let`, `\if`, `\ifx`, and `\ifnum` directives, expanding macros and
//! evaluating conditionals to produce an expanded token stream.

use rustlatex_lexer::{Category, Lexer, Token};
use std::collections::HashMap;

use crate::{Node, Parser};

/// A macro definition: parameter count + replacement body.
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// Number of parameters (0–9).
    pub param_count: u8,
    /// The replacement body tokens (containing `Token::Parameter(n)` placeholders).
    pub body: Vec<Token>,
}

/// A table of macro definitions, keyed by control sequence name.
pub type MacroTable = HashMap<String, MacroDef>;

/// The macro expander. Preprocesses a token stream by:
/// - Parsing and storing `\def`, `\newcommand`, `\renewcommand`, `\let` definitions
/// - Expanding macro calls by parameter substitution
/// - Evaluating `\if`, `\ifx`, `\ifnum` conditionals
pub struct Expander {
    /// The token stream being processed.
    tokens: Vec<Token>,
    /// Current position in the token stream.
    pos: usize,
    /// The macro definition table.
    pub macros: MacroTable,
}

impl Expander {
    /// Create a new expander from LaTeX source.
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Expander {
            tokens,
            pos: 0,
            macros: MacroTable::new(),
        }
    }

    /// Expand the token stream and parse the result into a [`Node::Document`].
    pub fn parse(&mut self) -> Node {
        let expanded = self.expand_all();
        let mut parser = Parser::from_tokens(expanded);
        parser.parse()
    }

    /// Peek at the current token.
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EndOfInput)
    }

    /// Advance and return the current token.
    fn advance(&mut self) -> Token {
        let tok = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token::EndOfInput);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    /// Check if we've consumed all tokens.
    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::EndOfInput)
    }

    /// Skip space tokens at the current position.
    fn skip_spaces(&mut self) {
        while matches!(self.peek(), Token::Space) {
            self.advance();
        }
    }

    /// Read a brace-delimited group of tokens (including nested groups).
    /// Expects current token to be `{`. Returns the tokens inside the braces.
    fn read_brace_group(&mut self) -> Vec<Token> {
        // consume '{'
        self.advance();
        let mut result = Vec::new();
        let mut depth = 1u32;
        loop {
            if self.at_end() {
                break;
            }
            let tok = self.advance();
            match &tok {
                Token::Character('{', Category::BeginGroup) => {
                    depth += 1;
                    result.push(tok);
                }
                Token::Character('}', Category::EndGroup) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    result.push(tok);
                }
                _ => {
                    result.push(tok);
                }
            }
        }
        result
    }

    /// Read a single undelimited argument: either a brace group or a single token.
    fn read_argument(&mut self) -> Vec<Token> {
        self.skip_spaces();
        match self.peek() {
            Token::Character('{', Category::BeginGroup) => self.read_brace_group(),
            _ => {
                let tok = self.advance();
                vec![tok]
            }
        }
    }

    /// Read a brace-delimited name string (e.g., `{foo}`). Returns the name.
    fn read_brace_name(&mut self) -> String {
        // consume '{'
        self.advance();
        let mut name = String::new();
        loop {
            match self.peek().clone() {
                Token::Character('}', Category::EndGroup) => {
                    self.advance();
                    break;
                }
                Token::EndOfInput => break,
                Token::Character(ch, _) => {
                    self.advance();
                    name.push(ch);
                }
                Token::ControlSequence(ref n) => {
                    // e.g. {\foo} — we return "foo" as the name
                    let n = n.clone();
                    self.advance();
                    name.push('\\');
                    name.push_str(&n);
                }
                Token::Space => {
                    self.advance();
                    // skip spaces inside the name braces
                }
                _ => {
                    self.advance();
                }
            }
        }
        name
    }

    /// Parse a `\def\name#1#2{body}` definition.
    fn parse_def(&mut self) {
        self.skip_spaces();

        // Read the control sequence being defined
        let name = match self.advance() {
            Token::ControlSequence(n) => n,
            _ => return, // malformed \def, skip
        };

        // Count parameter tokens in the parameter text
        let mut param_count: u8 = 0;
        loop {
            match self.peek() {
                Token::Parameter(_) => {
                    self.advance();
                    param_count += 1;
                }
                Token::Character('{', Category::BeginGroup) => break,
                Token::EndOfInput => return,
                _ => {
                    // skip delimited parameter text tokens (simplified)
                    self.advance();
                }
            }
        }

        // Read the body
        let body = self.read_brace_group();

        self.macros.insert(name, MacroDef { param_count, body });
    }

    /// Parse `\newcommand{\name}[n]{body}` or `\renewcommand{\name}[n]{body}`.
    fn parse_newcommand(&mut self) {
        self.skip_spaces();

        // Read the command name: {\name} or \name
        let name = match self.peek() {
            Token::Character('{', Category::BeginGroup) => {
                let n = self.read_brace_name();
                // Strip leading backslash if present
                if let Some(stripped) = n.strip_prefix('\\') {
                    stripped.to_string()
                } else {
                    n
                }
            }
            Token::ControlSequence(_) => {
                if let Token::ControlSequence(n) = self.advance() {
                    n
                } else {
                    return;
                }
            }
            _ => return,
        };

        // Optional: [n] for parameter count
        let mut param_count: u8 = 0;
        self.skip_spaces();
        if matches!(self.peek(), Token::Character('[', Category::Other)) {
            self.advance(); // consume '['
                            // Read the number
            if let Token::Character(c, _) = self.peek().clone() {
                if c.is_ascii_digit() {
                    self.advance();
                    param_count = c as u8 - b'0';
                }
            }
            // consume ']'
            if matches!(self.peek(), Token::Character(']', Category::Other)) {
                self.advance();
            }
        }

        self.skip_spaces();

        // Read the body
        let body = if matches!(self.peek(), Token::Character('{', Category::BeginGroup)) {
            self.read_brace_group()
        } else {
            Vec::new()
        };

        self.macros.insert(name, MacroDef { param_count, body });
    }

    /// Parse `\let\foo\bar` — alias \foo to have the same definition as \bar.
    fn parse_let(&mut self) {
        self.skip_spaces();

        let target = match self.advance() {
            Token::ControlSequence(n) => n,
            _ => return,
        };

        // Optional `=` between target and source
        self.skip_spaces();
        if matches!(self.peek(), Token::Character('=', Category::Other)) {
            self.advance();
        }
        self.skip_spaces();

        let source_tok = self.advance();
        match source_tok {
            Token::ControlSequence(ref src_name) => {
                if let Some(def) = self.macros.get(src_name).cloned() {
                    self.macros.insert(target, def);
                } else {
                    // Source is an undefined command — create an alias with zero params
                    // that expands to the control sequence itself
                    self.macros.insert(
                        target,
                        MacroDef {
                            param_count: 0,
                            body: vec![Token::ControlSequence(src_name.clone())],
                        },
                    );
                }
            }
            _ => {
                // \let\foo=<char token> — alias to the token
                self.macros.insert(
                    target,
                    MacroDef {
                        param_count: 0,
                        body: vec![source_tok],
                    },
                );
            }
        }
    }

    /// Evaluate `\if\tokenA\tokenB ... \else ... \fi`.
    /// Compares character codes of the two tokens.
    fn parse_if(&mut self) -> Vec<Token> {
        let tok_a = self.advance();
        let tok_b = self.advance();

        let char_a = self.token_char_code(&tok_a);
        let char_b = self.token_char_code(&tok_b);

        let condition = char_a == char_b;
        self.collect_conditional_branch(condition)
    }

    /// Evaluate `\ifx\tokenA\tokenB ... \else ... \fi`.
    /// Compares token meanings (same control sequence or same character+category).
    fn parse_ifx(&mut self) -> Vec<Token> {
        let tok_a = self.advance();
        let tok_b = self.advance();

        let condition = self.tokens_have_same_meaning(&tok_a, &tok_b);
        self.collect_conditional_branch(condition)
    }

    /// Evaluate `\ifnum <number> <rel> <number> ... \else ... \fi`.
    fn parse_ifnum(&mut self) -> Vec<Token> {
        self.skip_spaces();
        let lhs = self.read_number();
        self.skip_spaces();
        let rel = self.read_relation();
        self.skip_spaces();
        let rhs = self.read_number();

        let condition = match rel {
            '<' => lhs < rhs,
            '=' => lhs == rhs,
            '>' => lhs > rhs,
            _ => false,
        };

        self.collect_conditional_branch(condition)
    }

    /// Read an integer number from the token stream (possibly negative).
    fn read_number(&mut self) -> i64 {
        self.skip_spaces();
        let mut negative = false;
        if matches!(self.peek(), Token::Character('-', Category::Other)) {
            negative = true;
            self.advance();
            self.skip_spaces();
        } else if matches!(self.peek(), Token::Character('+', Category::Other)) {
            self.advance();
            self.skip_spaces();
        }

        let mut digits = String::new();
        loop {
            match self.peek() {
                Token::Character(c, _) if c.is_ascii_digit() => {
                    digits.push(*c);
                    self.advance();
                }
                _ => break,
            }
        }
        // Also skip one optional trailing space after a number in TeX
        if matches!(self.peek(), Token::Space) {
            self.advance();
        }

        let val: i64 = digits.parse().unwrap_or(0);
        if negative {
            -val
        } else {
            val
        }
    }

    /// Read a relational operator: `<`, `=`, or `>`.
    fn read_relation(&mut self) -> char {
        self.skip_spaces();
        match self.peek() {
            Token::Character('<', Category::Other) => {
                self.advance();
                '<'
            }
            Token::Character('=', Category::Other) => {
                self.advance();
                '='
            }
            Token::Character('>', Category::Other) => {
                self.advance();
                '>'
            }
            _ => {
                self.advance(); // consume unknown
                '?'
            }
        }
    }

    /// Extract the character code from a token for `\if` comparison.
    fn token_char_code(&self, tok: &Token) -> Option<char> {
        match tok {
            Token::Character(ch, _) => Some(*ch),
            Token::ControlSequence(name) => {
                // In TeX, \if compares the replacement text of macros,
                // but for simplicity we compare the first char of the name
                // or treat it as having char code 256 (no match with chars)
                if name.len() == 1 {
                    name.chars().next()
                } else {
                    None
                }
            }
            Token::Space => Some(' '),
            Token::Active(ch) => Some(*ch),
            _ => None,
        }
    }

    /// Check if two tokens have the same meaning for `\ifx`.
    fn tokens_have_same_meaning(&self, a: &Token, b: &Token) -> bool {
        match (a, b) {
            (Token::ControlSequence(na), Token::ControlSequence(nb)) => {
                // Same if both are undefined, or both have the same definition
                let def_a = self.macros.get(na);
                let def_b = self.macros.get(nb);
                match (def_a, def_b) {
                    (None, None) => true, // both undefined → equal
                    (Some(da), Some(db)) => da.param_count == db.param_count && da.body == db.body,
                    _ => false,
                }
            }
            (Token::Character(ca, cata), Token::Character(cb, catb)) => ca == cb && cata == catb,
            (Token::Space, Token::Space) => true,
            (Token::Active(a_ch), Token::Active(b_ch)) => a_ch == b_ch,
            _ => false,
        }
    }

    /// Collect the true or false branch of a conditional.
    /// If `condition` is true, collect the true branch; otherwise collect the else branch.
    /// Handles nested conditionals.
    fn collect_conditional_branch(&mut self, condition: bool) -> Vec<Token> {
        let mut true_branch = Vec::new();
        let mut false_branch = Vec::new();
        let mut in_else = false;
        let mut depth = 1u32; // nesting depth of \if..\fi

        loop {
            if self.at_end() {
                break;
            }

            match self.peek().clone() {
                Token::ControlSequence(ref name)
                    if name == "if" || name == "ifx" || name == "ifnum" =>
                {
                    let tok = self.advance();
                    depth += 1;
                    if in_else {
                        false_branch.push(tok);
                    } else {
                        true_branch.push(tok);
                    }
                }
                Token::ControlSequence(ref name) if name == "fi" => {
                    self.advance();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    let tok = Token::ControlSequence("fi".to_string());
                    if in_else {
                        false_branch.push(tok);
                    } else {
                        true_branch.push(tok);
                    }
                }
                Token::ControlSequence(ref name) if name == "else" && depth == 1 => {
                    self.advance();
                    in_else = true;
                }
                _ => {
                    let tok = self.advance();
                    if in_else {
                        false_branch.push(tok);
                    } else {
                        true_branch.push(tok);
                    }
                }
            }
        }

        if condition {
            true_branch
        } else {
            false_branch
        }
    }

    /// Perform parameter substitution on a macro body.
    fn substitute_params(body: &[Token], args: &[Vec<Token>]) -> Vec<Token> {
        let mut result = Vec::new();
        for tok in body {
            match tok {
                Token::Parameter(n) if *n >= 1 && (*n as usize) <= args.len() => {
                    result.extend(args[*n as usize - 1].iter().cloned());
                }
                Token::Parameter(0) => {
                    // ## → # in output
                    result.push(Token::Character('#', Category::Parameter));
                }
                _ => {
                    result.push(tok.clone());
                }
            }
        }
        result
    }

    /// Expand all tokens, processing definitions, macro calls, and conditionals.
    /// Returns the fully expanded token stream.
    pub fn expand_all(&mut self) -> Vec<Token> {
        let mut output = Vec::new();
        let mut iteration_limit = 100_000u32;

        while !self.at_end() && iteration_limit > 0 {
            iteration_limit -= 1;

            match self.peek().clone() {
                Token::ControlSequence(ref name) if name == "def" => {
                    self.advance(); // consume \def
                    self.parse_def();
                }
                Token::ControlSequence(ref name)
                    if name == "newcommand" || name == "renewcommand" =>
                {
                    self.advance(); // consume \newcommand or \renewcommand
                    self.parse_newcommand();
                }
                Token::ControlSequence(ref name) if name == "let" => {
                    self.advance(); // consume \let
                    self.parse_let();
                }
                Token::ControlSequence(ref name) if name == "if" => {
                    self.advance(); // consume \if
                    let branch_tokens = self.parse_if();
                    // Splice the branch tokens back into the stream for further expansion
                    self.splice_tokens(branch_tokens);
                }
                Token::ControlSequence(ref name) if name == "ifx" => {
                    self.advance(); // consume \ifx
                    let branch_tokens = self.parse_ifx();
                    self.splice_tokens(branch_tokens);
                }
                Token::ControlSequence(ref name) if name == "ifnum" => {
                    self.advance(); // consume \ifnum
                    let branch_tokens = self.parse_ifnum();
                    self.splice_tokens(branch_tokens);
                }
                Token::ControlSequence(ref name) => {
                    let name = name.clone();
                    if let Some(def) = self.macros.get(&name).cloned() {
                        self.advance(); // consume the macro call
                                        // Read arguments
                        let mut args = Vec::new();
                        for _ in 0..def.param_count {
                            let arg = self.read_argument();
                            args.push(arg);
                        }
                        // Substitute parameters
                        let expanded = Self::substitute_params(&def.body, &args);
                        // Splice back for further expansion
                        self.splice_tokens(expanded);
                    } else {
                        // Unknown command — pass through
                        let tok = self.advance();
                        output.push(tok);
                    }
                }
                _ => {
                    let tok = self.advance();
                    output.push(tok);
                }
            }
        }

        // Ensure EndOfInput at end
        if !output.iter().any(|t| matches!(t, Token::EndOfInput)) {
            output.push(Token::EndOfInput);
        }

        output
    }

    /// Splice tokens into the stream at the current position for re-expansion.
    fn splice_tokens(&mut self, tokens: Vec<Token>) {
        // Remove EndOfInput from spliced tokens if present
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t, Token::EndOfInput))
            .collect();

        // Insert at current position
        let pos = self.pos;
        for (i, tok) in tokens.into_iter().enumerate() {
            self.tokens.insert(pos + i, tok);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Test 1: Zero-arg \def ===
    #[test]
    fn test_def_zero_arg() {
        let src = r"\def\hello{world}\hello";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        // \hello should expand to "world"
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], Node::Text("world".to_string()));
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 2: \def with one parameter ===
    #[test]
    fn test_def_one_param() {
        let src = r"\def\greet#1{Hello, #1!}\greet{World}";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                // Should produce "Hello, World!"
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("Hello"),
                    "Expected 'Hello' in output, got: {:?}",
                    nodes
                );
                assert!(
                    text.contains("World"),
                    "Expected 'World' in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 3: \def with two parameters ===
    #[test]
    fn test_def_two_params() {
        let src = r"\def\pair#1#2{(#1, #2)}\pair{a}{b}";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("a") && text.contains("b"),
                    "Expected 'a' and 'b' in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 4: \newcommand with zero args ===
    #[test]
    fn test_newcommand_zero_args() {
        let src = r"\newcommand{\foo}{bar}\foo";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], Node::Text("bar".to_string()));
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 5: \newcommand with params ===
    #[test]
    fn test_newcommand_with_params() {
        let src = r"\newcommand{\bold}[1]{\textbf{#1}}\bold{hello}";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                // Should produce \textbf{hello}
                assert!(
                    nodes
                        .iter()
                        .any(|n| matches!(n, Node::Command { name, .. } if name == "textbf")),
                    "Expected \\textbf command in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 6: \renewcommand replaces existing ===
    #[test]
    fn test_renewcommand() {
        let src = r"\newcommand{\foo}{old}\renewcommand{\foo}{new}\foo";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], Node::Text("new".to_string()));
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 7: \let alias ===
    #[test]
    fn test_let_alias() {
        let src = r"\def\foo{hello}\let\bar\foo\bar";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], Node::Text("hello".to_string()));
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 8: \if true branch ===
    #[test]
    fn test_if_true() {
        let src = r"\if aa YES\else NO\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("YES"),
                    "Expected YES in output, got: {:?}",
                    nodes
                );
                assert!(
                    !text.contains("NO"),
                    "Should NOT contain NO, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 9: \if false branch ===
    #[test]
    fn test_if_false() {
        let src = r"\if ab YES\else NO\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("NO"),
                    "Expected NO in output, got: {:?}",
                    nodes
                );
                assert!(
                    !text.contains("YES"),
                    "Should NOT contain YES, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 10: \ifx same meaning ===
    #[test]
    fn test_ifx_same() {
        let src = r"\def\foo{x}\def\bar{x}\ifx\foo\bar SAME\else DIFF\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("SAME"),
                    "Expected SAME in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 11: \ifx different meaning ===
    #[test]
    fn test_ifx_different() {
        let src = r"\def\foo{x}\def\bar{y}\ifx\foo\bar SAME\else DIFF\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("DIFF"),
                    "Expected DIFF in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 12: \ifnum less than ===
    #[test]
    fn test_ifnum_less() {
        let src = r"\ifnum 1<2 YES\else NO\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("YES"),
                    "Expected YES for 1<2, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 13: \ifnum equal ===
    #[test]
    fn test_ifnum_equal() {
        let src = r"\ifnum 42=42 EQ\else NEQ\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    text.contains("EQ"),
                    "Expected EQ for 42=42, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 14: \ifnum greater than (false) ===
    #[test]
    fn test_ifnum_greater_false() {
        let src = r"\ifnum 1>2 YES\else NO\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(text.contains("NO"), "Expected NO for 1>2, got: {:?}", nodes);
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 15: Macro expansion with nested macro ===
    #[test]
    fn test_nested_macro_expansion() {
        let src = r"\def\inner{core}\def\outer{\inner}\outer";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], Node::Text("core".to_string()));
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 16: \if without \else ===
    #[test]
    fn test_if_without_else_true() {
        let src = r"\if aa YES\fi";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(text.contains("YES"), "Expected YES, got: {:?}", nodes);
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 17: \if without \else (false, nothing output) ===
    #[test]
    fn test_if_without_else_false() {
        let src = r"\if ab YES\fi rest";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        match doc {
            Node::Document(nodes) => {
                let text: String = nodes
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    !text.contains("YES"),
                    "Should not contain YES, got: {:?}",
                    nodes
                );
                assert!(
                    text.contains("rest"),
                    "Should contain 'rest', got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 18: \let with undefined source ===
    #[test]
    fn test_let_undefined_source() {
        let src = r"\let\foo\relax\foo";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        // \foo should expand to \relax (the body stored by \let)
        match doc {
            Node::Document(nodes) => {
                assert!(
                    nodes
                        .iter()
                        .any(|n| matches!(n, Node::Command { name, .. } if name == "relax")),
                    "Expected \\relax in output, got: {:?}",
                    nodes
                );
            }
            _ => panic!("Expected Document"),
        }
    }

    // === Test 19: MacroTable direct construction ===
    #[test]
    fn test_macro_table() {
        let mut table = MacroTable::new();
        table.insert(
            "test".to_string(),
            MacroDef {
                param_count: 0,
                body: vec![Token::Character('x', Category::Letter)],
            },
        );
        assert_eq!(table.len(), 1);
        assert!(table.contains_key("test"));
        assert_eq!(table["test"].param_count, 0);
    }

    // === Test 20: Expander preserves unknown commands ===
    #[test]
    fn test_unknown_commands_pass_through() {
        let src = r"\textbf{hello}";
        let mut exp = Expander::new(src);
        let doc = exp.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Command {
                name: "textbf".to_string(),
                args: vec![Node::Group(vec![Node::Text("hello".to_string())])]
            }])
        );
    }
}
