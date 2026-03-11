//! `rustlatex-engine` — LaTeX typesetting engine
//!
//! This crate implements the typesetting engine that transforms an AST into
//! a laid-out document using TeX's box/glue model. It will eventually
//! implement Knuth-Plass line breaking, page breaking, and the full TeX
//! typesetting algorithms.
//!
//! Currently implements a basic box/glue IR with greedy line breaking.

use rustlatex_parser::Node;

/// A node in the typesetting intermediate representation (box/glue model).
#[derive(Debug, Clone, PartialEq)]
pub enum BoxNode {
    /// A run of text with a computed width (in points).
    Text { text: String, width: f64 },
    /// Inter-word glue with natural width, stretchability, and shrinkability.
    Glue {
        natural: f64,
        stretch: f64,
        shrink: f64,
    },
    /// A fixed-width kern (non-breakable spacing).
    Kern { amount: f64 },
    /// A penalty value influencing line-break decisions.
    Penalty { value: i32 },
    /// A horizontal box containing sub-nodes.
    HBox {
        width: f64,
        height: f64,
        depth: f64,
        content: Vec<BoxNode>,
    },
    /// A vertical box containing sub-nodes.
    VBox { width: f64, content: Vec<BoxNode> },
}

/// Character width in points (10pt font, 6pt per char).
pub fn char_width(s: &str) -> f64 {
    s.len() as f64 * 6.0
}

/// Translate a parser AST node into a flat list of box/glue items.
///
/// This converts the high-level AST into the low-level typesetting IR that
/// the line-breaking algorithm operates on.
pub fn translate_node(node: &Node) -> Vec<BoxNode> {
    match node {
        Node::Text(s) => {
            let mut result = Vec::new();
            let words: Vec<&str> = s.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    result.push(BoxNode::Glue {
                        natural: 3.33,
                        stretch: 1.67,
                        shrink: 1.11,
                    });
                }
                result.push(BoxNode::Text {
                    text: word.to_string(),
                    width: char_width(word),
                });
            }
            result
        }
        Node::Paragraph(nodes) => nodes.iter().flat_map(translate_node).collect(),
        Node::Command { name, args } => {
            match name.as_str() {
                "textbf" | "textit" | "emph" => {
                    // For known formatting commands, translate their arguments
                    args.iter().flat_map(translate_node).collect()
                }
                _ => {
                    // Unknown commands → skip
                    vec![]
                }
            }
        }
        Node::Environment { content, .. } => content.iter().flat_map(translate_node).collect(),
        Node::InlineMath(_) | Node::DisplayMath(_) => {
            vec![BoxNode::Text {
                text: "(math)".to_string(),
                width: 20.0,
            }]
        }
        Node::Group(nodes) => nodes.iter().flat_map(translate_node).collect(),
        Node::Document(nodes) => nodes.iter().flat_map(translate_node).collect(),
        // Other node types (Superscript, Subscript, Fraction, Radical, MathGroup) are only
        // found inside math mode, which we already handle above.
        _ => vec![],
    }
}

/// Greedy line-breaking algorithm.
///
/// Walks the list of box/glue items, accumulating items into lines. When
/// adding a `Text` or `Kern` item would exceed `hsize`, breaks at the last
/// glue position and starts a new line.
///
/// Each resulting line is a `Vec<BoxNode>` with leading/trailing glue stripped.
pub fn break_into_lines(items: &[BoxNode], hsize: f64) -> Vec<Vec<BoxNode>> {
    if items.is_empty() {
        return vec![];
    }

    let mut lines: Vec<Vec<BoxNode>> = Vec::new();
    let mut current_line: Vec<BoxNode> = Vec::new();
    let mut current_width: f64 = 0.0;
    let mut last_glue_index: Option<usize> = None; // index in current_line

    for item in items {
        match item {
            BoxNode::Glue { .. } => {
                // Glue marks a potential break point but doesn't count toward width
                last_glue_index = Some(current_line.len());
                current_line.push(item.clone());
            }
            BoxNode::Text { width, .. } => {
                if current_width + width > hsize && last_glue_index.is_some() {
                    // Break at the last glue position
                    let glue_idx = last_glue_index.unwrap();
                    // Items before the glue go on the finished line
                    let remainder: Vec<BoxNode> = current_line.split_off(glue_idx);
                    let finished_line = strip_glue(current_line);
                    if !finished_line.is_empty() {
                        lines.push(finished_line);
                    }
                    // remainder starts with the glue; skip it, keep rest
                    current_line = if remainder.len() > 1 {
                        remainder[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    // Recalculate width for the new current line
                    current_width = current_line
                        .iter()
                        .map(|n| match n {
                            BoxNode::Text { width, .. } => *width,
                            BoxNode::Kern { amount } => *amount,
                            _ => 0.0,
                        })
                        .sum();
                    last_glue_index = None;
                }
                current_width += width;
                current_line.push(item.clone());
            }
            BoxNode::Kern { amount } => {
                if current_width + amount > hsize && last_glue_index.is_some() {
                    // Break at the last glue position
                    let glue_idx = last_glue_index.unwrap();
                    let remainder: Vec<BoxNode> = current_line.split_off(glue_idx);
                    let finished_line = strip_glue(current_line);
                    if !finished_line.is_empty() {
                        lines.push(finished_line);
                    }
                    current_line = if remainder.len() > 1 {
                        remainder[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    current_width = current_line
                        .iter()
                        .map(|n| match n {
                            BoxNode::Text { width, .. } => *width,
                            BoxNode::Kern { amount } => *amount,
                            _ => 0.0,
                        })
                        .sum();
                    last_glue_index = None;
                }
                current_width += amount;
                current_line.push(item.clone());
            }
            BoxNode::Penalty { .. } | BoxNode::HBox { .. } | BoxNode::VBox { .. } => {
                // Pass through without affecting width calculation for now
                current_line.push(item.clone());
            }
        }
    }

    // Flush remaining items
    let finished_line = strip_glue(current_line);
    if !finished_line.is_empty() {
        lines.push(finished_line);
    }

    lines
}

/// Strip leading and trailing `Glue` nodes from a line.
fn strip_glue(mut items: Vec<BoxNode>) -> Vec<BoxNode> {
    // Strip trailing glue
    while matches!(items.last(), Some(BoxNode::Glue { .. })) {
        items.pop();
    }
    // Strip leading glue
    while matches!(items.first(), Some(BoxNode::Glue { .. })) {
        items.remove(0);
    }
    items
}

/// A laid-out page ready for PDF rendering.
#[derive(Debug)]
pub struct Page {
    /// Page number (1-indexed).
    pub number: usize,
    /// Placeholder content — will become a proper box tree.
    pub content: String,
    /// The typeset box lines for this page.
    pub box_lines: Vec<Vec<BoxNode>>,
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
    /// Translates the AST to box/glue items, performs greedy line breaking,
    /// and packages the result into pages.
    pub fn typeset(&self) -> Vec<Page> {
        let items = translate_node(&self.document);
        let lines = break_into_lines(&items, 345.0);
        let content = format!("(stub) document node: {:?}", self.document);
        vec![Page {
            number: 1,
            content,
            box_lines: lines,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_parser::Parser;

    #[test]
    fn test_engine_stub() {
        let mut parser = Parser::new(r"\documentclass{article}");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
    }

    // ===== BoxNode construction tests =====

    #[test]
    fn test_boxnode_text_construction() {
        let node = BoxNode::Text {
            text: "hello".to_string(),
            width: 30.0,
        };
        if let BoxNode::Text { text, width } = &node {
            assert_eq!(text, "hello");
            assert!((width - 30.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_boxnode_glue_construction() {
        let node = BoxNode::Glue {
            natural: 3.33,
            stretch: 1.67,
            shrink: 1.11,
        };
        if let BoxNode::Glue {
            natural,
            stretch,
            shrink,
        } = &node
        {
            assert!((natural - 3.33).abs() < f64::EPSILON);
            assert!((stretch - 1.67).abs() < f64::EPSILON);
            assert!((shrink - 1.11).abs() < f64::EPSILON);
        } else {
            panic!("Expected BoxNode::Glue");
        }
    }

    #[test]
    fn test_boxnode_kern_construction() {
        let node = BoxNode::Kern { amount: 5.0 };
        if let BoxNode::Kern { amount } = &node {
            assert!((amount - 5.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected BoxNode::Kern");
        }
    }

    #[test]
    fn test_boxnode_penalty_construction() {
        let node = BoxNode::Penalty { value: -10000 };
        if let BoxNode::Penalty { value } = &node {
            assert_eq!(*value, -10000);
        } else {
            panic!("Expected BoxNode::Penalty");
        }
    }

    #[test]
    fn test_boxnode_hbox_construction() {
        let node = BoxNode::HBox {
            width: 100.0,
            height: 10.0,
            depth: 2.0,
            content: vec![BoxNode::Text {
                text: "hi".to_string(),
                width: 12.0,
            }],
        };
        if let BoxNode::HBox {
            width,
            height,
            depth,
            content,
        } = &node
        {
            assert!((width - 100.0).abs() < f64::EPSILON);
            assert!((height - 10.0).abs() < f64::EPSILON);
            assert!((depth - 2.0).abs() < f64::EPSILON);
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected BoxNode::HBox");
        }
    }

    #[test]
    fn test_boxnode_vbox_construction() {
        let node = BoxNode::VBox {
            width: 345.0,
            content: vec![BoxNode::Text {
                text: "line".to_string(),
                width: 24.0,
            }],
        };
        if let BoxNode::VBox { width, content } = &node {
            assert!((width - 345.0).abs() < f64::EPSILON);
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected BoxNode::VBox");
        }
    }

    // ===== AST→BoxList tests =====

    #[test]
    fn test_translate_text_node() {
        let node = Node::Text("hello world".to_string());
        let items = translate_node(&node);
        // Should be: Text("hello"), Glue, Text("world")
        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "hello".to_string(),
                width: 30.0
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "world".to_string(),
                width: 30.0
            }
        );
    }

    #[test]
    fn test_translate_paragraph_node() {
        let node = Node::Paragraph(vec![
            Node::Text("one two".to_string()),
            Node::Text("three".to_string()),
        ]);
        let items = translate_node(&node);
        // "one two" → Text("one"), Glue, Text("two")
        // "three" → Text("three")
        // total: 4 items
        assert_eq!(items.len(), 4);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "one".to_string(),
                width: 18.0
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "two".to_string(),
                width: 18.0
            }
        );
        assert_eq!(
            items[3],
            BoxNode::Text {
                text: "three".to_string(),
                width: 30.0
            }
        );
    }

    #[test]
    fn test_translate_inline_math() {
        let node = Node::InlineMath(vec![Node::Text("x".to_string())]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "(math)".to_string(),
                width: 20.0
            }
        );
    }

    #[test]
    fn test_translate_display_math() {
        let node = Node::DisplayMath(vec![Node::Text("E=mc^2".to_string())]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "(math)".to_string(),
                width: 20.0
            }
        );
    }

    #[test]
    fn test_translate_group_node() {
        let node = Node::Group(vec![Node::Text("inside".to_string())]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "inside".to_string(),
                width: 36.0
            }
        );
    }

    #[test]
    fn test_translate_textbf_command() {
        let node = Node::Command {
            name: "textbf".to_string(),
            args: vec![Node::Group(vec![Node::Text("bold text".to_string())])],
        };
        let items = translate_node(&node);
        // "bold text" → Text("bold"), Glue, Text("text")
        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "bold".to_string(),
                width: 24.0
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "text".to_string(),
                width: 24.0
            }
        );
    }

    #[test]
    fn test_translate_environment() {
        let node = Node::Environment {
            name: "document".to_string(),
            options: None,
            content: vec![Node::Text("content here".to_string())],
        };
        let items = translate_node(&node);
        // "content here" → Text("content"), Glue, Text("here")
        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "content".to_string(),
                width: 42.0
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "here".to_string(),
                width: 24.0
            }
        );
    }

    #[test]
    fn test_translate_unknown_command() {
        let node = Node::Command {
            name: "documentclass".to_string(),
            args: vec![Node::Group(vec![Node::Text("article".to_string())])],
        };
        let items = translate_node(&node);
        assert!(items.is_empty());
    }

    #[test]
    fn test_translate_document_node() {
        let node = Node::Document(vec![
            Node::Text("first".to_string()),
            Node::Text("second".to_string()),
        ]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "first".to_string(),
                width: 30.0
            }
        );
        assert_eq!(
            items[1],
            BoxNode::Text {
                text: "second".to_string(),
                width: 36.0
            }
        );
    }

    // ===== char_width tests =====

    #[test]
    fn test_char_width() {
        assert!((char_width("a") - 6.0).abs() < f64::EPSILON);
        assert!((char_width("hello") - 30.0).abs() < f64::EPSILON);
        assert!((char_width("") - 0.0).abs() < f64::EPSILON);
    }

    // ===== Line breaking tests =====

    #[test]
    fn test_break_into_lines_short_text() {
        // "hello world" fits in one line (30 + 30 = 60 < 345)
        let items = vec![
            BoxNode::Text {
                text: "hello".to_string(),
                width: 30.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "world".to_string(),
                width: 30.0,
            },
        ];
        let lines = break_into_lines(&items, 345.0);
        assert_eq!(lines.len(), 1);
        // Line should have: Text, Glue, Text (glue in the middle is kept)
        assert_eq!(lines[0].len(), 3);
    }

    #[test]
    fn test_break_into_lines_long_text() {
        // Create a sequence that exceeds hsize
        // Each word is 60pt wide, hsize = 100pt
        // word1 (60) + word2 (60) = 120 > 100 → should break
        let items = vec![
            BoxNode::Text {
                text: "aaaaaaaaaa".to_string(),
                width: 60.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "bbbbbbbbbb".to_string(),
                width: 60.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "cccccccccc".to_string(),
                width: 60.0,
            },
        ];
        let lines = break_into_lines(&items, 100.0);
        assert_eq!(lines.len(), 3);
        // Each line should have exactly one Text item
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[1].len(), 1);
        assert_eq!(lines[2].len(), 1);
    }

    #[test]
    fn test_break_into_lines_empty() {
        let items: Vec<BoxNode> = vec![];
        let lines = break_into_lines(&items, 345.0);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_break_into_lines_exact_fit() {
        // Two words that exactly fill a line
        let items = vec![
            BoxNode::Text {
                text: "aaa".to_string(),
                width: 50.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "bbb".to_string(),
                width: 50.0,
            },
        ];
        // hsize=100, 50 + 50 = 100, fits exactly
        let lines = break_into_lines(&items, 100.0);
        assert_eq!(lines.len(), 1);
    }

    // ===== Integration tests =====

    #[test]
    fn test_typeset_paragraph() {
        let mut parser = Parser::new("Hello world");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
        // Should have box_lines with at least one line
        assert!(!pages[0].box_lines.is_empty());
        // Content should still work as before
        assert!(pages[0].content.contains("(stub)"));
    }

    #[test]
    fn test_typeset_multi_paragraph() {
        let mut parser = Parser::new("first paragraph\n\nsecond paragraph");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        // Both paragraphs should produce items that end up in box_lines
        assert!(!pages[0].box_lines.is_empty());
    }

    #[test]
    fn test_typeset_with_math() {
        let mut parser = Parser::new("text $x^2$ more text");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].box_lines.is_empty());
        // Should contain a (math) text box somewhere
        let all_items: Vec<&BoxNode> = pages[0].box_lines.iter().flatten().collect();
        let has_math = all_items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "(math)"));
        assert!(has_math, "Expected a (math) placeholder in the output");
    }

    #[test]
    fn test_typeset_empty_document() {
        let mut parser = Parser::new(r"\documentclass{article}");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        // No translatable content, so box_lines should be empty
        assert!(pages[0].box_lines.is_empty());
    }

    #[test]
    fn test_translate_textit_command() {
        let node = Node::Command {
            name: "textit".to_string(),
            args: vec![Node::Group(vec![Node::Text("italic".to_string())])],
        };
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "italic".to_string(),
                width: 36.0
            }
        );
    }

    #[test]
    fn test_translate_emph_command() {
        let node = Node::Command {
            name: "emph".to_string(),
            args: vec![Node::Group(vec![Node::Text("emphasized".to_string())])],
        };
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "emphasized".to_string(),
                width: 60.0
            }
        );
    }

    #[test]
    fn test_break_into_lines_with_kern() {
        let items = vec![
            BoxNode::Text {
                text: "word".to_string(),
                width: 60.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Kern { amount: 50.0 },
        ];
        // 60 + 50 = 110 > 100 → should break at glue
        let lines = break_into_lines(&items, 100.0);
        assert_eq!(lines.len(), 2);
    }
}
