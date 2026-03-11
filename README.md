# RustLaTex

A Rust-based LaTeX compiler targeting semantic equivalence with — and eventually binary-identical output to — standard LaTeX compilers (pdflatex/lualatex).

[![CI](https://github.com/WenhanLyu/RustLaTex/actions/workflows/ci.yml/badge.svg)](https://github.com/WenhanLyu/RustLaTex/actions/workflows/ci.yml)

---

## Project Goal

RustLaTex aims to implement a complete LaTeX-to-PDF compiler written entirely in Rust. The pipeline follows TeX's classic architecture:

```
.tex source
    │
    ▼
┌─────────────────────┐
│  rustlatex-lexer    │  tokenize source → Token stream
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│  rustlatex-parser   │  Token stream → AST
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│  rustlatex-engine   │  AST → laid-out pages (box/glue model)
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│  rustlatex-pdf      │  Pages → PDF bytes
└─────────────────────┘
    │
    ▼
output.pdf
```

---

## Architecture Overview

The project is a Cargo workspace with five crates:

| Crate | Type | Description |
|-------|------|-------------|
| `rustlatex-lexer` | library | LaTeX tokenizer — converts raw source into a stream of tokens using TeX's category code mechanism |
| `rustlatex-parser` | library | AST parser — turns the token stream into a hierarchical Abstract Syntax Tree representing document structure |
| `rustlatex-engine` | library | Typesetting engine — applies TeX's box/glue model, Knuth-Plass line breaking, and page breaking to produce laid-out pages |
| `rustlatex-pdf` | library | PDF backend — emits PDF output from laid-out pages, targeting binary-identical output to pdflatex |
| `rustlatex-cli` | binary | Command-line tool — glues all crates together; accepts a `.tex` file argument and drives the pipeline |

---

## How to Build

```bash
# Build all crates
cargo build --all

# Build in release mode
cargo build --all --release
```

---

## How to Run

```bash
# Run on a .tex file
rustlatex input.tex

# Or via cargo
cargo run --bin rustlatex -- input.tex
```

Example output:
```
Compiling input.tex...

--- Source (42 bytes) ---
\documentclass{article}
\begin{document}
Hello, world!
\end{document}
--- End of source ---

[1/3] Tokenizing...
      12 token(s) produced.
[2/3] Parsing...
      AST root: Document([...])
[3/3] Typesetting and generating PDF (stub)...
      1 page(s) typeset.

Done. (PDF output is currently a stub — future milestones will write real PDF files.)
```

---

## How to Test

```bash
cargo test --all
```

---

## Crate Descriptions

### `rustlatex-lexer`

Implements TeX's tokenization rules, including:
- Category codes (catcodes) for all 16 TeX categories
- Control sequences (`\commandname`, `\ `, single-char sequences)
- Comment stripping (`%` to end of line)
- Space handling after control words

### `rustlatex-parser`

Parses the token stream into a tree of `Node` values:
- `Document` — top-level sequence of nodes
- `Command` — a control sequence with optional argument groups
- `Group` — a `{...}` group
- `Text` — plain text runs
- `InlineMath` — `$...$` math mode

### `rustlatex-engine`

The typesetting engine (stub for now, to be expanded in later milestones):
- Will implement TeX's box/glue model (hboxes, vboxes, glue, penalties)
- Knuth-Plass line-breaking algorithm
- Page breaking
- Macro expansion

### `rustlatex-pdf`

The PDF backend (stub for now):
- Will emit PDF 1.5 output
- Font embedding (Type1 / TrueType / OpenType)
- Cross-references, hyperlinks
- Targeting binary-identical output to pdflatex for a defined test corpus

### `rustlatex-cli`

The `rustlatex` binary:
- Accepts a single `.tex` file argument
- Runs the full pipeline and reports progress
- Exits 0 on success, non-zero on error

---

## Roadmap

See [roadmap.md](roadmap.md) for the full milestone plan.

---

## License

MIT (placeholder — to be decided)
