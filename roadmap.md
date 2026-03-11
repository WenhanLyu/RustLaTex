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
- **Strategy:** "Binary identical" is extremely ambitious. The right approach is: get basic output working first (M2-M5), then progressively harden toward binary identity (M6-M9).
- **Worker sizing:** Single-task assignments per worker work well. Keep milestones tight and verifiable. Leo (high model) can deliver large focused tasks in a single cycle.

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

### M4: Macro Expansion Engine — IN PROGRESS
Implement TeX macro expansion in `rustlatex-parser`:
- `\def`, `\newcommand`, `\renewcommand`
- `\let` alias creation
- Conditional expansion: `\if`, `\ifx`, `\ifnum`, `\else`, `\fi`
- MacroTable with parameter substitution (#1..#9)
- Integration with existing Parser: expand macros before/during AST construction
- 15+ new tests covering all features

- **Cycles budget:** 5
- **Status:** 🔄 In Progress (M4 starting now)

### M5: Math Mode Support
Implement math typesetting:
- Inline and display math parsing
- Basic math commands: `\frac`, `\sqrt`, `\sum`, `\int`
- Subscripts and superscripts
- Math font handling

- **Cycles budget:** 5
- **Status:** Pending

### M6: TeX Box/Glue Typesetting Engine
Implement TeX's typesetting model:
- Hboxes and vboxes
- Glue (flexible spacing)
- Penalties
- Line breaking (Knuth-Plass algorithm)
- Page breaking

This is the core algorithmic challenge. Requires careful study of TeX: The Program.

- **Cycles budget:** 10
- **Status:** Pending

### M7: Font Handling & Metrics
- Load and interpret TFM (TeX Font Metric) files
- Handle font families, sizes, encodings
- Kern pairs and ligatures

- **Cycles budget:** 4
- **Status:** Pending

### M8: PDF Backend
- Generate PDF 1.5 output
- Embed fonts (Type1/TrueType/OpenType)
- Handle cross-references, hyperlinks
- Match pdflatex's PDF structure for binary-identical output

- **Cycles budget:** 6
- **Status:** Pending

### M9: Integration & Binary-Identity Testing
- End-to-end test with real `.tex` documents
- Compare output byte-by-byte with pdflatex
- Fix all differences — font embedding, metadata, timestamps
- Achieve binary-identical output for a defined test corpus

- **Cycles budget:** 8
- **Status:** Pending

---

## Notes on "Binary Identical" Goal

True binary-identical output is extremely difficult because it depends on:
1. **Timestamps** — PDF metadata timestamps differ unless suppressed
2. **Random seeds** — some compilers use randomness
3. **Font subsetting** — the same subset algorithm must be used
4. **PDF object ordering** — exact same internal structure

In practice, we will target **semantic equivalence** first (same visual output), then work toward binary identity by matching pdflatex's specific behavior for timestamps, object ordering, and font embedding. The test corpus will be simple documents initially.
