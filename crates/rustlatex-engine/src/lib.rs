//! `rustlatex-engine` — LaTeX typesetting engine
//!
//! This crate implements the typesetting engine that transforms an AST into
//! a laid-out document using TeX's box/glue model. It implements both a greedy
//! line-breaking algorithm and the Knuth-Plass optimal line-breaking algorithm.

use rustlatex_parser::Node;

/// Text alignment mode for a line or block.
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum Alignment {
    /// Justify (stretch/shrink glue to fill hsize). This is the default.
    #[default]
    Justify,
    /// Center the line horizontally.
    Center,
    /// Ragged-right alignment (left-aligned, no justification).
    RaggedRight,
    /// Ragged-left alignment (right-aligned).
    RaggedLeft,
}

/// A single typeset line with its alignment mode.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputLine {
    pub alignment: Alignment,
    pub nodes: Vec<BoxNode>,
}

/// A node in the typesetting intermediate representation (box/glue model).
#[derive(Debug, Clone, PartialEq)]
pub enum BoxNode {
    /// A run of text with a computed width (in points).
    Text {
        text: String,
        width: f64,
        font_size: f64,
    },
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
    /// An alignment marker that sets the alignment mode for subsequent lines.
    AlignmentMarker { alignment: Alignment },
}

// ===== Font Metrics Trait and CM Roman Implementation =====

/// Trait providing font metric information for typesetting.
pub trait FontMetrics {
    /// Return the width of a single character in points.
    fn char_width(&self, ch: char) -> f64;

    /// Return the width of a space in points.
    fn space_width(&self) -> f64;

    /// Return the total width of a string by summing individual character widths.
    fn string_width(&self, s: &str) -> f64 {
        s.chars().map(|ch| self.char_width(ch)).sum()
    }
}

/// Font metrics using Computer Modern Roman 10pt (CM Roman) AFM widths (WX / 100 = pt at 10pt).
///
/// Character widths derived from CM Roman (cmr10) AFM data for accurate typesetting
/// that matches the embedded Type1 font in the PDF backend.
pub struct StandardFontMetrics;

impl FontMetrics for StandardFontMetrics {
    fn char_width(&self, ch: char) -> f64 {
        // CM Roman 10pt AFM widths (WX / 100 = pt at 10pt)
        match ch {
            // Lowercase
            'a' => 5.000,
            'b' => 5.556,
            'c' => 4.444,
            'd' => 5.556,
            'e' => 4.444,
            'f' => 3.056,
            'g' => 5.000,
            'h' => 5.556,
            'i' => 2.778,
            'j' => 3.056,
            'k' => 5.278,
            'l' => 2.778,
            'm' => 8.333,
            'n' => 5.556,
            'o' => 5.000,
            'p' => 5.556,
            'q' => 5.278,
            'r' => 3.917,
            's' => 3.944,
            't' => 3.889,
            'u' => 5.556,
            'v' => 5.278,
            'w' => 7.222,
            'x' => 5.278,
            'y' => 5.278,
            'z' => 4.444,
            // Uppercase
            'A' => 7.500,
            'B' => 7.083,
            'C' => 7.222,
            'D' => 7.639,
            'E' => 6.806,
            'F' => 6.528,
            'G' => 7.847,
            'H' => 7.500,
            'I' => 3.611,
            'J' => 5.139,
            'K' => 7.778,
            'L' => 6.250,
            'M' => 9.167,
            'N' => 7.500,
            'O' => 7.778,
            'P' => 6.806,
            'Q' => 7.778,
            'R' => 7.361,
            'S' => 5.556,
            'T' => 7.222,
            'U' => 7.500,
            'V' => 7.500,
            'W' => 10.278,
            'X' => 7.500,
            'Y' => 7.500,
            'Z' => 6.111,
            // Digits
            '0'..='9' => 5.000,
            _ => 5.000,
        }
    }

    fn space_width(&self) -> f64 {
        // CM Roman space width (from AFM: WX=333.333 / 100 = 3.333pt)
        3.333
    }
}

/// Character width in points — backward-compatible function.
///
/// Computes the total width of a string using CM Roman metrics
/// by summing the width of each individual character.
pub fn char_width(s: &str) -> f64 {
    let metrics = StandardFontMetrics;
    metrics.string_width(s)
}

/// Recursively convert a math AST node to a readable text string.
///
/// This walks the structured math AST and produces a plain-text representation
/// suitable for rendering as a `BoxNode::Text` item.
pub fn math_node_to_text(node: &Node) -> String {
    match node {
        Node::Text(s) => s.clone(),
        Node::Superscript { base, exponent } => {
            format!(
                "{}^{}",
                math_node_to_text(base),
                math_node_to_text(exponent)
            )
        }
        Node::Subscript { base, subscript } => {
            format!(
                "{}_{}",
                math_node_to_text(base),
                math_node_to_text(subscript)
            )
        }
        Node::Fraction {
            numerator,
            denominator,
        } => {
            format!(
                "{}/{}",
                math_node_to_text(numerator),
                math_node_to_text(denominator)
            )
        }
        Node::Radical {
            radicand,
            degree: None,
        } => {
            format!("√{}", math_node_to_text(radicand))
        }
        Node::Radical {
            radicand,
            degree: Some(d),
        } => {
            format!("^{}√{}", math_node_to_text(d), math_node_to_text(radicand))
        }
        Node::MathGroup(nodes) | Node::Group(nodes) => {
            nodes.iter().map(math_node_to_text).collect::<String>()
        }
        Node::Command { name, args } => {
            match name.as_str() {
                // Greek lowercase
                "alpha" => "α".to_string(),
                "beta" => "β".to_string(),
                "gamma" => "γ".to_string(),
                "delta" => "δ".to_string(),
                "epsilon" => "ε".to_string(),
                "zeta" => "ζ".to_string(),
                "eta" => "η".to_string(),
                "theta" => "θ".to_string(),
                "iota" => "ι".to_string(),
                "kappa" => "κ".to_string(),
                "lambda" => "λ".to_string(),
                "mu" => "μ".to_string(),
                "nu" => "ν".to_string(),
                "xi" => "ξ".to_string(),
                "pi" => "π".to_string(),
                "rho" => "ρ".to_string(),
                "sigma" => "σ".to_string(),
                "tau" => "τ".to_string(),
                "upsilon" => "υ".to_string(),
                "phi" => "φ".to_string(),
                "chi" => "χ".to_string(),
                "psi" => "ψ".to_string(),
                "omega" => "ω".to_string(),
                // Greek uppercase
                "Alpha" => "Α".to_string(),
                "Beta" => "Β".to_string(),
                "Gamma" => "Γ".to_string(),
                "Delta" => "Δ".to_string(),
                "Theta" => "Θ".to_string(),
                "Lambda" => "Λ".to_string(),
                "Pi" => "Π".to_string(),
                "Sigma" => "Σ".to_string(),
                "Phi" => "Φ".to_string(),
                "Omega" => "Ω".to_string(),
                // Math operators
                "cdot" => "·".to_string(),
                "times" => "×".to_string(),
                "div" => "÷".to_string(),
                "pm" => "±".to_string(),
                "mp" => "∓".to_string(),
                "leq" => "≤".to_string(),
                "geq" => "≥".to_string(),
                "neq" => "≠".to_string(),
                "infty" => "∞".to_string(),
                "sum" => "∑".to_string(),
                "prod" => "∏".to_string(),
                "int" => "∫".to_string(),
                "partial" => "∂".to_string(),
                "nabla" => "∇".to_string(),
                "in" => "∈".to_string(),
                "notin" => "∉".to_string(),
                "subset" => "⊂".to_string(),
                "cup" => "∪".to_string(),
                "cap" => "∩".to_string(),
                "cdots" => "⋯".to_string(),
                "ldots" => "…".to_string(),
                "to" => "→".to_string(),
                "leftarrow" => "←".to_string(),
                "rightarrow" => "→".to_string(),
                "Rightarrow" => "⇒".to_string(),
                "Leftrightarrow" => "⇔".to_string(),
                "forall" => "∀".to_string(),
                "exists" => "∃".to_string(),
                "land" => "∧".to_string(),
                "lor" => "∨".to_string(),
                "neg" => "¬".to_string(),
                // frac fallback (in case the parser produces a Command rather than Fraction)
                "frac" => {
                    if args.len() >= 2 {
                        format!(
                            "{}/{}",
                            math_node_to_text(&args[0]),
                            math_node_to_text(&args[1])
                        )
                    } else if args.len() == 1 {
                        math_node_to_text(&args[0])
                    } else {
                        "frac".to_string()
                    }
                }
                // sqrt fallback (in case the parser produces a Command rather than Radical)
                "sqrt" => {
                    if let Some(arg) = args.first() {
                        format!("√{}", math_node_to_text(arg))
                    } else {
                        "√".to_string()
                    }
                }
                // Unknown commands — use name as text
                other => other.to_string(),
            }
        }
        // Other node types (InlineMath, DisplayMath, etc. shouldn't appear
        // inside math mode, but handle gracefully)
        _ => String::new(),
    }
}

/// Translate a parser AST node into a flat list of box/glue items,
/// using the provided font metrics.
pub fn translate_node_with_metrics(node: &Node, metrics: &dyn FontMetrics) -> Vec<BoxNode> {
    match node {
        Node::Text(s) => {
            let mut result = Vec::new();
            let words: Vec<&str> = s.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    result.push(BoxNode::Glue {
                        natural: metrics.space_width(),
                        stretch: 1.67,
                        shrink: 1.11,
                    });
                }
                result.push(BoxNode::Text {
                    text: word.to_string(),
                    width: metrics.string_width(word),
                    font_size: 10.0,
                });
            }
            result
        }
        Node::Paragraph(nodes) => {
            let mut result: Vec<BoxNode> = nodes
                .iter()
                .flat_map(|n| translate_node_with_metrics(n, metrics))
                .collect();
            result.push(BoxNode::Glue {
                natural: 6.0,
                stretch: 2.0,
                shrink: 0.0,
            });
            result
        }
        Node::Command { name, args } => {
            match name.as_str() {
                "textbf" | "textit" | "emph" => {
                    // For known formatting commands, translate their arguments
                    args.iter()
                        .flat_map(|n| translate_node_with_metrics(n, metrics))
                        .collect()
                }
                "section" | "subsection" | "subsubsection" => {
                    let font_size = match name.as_str() {
                        "section" => 14.0_f64,
                        "subsection" => 12.0_f64,
                        _ => 11.0_f64, // subsubsection
                    };
                    // Extract title from first argument (which is a Group node)
                    let title = if let Some(arg) = args.first() {
                        match arg {
                            Node::Group(nodes) => nodes
                                .iter()
                                .filter_map(|n| {
                                    if let Node::Text(t) = n {
                                        Some(t.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" "),
                            Node::Text(t) => t.clone(),
                            _ => String::new(),
                        }
                    } else {
                        String::new()
                    };
                    let width = metrics.string_width(&title);
                    vec![
                        BoxNode::Kern { amount: 12.0 },
                        BoxNode::Text {
                            text: title,
                            width,
                            font_size,
                        },
                        BoxNode::Kern { amount: 6.0 },
                    ]
                }
                "LaTeX" => vec![BoxNode::Text {
                    text: "LaTeX".to_string(),
                    width: metrics.string_width("LaTeX"),
                    font_size: 10.0,
                }],
                "TeX" => vec![BoxNode::Text {
                    text: "TeX".to_string(),
                    width: metrics.string_width("TeX"),
                    font_size: 10.0,
                }],
                "today" => {
                    let date_str = "January 1, 2025".to_string();
                    vec![BoxNode::Text {
                        text: date_str.clone(),
                        width: metrics.string_width(&date_str),
                        font_size: 10.0,
                    }]
                }
                "\\" | "newline" => vec![BoxNode::Penalty { value: -10000 }],
                "centering" => vec![BoxNode::AlignmentMarker {
                    alignment: Alignment::Center,
                }],
                "raggedright" => vec![BoxNode::AlignmentMarker {
                    alignment: Alignment::RaggedRight,
                }],
                "raggedleft" => vec![BoxNode::AlignmentMarker {
                    alignment: Alignment::RaggedLeft,
                }],
                _ => {
                    // Unknown commands → skip
                    vec![]
                }
            }
        }
        Node::Environment { name, content, .. } => {
            match name.as_str() {
                "itemize" | "enumerate" => {
                    let is_enumerate = name == "enumerate";
                    // Split content at \item boundaries
                    let mut items: Vec<Vec<&Node>> = Vec::new();
                    let mut current: Option<Vec<&Node>> = None;
                    for node in content {
                        if matches!(node, Node::Command { name: cmd_name, args } if cmd_name == "item" && args.is_empty())
                        {
                            // Close previous item and start new one
                            if let Some(prev) = current.take() {
                                items.push(prev);
                            }
                            current = Some(Vec::new());
                        } else if let Some(ref mut cur) = current {
                            cur.push(node);
                        }
                        // Content before first \item — skip
                    }
                    if let Some(last) = current {
                        items.push(last);
                    }

                    let mut result = Vec::new();
                    // Before list: paragraph glue
                    result.push(BoxNode::Glue {
                        natural: 6.0,
                        stretch: 2.0,
                        shrink: 0.0,
                    });

                    for (i, item_nodes) in items.iter().enumerate() {
                        // Inter-item glue (not before first item)
                        if i > 0 {
                            result.push(BoxNode::Glue {
                                natural: 4.0,
                                stretch: 0.5,
                                shrink: 0.5,
                            });
                        }
                        // Indentation kern
                        result.push(BoxNode::Kern { amount: 20.0 });
                        // Label prefix
                        if is_enumerate {
                            let label = format!("{}. ", i + 1);
                            result.push(BoxNode::Text {
                                width: 12.0,
                                text: label,
                                font_size: 10.0,
                            });
                        } else {
                            result.push(BoxNode::Text {
                                text: "• ".to_string(),
                                width: 7.0,
                                font_size: 10.0,
                            });
                        }
                        // Item content
                        for node in item_nodes {
                            let mut translated = translate_node_with_metrics(node, metrics);
                            result.append(&mut translated);
                        }
                    }

                    // After list: paragraph glue
                    result.push(BoxNode::Glue {
                        natural: 6.0,
                        stretch: 2.0,
                        shrink: 0.0,
                    });

                    result
                }
                "center" => {
                    let mut result = vec![BoxNode::AlignmentMarker {
                        alignment: Alignment::Center,
                    }];
                    for node in content {
                        result.extend(translate_node_with_metrics(node, metrics));
                    }
                    result.push(BoxNode::AlignmentMarker {
                        alignment: Alignment::Justify,
                    });
                    result
                }
                _ => content
                    .iter()
                    .flat_map(|n| translate_node_with_metrics(n, metrics))
                    .collect(),
            }
        }
        Node::InlineMath(nodes) => {
            let text: String = nodes.iter().map(math_node_to_text).collect();
            vec![BoxNode::Text {
                width: metrics.string_width(&text),
                text,
                font_size: 10.0,
            }]
        }
        Node::DisplayMath(nodes) => {
            let text: String = nodes.iter().map(math_node_to_text).collect();
            vec![
                BoxNode::Penalty { value: -10000 },
                BoxNode::Text {
                    width: metrics.string_width(&text),
                    text,
                    font_size: 10.0,
                },
                BoxNode::Penalty { value: -10000 },
                BoxNode::Glue {
                    natural: 6.0,
                    stretch: 2.0,
                    shrink: 0.0,
                },
            ]
        }
        Node::Group(nodes) => nodes
            .iter()
            .flat_map(|n| translate_node_with_metrics(n, metrics))
            .collect(),
        Node::Document(nodes) => nodes
            .iter()
            .flat_map(|n| translate_node_with_metrics(n, metrics))
            .collect(),
        // Other node types (Superscript, Subscript, Fraction, Radical, MathGroup) are only
        // found inside math mode, which we already handle above.
        _ => vec![],
    }
}

/// Translate a parser AST node into a flat list of box/glue items.
///
/// This converts the high-level AST into the low-level typesetting IR that
/// the line-breaking algorithm operates on. Uses CM Roman 10pt metrics by default.
pub fn translate_node(node: &Node) -> Vec<BoxNode> {
    translate_node_with_metrics(node, &StandardFontMetrics)
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
            BoxNode::Penalty { .. }
            | BoxNode::HBox { .. }
            | BoxNode::VBox { .. }
            | BoxNode::AlignmentMarker { .. } => {
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

// ===== Line Breaker Trait and Implementations =====

/// Trait for line-breaking algorithms.
pub trait LineBreaker {
    /// Break a list of box/glue items into lines of at most `hsize` points wide.
    fn break_lines(&self, items: &[BoxNode], hsize: f64) -> Vec<Vec<BoxNode>>;
}

/// Greedy line-breaking algorithm that wraps at the first opportunity.
///
/// Walks the list of box/glue items, accumulating items into lines. When
/// adding a `Text` or `Kern` item would exceed `hsize`, breaks at the last
/// glue position.
pub struct GreedyLineBreaker;

impl GreedyLineBreaker {
    /// Create a new `GreedyLineBreaker`.
    pub fn new() -> Self {
        GreedyLineBreaker
    }
}

impl Default for GreedyLineBreaker {
    fn default() -> Self {
        GreedyLineBreaker::new()
    }
}

impl LineBreaker for GreedyLineBreaker {
    fn break_lines(&self, items: &[BoxNode], hsize: f64) -> Vec<Vec<BoxNode>> {
        break_into_lines(items, hsize)
    }
}

/// Knuth-Plass optimal line-breaking algorithm.
///
/// Uses dynamic programming to find the sequence of breakpoints that minimizes
/// total demerits across all lines. Glue items are feasible breakpoints;
/// penalty items can force or prevent breaks.
///
/// Parameters:
/// - `tolerance`: Maximum badness (default 200). Lines with badness > tolerance
///   are rejected unless no other option exists.
pub struct KnuthPlassLineBreaker {
    /// Maximum allowed badness for a line (default 200).
    pub tolerance: i32,
}

impl KnuthPlassLineBreaker {
    /// Create a new `KnuthPlassLineBreaker` with default tolerance of 200.
    pub fn new() -> Self {
        KnuthPlassLineBreaker { tolerance: 200 }
    }
}

impl Default for KnuthPlassLineBreaker {
    fn default() -> Self {
        KnuthPlassLineBreaker::new()
    }
}

/// Compute the natural width, total stretch, and total shrink of a slice of items.
fn measure_items(items: &[BoxNode]) -> (f64, f64, f64) {
    let mut width = 0.0_f64;
    let mut stretch = 0.0_f64;
    let mut shrink = 0.0_f64;
    for item in items {
        match item {
            BoxNode::Text { width: w, .. } => width += w,
            BoxNode::Kern { amount } => width += amount,
            BoxNode::Glue {
                natural,
                stretch: s,
                shrink: sh,
            } => {
                width += natural;
                stretch += s;
                shrink += sh;
            }
            _ => {}
        }
    }
    (width, stretch, shrink)
}

/// Compute the adjustment ratio `r` for a line segment.
///
/// `r = (hsize - natural_width) / stretch_or_shrink`
/// Returns `None` if the line cannot be set at all (too wide with no shrink).
fn adjustment_ratio(natural_width: f64, stretch: f64, shrink: f64, hsize: f64) -> Option<f64> {
    let diff = hsize - natural_width;
    if diff.abs() < f64::EPSILON {
        Some(0.0)
    } else if diff > 0.0 {
        // Need to stretch
        if stretch > 0.0 {
            Some(diff / stretch)
        } else {
            // Can't stretch, but line is underfull — use very large ratio
            Some(f64::INFINITY)
        }
    } else {
        // Need to shrink
        if shrink > 0.0 {
            Some(diff / shrink) // will be negative
        } else {
            // Can't shrink and line is overfull — infeasible
            None
        }
    }
}

/// Compute badness from adjustment ratio `r`.
///
/// `b = min(100 * |r|^3, 10000)`
fn compute_badness(r: f64) -> f64 {
    if r.is_infinite() {
        10000.0
    } else {
        (100.0 * r.abs().powi(3)).min(10000.0)
    }
}

/// Compute demerits for a line with badness `b` and optional penalty `p`.
///
/// For a penalty breakpoint: `d = (1 + b)^2 + p^2`
/// For a glue breakpoint: `d = (1 + b)^2`
fn compute_demerits(badness: f64, penalty: Option<i32>) -> f64 {
    let b_part = (1.0 + badness).powi(2);
    let p_part = penalty.map_or(0.0, |p| (p as f64).powi(2));
    b_part + p_part
}

impl LineBreaker for KnuthPlassLineBreaker {
    fn break_lines(&self, items: &[BoxNode], hsize: f64) -> Vec<Vec<BoxNode>> {
        if items.is_empty() {
            return vec![];
        }

        let n = items.len();

        // Collect breakpoint candidates: (position, penalty_value)
        //
        // A breakpoint at position `pos`:
        //   - Glue: the line ends BEFORE this glue (at `pos`), the next line
        //     starts AFTER this glue (at `pos + 1`).
        //   - Penalty: the line ends at `pos` (the penalty itself has 0 width),
        //     the next line starts at `pos + 1`.
        //
        // We add a synthetic end sentinel at position `n` to represent "end of text".
        let mut breakpoints: Vec<(usize, Option<i32>)> = Vec::new();

        for (i, item) in items.iter().enumerate() {
            match item {
                BoxNode::Glue { .. } => {
                    // Glue is a feasible breakpoint only if preceded by a box item
                    let preceded_by_box = items[..i].iter().rev().any(|x| {
                        matches!(
                            x,
                            BoxNode::Text { .. } | BoxNode::Kern { .. } | BoxNode::HBox { .. }
                        )
                    });
                    if preceded_by_box {
                        breakpoints.push((i, None));
                    }
                }
                BoxNode::Penalty { value } => {
                    if *value != 10000 {
                        // 10000 = prohibited; everything else is a breakpoint candidate
                        breakpoints.push((i, Some(*value)));
                    }
                }
                _ => {}
            }
        }

        // Add end sentinel (position n = past end of items)
        breakpoints.push((n, None));

        let num_bp = breakpoints.len();
        let inf = f64::INFINITY;

        // DP over breakpoints:
        //
        // We use a "previous breakpoints" encoding:
        //   - Index num_bp (virtual) represents the start of the paragraph.
        //   - dp[j] = minimum total demerits to break the paragraph up to breakpoints[j].
        //   - prev[j] = the index of the previous breakpoint (or num_bp for "start").
        //
        // For breakpoint j with position bp_j:
        //   The line content is items[line_start..bp_j], where:
        //     line_start = 0 if previous = start
        //     line_start = bp_prev + 1 if previous = breakpoints[prev]
        //
        // The sentinel at position n is the "last line" — for it, the last line
        // gets 0 additional demerits (underfull last lines are OK).

        let mut dp = vec![inf; num_bp + 1]; // dp[num_bp] = 0 (virtual start)
        let mut prev_arr = vec![num_bp; num_bp + 1]; // prev_arr[j] = index of prev bp

        dp[num_bp] = 0.0; // cost of being at the start is 0

        for j in 0..num_bp {
            let (bp_j, bp_pen_j) = breakpoints[j];
            let forced_j = bp_pen_j == Some(-10000);
            let is_sentinel = bp_j == n;

            // Consider all previous active nodes (breakpoints i < j, plus start = num_bp)
            let mut candidates: Vec<usize> = (0..j).collect();
            candidates.push(num_bp); // the virtual start node

            for &prev_idx in &candidates {
                let prev_cost = dp[prev_idx];
                if prev_cost >= inf {
                    continue; // previous node unreachable
                }

                // Compute the line content range
                let line_start = if prev_idx == num_bp {
                    0 // start of paragraph
                } else {
                    breakpoints[prev_idx].0 + 1 // after the previous break item
                };
                let line_end = bp_j;

                if line_start > line_end {
                    continue;
                }

                // Check: if the line content contains a forced break penalty (-10000),
                // this combination is invalid (the forced break must be respected).
                let line_items_slice = &items[line_start..line_end];
                let contains_forced_break = line_items_slice
                    .iter()
                    .any(|x| matches!(x, BoxNode::Penalty { value } if *value == -10000));
                if contains_forced_break {
                    continue; // Cannot bridge over a forced break
                }

                // Measure the line for adjustment ratio purposes.
                //
                // In TeX's KP algorithm, the adjustment ratio for a line broken at
                // a glue item includes the glue's own stretch and shrink.
                // The glue at the break position is consumed (not rendered) but its
                // stretch/shrink capacities contribute to the line's flexibility.
                //
                // Strategy: measure items[line_start..bp_j] for boxes/kerns,
                // plus include the break glue's stretch/shrink if bp is glue.
                // Natural width = sum of all box/kern/glue widths in slice
                let (nat_w_base, stretch_base, shrink_base) = measure_items(line_items_slice);

                // Add the break glue's contribution (if it's a glue breakpoint)
                let (nat_w, stretch, shrink) = if bp_pen_j.is_none() && bp_j < n {
                    if let Some(BoxNode::Glue {
                        natural: g_nat,
                        stretch: g_str,
                        shrink: g_shr,
                    }) = items.get(bp_j)
                    {
                        (
                            nat_w_base + g_nat,
                            stretch_base + g_str,
                            shrink_base + g_shr,
                        )
                    } else {
                        (nat_w_base, stretch_base, shrink_base)
                    }
                } else {
                    (nat_w_base, stretch_base, shrink_base)
                };

                // Compute the cost of this line
                let line_demerits = if is_sentinel {
                    // The last line: underfull is fine (0 demerits), but overfull is still
                    // infeasible if it exceeds the shrink limit.
                    let ratio = adjustment_ratio(nat_w, stretch, shrink, hsize);
                    match ratio {
                        None => continue,                // overfull with no shrink → infeasible
                        Some(r) if r < -1.0 => continue, // over-shrunk → infeasible
                        _ => 0.0,                        // underfull or perfect → 0 demerits
                    }
                } else if forced_j {
                    // Forced break: accept regardless of width, but add penalty² cost
                    let pen = bp_pen_j.unwrap_or(0) as f64;
                    pen * pen
                } else {
                    // Normal line: compute adjustment ratio and badness
                    let ratio = match adjustment_ratio(nat_w, stretch, shrink, hsize) {
                        None => continue, // overfull, no shrink → infeasible
                        Some(r) => r,
                    };
                    if ratio < -1.0 {
                        continue; // over-shrunk → infeasible
                    }
                    let badness = compute_badness(ratio);
                    if badness > self.tolerance as f64 {
                        continue; // too bad → reject
                    }
                    compute_demerits(badness, bp_pen_j)
                };

                let total_cost = prev_cost + line_demerits;
                if total_cost < dp[j] {
                    dp[j] = total_cost;
                    prev_arr[j] = prev_idx;
                }
            }
        }

        // The end sentinel is always the last breakpoint
        let end_bp = num_bp - 1;

        if dp[end_bp] >= inf {
            // No feasible solution — fall back to greedy
            return break_into_lines(items, hsize);
        }

        // Backtrack to find the optimal sequence of breakpoints
        let mut bp_sequence: Vec<usize> = Vec::new();
        let mut cur = end_bp;
        loop {
            bp_sequence.push(cur);
            let p = prev_arr[cur];
            if p == num_bp {
                // Reached the virtual start
                break;
            }
            cur = p;
        }
        bp_sequence.reverse();

        // Extract lines from the breakpoint sequence
        let mut lines: Vec<Vec<BoxNode>> = Vec::new();
        let mut line_start = 0usize;

        for &bp_idx in &bp_sequence {
            let (bp_pos, _) = breakpoints[bp_idx];
            let line_end = bp_pos; // items[line_start..line_end]

            if line_start <= line_end && line_end <= n {
                let line = strip_glue(items[line_start..line_end].to_vec());
                if !line.is_empty() {
                    lines.push(line);
                }
            }

            // Next line starts after the break item (glue or penalty)
            line_start = if bp_pos < n { bp_pos + 1 } else { n };
        }

        if lines.is_empty() {
            // Fallback: single line with everything stripped
            let line = strip_glue(items.to_vec());
            if !line.is_empty() {
                return vec![line];
            }
        }

        lines
    }
}

/// Break items into lines while tracking alignment markers.
/// AlignmentMarker nodes set the alignment for lines that follow them.
/// They are removed from the output lines (not rendered directly).
pub fn break_items_with_alignment(items: &[BoxNode], hsize: f64) -> Vec<OutputLine> {
    // Segment items by alignment spans
    let mut segments: Vec<(Alignment, Vec<BoxNode>)> = Vec::new();
    let mut current_alignment = Alignment::Justify;
    let mut current_items: Vec<BoxNode> = Vec::new();

    for item in items {
        if let BoxNode::AlignmentMarker { alignment } = item {
            if !current_items.is_empty() {
                segments.push((current_alignment, current_items.clone()));
                current_items.clear();
            }
            current_alignment = *alignment;
        } else {
            current_items.push(item.clone());
        }
    }
    if !current_items.is_empty() {
        segments.push((current_alignment, current_items));
    }

    let breaker = KnuthPlassLineBreaker::new();
    let mut result: Vec<OutputLine> = Vec::new();

    for (alignment, seg_items) in segments {
        let lines = breaker.break_lines(&seg_items, hsize);
        for nodes in lines {
            result.push(OutputLine { alignment, nodes });
        }
    }

    result
}

/// A laid-out page ready for PDF rendering.
#[derive(Debug)]
pub struct Page {
    /// Page number (1-indexed).
    pub number: usize,
    /// Placeholder content — will become a proper box tree.
    pub content: String,
    /// The typeset box lines for this page.
    pub box_lines: Vec<OutputLine>,
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
    /// Translates the AST to box/glue items, performs Knuth-Plass optimal line breaking,
    /// and packages the result into pages. Uses `StandardFontMetrics` (CM Roman 10pt).
    /// Splits into multiple pages when accumulated line height exceeds `vsize` (700pt).
    pub fn typeset(&self) -> Vec<Page> {
        let metrics = StandardFontMetrics;
        let items = translate_node_with_metrics(&self.document, &metrics);
        let all_lines = break_items_with_alignment(&items, 345.0);
        let content = format!("(stub) document node: {:?}", self.document);

        let vsize = 700.0_f64;
        let line_height = 12.0_f64;
        let mut pages: Vec<Page> = Vec::new();
        let mut current_page_lines: Vec<OutputLine> = Vec::new();
        let mut accumulated_height = 0.0_f64;

        for line in all_lines {
            if accumulated_height + line_height > vsize && !current_page_lines.is_empty() {
                pages.push(Page {
                    number: pages.len() + 1,
                    content: content.clone(),
                    box_lines: current_page_lines,
                });
                current_page_lines = Vec::new();
                accumulated_height = 0.0;
            }
            current_page_lines.push(line);
            accumulated_height += line_height;
        }
        if !current_page_lines.is_empty() {
            pages.push(Page {
                number: pages.len() + 1,
                content: content.clone(),
                box_lines: current_page_lines,
            });
        }
        if pages.is_empty() {
            // Always return at least one page for backward compatibility
            pages.push(Page {
                number: 1,
                content,
                box_lines: vec![],
            });
        }
        pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_parser::Parser;

    // Helper: compute CM10 width for a word
    fn cm10_width(s: &str) -> f64 {
        let metrics = StandardFontMetrics;
        metrics.string_width(s)
    }

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
        let w = cm10_width("hello");
        let node = BoxNode::Text {
            text: "hello".to_string(),
            width: w,
            font_size: 10.0,
        };
        if let BoxNode::Text { text, width, .. } = &node {
            assert_eq!(text, "hello");
            assert!((width - w).abs() < f64::EPSILON);
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
                font_size: 10.0,
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
                font_size: 10.0,
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
        // hello: h+e+l+l+o = 6.94+4.44+2.78+2.78+5.00 = 21.94
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "hello".to_string(),
                width: cm10_width("hello"),
                font_size: 10.0,
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // world: w+o+r+l+d = 7.50+5.00+3.92+2.78+5.56 = 24.76
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "world".to_string(),
                width: cm10_width("world"),
                font_size: 10.0,
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
        // + paragraph spacing Glue
        // total: 5 items
        assert_eq!(items.len(), 5);
        // one: o+n+e = 5.00+5.56+4.44 = 15.00
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "one".to_string(),
                width: cm10_width("one"),
                font_size: 10.0,
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // two: t+w+o = 3.89+7.50+5.00 = 16.39
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "two".to_string(),
                width: cm10_width("two"),
                font_size: 10.0,
            }
        );
        // three: t+h+r+e+e = 3.89+6.94+3.92+4.44+4.44 = 23.63
        assert_eq!(
            items[3],
            BoxNode::Text {
                text: "three".to_string(),
                width: cm10_width("three"),
                font_size: 10.0,
            }
        );
    }

    #[test]
    fn test_translate_inline_math() {
        let node = Node::InlineMath(vec![Node::Text("x".to_string())]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        // Should render "x", not the old "(math)" placeholder
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('x'),
                "Expected math text to contain 'x', got '{}'",
                text
            );
            assert_ne!(text, "(math)", "Should not produce (math) placeholder");
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_translate_display_math() {
        let node = Node::DisplayMath(vec![Node::Text("E=mc^2".to_string())]);
        let items = translate_node(&node);
        // DisplayMath produces: Penalty, Text, Penalty, Glue (4 items)
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], BoxNode::Penalty { value: -10000 }));
        if let BoxNode::Text { text, .. } = &items[1] {
            assert_ne!(text, "(math)", "Should not produce (math) placeholder");
        } else {
            panic!("Expected BoxNode::Text at index 1");
        }
        assert!(matches!(items[2], BoxNode::Penalty { value: -10000 }));
        assert!(matches!(items[3], BoxNode::Glue { .. }));
    }

    #[test]
    fn test_translate_group_node() {
        let node = Node::Group(vec![Node::Text("inside".to_string())]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        // inside: i+n+s+i+d+e = 2.78+5.56+3.89+2.78+5.56+4.44 = 25.01
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "inside".to_string(),
                width: cm10_width("inside"),
                font_size: 10.0,
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
        // bold: b+o+l+d = 5.56+5.00+2.78+5.56 = 18.90
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "bold".to_string(),
                width: cm10_width("bold"),
                font_size: 10.0,
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // text: t+e+x+t = 3.89+4.44+5.28+3.89 = 17.50
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "text".to_string(),
                width: cm10_width("text"),
                font_size: 10.0,
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
        // content: c+o+n+t+e+n+t = 4.44+5.00+5.56+3.89+4.44+5.56+3.89 = 32.78
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "content".to_string(),
                width: cm10_width("content"),
                font_size: 10.0,
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // here: h+e+r+e = 6.94+4.44+3.92+4.44 = 19.74
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "here".to_string(),
                width: cm10_width("here"),
                font_size: 10.0,
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
        // first: f+i+r+s+t = 3.33+2.78+3.92+3.89+3.89 = 17.81
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "first".to_string(),
                width: cm10_width("first"),
                font_size: 10.0,
            }
        );
        // second: s+e+c+o+n+d = 3.89+4.44+4.44+5.00+5.56+5.56 = 28.89
        assert_eq!(
            items[1],
            BoxNode::Text {
                text: "second".to_string(),
                width: cm10_width("second"),
                font_size: 10.0,
            }
        );
    }

    // ===== char_width tests (backward compat string-based function) =====

    #[test]
    fn test_char_width() {
        // 'a' = 5.000 in CM Roman
        assert!((char_width("a") - 5.000).abs() < 0.01);
        // char_width uses same StandardFontMetrics as cm10_width helper
        assert!((char_width("hello") - cm10_width("hello")).abs() < f64::EPSILON);
        // empty string
        assert!((char_width("") - 0.0).abs() < f64::EPSILON);
    }

    // ===== Line breaking tests =====

    #[test]
    fn test_break_into_lines_short_text() {
        // "hello world" fits in one line with Helvetica widths (21.12 + 23.89 < 345)
        let items = vec![
            BoxNode::Text {
                text: "hello".to_string(),
                width: cm10_width("hello"),
                font_size: 10.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "world".to_string(),
                width: cm10_width("world"),
                font_size: 10.0,
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
                font_size: 10.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "bbbbbbbbbb".to_string(),
                width: 60.0,
                font_size: 10.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "cccccccccc".to_string(),
                width: 60.0,
                font_size: 10.0,
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
                font_size: 10.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "bbb".to_string(),
                width: 50.0,
                font_size: 10.0,
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
        // Should contain a math text box (not the old "(math)" placeholder)
        let all_items: Vec<&BoxNode> = pages[0]
            .box_lines
            .iter()
            .flat_map(|l| l.nodes.iter())
            .collect();
        let has_math_text = all_items.iter().any(
            |n| matches!(n, BoxNode::Text { text, .. } if text.contains('x') || text.contains('2')),
        );
        assert!(has_math_text, "Expected math content (x^2) in the output");
        // Must NOT use the old placeholder
        let has_placeholder = all_items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "(math)"));
        assert!(!has_placeholder, "Should not produce (math) placeholder");
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
        // italic: i+t+a+l+i+c = 2.78+3.89+5.00+2.78+2.78+4.44 = 21.67
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "italic".to_string(),
                width: cm10_width("italic"),
                font_size: 10.0,
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
        // emphasized: e+m+p+h+a+s+i+z+e+d = 4.44+8.33+5.56+6.94+5.00+3.89+2.78+4.44+4.44+5.56 = 51.38
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "emphasized".to_string(),
                width: cm10_width("emphasized"),
                font_size: 10.0,
            }
        );
    }

    #[test]
    fn test_break_into_lines_with_kern() {
        let items = vec![
            BoxNode::Text {
                text: "word".to_string(),
                width: 60.0,
                font_size: 10.0,
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

    // ===== CM10 Font Metrics Tests =====

    #[test]
    fn test_cm10_lowercase_a() {
        let m = StandardFontMetrics;
        // CM Roman a = 5.000pt
        assert!((m.char_width('a') - 5.000).abs() < 0.01);
    }

    #[test]
    fn test_cm10_lowercase_m_is_widest_lowercase() {
        let m = StandardFontMetrics;
        assert!((m.char_width('m') - 8.33).abs() < 0.01);
        // m should be wider than any other lowercase letter
        for ch in 'a'..='z' {
            if ch != 'm' {
                assert!(
                    m.char_width('m') > m.char_width(ch),
                    "'m' should be wider than '{}'",
                    ch
                );
            }
        }
    }

    #[test]
    fn test_cm10_i_and_l_are_narrow() {
        let m = StandardFontMetrics;
        // CM Roman i = 2.778pt, l = 2.778pt
        assert!((m.char_width('i') - 2.778).abs() < 0.01);
        assert!((m.char_width('l') - 2.778).abs() < 0.01);
    }

    #[test]
    fn test_cm10_different_chars_different_widths() {
        let m = StandardFontMetrics;
        // These pairs should have different widths (CM Roman metrics)
        // m=8.333, i=2.778 → diff > 1.0 ✓
        assert!((m.char_width('m') - m.char_width('i')).abs() > 1.0);
        // w=7.222, l=2.778 → diff > 1.0 ✓
        assert!((m.char_width('w') - m.char_width('l')).abs() > 1.0);
        // h=5.556, f=3.056 → diff > 1.0 ✓
        assert!((m.char_width('h') - m.char_width('f')).abs() > 1.0);
        // b=5.556, c=4.444 → diff = 1.112 > 0.5 ✓
        assert!((m.char_width('b') - m.char_width('c')).abs() > 0.5);
        // r=3.917, t=3.889 → diff is small, use k vs i instead: k=5.278, i=2.778 → diff > 1.0
        assert!((m.char_width('k') - m.char_width('i')).abs() > 1.0);
    }

    #[test]
    fn test_cm10_uppercase_generally_wider_than_lowercase() {
        let m = StandardFontMetrics;
        // Most uppercase letters are wider than their lowercase counterparts
        assert!(m.char_width('A') > m.char_width('a'));
        assert!(m.char_width('B') > m.char_width('b'));
        assert!(m.char_width('D') > m.char_width('d'));
        assert!(m.char_width('H') > m.char_width('h')); // H=7.22 > h=5.56 (Helvetica)
        assert!(m.char_width('W') > m.char_width('w'));
    }

    #[test]
    fn test_cm10_space_width() {
        let m = StandardFontMetrics;
        // CM Roman space = 3.333pt (AFM WX=333.333 / 100)
        assert!((m.space_width() - 3.333).abs() < 0.01);
    }

    #[test]
    fn test_cm10_digit_widths() {
        let m = StandardFontMetrics;
        // All digits should be 5.000pt in CM Roman
        for ch in '0'..='9' {
            assert!(
                (m.char_width(ch) - 5.000).abs() < 0.01,
                "Digit '{}' should be 5.000pt",
                ch
            );
        }
    }

    #[test]
    fn test_cm10_string_width_hello() {
        let m = StandardFontMetrics;
        // hello (CM Roman): h(5.556) + e(4.444) + l(2.778) + l(2.778) + o(5.000) = 20.556
        let expected = 5.556 + 4.444 + 2.778 + 2.778 + 5.000;
        assert!((m.string_width("hello") - expected).abs() < 0.01);
    }

    #[test]
    fn test_cm10_string_width_world() {
        let m = StandardFontMetrics;
        // world (CM Roman): w(7.222) + o(5.000) + r(3.917) + l(2.778) + d(5.556) = 24.473
        let expected = 7.222 + 5.000 + 3.917 + 2.778 + 5.556;
        assert!((m.string_width("world") - expected).abs() < 0.01);
    }

    #[test]
    fn test_cm10_unknown_char_default() {
        let m = StandardFontMetrics;
        // Unknown characters should default to 5.000pt in CM Roman
        assert!((m.char_width('€') - 5.000).abs() < 0.01);
        assert!((m.char_width('→') - 5.000).abs() < 0.01);
    }

    #[test]
    fn test_cm10_w_is_wide() {
        let m = StandardFontMetrics;
        // CM Roman w = 7.222pt
        assert!((m.char_width('w') - 7.222).abs() < 0.01);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_cm10_uppercase_W_widest() {
        let m = StandardFontMetrics;
        assert!((m.char_width('W') - 10.278).abs() < 0.01);
        // W should be wider than all other uppercase letters
        for ch in 'A'..='Z' {
            if ch != 'W' {
                assert!(
                    m.char_width('W') > m.char_width(ch),
                    "'W' should be wider than '{}'",
                    ch
                );
            }
        }
    }

    #[test]
    fn test_translate_node_with_metrics_uses_custom_metrics() {
        // Create a simple custom font metrics for testing
        struct FixedMetrics;
        impl FontMetrics for FixedMetrics {
            fn char_width(&self, _ch: char) -> f64 {
                10.0
            }
            fn space_width(&self) -> f64 {
                5.0
            }
        }

        let node = Node::Text("ab cd".to_string());
        let items = translate_node_with_metrics(&node, &FixedMetrics);
        assert_eq!(items.len(), 3);
        // "ab" = 2 chars × 10.0 = 20.0
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "ab".to_string(),
                width: 20.0,
                font_size: 10.0,
            }
        );
        // Glue should use FixedMetrics space_width = 5.0
        if let BoxNode::Glue { natural, .. } = &items[1] {
            assert!((natural - 5.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected Glue");
        }
        // "cd" = 2 chars × 10.0 = 20.0
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "cd".to_string(),
                width: 20.0,
                font_size: 10.0,
            }
        );
    }

    #[test]
    fn test_font_metrics_trait_string_width_empty() {
        let m = StandardFontMetrics;
        assert!((m.string_width("") - 0.0).abs() < f64::EPSILON);
    }

    // ===== KnuthPlass Line Breaker Tests =====

    fn make_glue() -> BoxNode {
        BoxNode::Glue {
            natural: 5.0,
            stretch: 2.0,
            shrink: 1.0,
        }
    }

    fn make_text(width: f64) -> BoxNode {
        BoxNode::Text {
            text: "w".repeat((width as usize).max(1)),
            width,
            font_size: 10.0,
        }
    }

    #[test]
    fn test_kp_empty_items() {
        let kp = KnuthPlassLineBreaker::new();
        let lines = kp.break_lines(&[], 100.0);
        assert!(lines.is_empty(), "Empty items should produce no lines");
    }

    #[test]
    fn test_kp_single_item_text() {
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![make_text(40.0)];
        let lines = kp.break_lines(&items, 100.0);
        assert_eq!(lines.len(), 1, "Single item should produce one line");
    }

    #[test]
    fn test_kp_single_line_no_break() {
        // "hello world" at hsize=345 — both words fit on one line
        let kp = KnuthPlassLineBreaker::new();
        let m = StandardFontMetrics;
        let items = vec![
            BoxNode::Text {
                text: "hello".to_string(),
                width: m.string_width("hello"),
                font_size: 10.0,
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "world".to_string(),
                width: m.string_width("world"),
                font_size: 10.0,
            },
        ];
        let lines = kp.break_lines(&items, 345.0);
        assert_eq!(lines.len(), 1, "Short text should fit on one line");
    }

    #[test]
    fn test_kp_two_line_break() {
        // Two words each 60pt wide, hsize=100. They can't both fit → two lines.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![make_text(60.0), make_glue(), make_text(60.0)];
        let lines = kp.break_lines(&items, 100.0);
        assert_eq!(lines.len(), 2, "Two wide words should produce two lines");
    }

    #[test]
    fn test_kp_forced_break() {
        // A penalty of -10000 forces a line break at that position.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![
            make_text(20.0),
            BoxNode::Penalty { value: -10000 },
            make_text(20.0),
        ];
        // Even though both words fit on one line, the forced break splits them.
        let lines = kp.break_lines(&items, 100.0);
        assert_eq!(lines.len(), 2, "Forced penalty -10000 must create a break");
    }

    #[test]
    fn test_kp_prohibited_break() {
        // A penalty of +10000 prevents a break at glue between the words.
        // We put: word(30) + glue + prohibited(10000) + word(30) — the glue
        // is the only break candidate but the prohibited penalty follows it.
        // Actually, penalty 10000 is inserted between items — it IS the break point.
        // With value=10000, we skip it as a breakpoint, so the line should not break there.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![
            make_text(30.0),
            BoxNode::Penalty { value: 10000 }, // prohibited — no break here
            make_text(30.0),
        ];
        // Both words fit (30+30=60 < 100), and no valid breakpoints between them.
        let lines = kp.break_lines(&items, 100.0);
        assert_eq!(
            lines.len(),
            1,
            "Prohibited penalty 10000 should prevent break"
        );
    }

    #[test]
    fn test_kp_matches_greedy_simple() {
        // For a simple paragraph with natural breaks, KP should produce
        // the same number of lines as greedy.
        let kp = KnuthPlassLineBreaker::new();
        let greedy = GreedyLineBreaker::new();

        // Three words of 60pt each, hsize=100 — forces breaks every word.
        let items = vec![
            make_text(60.0),
            make_glue(),
            make_text(60.0),
            make_glue(),
            make_text(60.0),
        ];
        let kp_lines = kp.break_lines(&items, 100.0);
        let greedy_lines = greedy.break_lines(&items, 100.0);
        assert_eq!(
            kp_lines.len(),
            greedy_lines.len(),
            "KP and greedy should agree on obviously-forced breaks"
        );
    }

    #[test]
    fn test_kp_better_than_greedy() {
        // Four words of 40pt each, glue of 5pt natural, hsize=100.
        // Natural widths: 40+5+40=85 for two words (fits), 40+5+40+5+40=130 (doesn't).
        // Greedy: line1=[40,5,40]=85, line2=[40,5,40]=85 (2 lines)
        // KP: should also give 2 lines of equal width — no advantage here, but
        // both should produce exactly 2 lines and the same content.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![
            make_text(40.0),
            make_glue(),
            make_text(40.0),
            make_glue(),
            make_text(40.0),
            make_glue(),
            make_text(40.0),
        ];
        let kp_lines = kp.break_lines(&items, 100.0);
        // Should produce 2 lines (2 words per line)
        assert_eq!(
            kp_lines.len(),
            2,
            "KP should produce 2 lines for 4×40pt words with hsize=100"
        );
        // Each line should have at least one Text node
        for (i, line) in kp_lines.iter().enumerate() {
            let has_text = line.iter().any(|n| matches!(n, BoxNode::Text { .. }));
            assert!(has_text, "Line {} should contain text", i);
        }
    }

    #[test]
    fn test_greedy_linebreaker_trait() {
        // GreedyLineBreaker implements LineBreaker and matches break_into_lines()
        let greedy = GreedyLineBreaker::new();
        let items = vec![
            make_text(60.0),
            make_glue(),
            make_text(60.0),
            make_glue(),
            make_text(60.0),
        ];
        let trait_result = greedy.break_lines(&items, 100.0);
        let direct_result = break_into_lines(&items, 100.0);
        assert_eq!(
            trait_result.len(),
            direct_result.len(),
            "GreedyLineBreaker trait result must match break_into_lines()"
        );
    }

    #[test]
    fn test_kp_single_glue_only() {
        // Only a glue item — no boxes to precede it, so no breakpoints.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![BoxNode::Glue {
            natural: 5.0,
            stretch: 2.0,
            shrink: 1.0,
        }];
        // Glue with no preceding box → no breakpoints, returns single (empty) line
        // strip_glue will strip the glue, so lines should be empty.
        let lines = kp.break_lines(&items, 100.0);
        // Either empty vec or one empty line stripped — both acceptable
        for line in &lines {
            assert!(
                !line.is_empty(),
                "Non-empty lines should contain actual content"
            );
        }
    }

    #[test]
    fn test_kp_many_words_fit_on_one_line() {
        // Many small words that all fit on one line — KP returns one line.
        let kp = KnuthPlassLineBreaker::new();
        let mut items = Vec::new();
        // 10 words of 5pt each with 2pt glue = 10*5 + 9*2 = 68pt < 100pt
        for i in 0..10 {
            if i > 0 {
                items.push(BoxNode::Glue {
                    natural: 2.0,
                    stretch: 1.0,
                    shrink: 0.5,
                });
            }
            items.push(make_text(5.0));
        }
        let lines = kp.break_lines(&items, 100.0);
        assert_eq!(lines.len(), 1, "All words fitting should produce one line");
    }

    #[test]
    fn test_kp_every_word_too_wide_falls_back() {
        // Words wider than hsize with no stretch/shrink — every line is infeasible.
        // KP should fall back to greedy.
        let kp = KnuthPlassLineBreaker::new();
        let greedy = GreedyLineBreaker::new();
        let items = vec![
            make_text(120.0), // wider than hsize=100
            make_glue(),
            make_text(120.0),
        ];
        let kp_lines = kp.break_lines(&items, 100.0);
        let greedy_lines = greedy.break_lines(&items, 100.0);
        // Both should produce the same result (greedy fallback)
        assert_eq!(
            kp_lines.len(),
            greedy_lines.len(),
            "KP should fall back to greedy for infeasible lines"
        );
    }

    #[test]
    fn test_kp_forced_break_in_middle_of_many_words() {
        // A forced break in the middle of a long sequence.
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![
            make_text(20.0),
            make_glue(),
            make_text(20.0),
            BoxNode::Penalty { value: -10000 }, // force break here
            make_text(20.0),
            make_glue(),
            make_text(20.0),
        ];
        let lines = kp.break_lines(&items, 200.0);
        // The forced break splits into at least 2 sections
        assert!(
            lines.len() >= 2,
            "Forced break should produce at least 2 lines"
        );
    }

    #[test]
    fn test_kp_three_lines() {
        // 9 words of 40pt each, glue 5pt, hsize=100.
        // Each pair of words (40+5+40=85) fits; triple (40+5+40+5+40=130) doesn't.
        // Should produce (at least) 3 lines for 6 words.
        let kp = KnuthPlassLineBreaker::new();
        let mut items = Vec::new();
        for i in 0..6 {
            if i > 0 {
                items.push(make_glue());
            }
            items.push(make_text(40.0));
        }
        let lines = kp.break_lines(&items, 100.0);
        assert!(
            lines.len() >= 3,
            "6 words of 40pt with hsize=100 should need ≥3 lines"
        );
    }

    #[test]
    fn test_kp_adjustment_ratio_helpers() {
        // Test the helper functions directly
        // ratio = (hsize - nat_w) / stretch_or_shrink
        // Perfect fit: ratio = 0
        let r = adjustment_ratio(100.0, 10.0, 5.0, 100.0);
        assert_eq!(r, Some(0.0));

        // Need to stretch: (110 - 100) / 10 = 1.0
        let r = adjustment_ratio(100.0, 10.0, 5.0, 110.0);
        assert!((r.unwrap() - 1.0).abs() < f64::EPSILON);

        // Need to shrink: (90 - 100) / 5 = -2.0
        let r = adjustment_ratio(100.0, 10.0, 5.0, 90.0);
        assert!((r.unwrap() - (-2.0)).abs() < f64::EPSILON);

        // Can't shrink, overfull: None
        let r = adjustment_ratio(100.0, 10.0, 0.0, 90.0);
        assert_eq!(r, None);
    }

    #[test]
    fn test_kp_badness_helpers() {
        // b = 100 * |r|^3, capped at 10000
        assert!((compute_badness(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((compute_badness(1.0) - 100.0).abs() < f64::EPSILON);
        assert!((compute_badness(2.0) - 800.0).abs() < f64::EPSILON);
        // Cap at 10000
        assert!((compute_badness(10.0) - 10000.0).abs() < f64::EPSILON);
        assert!((compute_badness(f64::INFINITY) - 10000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kp_demerits_helpers() {
        // d = (1 + b)^2 for glue breakpoints
        let d = compute_demerits(0.0, None);
        assert!((d - 1.0).abs() < f64::EPSILON);

        let d = compute_demerits(9.0, None);
        assert!((d - 100.0).abs() < f64::EPSILON); // (1+9)^2 = 100

        // d = (1 + b)^2 + p^2 for penalty breakpoints
        let d = compute_demerits(0.0, Some(10));
        assert!((d - (1.0 + 100.0)).abs() < f64::EPSILON); // 1 + 100 = 101

        let d = compute_demerits(9.0, Some(0));
        assert!((d - 100.0).abs() < f64::EPSILON); // (1+9)^2 + 0 = 100
    }

    #[test]
    fn test_kp_engine_uses_kp() {
        // Engine::typeset() should use KP internally (not greedy).
        // Verify it still produces valid output.
        let mut parser =
            rustlatex_parser::Parser::new("The quick brown fox jumps over the lazy dog");
        let doc = parser.parse();
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].box_lines.is_empty());
    }

    #[test]
    fn test_kp_penalty_in_sequence() {
        // Mix of glue and penalty breakpoints
        let kp = KnuthPlassLineBreaker::new();
        let items = vec![
            make_text(30.0),
            make_glue(),
            make_text(30.0),
            BoxNode::Penalty { value: 50 }, // soft penalty
            make_text(30.0),
            make_glue(),
            make_text(30.0),
        ];
        // Total natural: 30+5+30+0+30+5+30=130 > 100, so must break somewhere
        let lines = kp.break_lines(&items, 100.0);
        assert!(!lines.is_empty(), "Should produce at least one line");
        // Verify no line exceeds hsize significantly (allowing for glue stretch)
        for line in &lines {
            let width: f64 = line
                .iter()
                .map(|n| match n {
                    BoxNode::Text { width, .. } => *width,
                    BoxNode::Kern { amount } => *amount,
                    BoxNode::Glue { natural, .. } => *natural,
                    _ => 0.0,
                })
                .sum();
            assert!(
                width <= 150.0,
                "No line should be excessively wide, got {}",
                width
            );
        }
    }

    // ===== M12: Document Structure Rendering Tests =====

    #[test]
    fn test_boxnode_text_has_font_size() {
        let node = BoxNode::Text {
            text: "hello".to_string(),
            width: 20.0,
            font_size: 10.0,
        };
        if let BoxNode::Text { font_size, .. } = node {
            assert_eq!(font_size, 10.0);
        } else {
            panic!("Expected Text node");
        }
    }

    #[test]
    fn test_section_command_produces_large_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "section".to_string(),
            args: vec![Node::Group(vec![Node::Text("Hello".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_14pt = nodes.iter().any(
            |n| matches!(n, BoxNode::Text { font_size, .. } if (*font_size - 14.0).abs() < 0.001),
        );
        assert!(has_14pt, "Expected 14pt text for section");
    }

    #[test]
    fn test_subsection_produces_12pt_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "subsection".to_string(),
            args: vec![Node::Group(vec![Node::Text("Sub".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_12pt = nodes.iter().any(
            |n| matches!(n, BoxNode::Text { font_size, .. } if (*font_size - 12.0).abs() < 0.001),
        );
        assert!(has_12pt);
    }

    #[test]
    fn test_subsubsection_produces_11pt_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "subsubsection".to_string(),
            args: vec![Node::Group(vec![Node::Text("Sub2".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_11pt = nodes.iter().any(
            |n| matches!(n, BoxNode::Text { font_size, .. } if (*font_size - 11.0).abs() < 0.001),
        );
        assert!(has_11pt);
    }

    #[test]
    fn test_paragraph_spacing_adds_glue() {
        let metrics = StandardFontMetrics;
        let node = Node::Paragraph(vec![Node::Text("Hello world".to_string())]);
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_glue = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Glue { natural, .. } if (*natural - 6.0).abs() < 0.001));
        assert!(has_glue, "Expected paragraph spacing glue");
    }

    #[test]
    fn test_multipage_layout_splits_pages() {
        // Create enough text to exceed 700pt (at 12pt/line, ~58+ lines needed)
        // Use long repeated text that will wrap to many lines within one paragraph
        let long_text = "The quick brown fox jumps over the lazy dog and then runs around the field again and again. ".repeat(50);
        let doc = Node::Document(vec![Node::Paragraph(vec![Node::Text(long_text)])]);
        let engine = Engine::new(doc);
        let result = engine.typeset();
        assert!(result.len() >= 2, "Expected 2+ pages, got {}", result.len());
    }

    #[test]
    fn test_latex_command_expands_to_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "LaTeX".to_string(),
            args: vec![],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_latex = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "LaTeX"));
        assert!(has_latex, "Expected LaTeX text node");
    }

    #[test]
    fn test_tex_command_expands_to_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "TeX".to_string(),
            args: vec![],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_tex = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "TeX"));
        assert!(has_tex);
    }

    #[test]
    fn test_today_command_produces_text() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "today".to_string(),
            args: vec![],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_date = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains("January")));
        assert!(has_date, "Expected date text from \\today");
    }

    #[test]
    fn test_newline_forces_break() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "\\".to_string(),
            args: vec![],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_penalty = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Penalty { value } if *value == -10000));
        assert!(has_penalty);
    }

    #[test]
    fn test_newline_command_forces_break() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "newline".to_string(),
            args: vec![],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_penalty = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Penalty { value } if *value == -10000));
        assert!(has_penalty);
    }

    #[test]
    fn test_section_has_vertical_kern_before() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "section".to_string(),
            args: vec![Node::Group(vec![Node::Text("X".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        assert!(
            matches!(nodes.first(), Some(BoxNode::Kern { amount }) if (*amount - 12.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_section_has_vertical_kern_after() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "section".to_string(),
            args: vec![Node::Group(vec![Node::Text("X".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        assert!(
            matches!(nodes.last(), Some(BoxNode::Kern { amount }) if (*amount - 6.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_subsection_kern_before() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "subsection".to_string(),
            args: vec![Node::Group(vec![Node::Text("X".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        assert!(
            matches!(nodes.first(), Some(BoxNode::Kern { amount }) if (*amount - 12.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_default_font_size_is_10() {
        let metrics = StandardFontMetrics;
        let node = Node::Text("hello".to_string());
        let nodes = translate_node_with_metrics(&node, &metrics);
        let all_10pt = nodes.iter().all(|n| {
            if let BoxNode::Text { font_size, .. } = n {
                (*font_size - 10.0).abs() < 0.001
            } else {
                true
            }
        });
        assert!(all_10pt);
    }

    #[test]
    fn test_multipage_layout_first_page_not_empty() {
        let long_text = "The quick brown fox jumps over the lazy dog and then runs around the field again and again. ".repeat(50);
        let doc = Node::Document(vec![Node::Paragraph(vec![Node::Text(long_text)])]);
        let engine = Engine::new(doc);
        let result = engine.typeset();
        assert!(!result.is_empty());
        assert!(!result[0].box_lines.is_empty());
    }

    #[test]
    fn test_font_size_propagated_to_nodes() {
        let doc = Node::Document(vec![Node::Paragraph(vec![Node::Text("test".to_string())])]);
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        for page in &pages {
            for line in &page.box_lines {
                for node in &line.nodes {
                    if let BoxNode::Text { font_size, .. } = node {
                        assert!(*font_size > 0.0, "font_size must be positive");
                    }
                }
            }
        }
    }

    #[test]
    fn test_section_title_text_content() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "section".to_string(),
            args: vec![Node::Group(vec![Node::Text("Introduction".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        let has_title = nodes
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "Introduction"));
        assert!(has_title, "Section should contain title text");
    }

    #[test]
    fn test_section_produces_three_nodes() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "section".to_string(),
            args: vec![Node::Group(vec![Node::Text("Title".to_string())])],
        };
        let nodes = translate_node_with_metrics(&node, &metrics);
        // Should produce: Kern(12.0), Text(title), Kern(6.0)
        assert_eq!(
            nodes.len(),
            3,
            "Section should produce exactly 3 nodes (kern, text, kern)"
        );
    }

    #[test]
    fn test_empty_document_produces_one_page() {
        let doc = Node::Document(vec![]);
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert_eq!(
            pages.len(),
            1,
            "Empty document should still produce one page"
        );
    }

    // ===== M13: Math Rendering Tests =====

    #[test]
    fn test_math_superscript_renders_as_text() {
        // $x^2$ should render as text containing "x" and "2", NOT "(math)"
        let mut parser = Parser::new("$x^2$");
        let doc = parser.parse();
        let items = translate_node(&doc);
        let text_content: String = items
            .iter()
            .filter_map(|n| {
                if let BoxNode::Text { text, .. } = n {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        assert!(
            text_content.contains('x'),
            "Expected 'x' in math output, got '{}'",
            text_content
        );
        assert!(
            text_content.contains('2'),
            "Expected '2' in math output, got '{}'",
            text_content
        );
        assert!(
            !text_content.contains("(math)"),
            "Should not contain '(math)' placeholder"
        );
    }

    #[test]
    fn test_math_subscript_renders_as_text() {
        // $x_i$ should render text containing "x" and "i"
        let node = Node::InlineMath(vec![Node::Subscript {
            base: Box::new(Node::Text("x".to_string())),
            subscript: Box::new(Node::Text("i".to_string())),
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('x'),
                "Expected 'x' in subscript text, got '{}'",
                text
            );
            assert!(
                text.contains('i'),
                "Expected 'i' in subscript text, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_fraction_renders_slash() {
        // $\frac{a}{b}$ should produce text containing "a", "/", "b"
        let node = Node::InlineMath(vec![Node::Fraction {
            numerator: Box::new(Node::MathGroup(vec![Node::Text("a".to_string())])),
            denominator: Box::new(Node::MathGroup(vec![Node::Text("b".to_string())])),
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('a'),
                "Expected 'a' in fraction, got '{}'",
                text
            );
            assert!(
                text.contains('/'),
                "Expected '/' in fraction, got '{}'",
                text
            );
            assert!(
                text.contains('b'),
                "Expected 'b' in fraction, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_sqrt_renders_radical() {
        // $\sqrt{x}$ should produce text containing "√"
        let node = Node::InlineMath(vec![Node::Radical {
            degree: None,
            radicand: Box::new(Node::MathGroup(vec![Node::Text("x".to_string())])),
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('√') || text.to_lowercase().contains("sqrt"),
                "Expected radical symbol in '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_greek_alpha() {
        // $\alpha$ should produce text containing "α"
        let node = Node::InlineMath(vec![Node::Command {
            name: "alpha".to_string(),
            args: vec![],
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('α'),
                "Expected 'α' for \\alpha, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_greek_beta() {
        // $\beta$ should produce text containing "β"
        let node = Node::InlineMath(vec![Node::Command {
            name: "beta".to_string(),
            args: vec![],
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('β'),
                "Expected 'β' for \\beta, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_greek_pi() {
        // $\pi$ should produce text containing "π"
        let node = Node::InlineMath(vec![Node::Command {
            name: "pi".to_string(),
            args: vec![],
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(text.contains('π'), "Expected 'π' for \\pi, got '{}'", text);
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_operator_times() {
        // $a \times b$ should produce text containing "×"
        let node = Node::InlineMath(vec![
            Node::Text("a".to_string()),
            Node::Command {
                name: "times".to_string(),
                args: vec![],
            },
            Node::Text("b".to_string()),
        ]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('×'),
                "Expected '×' for \\times, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_operator_leq() {
        // $x \leq y$ should produce text containing "≤"
        let node = Node::InlineMath(vec![
            Node::Text("x".to_string()),
            Node::Command {
                name: "leq".to_string(),
                args: vec![],
            },
            Node::Text("y".to_string()),
        ]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(text.contains('≤'), "Expected '≤' for \\leq, got '{}'", text);
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_math_operator_infty() {
        // $\infty$ should produce text containing "∞"
        let node = Node::InlineMath(vec![Node::Command {
            name: "infty".to_string(),
            args: vec![],
        }]);
        let items = translate_node(&node);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, .. } = &items[0] {
            assert!(
                text.contains('∞'),
                "Expected '∞' for \\infty, got '{}'",
                text
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    #[test]
    fn test_display_math_forces_linebreak() {
        // DisplayMath node should produce Penalty{value: -10000} items
        // Use $$...$$ which the parser recognises as DisplayMath
        let mut parser = Parser::new("$$E=mc^2$$");
        let doc = parser.parse();
        let items = translate_node(&doc);
        let has_forced_break = items
            .iter()
            .any(|n| matches!(n, BoxNode::Penalty { value } if *value == -10000));
        assert!(
            has_forced_break,
            "Display math should force line breaks (Penalty -10000)"
        );
    }

    #[test]
    fn test_inline_math_no_math_placeholder() {
        // InlineMath with text "x" should NOT produce BoxNode::Text with text == "(math)"
        let node = Node::InlineMath(vec![Node::Text("x".to_string())]);
        let items = translate_node(&node);
        let has_placeholder = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "(math)"));
        assert!(
            !has_placeholder,
            "InlineMath should not produce '(math)' placeholder"
        );
    }

    #[test]
    fn test_math_node_to_text_superscript_nested() {
        // x^{2+y} → text contains "2" and "y"
        let node = Node::Superscript {
            base: Box::new(Node::Text("x".to_string())),
            exponent: Box::new(Node::MathGroup(vec![
                Node::Text("2".to_string()),
                Node::Text("+".to_string()),
                Node::Text("y".to_string()),
            ])),
        };
        let result = math_node_to_text(&node);
        assert!(
            result.contains('2'),
            "Expected '2' in nested superscript: '{}'",
            result
        );
        assert!(
            result.contains('y'),
            "Expected 'y' in nested superscript: '{}'",
            result
        );
    }

    #[test]
    fn test_math_node_to_text_fraction_complex() {
        // \frac{x^2}{y_i} → text contains "x" and "y"
        let node = Node::Fraction {
            numerator: Box::new(Node::MathGroup(vec![Node::Superscript {
                base: Box::new(Node::Text("x".to_string())),
                exponent: Box::new(Node::Text("2".to_string())),
            }])),
            denominator: Box::new(Node::MathGroup(vec![Node::Subscript {
                base: Box::new(Node::Text("y".to_string())),
                subscript: Box::new(Node::Text("i".to_string())),
            }])),
        };
        let result = math_node_to_text(&node);
        assert!(
            result.contains('x'),
            "Expected 'x' in complex fraction: '{}'",
            result
        );
        assert!(
            result.contains('y'),
            "Expected 'y' in complex fraction: '{}'",
            result
        );
    }

    #[test]
    fn test_math_text_has_computed_width() {
        // Math text width should equal metrics.string_width(text), not hardcoded 20.0
        let metrics = StandardFontMetrics;
        let node = Node::InlineMath(vec![Node::Text("x".to_string())]);
        let items = translate_node_with_metrics(&node, &metrics);
        assert_eq!(items.len(), 1);
        if let BoxNode::Text { text, width, .. } = &items[0] {
            let expected_width = metrics.string_width(text);
            assert!(
                (width - expected_width).abs() < f64::EPSILON,
                "Math text width should be computed from metrics ({}), not hardcoded. Got {}, expected {}",
                text,
                width,
                expected_width
            );
            assert!(
                (width - 20.0).abs() > f64::EPSILON || text == "????",
                "Width should not be hardcoded 20.0 (unless text happens to be exactly 20.0pt wide)"
            );
        } else {
            panic!("Expected BoxNode::Text");
        }
    }

    // ===== M14: List rendering tests =====

    fn make_itemize(items: Vec<Vec<Node>>) -> Node {
        let mut content = Vec::new();
        for item_nodes in items {
            content.push(Node::Command {
                name: "item".to_string(),
                args: vec![],
            });
            content.extend(item_nodes);
        }
        Node::Environment {
            name: "itemize".to_string(),
            options: None,
            content,
        }
    }

    fn make_enumerate(items: Vec<Vec<Node>>) -> Node {
        let mut content = Vec::new();
        for item_nodes in items {
            content.push(Node::Command {
                name: "item".to_string(),
                args: vec![],
            });
            content.extend(item_nodes);
        }
        Node::Environment {
            name: "enumerate".to_string(),
            options: None,
            content,
        }
    }

    #[test]
    fn test_itemize_produces_bullet_prefix() {
        let node = make_itemize(vec![vec![Node::Text("apple".to_string())]]);
        let items = translate_node(&node);
        let has_bullet = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains('•')));
        assert!(has_bullet, "Expected a bullet • prefix in itemize output");
    }

    #[test]
    fn test_enumerate_produces_numbered_prefix() {
        let node = make_enumerate(vec![
            vec![Node::Text("first".to_string())],
            vec![Node::Text("second".to_string())],
            vec![Node::Text("third".to_string())],
        ]);
        let items = translate_node(&node);
        let has_1 = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains("1.")));
        let has_2 = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains("2.")));
        let has_3 = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains("3.")));
        assert!(has_1, "Expected '1.' in enumerate output");
        assert!(has_2, "Expected '2.' in enumerate output");
        assert!(has_3, "Expected '3.' in enumerate output");
    }

    #[test]
    fn test_itemize_item_has_indentation() {
        let node = make_itemize(vec![vec![Node::Text("apple".to_string())]]);
        let items = translate_node(&node);
        let has_kern = items.iter().any(
            |n| matches!(n, BoxNode::Kern { amount } if (*amount - 20.0).abs() < f64::EPSILON),
        );
        assert!(has_kern, "Expected Kern(20.0) indentation in itemize");
    }

    #[test]
    fn test_enumerate_item_has_indentation() {
        let node = make_enumerate(vec![vec![Node::Text("first".to_string())]]);
        let items = translate_node(&node);
        let has_kern = items.iter().any(
            |n| matches!(n, BoxNode::Kern { amount } if (*amount - 20.0).abs() < f64::EPSILON),
        );
        assert!(has_kern, "Expected Kern(20.0) indentation in enumerate");
    }

    #[test]
    fn test_itemize_three_items() {
        let node = make_itemize(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
            vec![Node::Text("c".to_string())],
        ]);
        let items = translate_node(&node);
        let bullet_count = items
            .iter()
            .filter(|n| matches!(n, BoxNode::Text { text, .. } if text.contains('•')))
            .count();
        assert_eq!(bullet_count, 3, "Expected 3 bullet prefixes for 3 items");
    }

    #[test]
    fn test_enumerate_three_items() {
        let node = make_enumerate(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
            vec![Node::Text("c".to_string())],
        ]);
        let items = translate_node(&node);
        let numbered_count = items
            .iter()
            .filter(|n| matches!(n, BoxNode::Text { text, .. } if text.ends_with(". ")))
            .count();
        assert_eq!(
            numbered_count, 3,
            "Expected 3 numbered prefixes for 3 items"
        );
    }

    #[test]
    fn test_list_surrounded_by_paragraph_glue() {
        let node = make_itemize(vec![vec![Node::Text("item".to_string())]]);
        let items = translate_node(&node);
        // First element should be Glue{natural:6.0}
        assert!(
            matches!(items.first(), Some(BoxNode::Glue { natural, .. }) if (*natural - 6.0).abs() < f64::EPSILON),
            "Expected Glue(6.0) at start of list"
        );
        // Last element should be Glue{natural:6.0}
        assert!(
            matches!(items.last(), Some(BoxNode::Glue { natural, .. }) if (*natural - 6.0).abs() < f64::EPSILON),
            "Expected Glue(6.0) at end of list"
        );
    }

    #[test]
    fn test_itemize_item_text_preserved() {
        let node = make_itemize(vec![vec![Node::Text("banana".to_string())]]);
        let items = translate_node(&node);
        let has_text = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "banana"));
        assert!(has_text, "Expected item text 'banana' in itemize output");
    }

    #[test]
    fn test_enumerate_item_text_preserved() {
        let node = make_enumerate(vec![vec![Node::Text("orange".to_string())]]);
        let items = translate_node(&node);
        let has_text = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "orange"));
        assert!(has_text, "Expected item text 'orange' in enumerate output");
    }

    #[test]
    fn test_itemize_inter_item_glue() {
        let node = make_itemize(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
        ]);
        let items = translate_node(&node);
        // Between items there should be a Glue{natural:4.0, stretch:0.5, shrink:0.5}
        let has_inter_glue = items.iter().any(|n| {
            matches!(n, BoxNode::Glue { natural, stretch, shrink }
                if (*natural - 4.0).abs() < f64::EPSILON
                && (*stretch - 0.5).abs() < f64::EPSILON
                && (*shrink - 0.5).abs() < f64::EPSILON)
        });
        assert!(
            has_inter_glue,
            "Expected inter-item Glue(4.0, 0.5, 0.5) in itemize"
        );
    }

    #[test]
    fn test_enumerate_inter_item_glue() {
        let node = make_enumerate(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
        ]);
        let items = translate_node(&node);
        let has_inter_glue = items.iter().any(|n| {
            matches!(n, BoxNode::Glue { natural, stretch, shrink }
                if (*natural - 4.0).abs() < f64::EPSILON
                && (*stretch - 0.5).abs() < f64::EPSILON
                && (*shrink - 0.5).abs() < f64::EPSILON)
        });
        assert!(
            has_inter_glue,
            "Expected inter-item Glue(4.0, 0.5, 0.5) in enumerate"
        );
    }

    #[test]
    fn test_enumerate_second_item_prefix_is_2() {
        let node = make_enumerate(vec![
            vec![Node::Text("first".to_string())],
            vec![Node::Text("second".to_string())],
        ]);
        let items = translate_node(&node);
        let has_2 = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "2. "));
        assert!(has_2, "Expected '2. ' label for second enumerate item");
    }

    #[test]
    fn test_enumerate_third_item_prefix_is_3() {
        let node = make_enumerate(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
            vec![Node::Text("c".to_string())],
        ]);
        let items = translate_node(&node);
        let has_3 = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "3. "));
        assert!(has_3, "Expected '3. ' label for third enumerate item");
    }

    #[test]
    fn test_itemize_no_numbering() {
        let node = make_itemize(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
        ]);
        let items = translate_node(&node);
        let has_numbered = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text == "1. " || text == "2. "));
        assert!(
            !has_numbered,
            "itemize should NOT produce numbered prefixes like '1. '"
        );
    }

    #[test]
    fn test_enumerate_no_bullet() {
        let node = make_enumerate(vec![
            vec![Node::Text("a".to_string())],
            vec![Node::Text("b".to_string())],
        ]);
        let items = translate_node(&node);
        let has_bullet = items
            .iter()
            .any(|n| matches!(n, BoxNode::Text { text, .. } if text.contains('•')));
        assert!(
            !has_bullet,
            "enumerate should NOT produce bullet • prefixes"
        );
    }

    #[test]
    fn test_enumerate_label_width_is_12() {
        let node = make_enumerate(vec![vec![Node::Text("item".to_string())]]);
        let items = translate_node(&node);
        let label_node = items
            .iter()
            .find(|n| matches!(n, BoxNode::Text { text, .. } if text == "1. "));
        if let Some(BoxNode::Text { width, .. }) = label_node {
            assert!(
                (*width - 12.0).abs() < f64::EPSILON,
                "Expected enumerate label width 12.0, got {}",
                width
            );
        } else {
            panic!("Expected a Text node with '1. '");
        }
    }

    #[test]
    fn test_itemize_bullet_width_is_7() {
        let node = make_itemize(vec![vec![Node::Text("item".to_string())]]);
        let items = translate_node(&node);
        let bullet_node = items
            .iter()
            .find(|n| matches!(n, BoxNode::Text { text, .. } if text.contains('•')));
        if let Some(BoxNode::Text { width, .. }) = bullet_node {
            assert!(
                (*width - 7.0).abs() < f64::EPSILON,
                "Expected itemize bullet width 7.0, got {}",
                width
            );
        } else {
            panic!("Expected a Text node with bullet •");
        }
    }

    // ===== M16: Alignment tests =====

    #[test]
    fn test_alignment_enum_default() {
        assert_eq!(Alignment::default(), Alignment::Justify);
    }

    #[test]
    fn test_alignment_marker_boxnode_centering() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "centering".to_string(),
            args: vec![],
        };
        let result = translate_node_with_metrics(&node, &metrics);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            BoxNode::AlignmentMarker {
                alignment: Alignment::Center
            }
        );
    }

    #[test]
    fn test_alignment_marker_boxnode_raggedright() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "raggedright".to_string(),
            args: vec![],
        };
        let result = translate_node_with_metrics(&node, &metrics);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            BoxNode::AlignmentMarker {
                alignment: Alignment::RaggedRight
            }
        );
    }

    #[test]
    fn test_alignment_marker_boxnode_raggedleft() {
        let metrics = StandardFontMetrics;
        let node = Node::Command {
            name: "raggedleft".to_string(),
            args: vec![],
        };
        let result = translate_node_with_metrics(&node, &metrics);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            BoxNode::AlignmentMarker {
                alignment: Alignment::RaggedLeft
            }
        );
    }

    #[test]
    fn test_center_environment_produces_alignment_markers() {
        let metrics = StandardFontMetrics;
        let node = Node::Environment {
            name: "center".to_string(),
            options: None,
            content: vec![Node::Text("hello".to_string())],
        };
        let result = translate_node_with_metrics(&node, &metrics);
        // First item is AlignmentMarker::Center, last is AlignmentMarker::Justify
        assert!(matches!(
            result.first(),
            Some(BoxNode::AlignmentMarker {
                alignment: Alignment::Center
            })
        ));
        assert!(matches!(
            result.last(),
            Some(BoxNode::AlignmentMarker {
                alignment: Alignment::Justify
            })
        ));
    }

    #[test]
    fn test_break_items_with_alignment_default_justify() {
        let items = vec![BoxNode::Text {
            text: "hello".to_string(),
            width: 30.0,
            font_size: 10.0,
        }];
        let lines = break_items_with_alignment(&items, 345.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].alignment, Alignment::Justify);
    }

    #[test]
    fn test_break_items_with_alignment_centering() {
        let items = vec![
            BoxNode::AlignmentMarker {
                alignment: Alignment::Center,
            },
            BoxNode::Text {
                text: "hello".to_string(),
                width: 30.0,
                font_size: 10.0,
            },
        ];
        let lines = break_items_with_alignment(&items, 345.0);
        assert!(!lines.is_empty());
        assert_eq!(lines[0].alignment, Alignment::Center);
    }

    #[test]
    fn test_break_items_with_alignment_raggedright() {
        let items = vec![
            BoxNode::AlignmentMarker {
                alignment: Alignment::RaggedRight,
            },
            BoxNode::Text {
                text: "hi".to_string(),
                width: 20.0,
                font_size: 10.0,
            },
        ];
        let lines = break_items_with_alignment(&items, 345.0);
        assert!(!lines.is_empty());
        assert_eq!(lines[0].alignment, Alignment::RaggedRight);
    }

    #[test]
    fn test_break_items_with_alignment_raggedleft() {
        let items = vec![
            BoxNode::AlignmentMarker {
                alignment: Alignment::RaggedLeft,
            },
            BoxNode::Text {
                text: "hi".to_string(),
                width: 20.0,
                font_size: 10.0,
            },
        ];
        let lines = break_items_with_alignment(&items, 345.0);
        assert!(!lines.is_empty());
        assert_eq!(lines[0].alignment, Alignment::RaggedLeft);
    }

    #[test]
    fn test_break_items_alignment_switches_mid_document() {
        let items = vec![
            BoxNode::Text {
                text: "normal".to_string(),
                width: 40.0,
                font_size: 10.0,
            },
            BoxNode::AlignmentMarker {
                alignment: Alignment::Center,
            },
            BoxNode::Text {
                text: "centered".to_string(),
                width: 50.0,
                font_size: 10.0,
            },
        ];
        let lines = break_items_with_alignment(&items, 345.0);
        assert!(lines.len() >= 2);
        assert_eq!(lines[0].alignment, Alignment::Justify);
        assert_eq!(lines[1].alignment, Alignment::Center);
    }

    #[test]
    fn test_alignment_marker_not_in_output_nodes() {
        let items = vec![
            BoxNode::AlignmentMarker {
                alignment: Alignment::Center,
            },
            BoxNode::Text {
                text: "text".to_string(),
                width: 30.0,
                font_size: 10.0,
            },
        ];
        let lines = break_items_with_alignment(&items, 345.0);
        // AlignmentMarker should be stripped from nodes
        for line in &lines {
            for node in &line.nodes {
                assert!(!matches!(node, BoxNode::AlignmentMarker { .. }));
            }
        }
    }

    #[test]
    fn test_output_line_struct() {
        let line = OutputLine {
            alignment: Alignment::Center,
            nodes: vec![BoxNode::Text {
                text: "x".to_string(),
                width: 6.0,
                font_size: 10.0,
            }],
        };
        assert_eq!(line.alignment, Alignment::Center);
        assert_eq!(line.nodes.len(), 1);
    }

    #[test]
    fn test_typeset_with_centering_command() {
        let doc = Node::Document(vec![
            Node::Command {
                name: "centering".to_string(),
                args: vec![],
            },
            Node::Paragraph(vec![Node::Text("Hello World".to_string())]),
        ]);
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert!(!pages.is_empty());
        let has_centered = pages
            .iter()
            .any(|p| p.box_lines.iter().any(|l| l.alignment == Alignment::Center));
        assert!(has_centered, "Expected at least one centered line");
    }

    #[test]
    fn test_typeset_center_environment() {
        let doc = Node::Document(vec![Node::Environment {
            name: "center".to_string(),
            options: None,
            content: vec![Node::Text("Centered text".to_string())],
        }]);
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert!(!pages.is_empty());
        let has_centered = pages
            .iter()
            .any(|p| p.box_lines.iter().any(|l| l.alignment == Alignment::Center));
        assert!(
            has_centered,
            "Expected centered lines from center environment"
        );
    }

    #[test]
    fn test_typeset_raggedright_command() {
        let doc = Node::Document(vec![
            Node::Command {
                name: "raggedright".to_string(),
                args: vec![],
            },
            Node::Paragraph(vec![Node::Text("Left text".to_string())]),
        ]);
        let engine = Engine::new(doc);
        let pages = engine.typeset();
        assert!(!pages.is_empty());
        let has_ragged = pages.iter().any(|p| {
            p.box_lines
                .iter()
                .any(|l| l.alignment == Alignment::RaggedRight)
        });
        assert!(has_ragged);
    }

    #[test]
    fn test_break_items_empty_produces_no_lines() {
        let lines = break_items_with_alignment(&[], 345.0);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_center_env_content_has_no_alignment_markers_in_nodes() {
        let metrics = StandardFontMetrics;
        let node = Node::Environment {
            name: "center".to_string(),
            options: None,
            content: vec![Node::Text("test".to_string())],
        };
        let items = translate_node_with_metrics(&node, &metrics);
        let lines = break_items_with_alignment(&items, 345.0);
        for line in &lines {
            for n in &line.nodes {
                assert!(!matches!(n, BoxNode::AlignmentMarker { .. }));
            }
        }
    }
}
