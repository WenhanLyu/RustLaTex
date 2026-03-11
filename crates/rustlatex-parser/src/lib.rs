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
}

impl Parser {
    /// Create a parser from a LaTeX source string.
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    /// Create a parser from a pre-built token vector (used by the expander).
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
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
                    let content = self.parse_body(Some(&env_name));
                    Some(ParseEvent::Node(Node::Environment {
                        name: env_name,
                        options: None,
                        content,
                    }))
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
                Token::Character(ch, _) => {
                    self.advance();
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
                    nodes.push(Node::Text(text));
                }
                Token::Space => {
                    self.advance();
                    // In math mode, spaces are usually ignored but we keep them as text
                    nodes.push(Node::Text(" ".to_string()));
                }
                _ => {
                    self.advance();
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
        let mut parser = Parser::new(r"$x^2$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::InlineMath(vec![Node::Text("x^2".to_string())])])
        );
    }

    #[test]
    fn test_display_math() {
        let mut parser = Parser::new(r"$$E=mc^2$$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::DisplayMath(vec![Node::Text(
                "E=mc^2".to_string()
            )])])
        );
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
        let mut parser = Parser::new(r"$$\frac{a}{b}$$");
        let doc = parser.parse();
        assert_eq!(
            doc,
            Node::Document(vec![Node::DisplayMath(vec![Node::Command {
                name: "frac".to_string(),
                args: vec![
                    Node::Group(vec![Node::Text("a".to_string())]),
                    Node::Group(vec![Node::Text("b".to_string())]),
                ]
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
}
