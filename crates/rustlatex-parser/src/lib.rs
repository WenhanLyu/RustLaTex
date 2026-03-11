//! `rustlatex-parser` — LaTeX AST parser
//!
//! This crate implements the second stage of the LaTeX compilation pipeline:
//! parsing the token stream produced by `rustlatex-lexer` into an Abstract
//! Syntax Tree (AST) representing the document structure.

use rustlatex_lexer::{Lexer, Token};

/// A node in the LaTeX document AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A LaTeX document containing a sequence of nodes.
    Document(Vec<Node>),
    /// A control sequence command with optional argument nodes.
    Command {
        name: String,
        args: Vec<Node>,
    },
    /// A group delimited by `{` and `}`.
    Group(Vec<Node>),
    /// Plain text content.
    Text(String),
    /// A math mode expression (inline).
    InlineMath(Vec<Node>),
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

    /// Peek at the current token.
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EndOfInput)
    }

    /// Advance past the current token and return it.
    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::EndOfInput);
        self.pos += 1;
        tok
    }

    /// Parse the entire document into a [`Node::Document`].
    pub fn parse(&mut self) -> Node {
        let nodes = self.parse_nodes(false);
        Node::Document(nodes)
    }

    /// Parse a sequence of nodes, stopping at end of input or `}`.
    fn parse_nodes(&mut self, in_group: bool) -> Vec<Node> {
        use rustlatex_lexer::Category;
        let mut nodes = Vec::new();

        loop {
            match self.peek().clone() {
                Token::EndOfInput => break,
                Token::Character('}', Category::EndGroup) => {
                    if in_group {
                        self.advance(); // consume the `}`
                    }
                    break;
                }
                Token::ControlSequence(name) => {
                    self.advance();
                    let node = Node::Command { name, args: vec![] };
                    nodes.push(node);
                }
                Token::Character('{', Category::BeginGroup) => {
                    self.advance(); // consume `{`
                    let inner = self.parse_nodes(true);
                    nodes.push(Node::Group(inner));
                }
                Token::Character(ch, _) => {
                    self.advance();
                    // Accumulate consecutive text characters
                    let mut text = ch.to_string();
                    loop {
                        // Clone to avoid holding an immutable borrow while calling advance()
                        let next = self.peek().clone();
                        match next {
                            Token::Character(c, cat) => {
                                use Category::*;
                                match cat {
                                    BeginGroup | EndGroup | MathShift => break,
                                    _ => {
                                        self.advance();
                                        text.push(c);
                                    }
                                }
                            }
                            _ => break,
                        }
                    }
                    nodes.push(Node::Text(text));
                }
            }
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            doc,
            Node::Document(vec![Node::Text("Hello".to_string())])
        );
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
}
