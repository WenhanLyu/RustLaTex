//! `rustlatex-engine` — LaTeX typesetting engine
//!
//! This crate implements the typesetting engine that transforms an AST into
//! a laid-out document using TeX's box/glue model. It will eventually
//! implement Knuth-Plass line breaking, page breaking, and the full TeX
//! typesetting algorithms.
//!
//! Currently a stub — future milestones will fill this in.

use rustlatex_parser::Node;

/// A laid-out page ready for PDF rendering.
#[derive(Debug)]
pub struct Page {
    /// Page number (1-indexed).
    pub number: usize,
    /// Placeholder content — will become a proper box tree.
    pub content: String,
}

/// The typesetting engine processes an AST and produces pages.
pub struct Engine {
    /// The parsed document AST.
    document: Node,
}

impl Engine {
    /// Create a new engine from a parsed document.
    pub fn new(document: Node) -> Self {
        Engine { document }
    }

    /// Typeset the document and return pages.
    ///
    /// This is currently a stub that returns a single placeholder page.
    pub fn typeset(&self) -> Vec<Page> {
        let content = format!("(stub) document node: {:?}", self.document);
        vec![Page { number: 1, content }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_parser::{Node, Parser};

    #[test]
    fn test_engine_stub() {
        let mut parser = Parser::new(r"\documentclass{article}");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
    }
}
