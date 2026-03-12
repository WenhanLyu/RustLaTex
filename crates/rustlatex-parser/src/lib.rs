//! `rustlatex-parser` — LaTeX AST parser
//!
//! This crate implements the second stage of the LaTeX compilation pipeline:
//! parsing the token stream produced by `rustlatex-lexer` into an Abstract
//! Syntax Tree (AST) representing the document structure.
//!
//! The [`expander`] module adds M4-level macro expansion as a preprocessing
//! stage: `\def`, `\newcommand`, `\renewcommand`, `\let`, `\if`, `\ifx`,
//! and `\ifnum` are evaluated before the token stream reaches the parser.

pub mod expander;

pub use expander::{Expander, MacroDef, MacroTable};
use rustlatex_lexer::{Category, Lexer, Token};

/// A node in the LaTeX document AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A LaTeX document containing a sequence of nodes.
    Document(Vec<Node>),
    /// A control sequence command with optional argument nodes.
    Command { name: String, args: Vec<Node> },
    /// A group delimited by `{` and `}`.
    Group(Vec<Node>),
    /// Plain text content.
    Text(String),
    /// A math mode expression (inline): `$...$`.
    InlineMath(Vec<Node>),
    /// A LaTeX environment: `\begin{name}...\end{name}`.
    Environment {
        name: String,
        options: Option<Vec<Node>>,
        content: Vec<Node>,
    },
    /// A paragraph (sequence of nodes separated by blank lines).
    Paragraph(Vec<Node>),
    /// Display math mode: `$$...$$`.
    DisplayMath(Vec<Node>),
    /// Superscript: `base^exponent` in math mode.
    Superscript {
        base: Box<Node>,
        exponent: Box<Node>,
    },
    /// Subscript: `base_subscript` in math mode.
    Subscript {
        base: Box<Node>,
        subscript: Box<Node>,
    },
    /// Fraction: `\frac{numerator}{denominator}` in math mode.
    Fraction {
        numerator: Box<Node>,
        denominator: Box<Node>,
    },
    /// Radical: `\sqrt[degree]{radicand}` in math mode.
    Radical {
        degree: Option<Box<Node>>,
        radicand: Box<Node>,
    },
    /// A `{...}` group inside math mode.
    MathGroup(Vec<Node>),
}

/// Internal sentinel returned when `\end{name}` is encountered during parsing.
#[derive(Debug, Clone, PartialEq)]
enum ParseEvent {
    Node(Node),
    EndEnvironment(String),
}

/// Parser state machine.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// The original source text, used for verbatim environments.
    #[allow(dead_code)]
    source: Option<String>,
}

impl Parser {
    /// Create a parser from a LaTeX source string.
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            pos: 0,
            source: Some(source.to_string()),
        }
    }

    /// Create a parser from a pre-built token vector (used by the expander).
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            source: None,
        }
    }

    /// Peek at the current token.
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EndOfInput)
    }

    /// Peek at the token at offset `n` from current position.
    fn peek_at(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or(&Token::EndOfInput)
    }

    /// Advance past the current token and return it.
    fn advance(&mut self) -> Token {
        let tok = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token::EndOfInput);
        self.pos += 1;
        tok
    }

    /// Parse the entire document into a [`Node::Document`].
    pub fn parse(&mut self) -> Node {
        let nodes = self.parse_body(None);
        Node::Document(nodes)
    }

    /// Read a brace-delimited name: `{name}`. Returns the name string.
    /// Expects current token to be `{`.
    fn parse_brace_name(&mut self) -> String {
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
                Token::ControlSequence(_) => {
                    // unexpected but don't loop forever
                    break;
                }
                Token::Space => {
                    self.advance();
                    // skip spaces in env name
                }
                _ => {
                    self.advance();
                }
            }
        }
        name
    }

    /// After consuming a command name, greedily consume optional `[...]` and
    /// mandatory `{...}` arguments.
    fn parse_command_args(&mut self) -> Vec<Node> {
        let mut args = Vec::new();

        loop {
            match self.peek().clone() {
                // Optional argument: [...]
                Token::Character('[', Category::Other) => {
                    self.advance(); // consume '['
                    let inner = self.parse_until_bracket();
                    args.push(Node::Group(inner));
                }
                // Mandatory argument: {...}
                Token::Character('{', Category::BeginGroup) => {
                    self.advance(); // consume '{'
                    let inner = self.parse_nodes_inner(true, None);
                    args.push(Node::Group(inner));
                }
                _ => break,
            }
        }

        args
    }

    /// Parse nodes until `]` is encountered (for optional arguments).
    fn parse_until_bracket(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            match self.peek().clone() {
                Token::Character(']', Category::Other) => {
                    self.advance(); // consume ']'
                    break;
                }
                Token::EndOfInput => break,
                Token::Character('}', Category::EndGroup) => break,
                Token::ControlSequence(ref name) => {
                    let name = name.clone();
                    self.advance();
                    let args = self.parse_command_args();
                    nodes.push(Node::Command { name, args });
                }
                Token::Character('{', Category::BeginGroup) => {
                    self.advance();
                    let inner = self.parse_nodes_inner(true, None);
                    nodes.push(Node::Group(inner));
                }
                Token::Character(ch, cat) => {
                    // Accumulate text, but stop at ], {, }, $
                    if cat == Category::MathShift {
                        // handle math inside brackets if needed
                        if let Some(ParseEvent::Node(n)) = self.parse_single_event(None) {
                            nodes.push(n);
                        }
                    } else {
                        self.advance();
                        let mut text = ch.to_string();
                        loop {
                            match self.peek().clone() {
                                Token::Character(']', Category::Other) => break,
                                Token::Character(c, cat2) => match cat2 {
                                    Category::BeginGroup
                                    | Category::EndGroup
                                    | Category::MathShift => break,
                                    _ => {
                                        self.advance();
                                        text.push(c);
                                    }
                                },
                                Token::Space => {
                                    self.advance();
                                    text.push(' ');
                                }
                                _ => break,
                            }
                        }
                        nodes.push(Node::Text(text));
                    }
                }
                Token::Space => {
                    self.advance();
                    let mut text = String::from(" ");
                    loop {
                        match self.peek().clone() {
                            Token::Character(']', Category::Other) => break,
                            Token::Character(c, cat) => match cat {
                                Category::BeginGroup | Category::EndGroup | Category::MathShift => {
                                    break
                                }
                                _ => {
                                    self.advance();
                                    text.push(c);
                                }
                            },
                            Token::Space => {
                                self.advance();
                                text.push(' ');
                            }
                            _ => break,
                        }
                    }
                    nodes.push(Node::Text(text));
                }
                _ => {
                    self.advance();
                }
            }
        }
        nodes
    }

    /// Parse the body of a document or environment. If `end_env` is Some, stop
    /// when `\end{name}` matching `end_env` is found. Handles paragraph grouping.
    fn parse_body(&mut self, end_env: Option<&str>) -> Vec<Node> {
        let events = self.parse_events(end_env);

        // Check if there are any Par-based paragraph boundaries
        let has_par = events
            .iter()
            .any(|e| matches!(e, ParseEvent::Node(Node::Text(t)) if t == "\u{FFFF}PAR\u{FFFF}"));

        if !has_par {
            // No paragraph breaks — return nodes as-is (backward compatibility)
            return events
                .into_iter()
                .filter_map(|e| match e {
                    ParseEvent::Node(n) => Some(n),
                    ParseEvent::EndEnvironment(_) => None,
                })
                .collect();
        }

        // Split by paragraph markers
        let mut paragraphs: Vec<Vec<Node>> = Vec::new();
        let mut current: Vec<Node> = Vec::new();

        for event in events {
            match event {
                ParseEvent::Node(Node::Text(ref t)) if t == "\u{FFFF}PAR\u{FFFF}" => {
                    if !current.is_empty() {
                        paragraphs.push(current);
                        current = Vec::new();
                    }
                }
                ParseEvent::Node(n) => current.push(n),
                ParseEvent::EndEnvironment(_) => {}
            }
        }
        if !current.is_empty() {
            paragraphs.push(current);
        }

        paragraphs.into_iter().map(Node::Paragraph).collect()
    }

    /// Parse a flat list of events (nodes + end-environment sentinels).
    fn parse_events(&mut self, end_env: Option<&str>) -> Vec<ParseEvent> {
        let mut events = Vec::new();

        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character('}', Category::EndGroup) => {
                    // If we're inside a group, the caller handles this
                    break;
                }
                Token::Par => {
                    self.advance();
                    // Insert a sentinel for paragraph boundary
                    events.push(ParseEvent::Node(Node::Text(
                        "\u{FFFF}PAR\u{FFFF}".to_string(),
                    )));
                }
                _ => {
                    if let Some(event) = self.parse_single_event(end_env) {
                        match &event {
                            ParseEvent::EndEnvironment(name) => {
                                if let Some(expected) = end_env {
                                    if name == expected {
                                        break;
                                    }
                                }
                                // Mismatched end — just drop it
                                break;
                            }
                            ParseEvent::Node(_) => {
                                events.push(event);
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Parse a single event (node or end-environment sentinel).
    /// Returns None if we hit end-of-input or end-group without producing anything.
    fn parse_single_event(&mut self, _end_env: Option<&str>) -> Option<ParseEvent> {
        match self.peek().clone() {
            Token::EndOfInput => None,
            Token::Character('}', Category::EndGroup) => None,
            Token::ControlSequence(ref name) => {
                let name = name.clone();
                self.advance();

                if name == "begin" {
                    // Parse environment
                    let env_name = self.parse_brace_name();
                    if env_name == "verbatim" {
                        // Verbatim environment: consume raw tokens until \end{verbatim}
                        let raw_content = self.parse_verbatim_content();
                        Some(ParseEvent::Node(Node::Environment {
                            name: env_name,
                            options: None,
                            content: vec![Node::Text(raw_content)],
                        }))
                    } else {
                        let content = self.parse_body(Some(&env_name));
                        Some(ParseEvent::Node(Node::Environment {
                            name: env_name,
                            options: None,
                            content,
                        }))
                    }
                } else if name == "end" {
                    let env_name = self.parse_brace_name();
                    Some(ParseEvent::EndEnvironment(env_name))
                } else {
                    let args = self.parse_command_args();
                    Some(ParseEvent::Node(Node::Command { name, args }))
                }
            }
            Token::Character('{', Category::BeginGroup) => {
                self.advance(); // consume '{'
                let inner = self.parse_nodes_inner(true, None);
                Some(ParseEvent::Node(Node::Group(inner)))
            }
            Token::Character('$', Category::MathShift) => {
                // Check for display math ($$)
                if matches!(self.peek_at(1), Token::Character('$', Category::MathShift)) {
                    // Display math
                    self.advance(); // first $
                    self.advance(); // second $
                    let inner = self.parse_math_content(true);
                    Some(ParseEvent::Node(Node::DisplayMath(inner)))
                } else {
                    // Inline math
                    self.advance(); // consume $
                    let inner = self.parse_math_content(false);
                    Some(ParseEvent::Node(Node::InlineMath(inner)))
                }
            }
            Token::Character(ch, _cat) => {
                self.advance();
                // Accumulate consecutive text characters (letters, other, spaces)
                let mut text = ch.to_string();
                loop {
                    match self.peek().clone() {
                        Token::Character(c, cat) => match cat {
                            Category::BeginGroup | Category::EndGroup | Category::MathShift => {
                                break
                            }
                            _ => {
                                self.advance();
                                text.push(c);
                            }
                        },
                        Token::Space => {
                            self.advance();
                            text.push(' ');
                        }
                        _ => break,
                    }
                }
                Some(ParseEvent::Node(Node::Text(text)))
            }
            Token::Space => {
                self.advance();
                // Accumulate space with following text
                let mut text = String::from(" ");
                loop {
                    match self.peek().clone() {
                        Token::Character(c, cat) => match cat {
                            Category::BeginGroup | Category::EndGroup | Category::MathShift => {
                                break
                            }
                            _ => {
                                self.advance();
                                text.push(c);
                            }
                        },
                        Token::Space => {
                            self.advance();
                            text.push(' ');
                        }
                        _ => break,
                    }
                }
                Some(ParseEvent::Node(Node::Text(text)))
            }
            Token::Par => {
                // Handled in parse_events; shouldn't reach here normally
                self.advance();
                None
            }
            Token::Active(ch) => {
                self.advance();
                Some(ParseEvent::Node(Node::Text(ch.to_string())))
            }
            Token::Parameter(n) => {
                self.advance();
                Some(ParseEvent::Node(Node::Text(format!("#{}", n))))
            }
        }
    }

    /// Parse a single math atom: a `{...}` group (→ `MathGroup`) or a single
    /// character/token (→ `Text`). Used as the operand for `^` and `_`.
    fn parse_math_atom(&mut self) -> Node {
        match self.peek().clone() {
            Token::Character('{', Category::BeginGroup) => {
                self.advance(); // consume '{'
                let inner = self.parse_math_group_inner();
                Node::MathGroup(inner)
            }
            Token::ControlSequence(ref name) => {
                let name = name.clone();
                self.advance();
                Node::Command { name, args: vec![] }
            }
            Token::Character(ch, _) => {
                self.advance();
                Node::Text(ch.to_string())
            }
            Token::Space => {
                self.advance();
                Node::Text(" ".to_string())
            }
            _ => Node::Text(String::new()),
        }
    }

    /// Parse the contents of a `{...}` group in math mode, consuming the
    /// closing `}`. Returns a `Vec<Node>` suitable for `MathGroup`.
    fn parse_math_group_inner(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character('}', Category::EndGroup) => {
                    self.advance(); // consume '}'
                    break;
                }
                _ => {
                    if let Some(node) = self.parse_math_node() {
                        // Check for ^ or _ following this node
                        let node = self.maybe_attach_scripts(node);
                        nodes.push(node);
                    }
                }
            }
        }
        nodes
    }

    /// Try to attach `^` (superscript) or `_` (subscript) to `base`.
    /// Handles chained scripts like `x^2_i` or `x_i^n`.
    fn maybe_attach_scripts(&mut self, mut base: Node) -> Node {
        loop {
            match self.peek().clone() {
                Token::Character('^', Category::Superscript) => {
                    self.advance(); // consume '^'
                    let exponent = self.parse_math_atom();
                    base = Node::Superscript {
                        base: Box::new(base),
                        exponent: Box::new(exponent),
                    };
                }
                Token::Character('_', Category::Subscript) => {
                    self.advance(); // consume '_'
                    let subscript = self.parse_math_atom();
                    base = Node::Subscript {
                        base: Box::new(base),
                        subscript: Box::new(subscript),
                    };
                }
                _ => break,
            }
        }
        base
    }

    /// Parse a single math-mode node (without script attachment).
    /// Returns `None` only when we should stop (end of input, `$`, `}`).
    fn parse_math_node(&mut self) -> Option<Node> {
        match self.peek().clone() {
            Token::EndOfInput => None,
            Token::Character('$', Category::MathShift) => None,
            Token::Character('}', Category::EndGroup) => None,
            Token::ControlSequence(ref name) => {
                let name = name.clone();
                self.advance();
                if name == "frac" {
                    // \frac{numerator}{denominator}
                    let num = self.parse_math_mandatory_group();
                    let den = self.parse_math_mandatory_group();
                    Some(Node::Fraction {
                        numerator: Box::new(num),
                        denominator: Box::new(den),
                    })
                } else if name == "sqrt" {
                    // \sqrt[degree]{radicand}  or  \sqrt{radicand}
                    let degree = if matches!(self.peek(), Token::Character('[', Category::Other)) {
                        self.advance(); // consume '['
                        let inner = self.parse_math_optional_bracket();
                        Some(Box::new(Node::MathGroup(inner)))
                    } else {
                        None
                    };
                    let radicand = self.parse_math_mandatory_group();
                    Some(Node::Radical {
                        degree,
                        radicand: Box::new(radicand),
                    })
                } else {
                    // Generic math command — parse {…} args as MathGroup
                    let args = self.parse_math_command_args();
                    Some(Node::Command { name, args })
                }
            }
            Token::Character('{', Category::BeginGroup) => {
                self.advance(); // consume '{'
                let inner = self.parse_math_group_inner();
                Some(Node::MathGroup(inner))
            }
            Token::Character(ch, _) => {
                self.advance();
                // Single character — stop accumulating at superscript/subscript/group chars
                Some(Node::Text(ch.to_string()))
            }
            Token::Space => {
                self.advance();
                Some(Node::Text(" ".to_string()))
            }
            _ => {
                self.advance();
                None
            }
        }
    }

    /// Parse a mandatory `{…}` argument in math mode → `MathGroup`.
    fn parse_math_mandatory_group(&mut self) -> Node {
        match self.peek().clone() {
            Token::Character('{', Category::BeginGroup) => {
                self.advance(); // consume '{'
                let inner = self.parse_math_group_inner();
                Node::MathGroup(inner)
            }
            // Bare token (no braces) — treat as single-token group
            _ => {
                let atom = self.parse_math_atom();
                Node::MathGroup(vec![atom])
            }
        }
    }

    /// Parse the content of `[…]` inside math mode (for `\sqrt[n]{…}`).
    /// Consumes the closing `]`.
    fn parse_math_optional_bracket(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character(']', Category::Other) => {
                    self.advance(); // consume ']'
                    break;
                }
                _ => {
                    if let Some(node) = self.parse_math_node() {
                        let node = self.maybe_attach_scripts(node);
                        nodes.push(node);
                    }
                }
            }
        }
        nodes
    }

    /// Parse `{…}` args for a generic math command. Produces `MathGroup` nodes
    /// (not `Group`) so math structure is preserved.
    fn parse_math_command_args(&mut self) -> Vec<Node> {
        let mut args = Vec::new();
        loop {
            match self.peek().clone() {
                Token::Character('[', Category::Other) => {
                    self.advance(); // consume '['
                    let inner = self.parse_math_optional_bracket();
                    args.push(Node::MathGroup(inner));
                }
                Token::Character('{', Category::BeginGroup) => {
                    self.advance(); // consume '{'
                    let inner = self.parse_math_group_inner();
                    args.push(Node::MathGroup(inner));
                }
                _ => break,
            }
        }
        args
    }

    /// Parse math mode content until closing delimiter.
    /// If `display` is true, stop at `$$`; otherwise stop at single `$`.
    fn parse_math_content(&mut self, display: bool) -> Vec<Node> {
        let mut nodes = Vec::new();

        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character('$', Category::MathShift) => {
                    if display {
                        // Need $$
                        if matches!(self.peek_at(1), Token::Character('$', Category::MathShift)) {
                            self.advance(); // first $
                            self.advance(); // second $
                            break;
                        } else {
                            // Single $ inside display math — treat as content? Unlikely but handle
                            self.advance();
                            nodes.push(Node::Text("$".to_string()));
                        }
                    } else {
                        // Single $ closes inline math
                        self.advance();
                        break;
                    }
                }
                _ => {
                    if let Some(node) = self.parse_math_node() {
                        let node = self.maybe_attach_scripts(node);
                        nodes.push(node);
                    }
                }
            }
        }

        nodes
    }

    /// Parse a sequence of nodes inside a group (in_group=true) or body.
    /// This is the low-level version that does NOT do paragraph grouping.
    fn parse_nodes_inner(&mut self, in_group: bool, end_env: Option<&str>) -> Vec<Node> {
        let mut nodes = Vec::new();

        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character('}', Category::EndGroup) => {
                    if in_group {
                        self.advance(); // consume '}'
                    }
                    break;
                }
                Token::Par => {
                    self.advance();
                    // skip Par tokens inside groups
                }
                _ => {
                    if let Some(event) = self.parse_single_event(end_env) {
                        match event {
                            ParseEvent::Node(n) => nodes.push(n),
                            ParseEvent::EndEnvironment(name) => {
                                if let Some(expected) = end_env {
                                    if name == expected {
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        nodes
    }

    /// Consume tokens inside a verbatim environment until `\end{verbatim}` is found.
    /// Returns the raw text content without interpreting LaTeX commands.
    fn parse_verbatim_content(&mut self) -> String {
        let mut raw = String::new();
        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::ControlSequence(ref cs_name) if cs_name == "end" => {
                    // Check if this is \end{verbatim}
                    let saved_pos = self.pos;
                    self.advance(); // consume \end
                                    // Check for {verbatim}
                    if matches!(self.peek(), Token::Character('{', Category::BeginGroup)) {
                        let env_name = self.parse_brace_name();
                        if env_name == "verbatim" {
                            // Done — consumed \end{verbatim}
                            break;
                        }
                        // Not verbatim — add \end{name} as raw text and continue
                        raw.push_str(&format!("\\end{{{}}}", env_name));
                    } else {
                        // \end not followed by { — treat as raw text
                        raw.push_str("\\end");
                        // pos already advanced past \end, don't restore
                        let _ = saved_pos;
                    }
                }
                Token::ControlSequence(ref cs_name) => {
                    let cs = cs_name.clone();
                    self.advance();
                    raw.push('\\');
                    raw.push_str(&cs);
                }
                Token::Character(ch, _) => {
                    self.advance();
                    raw.push(ch);
                }
                Token::Space => {
                    self.advance();
                    raw.push(' ');
                }
                Token::Par => {
                    self.advance();
                    raw.push('\n');
                    raw.push('\n');
                }
                Token::Active(ch) => {
                    self.advance();
                    raw.push(ch);
                }
                Token::Parameter(n) => {
                    self.advance();
                    raw.push('#');
                    // n is a u8 digit
                    raw.push(char::from(b'0' + n));
                }
            }
        }
        raw
    }

    // Keep the old method signature for backward compatibility in case anything references it,
    // but we won't use it internally anymore.
    #[allow(dead_code)]
    fn parse_nodes(&mut self, in_group: bool) -> Vec<Node> {
        self.parse_nodes_inner(in_group, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Existing 4 tests (must still pass) =====

    #[test]
    fn test_parse_empty() {
        let mut parser = Parser::new("");
        let doc = parser.parse();
        assert_eq!(doc, Node::Document(vec![]));
    }

    #[test]
    fn test_parse_command() {
        let mut parser = Parser::new(r"\hello");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Command {
                name: "hello".to_string(),
                args: vec![]
            }])
        );
    }

    #[test]
    fn test_parse_text() {
        let mut parser = Parser::new("Hello");
        let doc = parser.parse();
        assert_eq!(doc, Node::Document(vec![Node::Text("Hello".to_string())]));
    }

    #[test]
    fn test_parse_group() {
        let mut parser = Parser::new("{hello}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Group(vec![Node::Text("hello".to_string())])])
        );
    }

    // ===== New tests (M3) =====

    #[test]
    fn test_command_with_mandatory_arg() {
        let mut parser = Parser::new(r"\textbf{bold}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Command {
                name: "textbf".to_string(),
                args: vec![Node::Group(vec![Node::Text("bold".to_string())])]
            }])
        );
    }

    #[test]
    fn test_command_with_optional_and_mandatory_arg() {
        let mut parser = Parser::new(r"\section[short]{Long}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Command {
                name: "section".to_string(),
                args: vec![
                    Node::Group(vec![Node::Text("short".to_string())]),
                    Node::Group(vec![Node::Text("Long".to_string())]),
                ]
            }])
        );
    }

    #[test]
    fn test_inline_math() {
        // $x^2$ now produces a structured Superscript node
        let mut parser = Parser::new(r"$x^2$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Text("x".to_string())),
                exponent: Box::new(Node::Text("2".to_string())),
            }])])
        );
    }

    #[test]
    fn test_display_math() {
        // $$E=mc^2$$ — 'E' followed by '=', 'm', 'c' (text chars), then ^2 (superscript)
        let mut parser = Parser::new(r"$$E=mc^2$$");
        let doc = parser.parse();
        // E, =, m, c get accumulated... but ^ stops accumulation (single chars now)
        // With single-char accumulation: Text("E"), Text("="), Text("m"), Superscript{base:Text("c"), exp:Text("2")}
        match doc {
            Node::Document(ref nodes) => {
                assert_eq!(nodes.len(), 1);
                assert!(matches!(&nodes[0], Node::DisplayMath(_)));
                if let Node::DisplayMath(ref inner) = nodes[0] {
                    // Must contain a Superscript node
                    assert!(inner.iter().any(|n| matches!(n, Node::Superscript { .. })));
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn test_simple_environment() {
        let mut parser = Parser::new(r"\begin{document}hello\end{document}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "document".to_string(),
                options: None,
                content: vec![Node::Text("hello".to_string())]
            }])
        );
    }

    #[test]
    fn test_nested_environments() {
        let src = r"\begin{document}\begin{itemize}inner\end{itemize}\end{document}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "document".to_string(),
                options: None,
                content: vec![Node::Environment {
                    name: "itemize".to_string(),
                    options: None,
                    content: vec![Node::Text("inner".to_string())]
                }]
            }])
        );
    }

    #[test]
    fn test_environment_with_content_commands() {
        let src = r"\begin{document}\textbf{hi}\end{document}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "document".to_string(),
                options: None,
                content: vec![Node::Command {
                    name: "textbf".to_string(),
                    args: vec![Node::Group(vec![Node::Text("hi".to_string())])]
                }]
            }])
        );
    }

    #[test]
    fn test_paragraph_from_par_token() {
        // "a\n\nb" → lexer produces: a Space Par b
        let mut parser = Parser::new("a\n\nb");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![
                Node::Paragraph(vec![Node::Text("a ".to_string())]),
                Node::Paragraph(vec![Node::Text("b".to_string())]),
            ])
        );
    }

    #[test]
    fn test_multiple_paragraphs() {
        let mut parser = Parser::new("first\n\nsecond\n\nthird");
        let doc = parser.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 3);
                assert!(matches!(&nodes[0], Node::Paragraph(_)));
                assert!(matches!(&nodes[1], Node::Paragraph(_)));
                assert!(matches!(&nodes[2], Node::Paragraph(_)));
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn test_inline_math_in_paragraph() {
        let mut parser = Parser::new("text $x$ more\n\nnext");
        let doc = parser.parse();
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 2);
                // First paragraph should contain text, inline math, text
                if let Node::Paragraph(ref inner) = nodes[0] {
                    assert!(inner.len() >= 2); // at least text + math
                                               // Check that inline math is present
                    assert!(inner.iter().any(|n| matches!(n, Node::InlineMath(_))));
                } else {
                    panic!("Expected Paragraph, got {:?}", nodes[0]);
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn test_itemize_environment() {
        let src = r"\begin{itemize}\item foo\end{itemize}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "itemize".to_string(),
                options: None,
                content: vec![
                    Node::Command {
                        name: "item".to_string(),
                        args: vec![]
                    },
                    Node::Text("foo".to_string()),
                ]
            }])
        );
    }

    #[test]
    fn test_nested_groups() {
        let mut parser = Parser::new("{{a}b}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Group(vec![
                Node::Group(vec![Node::Text("a".to_string())]),
                Node::Text("b".to_string()),
            ])])
        );
    }

    #[test]
    fn test_command_no_args() {
        let mut parser = Parser::new(r"\noindent");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Command {
                name: "noindent".to_string(),
                args: vec![]
            }])
        );
    }

    #[test]
    fn test_display_math_with_command() {
        // $$\frac{a}{b}$$ now produces a structured Fraction node
        let mut parser = Parser::new(r"$$\frac{a}{b}$$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::DisplayMath(vec![Node::Fraction {
                numerator: Box::new(Node::MathGroup(vec![Node::Text("a".to_string())])),
                denominator: Box::new(Node::MathGroup(vec![Node::Text("b".to_string())])),
            }])])
        );
    }

    #[test]
    fn test_enumerate_environment() {
        let src = r"\begin{enumerate}\item one\item two\end{enumerate}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "enumerate".to_string(),
                options: None,
                content: vec![
                    Node::Command {
                        name: "item".to_string(),
                        args: vec![]
                    },
                    Node::Text("one".to_string()),
                    Node::Command {
                        name: "item".to_string(),
                        args: vec![]
                    },
                    Node::Text("two".to_string()),
                ]
            }])
        );
    }

    #[test]
    fn test_equation_environment() {
        let src = r"\begin{equation}x^2 + y^2 = z^2\end{equation}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::Environment {
                name: "equation".to_string(),
                options: None,
                content: vec![Node::Text("x^2 + y^2 = z^2".to_string())]
            }])
        );
    }

    #[test]
    fn test_command_followed_by_text() {
        // \hello world — \hello has no args because next token is text, not { or [
        let mut parser = Parser::new(r"\hello world");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![
                Node::Command {
                    name: "hello".to_string(),
                    args: vec![]
                },
                Node::Text("world".to_string()),
            ])
        );
    }

    #[test]
    fn test_multiple_commands_with_args() {
        let mut parser = Parser::new(r"\textbf{bold}\textit{italic}");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![
                Node::Command {
                    name: "textbf".to_string(),
                    args: vec![Node::Group(vec![Node::Text("bold".to_string())])]
                },
                Node::Command {
                    name: "textit".to_string(),
                    args: vec![Node::Group(vec![Node::Text("italic".to_string())])]
                },
            ])
        );
    }

    // ===== M5 Math AST Enhancement Tests =====

    /// `$x^2$` → InlineMath([Superscript{base:Text("x"), exponent:Text("2")}])
    #[test]
    fn test_math_superscript_simple() {
        let mut parser = Parser::new(r"$x^2$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Text("x".to_string())),
                exponent: Box::new(Node::Text("2".to_string())),
            }])])
        );
    }

    /// `$x^{n+1}$` → InlineMath([Superscript{base:Text("x"), exponent:MathGroup([...])}])
    #[test]
    fn test_math_superscript_group_exponent() {
        let mut parser = Parser::new(r"$x^{n+1}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Text("x".to_string())),
                exponent: Box::new(Node::MathGroup(vec![
                    Node::Text("n".to_string()),
                    Node::Text("+".to_string()),
                    Node::Text("1".to_string()),
                ])),
            }])])
        );
    }

    /// `$x_i$` → InlineMath([Subscript{base:Text("x"), subscript:Text("i")}])
    #[test]
    fn test_math_subscript_simple() {
        let mut parser = Parser::new(r"$x_i$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Subscript {
                base: Box::new(Node::Text("x".to_string())),
                subscript: Box::new(Node::Text("i".to_string())),
            }])])
        );
    }

    /// `$x_{ij}$` → InlineMath([Subscript{base:Text("x"), subscript:MathGroup([...])}])
    #[test]
    fn test_math_subscript_group() {
        let mut parser = Parser::new(r"$x_{ij}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Subscript {
                base: Box::new(Node::Text("x".to_string())),
                subscript: Box::new(Node::MathGroup(vec![
                    Node::Text("i".to_string()),
                    Node::Text("j".to_string()),
                ])),
            }])])
        );
    }

    /// `$\frac{a}{b}$` → InlineMath([Fraction{numerator:MathGroup([Text("a")]), denominator:MathGroup([Text("b")])}])
    #[test]
    fn test_math_fraction() {
        let mut parser = Parser::new(r"$\frac{a}{b}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Fraction {
                numerator: Box::new(Node::MathGroup(vec![Node::Text("a".to_string())])),
                denominator: Box::new(Node::MathGroup(vec![Node::Text("b".to_string())])),
            }])])
        );
    }

    /// `$\sqrt{x}$` → InlineMath([Radical{degree:None, radicand:MathGroup([Text("x")])}])
    #[test]
    fn test_math_sqrt_no_degree() {
        let mut parser = Parser::new(r"$\sqrt{x}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Radical {
                degree: None,
                radicand: Box::new(Node::MathGroup(vec![Node::Text("x".to_string())])),
            }])])
        );
    }

    /// `$\sqrt[n]{x}$` → InlineMath([Radical{degree:Some(MathGroup([Text("n")])), radicand:MathGroup([Text("x")])}])
    #[test]
    fn test_math_sqrt_with_degree() {
        let mut parser = Parser::new(r"$\sqrt[n]{x}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Radical {
                degree: Some(Box::new(Node::MathGroup(vec![Node::Text("n".to_string())]))),
                radicand: Box::new(Node::MathGroup(vec![Node::Text("x".to_string())])),
            }])])
        );
    }

    /// `${abc}$` → InlineMath([MathGroup([Text("a"), Text("b"), Text("c")])])
    #[test]
    fn test_math_group() {
        let mut parser = Parser::new(r"${abc}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::MathGroup(vec![
                Node::Text("a".to_string()),
                Node::Text("b".to_string()),
                Node::Text("c".to_string()),
            ])])])
        );
    }

    /// `$$\sum_{i=0}^{n} x_i$$` — DisplayMath with Subscript and Superscript on Command("sum")
    #[test]
    fn test_display_math_sum_with_scripts() {
        let mut parser = Parser::new(r"$$\sum_{i=0}^{n} x_i$$");
        let doc = parser.parse();
        match doc {
            Node::Document(ref nodes) => {
                assert_eq!(nodes.len(), 1);
                if let Node::DisplayMath(ref inner) = nodes[0] {
                    // First node should be a Superscript wrapping a Subscript wrapping Command("sum")
                    assert!(inner.len() >= 2); // sum_i^n + x_i (space + subscript)
                                               // The sum command should have sub/super scripts attached
                    let first = &inner[0];
                    assert!(
                        matches!(first, Node::Superscript { .. })
                            || matches!(first, Node::Subscript { .. })
                    );
                } else {
                    panic!("Expected DisplayMath");
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    /// Combined: `$\frac{x^2}{y_i}$`
    #[test]
    fn test_math_fraction_with_scripts() {
        let mut parser = Parser::new(r"$\frac{x^2}{y_i}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Fraction {
                numerator: Box::new(Node::MathGroup(vec![Node::Superscript {
                    base: Box::new(Node::Text("x".to_string())),
                    exponent: Box::new(Node::Text("2".to_string())),
                }])),
                denominator: Box::new(Node::MathGroup(vec![Node::Subscript {
                    base: Box::new(Node::Text("y".to_string())),
                    subscript: Box::new(Node::Text("i".to_string())),
                }])),
            }])])
        );
    }

    /// `$x^2_i$` — chained superscript then subscript
    #[test]
    fn test_math_chained_super_then_sub() {
        let mut parser = Parser::new(r"$x^2_i$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Subscript {
                base: Box::new(Node::Superscript {
                    base: Box::new(Node::Text("x".to_string())),
                    exponent: Box::new(Node::Text("2".to_string())),
                }),
                subscript: Box::new(Node::Text("i".to_string())),
            }])])
        );
    }

    /// `$\sqrt[3]{x^2}$` — cube root of x squared
    #[test]
    fn test_math_sqrt_cube_root_with_superscript() {
        let mut parser = Parser::new(r"$\sqrt[3]{x^2}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Radical {
                degree: Some(Box::new(Node::MathGroup(vec![Node::Text("3".to_string())]))),
                radicand: Box::new(Node::MathGroup(vec![Node::Superscript {
                    base: Box::new(Node::Text("x".to_string())),
                    exponent: Box::new(Node::Text("2".to_string())),
                }])),
            }])])
        );
    }

    /// `$$\frac{a}{b}$$` — display math fraction (already updated in test_display_math_with_command)
    #[test]
    fn test_display_math_fraction() {
        let mut parser = Parser::new(r"$$\frac{1}{2}$$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::DisplayMath(vec![Node::Fraction {
                numerator: Box::new(Node::MathGroup(vec![Node::Text("1".to_string())])),
                denominator: Box::new(Node::MathGroup(vec![Node::Text("2".to_string())])),
            }])])
        );
    }

    /// `$a^{b^c}$` — nested superscripts
    #[test]
    fn test_math_nested_superscripts() {
        let mut parser = Parser::new(r"$a^{b^c}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Text("a".to_string())),
                exponent: Box::new(Node::MathGroup(vec![Node::Superscript {
                    base: Box::new(Node::Text("b".to_string())),
                    exponent: Box::new(Node::Text("c".to_string())),
                }])),
            }])])
        );
    }

    /// `$\alpha^2$` — command as base with superscript
    #[test]
    fn test_math_command_base_superscript() {
        let mut parser = Parser::new(r"$\alpha^2$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Command {
                    name: "alpha".to_string(),
                    args: vec![],
                }),
                exponent: Box::new(Node::Text("2".to_string())),
            }])])
        );
    }

    /// `$x_{i=0}^{n}$` — both subscript and superscript with groups
    #[test]
    fn test_math_subscript_group_and_superscript_group() {
        let mut parser = Parser::new(r"$x_{i=0}^{n}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Superscript {
                base: Box::new(Node::Subscript {
                    base: Box::new(Node::Text("x".to_string())),
                    subscript: Box::new(Node::MathGroup(vec![
                        Node::Text("i".to_string()),
                        Node::Text("=".to_string()),
                        Node::Text("0".to_string()),
                    ])),
                }),
                exponent: Box::new(Node::MathGroup(vec![Node::Text("n".to_string())])),
            }])])
        );
    }

    // ===== M19: Verbatim environment tests =====

    #[test]
    fn test_verbatim_preserves_raw_text() {
        let src = r"\begin{verbatim}hello world\end{verbatim}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        match doc {
            Node::Document(ref nodes) => {
                assert_eq!(nodes.len(), 1);
                if let Node::Environment {
                    name,
                    options,
                    content,
                } = &nodes[0]
                {
                    assert_eq!(name, "verbatim");
                    assert_eq!(*options, None);
                    assert_eq!(content.len(), 1);
                    if let Node::Text(t) = &content[0] {
                        assert!(t.contains("hello world"), "Expected raw text, got '{}'", t);
                    } else {
                        panic!("Expected Text node inside verbatim");
                    }
                } else {
                    panic!("Expected Environment node");
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn test_verbatim_does_not_parse_commands() {
        let src = r"\begin{verbatim}\textbf{bold}\end{verbatim}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        match doc {
            Node::Document(ref nodes) => {
                assert_eq!(nodes.len(), 1);
                if let Node::Environment { content, .. } = &nodes[0] {
                    assert_eq!(content.len(), 1);
                    if let Node::Text(t) = &content[0] {
                        // The raw content should contain \textbf literally
                        assert!(
                            t.contains("\\textbf"),
                            "Expected \\textbf in raw verbatim text, got '{}'",
                            t
                        );
                    } else {
                        panic!("Expected Text node");
                    }
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn test_verbatim_preserves_special_chars() {
        let src = r"\begin{verbatim}$x^2$ & # % ~\end{verbatim}";
        let mut parser = Parser::new(src);
        let doc = parser.parse();
        if let Node::Document(ref nodes) = doc {
            if let Node::Environment { content, .. } = &nodes[0] {
                if let Node::Text(t) = &content[0] {
                    assert!(t.contains('$'), "Expected $ in verbatim, got '{}'", t);
                    assert!(t.contains('^'), "Expected ^ in verbatim, got '{}'", t);
                } else {
                    panic!("Expected Text node");
                }
            }
        }
    }

    /// `$\frac{\sqrt{a}}{b^2}$` — fraction with sqrt in numerator
    #[test]
    fn test_math_fraction_sqrt_numerator() {
        let mut parser = Parser::new(r"$\frac{\sqrt{a}}{b^2}$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Fraction {
                numerator: Box::new(Node::MathGroup(vec![Node::Radical {
                    degree: None,
                    radicand: Box::new(Node::MathGroup(vec![Node::Text("a".to_string())])),
                }])),
                denominator: Box::new(Node::MathGroup(vec![Node::Superscript {
                    base: Box::new(Node::Text("b".to_string())),
                    exponent: Box::new(Node::Text("2".to_string())),
                }])),
            }])])
        );
    }
}
