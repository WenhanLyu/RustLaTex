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

_(Updated each cycle)_

## Milestones

### M1: Project Foundation & Rust Workspace Setup ✅ PLANNED
Set up a well-structured Rust workspace with CI, basic project scaffolding, and clear crate organization. This lays the foundation for all subsequent development.

- **Deliverables:**
  - `Cargo.toml` workspace with crates: `rustlatex-lexer`, `rustlatex-parser`, `rustlatex-engine`, `rustlatex-pdf`, `rustlatex-cli`
  - CI pipeline (GitHub Actions) running `cargo build` and `cargo test`
  - Basic CLI binary that accepts a `.tex` file argument
  - README documenting the architecture
- **Cycles budget:** 3
- **Status:** Pending

### M2: LaTeX Lexer (Tokenizer)
Implement a complete LaTeX tokenizer that handles:
- Control sequences (`\commandname`, `\ `, `\@`)
- Active characters
- Grouping (`{`, `}`)
- Math mode (`$`, `$$`, `\(`, `\[`)
- Comments (`%`)
- Parameter tokens (`#1`-`#9`)
- Category codes (catcodes) — TeX's core tokenization mechanism

Output: stream of TeX tokens matching TeX's Category Code rules.

- **Cycles budget:** 4
- **Status:** Pending

### M3: LaTeX Parser & Basic Document Structure
Parse tokenized input into an AST representing:
- Document structure: `\documentclass`, `\begin{document}`, `\end{document}`
- Common environments: `itemize`, `enumerate`, `verbatim`, `figure`, `table`
- Sections: `\section`, `\subsection`, etc.
- Basic text formatting: `\textbf`, `\textit`, `\emph`
- `\usepackage` declarations

- **Cycles budget:** 5
- **Status:** Pending

### M4: Macro Expansion Engine
Implement TeX's macro expansion:
- `\def`, `\newcommand`, `\renewcommand`
- `\let`, `\futurelet`
- Conditional expansion: `\if`, `\ifx`, `\ifnum`, `\else`, `\fi`
- `\input`, `\include` file inclusion

- **Cycles budget:** 5
- **Status:** Pending

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
