# RustLaTex Roadmap

## Project Goal

Build a Rust-based LaTeX compiler that generates binary-identical PDF output compared to reference LaTeX compilers (pdflatex/lualatex).

## Architecture Overview

A LaTeX compiler pipeline:
1. **Lexer/Tokenizer** — tokenize LaTeX source into tokens (commands, text, math, etc.)
2. **Parser** — parse tokens into an AST (document structure, environments, commands)
3. **Semantic Analysis** — resolve macros, expand commands, process environments
4. **Typesetting Engine** — lay out text, math, figures using TeX's box/glue model
5. **PDF Backend** — emit PDF output conforming to PDF spec (matching pdflatex output)

Binary-identical output requires:
- Exact same font metrics (use same TFM/OTF fonts)
- Exact same TeX typesetting algorithms (line breaking, page breaking)
- Exact same PDF structure/encoding

## Lessons Learned

- **Cycle 1-4 (M1):** M1 completed in 3 cycles (under budget). The workspace setup was well-understood, Ares's team executed efficiently. Leo was hired mid-milestone to fix CI/fmt issues.
- **Cycle 5-6 (M2):** M2 completed in 2 cycles (under budget of 4). Leo delivered the full lexer implementation with all 16 catcodes, mutable catcode table, 28 tests, parameter tokens, active chars, comment handling, and Par handling. Apollo verified all checks pass (34 tests total).
- **Cycle 7-12 (M3):** M3 completed in 2 implementation cycles + 1 verification. Leo delivered the full parser upgrade: Environment, Paragraph, DisplayMath nodes, argument/optional arg parsing, 22 tests total. Apollo verified all 52 tests pass, CI clean.
- **Cycle 13-19 (M4):** M4 completed in 2 implementation cycles + 1 verification (with 1 fix round). Leo delivered MacroTable, \def, \newcommand, \renewcommand, \let, \if/\ifx/\ifnum conditionals, 21 new tests. Apollo verified all 73 tests pass, CI clean.
- **Cycle 20-22 (M5):** M5 completed in 1 implementation cycle + 1 verification. Ares implemented math AST nodes directly (Superscript, Subscript, Fraction, Radical, MathGroup). Apollo verified 90 total tests pass, CI clean.
- **Cycle 23-25 (M6):** M6 completed in 1 implementation cycle + 1 verification. Leo implemented BoxNode enum (6 variants), AST→BoxList translator, greedy line breaking, and updated Engine::typeset(). Apollo verified 117 tests pass, CI clean.
- **Cycle 26-28 (M7):** M7 completed in 1 implementation cycle + 1 verification. Leo implemented FontMetrics trait, StandardFontMetrics struct (CM Roman 10pt), translate_node_with_metrics(), Engine uses StandardFontMetrics by default. Apollo verified 131 tests pass, CI clean.
- **Cycle 29-33 (M8):** M8 completed in 1 implementation cycle + 1 verification. Leo implemented real PDF backend: pdf-writer 0.9, A4 page layout, Base-14 Helvetica font, BoxNode rendering to PDF content streams, CLI writes .pdf file. Apollo verified 138 tests pass, CI clean.
- **Cycle 34-38 (M9):** M9 completed in 1 implementation cycle + 1 verification. Ares implemented Knuth-Plass DP line-breaking: LineBreaker trait, GreedyLineBreaker, KnuthPlassLineBreaker (O(n²) DP, badness/demerits, tolerance=200), 19 new tests. Apollo verified 157 tests pass, CI clean.
- **Cycle 39-41 (M10):** M10 completed in 1 implementation cycle + 1 verification. Ares implemented integration tests (20 tests, 4 .tex corpus files), Helvetica metric alignment, CLI error handling. Apollo verified 182 tests pass, CI clean.
- **Cycle 42-45 (M11):** M11 completed in 1 implementation cycle + 1 verification. Ares embedded cmr10.pfb (Type1 font), updated StandardFontMetrics to CM Roman AFM widths, added Type1 font dict+descriptor+file to PDF. Apollo verified 196 tests pass, CI clean.
- **Cycle 46-49 (M12):** M12 completed in 1 implementation cycle (verification pending). Leo delivered font_size field on BoxNode::Text, section/subsection at 14/12/11pt with kerns, paragraph spacing, multi-page layout, \LaTeX/\TeX/\today expansion, forced breaks. 20 new engine tests, 216 total tests pass.
- **Strategy:** "Binary identical" is extremely ambitious. The right approach is: get basic output working first (M2-M5), then progressively harden toward binary identity (M6-M9). M10 focuses on integration quality and font consistency before binary-identity work. M11 embeds real CM Type1 fonts. M12 targets document structure rendering (sections, spacing, multi-page layout).
- **Worker sizing:** Single-task assignments per worker work well. Keep milestones tight and verifiable. Leo (high model) can deliver large focused tasks in a single cycle.
- **M6 approach:** Box/glue engine is complex — break it into: M6 (box/glue data model + AST→boxes translator), M7 (font metrics + TFM), M8 (PDF backend), M9 (Knuth-Plass + integration). This ensures steady progress without overloading a single milestone.
- **Font resources available:** cmr10.afm at `/Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages/matplotlib/mpl-data/fonts/afm/cmr10.afm` and cmr10.pfb at `/System/Volumes/Data/Users/wenhanlyu/.local/lib/python2.7/site-packages/matplotlib/tests/cmr10.pfb` — both available for M11 font embedding.
- **pdflatex not installed:** M12 binary-identity testing requires installing pdflatex. Consider Homebrew install or alternative before starting M12.

## Milestones

### M1: Project Foundation & Rust Workspace Setup ✅ COMPLETE
Set up a well-structured Rust workspace with CI, basic project scaffolding, and clear crate organization.

- **Deliverables:** 5-crate workspace, CI (GitHub Actions), CLI binary, README
- **Cycles budget:** 3 | **Cycles actual:** 3
- **Status:** ✅ Complete — verified by Apollo (cycle 4)

### M2: LaTeX Lexer (Tokenizer) ✅ COMPLETE
Implement a complete, production-quality LaTeX tokenizer in `rustlatex-lexer`.

- **Deliverables:** CatcodeTable (256-entry), all 16 catcodes, mutable table, parameter tokens, active chars, Par/Space tokens, comment handling, 28 unit tests
- **Cycles budget:** 4 | **Cycles actual:** 2
- **Status:** ✅ Complete — verified by Apollo (commit 05518e3)

### M3: LaTeX Parser & Basic Document Structure ✅ COMPLETE
Parse tokenized input into an AST representing:
- Document structure: `\documentclass`, `\begin{document}`, `\end{document}`
- Common environments: `itemize`, `enumerate`, `verbatim`, `figure`, `table`
- Sections: `\section`, `\subsection`, etc.
- Basic text formatting: `\textbf`, `\textit`, `\emph`
- `\usepackage` declarations
- Argument parsing: `\cmd{arg}` with mandatory `{}` args and optional `[opt]` args

- **Cycles budget:** 5 | **Cycles actual:** 2
- **Status:** ✅ Complete — verified by Apollo (commit b03889f, 52 tests)

### M4: Macro Expansion Engine ✅ COMPLETE
Implement TeX macro expansion in `rustlatex-parser`:
- `\def`, `\newcommand`, `\renewcommand`
- `\let` alias creation
- Conditional expansion: `\if`, `\ifx`, `\ifnum`, `\else`, `\fi`
- MacroTable with parameter substitution (#1..#9)
- Integration with existing Parser: expand macros before/during AST construction
- 21 new tests covering all features

- **Cycles budget:** 5 | **Cycles actual:** 3
- **Status:** ✅ Complete — verified by Apollo (commit 8da83d2, 73 tests total)

### M5: Math Mode AST Enhancement ✅ COMPLETE
Enhance the math mode parser in `rustlatex-parser` to produce structured AST nodes instead of raw text:
- `Superscript`, `Subscript`, `Fraction`, `Radical`, `MathGroup` nodes
- 17 new math tests, all existing 73 tests continue to pass

- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (90 tests total)

### M6: Box/Glue Data Model & AST→BoxList Translator ✅ COMPLETE
Implement the typesetting IR (intermediate representation) in `rustlatex-engine`:

**Box/Glue data model:**
- `BoxNode` enum: `HBox`, `VBox`, `Text`, `Glue`, `Kern`, `Penalty`, `Rule` variants
- `Glue` struct: `{ natural: f64, stretch: f64, shrink: f64 }` (scaled points or float)
- `Dimension` type (scaled points as i64, or f64 for initial implementation)
- `HBox { width, height, depth, content: Vec<BoxNode> }`
- `VBox { width, height, content: Vec<BoxNode> }`

**AST→BoxList translator:**
- Traverse AST `Node` tree and produce a `Vec<BoxNode>` (the "horizontal list")
- Handle: `Text` → sequence of character `BoxNode::Text` items + inter-word glue
- Handle: `Command` for font/formatting commands (`\textbf`, `\textit`) — stub, no real font change
- Handle: `Paragraph(nodes)` → horizontal list of items followed by paragraph glue
- Handle: `Environment` → vertical list of boxed paragraphs
- Handle: `InlineMath` / `DisplayMath` → placeholder `BoxNode::Text("(math)")` (full math layout is later)

**Naive line breaking (greedy):**
- Implement a greedy line-breaking algorithm (first-fit, no Knuth-Plass yet)
- Break horizontal lists at glue points to produce lines of a given `\hsize` (hardcoded 345pt for A4)
- Stack lines into pages using a fixed `\vsize` (hardcoded 550pt for A4)

**Output:**
- `Engine::typeset()` returns `Vec<Page>` where each `Page` has a `Vec<Vec<BoxNode>>` (lines) — replace the current placeholder `String`

**Tests (15+):**
- Test `BoxNode` construction and basic properties
- Test AST→BoxList for simple text paragraph
- Test naive line breaking with known text width
- Test multi-paragraph documents produce multiple paragraph groups
- All existing 90 tests continue to pass

- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 84806c3, 117 tests)

### M7: Font Handling & Real Character Widths ✅ COMPLETE
Implement font metrics support so the typesetting engine uses accurate character widths instead of the 6pt-per-character approximation.

- **Deliverables:** `FontMetrics` trait, `StandardFontMetrics` (CM Roman 10pt hardcoded), `translate_node_with_metrics()`, Engine uses real metrics, 14 new tests
- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit a283d5c, 131 tests total)

### M8: PDF Backend (Real Output) ✅ COMPLETE
Generate real, viewable PDF output using the `pdf-writer` crate (Rust).

- **Deliverables:** Real PDF 1.7 output, A4 pages, Base-14 Helvetica, BoxNode→PDF rendering, CLI writes .pdf file, 8 PDF tests
- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit faecd86, 138 tests total)

### M9: Knuth-Plass Line Breaking ✅ COMPLETE
Replace the greedy `break_into_lines()` with the Knuth-Plass optimal line-breaking algorithm.

- **Deliverables:** `LineBreaker` trait, `GreedyLineBreaker`, `KnuthPlassLineBreaker` (DP, badness/demerits, tolerance=200, forced/prohibited breaks), 19 new tests, Engine uses KP by default
- **Cycles budget:** 6 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (157 tests total)

### M10: End-to-End Integration Tests + Font/Rendering Consistency ✅ COMPLETE
Validate the full pipeline with real `.tex` documents and fix the font/metrics consistency gap.

- **Deliverables:** 20 integration tests, 4 .tex corpus files, Helvetica metrics alignment, CLI error handling, 5 CLI tests
- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 1a2254d, 182 tests total)

### M11: Real TeX Font Embedding (Type1 / Computer Modern) ✅ COMPLETE
Embed actual Computer Modern Roman Type1 font (cmr10) in the PDF output, using real AFM metrics.

- **Deliverables:** cmr10.pfb embedded, CM Roman AFM metrics in engine, Type1 font dict+descriptor+file in PDF, 14 new tests
- **Cycles budget:** 6 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 93a8af4, 196 tests total)

### M12: Document Structure Rendering (Sections, Multi-page, Paragraph Spacing) ✅ COMPLETE
Make the PDF output visually resemble a real LaTeX-compiled document by implementing proper rendering of document structure.

- **Deliverables:** font_size field on BoxNode::Text, section/subsection/subsubsection at 14/12/11pt, paragraph spacing (6pt glue), multi-page layout (vsize=700pt), \LaTeX/\TeX/\today expansion, \\/\newline forced breaks, 20 new engine tests
- **Cycles budget:** 6 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Leo (commit 2b2e00e, 216 tests total)

### M13: Basic Math Rendering (Inline Math Text Rendering)
Replace the `(math)` placeholder with actual rendered text representations of inline and display math expressions by walking the structured math AST.

**Scope in `rustlatex-engine`:**
- Walk `Node::InlineMath(nodes)` and `Node::DisplayMath(nodes)` to produce readable text
- Handle `Node::Superscript { base, exponent }` → render as "base^exponent" text (e.g., `x^2` → "x²" or "x^2")
- Handle `Node::Subscript { base, subscript }` → render as "base_subscript" text
- Handle `Node::Fraction { numerator, denominator }` → render as "numerator/denominator" text
- Handle `Node::Radical { radicand, .. }` → render as "√radicand" text
- Handle `Node::MathGroup(nodes)` → render contained nodes
- Handle Greek letter commands in math: `\alpha` → "α", `\beta` → "β", `\gamma` → "γ", `\delta` → "δ", `\pi` → "π", `\theta` → "θ", `\lambda` → "λ", `\mu` → "μ", `\sigma` → "σ", `\omega` → "ω"
- Handle math operators in math: `\cdot` → "·", `\times` → "×", `\div` → "÷", `\pm` → "±", `\leq` → "≤", `\geq` → "≥", `\neq` → "≠", `\infty` → "∞"
- Inline math renders inline (surrounded by space glue)
- Display math renders on its own line with extra vertical space

**Tests (15+):**
- Test `$x^2$` renders as text containing "x" and "2" (no "(math)")
- Test `$\alpha + \beta$` renders as text containing "α" and "β"
- Test `$\frac{a}{b}$` renders as text containing "a/b" form
- Test `$\sqrt{x}$` renders as text containing "√"
- Test display math `\[ E = mc^2 \]` renders as structured text (not "(math)")
- All 216 existing tests continue to pass

- **Cycles budget:** 6
- **Status:** Pending

### M14: Integration & Visual Quality Testing
- Install pdflatex via MacTeX/BasicTeX (`brew install --cask basictex`)
- End-to-end visual comparison: render both PDFs to images (with `gs` or `convert`) and compute pixel diff
- Fix layout differences until output is visually close

- **Cycles budget:** 10
- **Status:** Pending

---

## Notes on "Binary Identical" Goal

True binary-identical output is extremely difficult because it depends on:
1. **Timestamps** — PDF metadata timestamps differ unless suppressed
2. **Random seeds** — some compilers use randomness
3. **Font subsetting** — the same subset algorithm must be used
4. **PDF object ordering** — exact same internal structure

In practice, we will target **semantic equivalence** first (same visual output), then work toward binary identity by matching pdflatex's specific behavior for timestamps, object ordering, and font embedding. The test corpus will be simple documents initially.
