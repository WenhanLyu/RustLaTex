//! `rustlatex-engine` — LaTeX typesetting engine
//!
//! This crate implements the typesetting engine that transforms an AST into
//! a laid-out document using TeX's box/glue model. It implements both a greedy
//! line-breaking algorithm and the Knuth-Plass optimal line-breaking algorithm.

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

// ===== Font Metrics Trait and CM Roman 10pt Implementation =====

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

/// Font metrics for Computer Modern Roman 10pt (cmr10), based on TFM data.
pub struct StandardFontMetrics;

impl FontMetrics for StandardFontMetrics {
    fn char_width(&self, ch: char) -> f64 {
        match ch {
            'a' => 5.00,
            'b' => 5.56,
            'c' => 4.44,
            'd' => 5.56,
            'e' => 4.44,
            'f' => 3.33,
            'g' => 5.00,
            'h' => 6.94,
            'i' => 2.78,
            'j' => 3.06,
            'k' => 5.56,
            'l' => 2.78,
            'm' => 8.33,
            'n' => 5.56,
            'o' => 5.00,
            'p' => 5.56,
            'q' => 5.28,
            'r' => 3.92,
            's' => 3.89,
            't' => 3.89,
            'u' => 6.94,
            'v' => 5.28,
            'w' => 7.50,
            'x' => 5.28,
            'y' => 5.28,
            'z' => 4.44,
            'A' => 7.22,
            'B' => 6.67,
            'C' => 6.67,
            'D' => 7.22,
            'E' => 6.11,
            'F' => 5.56,
            'G' => 7.22,
            'H' => 7.22,
            'I' => 2.78,
            'J' => 3.89,
            'K' => 7.22,
            'L' => 6.11,
            'M' => 8.33,
            'N' => 7.22,
            'O' => 7.78,
            'P' => 6.11,
            'Q' => 7.78,
            'R' => 6.94,
            'S' => 5.56,
            'T' => 6.67,
            'U' => 7.22,
            'V' => 7.22,
            'W' => 9.44,
            'X' => 6.94,
            'Y' => 7.22,
            'Z' => 6.11,
            '0'..='9' => 5.56,
            _ => 6.0,
        }
    }

    fn space_width(&self) -> f64 {
        3.33
    }
}

/// Character width in points — backward-compatible function.
///
/// Computes the total width of a string using CM Roman 10pt metrics
/// by summing the width of each individual character.
pub fn char_width(s: &str) -> f64 {
    let metrics = StandardFontMetrics;
    metrics.string_width(s)
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
                });
            }
            result
        }
        Node::Paragraph(nodes) => nodes
            .iter()
            .flat_map(|n| translate_node_with_metrics(n, metrics))
            .collect(),
        Node::Command { name, args } => {
            match name.as_str() {
                "textbf" | "textit" | "emph" => {
                    // For known formatting commands, translate their arguments
                    args.iter()
                        .flat_map(|n| translate_node_with_metrics(n, metrics))
                        .collect()
                }
                _ => {
                    // Unknown commands → skip
                    vec![]
                }
            }
        }
        Node::Environment { content, .. } => content
            .iter()
            .flat_map(|n| translate_node_with_metrics(n, metrics))
            .collect(),
        Node::InlineMath(_) | Node::DisplayMath(_) => {
            vec![BoxNode::Text {
                text: "(math)".to_string(),
                width: 20.0,
            }]
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
    /// Translates the AST to box/glue items, performs Knuth-Plass optimal line breaking,
    /// and packages the result into pages. Uses `StandardFontMetrics` (CM Roman 10pt).
    pub fn typeset(&self) -> Vec<Page> {
        let metrics = StandardFontMetrics;
        let items = translate_node_with_metrics(&self.document, &metrics);
        let breaker = KnuthPlassLineBreaker::new();
        let lines = breaker.break_lines(&items, 345.0);
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
        };
        if let BoxNode::Text { text, width } = &node {
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
        // hello: h+e+l+l+o = 6.94+4.44+2.78+2.78+5.00 = 21.94
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "hello".to_string(),
                width: cm10_width("hello")
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // world: w+o+r+l+d = 7.50+5.00+3.92+2.78+5.56 = 24.76
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "world".to_string(),
                width: cm10_width("world")
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
        // one: o+n+e = 5.00+5.56+4.44 = 15.00
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "one".to_string(),
                width: cm10_width("one")
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // two: t+w+o = 3.89+7.50+5.00 = 16.39
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "two".to_string(),
                width: cm10_width("two")
            }
        );
        // three: t+h+r+e+e = 3.89+6.94+3.92+4.44+4.44 = 23.63
        assert_eq!(
            items[3],
            BoxNode::Text {
                text: "three".to_string(),
                width: cm10_width("three")
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
        // inside: i+n+s+i+d+e = 2.78+5.56+3.89+2.78+5.56+4.44 = 25.01
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "inside".to_string(),
                width: cm10_width("inside")
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
                width: cm10_width("bold")
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // text: t+e+x+t = 3.89+4.44+5.28+3.89 = 17.50
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "text".to_string(),
                width: cm10_width("text")
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
                width: cm10_width("content")
            }
        );
        assert!(matches!(items[1], BoxNode::Glue { .. }));
        // here: h+e+r+e = 6.94+4.44+3.92+4.44 = 19.74
        assert_eq!(
            items[2],
            BoxNode::Text {
                text: "here".to_string(),
                width: cm10_width("here")
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
                width: cm10_width("first")
            }
        );
        // second: s+e+c+o+n+d = 3.89+4.44+4.44+5.00+5.56+5.56 = 28.89
        assert_eq!(
            items[1],
            BoxNode::Text {
                text: "second".to_string(),
                width: cm10_width("second")
            }
        );
    }

    // ===== char_width tests (backward compat string-based function) =====

    #[test]
    fn test_char_width() {
        // 'a' = 5.00 in CM10
        assert!((char_width("a") - 5.00).abs() < 0.01);
        // 'hello' = h+e+l+l+o = 6.94+4.44+2.78+2.78+5.00 = 21.94
        assert!((char_width("hello") - cm10_width("hello")).abs() < f64::EPSILON);
        // empty string
        assert!((char_width("") - 0.0).abs() < f64::EPSILON);
    }

    // ===== Line breaking tests =====

    #[test]
    fn test_break_into_lines_short_text() {
        // "hello world" fits in one line with CM10 widths (21.94 + 24.76 < 345)
        let items = vec![
            BoxNode::Text {
                text: "hello".to_string(),
                width: cm10_width("hello"),
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "world".to_string(),
                width: cm10_width("world"),
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
        // italic: i+t+a+l+i+c = 2.78+3.89+5.00+2.78+2.78+4.44 = 21.67
        assert_eq!(
            items[0],
            BoxNode::Text {
                text: "italic".to_string(),
                width: cm10_width("italic")
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
                width: cm10_width("emphasized")
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

    // ===== CM10 Font Metrics Tests =====

    #[test]
    fn test_cm10_lowercase_a() {
        let m = StandardFontMetrics;
        assert!((m.char_width('a') - 5.00).abs() < 0.01);
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
        assert!((m.char_width('i') - 2.78).abs() < 0.01);
        assert!((m.char_width('l') - 2.78).abs() < 0.01);
    }

    #[test]
    fn test_cm10_different_chars_different_widths() {
        let m = StandardFontMetrics;
        // These pairs should have different widths
        assert!((m.char_width('m') - m.char_width('i')).abs() > 1.0);
        assert!((m.char_width('w') - m.char_width('l')).abs() > 1.0);
        assert!((m.char_width('h') - m.char_width('f')).abs() > 1.0);
        assert!((m.char_width('b') - m.char_width('c')).abs() > 0.5);
        assert!((m.char_width('d') - m.char_width('e')).abs() > 0.5);
    }

    #[test]
    fn test_cm10_uppercase_generally_wider_than_lowercase() {
        let m = StandardFontMetrics;
        // Most uppercase letters are wider than their lowercase counterparts
        assert!(m.char_width('A') > m.char_width('a'));
        assert!(m.char_width('B') > m.char_width('b'));
        assert!(m.char_width('D') > m.char_width('d'));
        assert!(m.char_width('H') > m.char_width('h')); // H=7.22 > h=6.94
        assert!(m.char_width('W') > m.char_width('w'));
    }

    #[test]
    fn test_cm10_space_width() {
        let m = StandardFontMetrics;
        assert!((m.space_width() - 3.33).abs() < 0.01);
    }

    #[test]
    fn test_cm10_digit_widths() {
        let m = StandardFontMetrics;
        // All digits should be 5.56pt (monospaced digits in CM)
        for ch in '0'..='9' {
            assert!(
                (m.char_width(ch) - 5.56).abs() < 0.01,
                "Digit '{}' should be 5.56pt",
                ch
            );
        }
    }

    #[test]
    fn test_cm10_string_width_hello() {
        let m = StandardFontMetrics;
        // hello: h(6.94) + e(4.44) + l(2.78) + l(2.78) + o(5.00) = 21.94
        let expected = 6.94 + 4.44 + 2.78 + 2.78 + 5.00;
        assert!((m.string_width("hello") - expected).abs() < 0.01);
    }

    #[test]
    fn test_cm10_string_width_world() {
        let m = StandardFontMetrics;
        // world: w(7.50) + o(5.00) + r(3.92) + l(2.78) + d(5.56) = 24.76
        let expected = 7.50 + 5.00 + 3.92 + 2.78 + 5.56;
        assert!((m.string_width("world") - expected).abs() < 0.01);
    }

    #[test]
    fn test_cm10_unknown_char_default() {
        let m = StandardFontMetrics;
        // Unknown characters should default to 6.0pt
        assert!((m.char_width('€') - 6.0).abs() < 0.01);
        assert!((m.char_width('→') - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_cm10_w_is_wide() {
        let m = StandardFontMetrics;
        assert!((m.char_width('w') - 7.50).abs() < 0.01);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_cm10_uppercase_W_widest() {
        let m = StandardFontMetrics;
        assert!((m.char_width('W') - 9.44).abs() < 0.01);
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
                width: 20.0
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
                width: 20.0
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
            },
            BoxNode::Glue {
                natural: 3.33,
                stretch: 1.67,
                shrink: 1.11,
            },
            BoxNode::Text {
                text: "world".to_string(),
                width: m.string_width("world"),
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
}
