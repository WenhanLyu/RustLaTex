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
- **Cycle 46-49 (M12):** M12 completed in 1 implementation cycle + 1 verification. Leo delivered font_size field on BoxNode::Text, section/subsection at 14/12/11pt with kerns, paragraph spacing, multi-page layout, \LaTeX/\TeX/\today expansion, forced breaks. 20 new engine tests, 216 total tests pass.
- **Cycle 50-51 (M13):** M13 completed in 1 implementation cycle + 1 verification. Ares delivered math_node_to_text(), Greek letters/math operators, inline/display math rendering (no more "(math)" placeholder). 15 new tests, 231 total tests pass.
- **Strategy:** "Binary identical" is extremely ambitious. The right approach is: get basic output working first (M2-M5), then progressively harden toward binary identity (M6-M9). M10 focuses on integration quality and font consistency before binary-identity work. M11 embeds real CM Type1 fonts. M12 targets document structure rendering (sections, spacing, multi-page layout).
- **Worker sizing:** Single-task assignments per worker work well. Keep milestones tight and verifiable. Leo (high model) can deliver large focused tasks in a single cycle.
- **M6 approach:** Box/glue engine is complex — break it into: M6 (box/glue data model + AST→boxes translator), M7 (font metrics + TFM), M8 (PDF backend), M9 (Knuth-Plass + integration). This ensures steady progress without overloading a single milestone.
- **Font resources available:** cmr10.afm at `/Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages/matplotlib/mpl-data/fonts/afm/cmr10.afm` and cmr10.pfb at `/System/Volumes/Data/Users/wenhanlyu/.local/lib/python2.7/site-packages/matplotlib/tests/cmr10.pfb` — both available for M11 font embedding.
- **pdflatex not installed:** M12 binary-identity testing requires installing pdflatex. Consider Homebrew install or alternative before starting M12.
- **Cycle 52-54 (M14):** M14 completed in 1 implementation cycle. Leo delivered itemize/enumerate rendering with bullet/number prefixes, 20pt indentation, inter-item glue, 17 new tests. CI green, 248 total tests pass.
- **GhostScript available:** `gs` at /opt/homebrew/bin/gs (v10.06.0) — can render PDFs to PNG for visual validation. Use this for M15 integration tests.
- **Cycle 55-57 (M15):** M15 completed in 1 implementation cycle. Ares delivered 16 GhostScript integration tests (5 example file tests + 11 inline tests), CI installs ghostscript. Apollo verified all 264 tests pass, CI green.
- **M16 scope:** Focus on text alignment in PDF backend (justify/center/raggedright). Justification requires computing inter-word spacing adjustments per line. The KP line breaker already computes break points — the PDF renderer needs to use the adjustment ratios. Add \centering, \raggedright, \raggedleft command support in the engine. Keep hyphenation simple (pattern-based prefix suffix, Aho-Corasick not needed).
- **Cycle 58-62 (M16):** M16 completed in 1 implementation cycle + 1 verification. Leo delivered Alignment enum (Justify/Center/RaggedRight/RaggedLeft), AlignmentMarker BoxNode variant, OutputLine struct, \centering/\raggedright/\raggedleft/center-environment support, PDF per-line x-offset computation. 19 new tests, 283 total tests pass, CI green.
- **M17 scope:** Tables (tabular environment). tabular column spec parsing (l/r/c/|), \hline rules, cell content rendering, & column separator, \multicolumn support. Engine must produce table box layout. PDF backend must render table cells with proper alignment and column widths.
- **Cycle 58-62 (M17):** M17 completed in 1 implementation cycle. Leo delivered tabular environment: column spec parsing (l/r/c ignoring |), row splitting at \\, cell splitting at &, \hline as BoxNode::Rule, PDF backend renders Rule as filled rect. 17 new tests, 300 total tests pass, CI green.
- **M18 scope:** Figures & Cross-References. \begin{figure} environment with placeholder rendering, \label/\ref/\pageref system (resolve references in two passes), automatic figure/section/table numbering, \caption rendering. Keep implementation practical: collect labels in first pass, substitute refs in second pass.
- **Cycle 62-64 (M18):** M18 completed in 1 implementation cycle + 1 verification (with 1 fix round for forward-ref test). Leo delivered: two-pass label/ref system, \label/\ref/\pageref, figure environment with caption numbering, section numbering, 20 new tests. Apollo verified 320 total tests pass, CI green.
- **M19 scope:** CLI improvements + verbatim environment + more text commands. Fix CLI output path (currently ignores second arg), add \begin{verbatim} environment (monospace, no command parsing), add \texttt{} command (inline code), add \underline{}, fix UTF-8/special chars in PDF output. These are high-value user-visible improvements.
- **pdflatex not available locally:** Comparison testing deferred. GhostScript available for smoke tests.
- **Cycle 62-67 (M19):** M19 completed in 1 implementation cycle + 1 verification. Leo delivered CLI output path fix, verbatim environment, \texttt/\underline/\textsc/\mbox/\noindent commands. 18 new tests, 338 total tests pass.
- **M20 scope:** Focus on core TeX behaviors: paragraph indentation (20pt first-line indent, suppressed after section headings), page break commands (\newpage/\clearpage/\pagebreak), \vspace, and inter-sentence spacing (wider glue after sentence-ending punctuation). These are visible in every real LaTeX document.
- **Cycle 60-62 (M20):** M20 completed in 1 implementation cycle + 1 verification. Leo delivered paragraph indentation (20pt Kern, suppressed after headings and via \noindent), \newpage/\clearpage/\pagebreak (Penalty{-10001}), \vspace/\vspace* dimension parsing, inter-sentence spacing (1.5x glue, abbreviation exception). 22 new tests, 360 total tests pass.
- **M21 scope:** Title/author/date (\title, \author, \date, \maketitle) and page numbers in PDF footer. These are present in virtually every real LaTeX document. \maketitle emits a centered title block; PDF backend renders page numbers in footer.
- **Cycle ~132 (M31):** M31 completed in 1 implementation cycle. Ares delivered: eprintln! visibility for pixel similarity test, section heading spacing fixes (before section: 24pt, after: 8pt; before subsection: 18pt, after: 6pt), 21 new tests. 579 total tests pass, CI green.
- **M32 scope:** Embed CM Bold (cmbx10), CM Italic (cmti10), CM Bold Italic (cmbxti10), and CM Typewriter (cmtt10) fonts in PDF output. Replace Helvetica variants with actual CM fonts. Update engine width metrics to use AFM data (cmtt10 is 5.25pt monospace, not 6.0pt). This significantly improves visual output quality.
- **Cycle 86-88 (M21):** M21 completed in 1 implementation cycle. Leo delivered \title/\author/\date/\maketitle system + PDF page number footer. TranslationContext extended with title/author/date fields. \maketitle emits centered title block at 17pt/12pt. PDF footer renders page numbers. 17 new tests, 377 total tests pass, CI green.
- **M22 scope:** Footnotes (\footnote), abstract environment, horizontal spacing (\hspace, \hfill, \vfill), and URL/hyperlink commands (\href, \url). These are present in the majority of real academic LaTeX documents and are completely missing from the current implementation.
- **Cycle 89-94 (M22):** M22 completed in 1 implementation cycle + 1 fix round + 1 verification. Leo delivered \footnote superscript markers + page-bottom rendering, \begin{abstract} centered heading, \hspace/\hfill/\vfill/\quad/\qquad/\,/\; spacing, \url/\href URL commands. Ares fixed abstract 6pt glue + 30pt kern indentation + PDF footnote rule width. 25 new tests, 402 total tests pass, CI green.
- **M23 scope:** Color support (\textcolor, \color, \colorbox, xcolor named colors) and image inclusion (\includegraphics with PNG XObject embedding). Color requires adding a color field to BoxNode::Text and DeviceRGB PDF operators. Image inclusion requires BoxNode::ImagePlaceholder and PNG XObject embedding.
- **Cycle 95-98 (M23):** M23 completed in 1 implementation cycle + 1 verification. Leo delivered Color struct, 16 named colors, \textcolor/\color/\colorbox, BoxNode::ImagePlaceholder, \includegraphics with width/height/scale parsing, PDF rg operators for colored text, grey rectangle for image placeholders. 24 new tests, 426 total tests pass, CI green.
- **M24 scope:** Equation environments (equation, align, align*) + theorem-like environments (\newtheorem, theorem, lemma, proof) + Table of Contents (\tableofcontents) + description list environment. These are the core missing academic document features present in virtually all research papers.
- **Cycle 99-101 (M24):** M24 completed in 1 implementation cycle + 1 verification. Leo delivered equation/equation*/align/align* with auto-numbering, \newtheorem + pre-registered theorem/lemma/definition/corollary/proposition/remark/example, proof environment with QED □, \tableofcontents two-pass rendering, description list \item[term]. 16 new tests, 442 total tests pass, CI green.
- **M25 scope:** Bibliography system (\cite/\bibitem/thebibliography), \newenvironment (custom environment definitions), and \input file inclusion. These complete the core academic LaTeX feature set — virtually every research paper uses citations and custom environments.
- **Cycle 102-104 (M25):** M25 completed in 1 implementation cycle + 1 verification. Leo delivered bibliography system (\bibitem/\cite two-pass), \newenvironment/\renewenvironment, \input/\include file inclusion with working_dir. 17 new tests, 459 total tests pass, CI green.
- **M26 scope:** TeX hyphenation (pattern-based English hyphenation to improve line breaking quality) + LaTeX counter system (\setcounter, \addtocounter, \value, \arabic, \roman, \alph). These significantly improve document quality and enable richer document customization.
- **Cycle 109-111 (M26):** M26 completed in 1 implementation cycle. Leo delivered Hyphenator with Liang's algorithm, ~50 English patterns, \hyphenation exception command, \- soft hyphen, full counter system (\setcounter/\addtocounter/\newcounter/\stepcounter/\arabic/\roman/\Roman/\alph/\Alph/\fnsymbol). 33 new tests, 492 total tests pass, CI green.
- **M27 scope:** Font style support (bold, italic, bold-italic, typewriter) in PDF output. Currently \textbf/\textit/\emph/\texttt all render identically — they don't change font face in the PDF. For the project goal of "binary identical" output, font styles must produce visually correct PDF output using appropriate font resources.
- **Cycle 113-120 (M27):** M27 completed in 1 implementation cycle + 1 fix round + 1 verification. Leo delivered FontStyle enum (Normal/Bold/Italic/BoldItalic/Typewriter), font_style field on BoxNode::Text, TranslationContext font style tracking + group scoping, \bfseries/\itshape/\ttfamily/\normalfont declarations, 5 Base-14 PDF fonts (Helvetica variants + Courier). Ares added 2 missing tests. 24 new tests, 516 total tests pass, CI green.
- **M28 scope:** Per-font-style character width metrics. Currently all font styles (Bold, Italic, BoldItalic, Typewriter) use CM Roman width metrics for line-breaking and PDF positioning, even though the PDF backend selects different fonts. This causes incorrect text layout. M28 adds separate width tables for Helvetica-Bold, Helvetica-Oblique, Helvetica-BoldOblique, and Courier, and uses them consistently in both the engine (line-breaking) and PDF backend (character positioning).
- **Cycle 120-123 (M28):** M28 completed in 1 implementation cycle + 1 verification. char_width_for_style/space_width_for_style/string_width_for_style added to FontMetrics trait. Typewriter=6.0pt monospace, Bold/BoldItalic=1.05×, Italic=Normal. Engine translator uses per-style widths throughout. 20 new tests, 536 total tests pass, CI green.
- **M29 scope:** pdflatex comparison infrastructure. Install texlive-base + texlive-fonts-recommended in CI, add integration tests that compile simple .tex files with BOTH our compiler and pdflatex, render both to PNG via GhostScript, compute pixel similarity. Establishes the baseline measurement for progress toward "binary identical" goal. pdflatex not available locally — CI (ubuntu-latest) is the test environment.
- **Cycle ~130 (M29):** M29 completed in 1 implementation cycle. Leo delivered texlive CI install, examples/compare.tex, and 5 comparison tests (our PDF, pdflatex PDF, gs render ours, gs render pdflatex, pixel similarity log). 541 total tests pass, CI green.
- **Diana's M30 research:** Diana identified 5 critical rendering gaps vs pdflatex: (1) PDF hsize mismatch 495pt vs 345pt — CRITICAL BUG stretching every line; (2) Wrong page margins (50pt vs 72.27pt); (3) Section headings not bold; (4) \[...\] display math not recognized; (5) List items flow as paragraph text. Fixes #1+#2 estimated to push similarity from ~25% to ~60%.
- **Cycle ~131 (M30):** M30 completed in 1 implementation cycle. Leo fixed all 5 critical rendering gaps. 558 total tests pass, CI green.
- **Pixel similarity score now visible in CI:** M31 fixed eprintln! — score now appears in CI stderr logs.
- **Cycle ~133 (M32):** M32 completed in 1 implementation cycle + 1 verification. Leo replaced Helvetica/Courier Base-14 stubs with embedded cmbx10/cmti10/cmbxti10/cmtt10 Type1 fonts. Updated StandardFontMetrics: Bold uses cmbx10 per-char widths, Typewriter = 5.25pt monospace. Apollo verified 594 tests pass, CI green. No more Helvetica/Courier in PDF output.
- **M33 research (Diana):** OT1 encoding is the primary correctness bug — CM fonts use OT1 not StandardEncoding. `< > { } | \ "` all render blank/wrong. Bullet "•" is another visible bug. Superscript rendering (proper size+rise) is the biggest visual similarity gap. CI pixel comparison uses raw bytes (not decoded pixels) — fundamentally flawed measurement. Estimated visual similarity ~55-70% after M32.
- **Cycle (M33):** M33 completed in 1 implementation cycle. Leo delivered OT1 /Differences encoding array for all 5 CM fonts (cmr10/cmbx10/cmti10/cmbxti10/cmtt10), NON_SYMBOLIC→SYMBOLIC font descriptor flag change, bullet "•" replaced with "-", CI nocapture step added. 15 new tests, 609 total tests pass, CI green.
- **M33 scope:** OT1 encoding fix + bullet fix + CI visibility. Superscript rendering deferred to M34.
- **Cycle (M34):** M34 completed in 1 implementation cycle. Leo delivered vertical_offset field on BoxNode::Text, math_node_to_boxes() with proper superscript (font_size=7.0, vertical_offset=+4.0) and subscript (font_size=7.0, vertical_offset=-2.0) rendering, PDF Ts operator via set_rise(). 17 new tests, 626 total tests pass, CI green.
- **Cycle (M35):** M35 completed in 1 implementation cycle. Ares delivered math italic for single ASCII letter variables (FontStyle::Italic in math_node_to_boxes_inner), PPM pixel comparison (render_pdf_to_ppm + compare_ppm_files helpers), expanded compare.tex with subsection and display math. 22 new tests, 647 total tests pass, CI green.
- **M35 scope:** Math variables use Italic font style (matches cmmi10 in pdflatex). Fix pixel similarity comparison to use PPM raw pixel data (not PNG bytes) for accurate visual measurement. Expand compare.tex. Target 641+ tests.
- **Diana's M36 research:** Identified 4 practical improvements: (1) Bullet fix — itemize uses '-' but pdflatex uses cmsy10 bullet; implement as BoxNode::Bullet rendered as PDF filled circle (+0.5-1.0%); (2) OT1 ligature substitution fi/fl/ff/ffi/ffl in PDF backend (+0.01-0.05%); (3) Fix parindent 20pt→15pt (+0.1%); (4) Display math spacing: abovedisplayskip/belowdisplayskip=12pt (+0.2-0.4%). cmmi10/cmsy10 pfb files not available locally but available in CI texlive.
- **M36 scope:** Implement the 4 practical improvements from Diana's research. No new font embedding required. Target 662+ tests.
- **Cycle (M36):** M36 completed in 1 implementation cycle. Leo delivered BoxNode::Bullet (PDF filled circle), OT1 ligature substitution (fi/fl/ff/ffi/ffl), parindent 20pt→15pt, display math 12pt spacing. 25+ new tests, 672 total tests pass, CI green.
- **Cycle (M37):** M37 completed in 1 implementation cycle. Ares/Leo delivered FontStyle::MathItalic variant, cmmi10 AFM width metrics for line-breaking, display math horizontal centering. 18+ new tests, 690 total tests pass. Note: MathItalic still maps to cmti10 in PDF (cmmi10.pfb not yet embedded — not available locally).
- **M38 scope:** Embed cmmi10.pfb (CM Math Italic) and cmsy10.pfb (CM Math Symbols) in PDF output. Map FontStyle::MathItalic to F7 (cmmi10 instead of cmti10). Render BoxNode::Bullet using cmsy10 glyph 15 instead of Bezier circle. Shift page Ref IDs from 18+ to 24+ to accommodate 6 new fixed refs. Diana's research confirmed fonts available at /tmp/amsfonts_extract/ (SIL OFL license) — copy to crates/rustlatex-pdf/fonts/ and commit.
- **Diana's M38 research (issue #40, closed):** Confirmed cmmi10/cmsy10 downloadable from CTAN amsfonts (SIL OFL 1.1). cmmi10 Latin letters at same ASCII positions as OT1. cmsy10 bullet at position 15 (5pt advance, 3.87pt diameter). Expected +0.5-0.9% pixel similarity improvement. Low implementation risk.
- **Cycle (M38):** M38 completed in 1 implementation cycle (Ares). Embedded cmmi10.pfb (F7/MathItalic) + cmsy10.pfb (F8/bullet). Shifted page Refs from 18 to 24. Bullet now uses cmsy10 glyph 15 (5pt advance). 706 total tests pass, CI green (commit 315adba).
- **M39 scope:** Math operator spacing (thin/thick spaces around binary operators and relations in math mode) + fix CI pixel similarity visibility. Binary ops (+/-/×) get 1.667pt on each side; relations (=/</>/) get 2.778pt on each side. This is the most visible remaining gap in compare.tex.
- **M40 research (Diana):** Investigate character pair kerning (cmr10 AFM kern pairs), math mode rendering accuracy, word spacing exact values, page geometry validation, and actual pixel similarity score.

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
- **Status:** ✅ Complete — verified by Apollo (commit 0464a28, 231 tests total)

### M14: List Rendering (itemize/enumerate with bullets/numbers) ✅ COMPLETE
Implement proper visual rendering of LaTeX list environments in the engine.

- **Deliverables:** itemize/enumerate rendering with bullet/number prefixes, 20pt indentation, inter-item glue, list glue before/after; 17 new list tests
- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by CI (commit 02a7722, 248 tests total)

### M15: GhostScript Integration Tests + Visual Smoke Tests ✅ COMPLETE
- Use GhostScript (`gs`) to render our output PDFs to images
- Run our compiler on all example .tex files and verify they produce valid, non-empty PDFs
- Add integration test: compile each example, render with gs, verify PNG is non-empty
- Note: pdflatex comparison deferred (requires sudo install)

- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 773472f, 264 tests total)

### M16: Text Alignment & Justified Output ✅ COMPLETE
- Implement text alignment modes: justified (default), centered, ragged-right, ragged-left
- Add `Alignment` enum to engine: `Justify`, `Center`, `RaggedRight`, `RaggedLeft`
- Handle `\centering`, `\raggedright`, `\raggedleft` commands in the translator
- PDF backend: compute x-position offset per line based on alignment and actual line width
- For justified text: distribute remaining space proportionally across inter-word glue
- 19 new tests covering each alignment mode
- All 264 existing tests continue to pass
- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit ed1ece8, 283 tests total)

### M17: Tables (tabular environment) ✅ COMPLETE
- Implement `\begin{tabular}{lrc}` with column spec parsing
- Cell content rendering with alignment (l/r/c)
- Column separators (vertical rules `|`)
- Horizontal rules (`\hline`)
- 17 new tests covering column spec parsing, cell rendering, hline, multi-column documents
- All 283 existing tests continue to pass

- **Cycles budget:** 6 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by CI (commit f35e77e, 300 tests total)

### M18: Figures & Cross-References ✅ COMPLETE
Implement a practical label/reference system and figure environment:
- Figure environment: `\begin{figure}...\end{figure}` renders as a boxed region with caption
- `\caption{text}` inside figure: renders "Figure N: text" where N is auto-incremented
- `\label{key}` command: registers a label (figure number or section number) in a label table
- `\ref{key}` command: resolves to the associated number (e.g., "2" for figure 2)
- `\pageref{key}` command: resolves to the page number (e.g., "1")
- Section numbering: `\section` auto-increments a counter; `\label` after section captures that counter
- Two-pass rendering: first pass collects labels, second pass substitutes `\ref` values
- 20 new tests: figure rendering, caption numbering, \label/\ref resolution, section ref, forward references
- All 300 existing tests continue to pass
- **Cycles budget:** 6 | **Cycles actual:** 1 (+ 1 fix round)
- **Status:** ✅ Complete — verified by Apollo (commit 4d4c030, 320 tests total)

### M19: CLI Output Path + Verbatim Environment + Text Commands ✅ COMPLETE
Improve usability and completeness of the compiler:

**CLI fix (rustlatex-cli):**
- Fix output path: if a second argument is provided, use it as the output PDF path (currently ignored)
- Write the PDF to the user-specified path (or derive from input filename as fallback)

**Verbatim environment (rustlatex-parser + rustlatex-engine):**
- Parse `\begin{verbatim}...\end{verbatim}` as a special environment (no command interpretation inside)
- Engine: render verbatim content as monospaced text (use Courier or CM Typewriter font size 10pt)
- Each line of verbatim becomes a BoxNode::Text line, no line-breaking applied

**New text commands (rustlatex-engine):**
- `\texttt{text}` — inline monospace/typewriter text (render at same size, mark as monospace)
- `\underline{text}` — underlined text (render with a Rule beneath the text box)
- `\textsc{text}` — small caps (render as uppercase text for now)
- `\noindent` — suppress paragraph indentation (no-op is acceptable for now)
- `\mbox{text}` — unbreakable horizontal box

**Tests (15+ new):**
- Test CLI accepts two arguments and writes to specified path
- Test `\begin{verbatim}` produces output with the verbatim content
- Test `\texttt{code}` produces text output
- Test `\underline{text}` produces text + rule output
- All 320 existing tests continue to pass
- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 3c89fff, 338 tests total)

### M20: Paragraph Indentation + Page Break Commands + Inter-sentence Spacing
Implement core TeX typesetting behaviors that are present in every LaTeX document:

**Paragraph first-line indentation (rustlatex-engine):**
- Standard LaTeX indents the first line of each paragraph by `\parindent` = 20pt (1.5em at 10pt)
- First paragraph after a section heading is NOT indented (standard LaTeX behavior)
- `\noindent` already implemented (suppresses indent); now actually use it to trigger no-indent
- Add a `BoxNode::Kern(20.0)` at the start of each paragraph's box list (except post-heading paragraphs)
- Track "after_heading" state in TranslationContext to suppress indentation

**Page break commands (rustlatex-engine + rustlatex-parser):**
- `\newpage` — force a page break (emit a Penalty{value:-10001} or Page break marker)
- `\clearpage` — same as \newpage for now (flush and start new page)
- `\pagebreak` — same as \newpage for practical purposes
- `\vspace{len}` — vertical space insertion (emit Glue with specified natural size, parse pt/em/ex)
- `\vspace*{len}` — same as \vspace (star variant; no-op for the * for now)

**Inter-sentence spacing (rustlatex-engine):**
- TeX uses extra space after sentence-ending punctuation (`.`, `!`, `?`) followed by whitespace
- Implement: after a word ending in `.`, `!`, or `?`, if followed by space, emit wider inter-word glue (1.5x natural)
- Exception: do NOT apply extra space after abbreviations (a capital letter followed by `.`)
- This matches pdflatex's default behavior (before `\frenchspacing`)

**Tests (15+ new):**
- Test that paragraph has leading Kern(20.0) for indentation
- Test that first paragraph after `\section{}` has no leading Kern (no indent)
- Test `\noindent` suppresses the paragraph indent
- Test `\newpage` produces a page break (verify multi-page output from newpage)
- Test `\vspace{10pt}` emits a Glue node with natural=10.0
- Test inter-sentence spacing: "Hello. World" has wider glue than "hello world"
- All 338 existing tests continue to pass
- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by Apollo (commit 13fbcaa, 360 tests total)

### M21: Title Page (\maketitle) + Page Numbers in PDF Footer ✅ COMPLETE
Implement the LaTeX title block and page number rendering — features present in nearly every real LaTeX document.

- **Deliverables:** \title/\author/\date/\maketitle system, PDF page number footer (CMR10, centered, 30pt from bottom), 17 new tests
- **Cycles budget:** 4 | **Cycles actual:** 1
- **Status:** ✅ Complete — verified by CI (commit 51cf47d, 377 tests total)

### M22: Footnotes + Abstract + Horizontal Spacing + URLs ✅ COMPLETE
Implement common LaTeX features missing from the current implementation.

**Footnote system (rustlatex-engine + rustlatex-pdf):**
- `\footnote{text}` — render footnote text at the bottom of the current page; auto-numbered superscript in main text
- Footnote counter increments per-page; superscript marker "¹", "²", etc. appears in text
- At page bottom: horizontal rule + footnote text at 8pt, numbered to match superscript
- Simple implementation: collect footnotes per page during typesetting, render in PDF footer area above page numbers

**Abstract environment (rustlatex-engine):**
- `\begin{abstract}...\end{abstract}` — render centered heading "Abstract" followed by indented paragraph text
- Abstract heading at 12pt, text at 10pt, with 12pt vertical space before/after

**Horizontal spacing (rustlatex-engine):**
- `\hspace{len}` — insert horizontal glue of specified size (parse pt/em/ex dimensions)
- `\hspace*{len}` — same as \hspace for now
- `\hfill` — infinite horizontal stretch glue (pushes content to the right or fills line)
- `\vfill` — infinite vertical stretch glue (fills vertical space)
- `\quad` — 1em horizontal space (10pt at default size)
- `\qquad` — 2em horizontal space (20pt at default size)
- `\,` — thin space (3pt)
- `\;` — thick space (5pt)

**URL/hyperlink commands (rustlatex-engine):**
- `\url{http://...}` — render URL text in typewriter font (same as \texttt)
- `\href{url}{text}` — render text portion in typewriter font (URL is currently not clickable — PDF links are future work)
- `\textbf{text}` already works; ensure `\emph{text}` works as italic

**Tests (15+ new):**
- Test `\footnote{text}` produces a superscript marker in main text
- Test footnote content appears in engine output
- Test `\begin{abstract}` renders "Abstract" heading
- Test `\hspace{10pt}` produces a Kern(10.0) in output
- Test `\hfill` produces a Glue with large stretch value
- Test `\quad` produces a Kern(10.0)
- Test `\url{...}` renders URL text in typewriter font
- Test `\href{url}{text}` renders link text
- All 377 existing tests continue to pass

- **Cycles budget:** 4
- **Status:** ✅ Complete — verified by Apollo (commit 30849f6, 402 tests total)

### M23: Color Support + Image Inclusion
Implement color support and basic image inclusion — features present in virtually all modern LaTeX documents.

**Color system (rustlatex-engine + rustlatex-pdf):**
- Add `Color` type to engine: RGB values (r, g, b: f64 in 0.0..1.0) + named colors
- Add `color` field to `BoxNode::Text` (default: black = (0,0,0))
- Handle `\textcolor{color}{text}` — renders text in specified color
- Handle `\color{color}` — sets current color for subsequent text
- Handle `\colorbox{color}{text}` — background color box (filled Rect behind text)
- Support xcolor named colors: black, white, red, green, blue, cyan, magenta, yellow, gray, orange, purple, brown, lime, teal, violet, pink
- Support RGB specification: `\textcolor[rgb]{r,g,b}{text}`
- PDF backend: emit `rg` DeviceRGB operators for colored text

**Image inclusion (rustlatex-engine + rustlatex-pdf):**
- `\includegraphics[width=Xpt]{filename}` — placeholder box if file not found
- Add `BoxNode::ImagePlaceholder { filename: String, width: f64, height: f64 }` variant
- If PNG file exists: embed as XObject in PDF
- Parse optional key=value args: width, height, scale
- `\usepackage{graphicx}` and `\usepackage{xcolor}` — parse and ignore

**Tests (15+ new):**
- Test `\textcolor{red}{text}` produces Text node with color=(1,0,0)
- Test `\textcolor[rgb]{0.5,0.5,0.5}{text}` produces correct color
- Test `\colorbox{yellow}{text}` produces nodes with background
- Test `\includegraphics[width=100pt]{missing.png}` produces ImagePlaceholder(width=100)
- Test color names map to RGB values correctly
- All 402 existing tests continue to pass

- **Cycles budget:** 4
- **Status:** ✅ Complete — verified by Apollo (commit 372cdd3, 426 tests total)

### M24: Equation Environments + Theorem-Like Environments + Table of Contents ✅ COMPLETE
Implement key academic document features:

- **Deliverables:** equation/equation*/align/align* with auto-numbering, \newtheorem + pre-registered theorem types, proof env with QED □, \tableofcontents two-pass, description list \item[term], \label in equations, 16 new tests
- **Cycles budget:** 5 | **Cycles actual:** 1 (+ 1 verification)
- **Status:** ✅ Complete — verified by Vera (commit 6084dfe, 442 tests total)

### M25: Bibliography System + \newenvironment + \input File Inclusion
Implement the remaining core academic LaTeX features:

**Bibliography system (rustlatex-engine + rustlatex-parser):**
- `\begin{thebibliography}{99}...\end{thebibliography}` environment
- `\bibitem{key}` inside thebibliography — registers a citation with auto-number
- `\bibitem[label]{key}` — optional explicit label
- `\cite{key}` — renders as "[N]" where N is the bibitem number
- `\cite[note]{key}` — renders as "[N, note]"
- Multiple citations: `\cite{key1,key2}` → "[1, 2]"
- Two-pass: first pass collects \bibitem keys, second pass resolves \cite

**\newenvironment (rustlatex-engine + rustlatex-parser):**
- `\newenvironment{name}{begin-code}{end-code}` — defines a new environment
- `\renewenvironment{name}{begin-code}{end-code}` — redefines existing environment
- When `\begin{name}` is encountered: expand begin-code, then process content, then expand end-code
- Parameters (#1..#9) in begin-code take arguments from `\begin{name}[opt]{arg1}`
- This enables document-level abstractions like `\newenvironment{myquote}{\begin{quote}\itshape}{\end{quote}}`

**\input file inclusion (rustlatex-cli + rustlatex-parser):**
- `\input{filename}` — read and process the specified .tex file inline
- `\include{filename}` — same as \input for now (full LaTeX \include has clearpage behavior, simplified here)
- Parser reads the file from disk (relative to the main document) and inserts its tokens
- If file not found: emit a warning text node, continue parsing

**Tests (15+ new):**
- Test `\bibitem{key}` + `\cite{key}` resolves to "[1]"
- Test `\bibitem[A]{key}` + `\cite{key}` resolves to "[A]"
- Test multiple `\bibitem` entries auto-number correctly
- Test `\cite{key1,key2}` renders as "[1, 2]"
- Test `\begin{thebibliography}` renders "References" heading
- Test `\newenvironment{myenv}{prefix}{suffix}` + `\begin{myenv}content\end{myenv}` expands correctly
- Test `\renewenvironment` overwrites the prior definition
- Test `\input{file.tex}` includes file content (write a temp file during test)
- All 442 existing tests continue to pass

- **Cycles budget:** 5
- **Status:** ✅ Complete — verified by Vera (commit 76cc62a, 459 tests total)

### M26: TeX Hyphenation + LaTeX Counter System ✅ COMPLETE
Improve line-breaking quality and document customization:

**TeX Pattern-Based Hyphenation (rustlatex-engine):**
- Implement Liang's hyphenation algorithm with a minimal built-in English pattern set
- Add `Hyphenator` struct with pattern lookup to find allowed hyphenation points in words
- Integrate with KP line-breaker: insert discretionary Penalty+Kern nodes at hyphen points
- Hyphen penalty = 50 (discourage but allow), explicit hyphens penalty = 0
- `\hyphenation{word-list}` command to specify manual hyphenation for specific words
- `\- ` soft hyphen insertion (explicit discretionary break)

**LaTeX Counter System (rustlatex-engine):**
- `\setcounter{name}{N}` — set a named counter to integer N
- `\addtocounter{name}{N}` — add N to a named counter (N can be negative)
- `\value{name}` — expands to the integer value of the counter (for use in \setcounter)
- `\newcounter{name}` — declare a new counter (initialized to 0)
- `\stepcounter{name}` — increment counter by 1 (same as \addtocounter{name}{1})
- Counter display commands:
  - `\arabic{counter}` — renders counter as arabic numeral (1, 2, 3...)
  - `\roman{counter}` — renders as lowercase roman (i, ii, iii...)
  - `\Roman{counter}` — renders as uppercase roman (I, II, III...)  
  - `\alph{counter}` — renders as lowercase letter (a, b, c...)
  - `\Alph{counter}` — renders as uppercase letter (A, B, C...)
  - `\fnsymbol{counter}` — renders as footnote symbol (*, †, ‡...)
- Pre-defined counters: `section`, `subsection`, `subsubsection`, `figure`, `table`, `equation`, `enumi`, `enumii`, `enumiii`, `page`
- Existing section/figure counters should USE the counter system internally

**Tests (15+ new):**
- Test `\setcounter{page}{5}` sets page counter to 5
- Test `\addtocounter{section}{2}` increments section counter
- Test `\newcounter{myctr}` creates counter at 0
- Test `\arabic{section}` renders current section number
- Test `\roman{page}` renders page number as roman numeral
- Test `\alph{enumi}` renders list counter as letter
- Test hyphenation produces discretionary breaks in long words
- Test `\hyphenation{algo-rithm}` respects manual hyphenation
- Test `\-` inserts soft hyphen break
- All 459 existing tests continue to pass

- **Cycles budget:** 5 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented, 492 tests pass, CI green (commit ce0908f)

### M27: Font Style Support (Bold/Italic/Typewriter in PDF Output) ✅ COMPLETE
Implement proper font face differentiation in PDF output so that `\textbf`, `\textit`, `\emph`, and `\texttt` produce visually distinct output using appropriate PDF font resources.

- **Deliverables:** FontStyle enum (Normal/Bold/Italic/BoldItalic/Typewriter), font_style on BoxNode::Text, TranslationContext font tracking + group scoping, \bfseries/\itshape/\ttfamily/\normalfont, 5 Base-14 PDF fonts, 24 new tests
- **Cycles budget:** 5 | **Cycles actual:** 2 (1 impl + 1 fix round)
- **Status:** ✅ Complete — verified by Apollo (commit c34c8a9, 516 tests total)

### M28: Per-Font-Style Character Width Metrics ✅ COMPLETE
Fix the character width metrics so each font style uses accurate widths for line-breaking and PDF text positioning.

**Background:** M27 added visual font differentiation in PDF (correct font face selected). However, all font styles still use the same CM Roman/Helvetica width table for measuring character widths. This means bold text (wider than normal in Helvetica-Bold) and typewriter text (monospace in Courier) still break lines as if they were normal weight Helvetica. This causes line-breaking to be incorrect for styled text.

**Engine changes (rustlatex-engine):**
- Add width tables for each font style to `FontMetrics` trait and `StandardFontMetrics`
- Add method `char_width_for_style(c: char, style: FontStyle, size: f64) -> f64` to FontMetrics
- Helvetica Normal widths: current widths (already approximated from Helvetica AFM)
- Helvetica-Bold widths: slightly wider than Normal (apply 1.05× scale as approximation, or use actual AFM widths if available)
- Helvetica-Oblique widths: same as Normal (oblique has same metrics as upright)
- Helvetica-BoldOblique widths: same as Bold
- Courier widths: monospace — every character is exactly 600 units wide (at 1000 units/em scale)
- Engine translator: when computing `BoxNode::Text.width`, use `char_width_for_style(c, font_style, font_size)`

**PDF backend changes (rustlatex-pdf):**
- When computing text width for `Tf` (font) and `Tj` (show text) operators, use per-font-style widths consistent with engine metrics
- This ensures PDF text positioning matches engine layout expectations

**Tests (15+ new):**
- Test Courier (Typewriter) width for any printable char = 6.0pt (at 10pt, since 600/1000 × 10 = 6.0)
- Test that `\texttt{hello}` produces Text nodes whose width = 5 × 6.0 = 30.0pt
- Test that bold text width is >= normal text width (bold is not thinner than normal)
- Test that `char_width_for_style` returns different values for Normal vs Typewriter
- Test that engine uses correct width when translating `\texttt{x}`
- Test that engine uses Courier width for typewriter nodes in line-breaking context
- Test FontMetrics trait method exists for style-aware width lookup
- All 516 existing tests continue to pass

- **Cycles budget:** 4 | **Cycles actual:** 1 (+ 1 verification)
- **Status:** ✅ Complete — verified by Apollo (commit fce36ac, 536 tests total)

### M29: pdflatex Comparison Infrastructure
Establish the ability to compare our compiler's output against pdflatex output in CI.

**CI changes (.github/workflows/ci.yml):**
- Add `texlive-base texlive-fonts-recommended texlive-latex-base` to the apt-get install step (already has ghostscript)
- Verify `pdflatex --version` works in CI

**Test files (examples/):**
- Use existing simple .tex files (hello.tex, sections.tex, math.tex, lists.tex)
- Create a new `compare.tex` with basic text, one section, inline math, and a list — representative enough to reveal layout differences

**Rust integration test (crates/rustlatex-engine/tests/ or a new comparison test):**
- Compile `compare.tex` with our compiler → `our_output.pdf`
- Compile `compare.tex` with pdflatex → `ref_output.pdf`  
- Render page 1 of each to PNG via GhostScript at 150 DPI
- Compute pixel difference (using a simple byte-level metric in the test)
- Assert both PDFs are non-empty and valid (gs can render them)
- Log pixel similarity score — don't fail test on similarity (we know output will differ initially), just log it
- This establishes a baseline for future comparison

**Shell script approach (simpler alternative):**
- Add a `compare_test.sh` script that runs both compilers and uses `gs` + `diff` or `compare`
- Run this script as part of a CI step
- Output the similarity score to CI logs

**Tests (5+ new):**
- Test that pdflatex is available in CI (skip test if not available via `#[cfg]` or env check)
- Test that our compiler produces non-empty PDF for compare.tex
- Test that pdflatex produces non-empty PDF for compare.tex  
- Test that both PDFs can be rendered to PNG by GhostScript
- Log pixel similarity between our output and pdflatex output

**Goal:** After this milestone, we will know *how* different our output is from pdflatex on a simple document, which will guide M30+ fixes.

- **Cycles budget:** 3 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo committed 2344578, 541 tests pass, CI green. Comparison infrastructure established.

### M30: Fix Critical Rendering Gaps vs pdflatex
Fix the top 5 rendering gaps identified by Diana's research to dramatically improve visual quality:

1. **PDF hsize mismatch (CRITICAL BUG):** Change PDF justification hsize from 495pt to 345pt, update margin_left to 72.27pt
2. **Page margins + line height:** margin_left=72.27pt, body text at 733pt from bottom (109pt from top), line_height=12pt
3. **Section headings bold:** FontStyle::Bold for section/subsection/subsubsection title text
4. **Support \[...\] display math syntax:** Parser recognizes \[ ... \] as DisplayMath
5. **Force line breaks between list items:** Add Penalty{-10000} between list items

**Goal:** Dramatically improve visual similarity vs pdflatex (estimated ~25% → ~60%+ similarity). 15+ new tests, all 541 existing tests pass.

- **Cycles budget:** 3 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented commit 72ed772, 558 tests pass, CI green

### M31: Section/Paragraph Spacing + Pixel Similarity Visibility ✅ COMPLETE
Improve visual quality by fixing spacing to match LaTeX article class and surface the pixel similarity score in CI.

**Goals:**
1. **Surface pixel similarity**: Change `println!` → `eprintln!` in `test_pixel_similarity_logged` so the score is visible in CI logs (stderr is not captured)
2. **Fix section heading spacing**: LaTeX article class uses ~24pt before `\section`, ~8pt after; ~18pt before `\subsection`, ~6pt after. Currently we use 12pt/6pt.
3. **Fix paragraph spacing model**: LaTeX uses `\parskip=0pt` — no extra space between paragraphs beyond baselineskip. Verify our 6pt inter-paragraph glue is appropriate or adjust.
4. **15+ new tests** covering spacing values for sections, paragraphs

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Ares implemented, Apollo verified (commit 46a2dbc, 579 tests total)

### M32: Embed CM Bold/Italic/Typewriter Fonts + Accurate Width Metrics ✅ COMPLETE
Replace Helvetica font variants in PDF output with actual Computer Modern fonts for visual accuracy matching pdflatex.

**Goals:**
1. Embed cmbx10.pfb (CM Bold), cmti10.pfb (CM Italic), cmbxti10.pfb (CM Bold Italic), cmtt10.pfb (CM Typewriter)
2. Replace F3/F4/F5/F6 (currently Helvetica variants + Courier) with CM Type1 fonts
3. Update engine width metrics: cmbx10 AFM widths, cmti10 AFM widths, cmtt10 = 5.25pt monospace (not 6.0pt)
4. 15+ tests covering font embedding and width accuracy

- **Cycles budget:** 2 | **Cycles actual:** 1 + 1 verification
- **Status:** ✅ Complete — Apollo verified 594 tests pass, CI green (commit a7790a7)

### M33: OT1 Encoding Fix + Bullet Character Fix + CI Similarity Visibility ✅ COMPLETE
Fix the top rendering gaps identified by Diana's M32 research:

1. OT1 /Differences encoding for all 5 CM font dicts (NON_SYMBOLIC→SYMBOLIC)
2. Bullet "•" replaced with "-" (avoids UTF-8 encoding issues with 8-bit CM fonts)
3. CI step with --nocapture for pixel similarity visibility

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented, 609 tests pass, CI green

### M34: Proper Superscript/Subscript Rendering in PDF ✅ COMPLETE
Implement proper superscript and subscript rendering using the PDF Ts (text rise) operator.

- **Deliverables:** vertical_offset field on BoxNode::Text, math_node_to_boxes() function, PDF Ts/set_rise() operator, 17 new tests
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented, 626 tests pass, CI green (commit 436889b)

### M35: Math Italic Style + Real Pixel Similarity Comparison
Improve math rendering quality and fix visual similarity measurement infrastructure.

**Goals:**
1. **Math variables use Italic** — In math_node_to_boxes_inner(), use FontStyle::Italic for single-letter math tokens (a-z, A-Z). This matches pdflatex/cmmi10 behavior where math variables appear in math italic.
2. **Fix pixel similarity** — Change GS rendering from PNG to PPM (uncompressed), compute actual per-pixel similarity score. Add compare_ppm_files() helper.
3. **Expand compare.tex** — Add subsection and display math to get better pixel comparison coverage.

- **Tests:** 22 new, 647 total
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented, 647 tests pass, CI green (commit 0969eab)

### M36: Rendering Quality Improvements (bullet, ligatures, parindent, display math) ✅ COMPLETE
Implement four targeted visual quality improvements identified by Diana's M36 research:

1. **Bullet fix** — Add BoxNode::Bullet variant in engine, render as PDF filled circle. Change itemize to emit Bullet instead of Text("-"). Estimated +0.5-1.0% pixel similarity.
2. **OT1 ligature substitution** — In rustlatex-pdf, add apply_ot1_ligatures() mapping fi→12, fl→13, ff→11, ffi→14, ffl→15 for CM text fonts (F1/F3/F4/F5 only). ~+0.01-0.05%.
3. **Fix parindent 20pt→15pt** — Paragraph indent Kern changes from 20.0 to 15.0. ~+0.1%.
4. **Display math spacing** — abovedisplayskip/belowdisplayskip = 12pt. ~+0.2-0.4%.

- **Tests:** 25+ new, 672 total
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 524b22c), 672 tests pass

### M37: FontStyle::MathItalic + cmmi10 Width Metrics + Display Math Centering ✅ COMPLETE
Improve math rendering quality with accurate width metrics and visual centering.

**Goals:**
1. **FontStyle::MathItalic variant** — Add new FontStyle::MathItalic to engine enum (distinct from Italic). Use for single-letter math variables in math_node_to_boxes_inner(). In PDF backend, map MathItalic to cmti10 (F4) for now.

2. **cmmi10 AFM width metrics** — Extract character widths from cmmi10.afm at matplotlib path. Add char_width_for_style() for MathItalic using these widths. More accurate line-breaking for math content.

3. **Display math horizontal centering** — Center display math horizontally within text body (x = margin_left + (body_width - line_width) / 2). Use AlignmentMarker::Center for display math paragraphs.

- **Tests:** 18 new, 690 total
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 13b2626), 690 tests pass, CI green

### M38: Embed cmmi10.pfb + cmsy10.pfb Math Fonts ✅ COMPLETE
Improve math font visual fidelity by embedding actual Computer Modern Math fonts.

**Goals:**
1. **Embed cmmi10.pfb (Math Italic font)** — Registered as F7 in PDF with OML encoding. FontStyle::MathItalic maps to F7. Fixes engine/PDF width mismatch.
2. **Embed cmsy10.pfb (Math Symbol font)** — Registered as F8. BoxNode::Bullet uses cmsy10 glyph 15 (5pt advance) instead of Bezier circle.
3. **Shift page Ref IDs** — Pages moved from 18+i*2 to 24+i*2. Refs 18-23 for cmmi10/cmsy10.
4. **OT1 ligature guard** — F7/F8 excluded from OT1 ligature substitution (correct).

- **Tests:** 706 total pass, CI green
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Ares implemented (commit 315adba)



### M39: Math Operator Spacing + CI Pixel Similarity Fix
Improve math rendering quality by adding proper thin/thick spaces around math operators.

**Goals:**
1. **Math operator spacing** — In math_node_to_boxes_inner(), add Kern nodes around binary operators (+,-,×,÷,±,∓,·) and relations (=,<,>,≤,≥,≠,∈,⊂,∪,∩,→,←): binary ops get 1.667pt kern on each side, relations get 2.778pt kern on each side. This matches pdflatex behavior.
2. **CI pixel similarity fix** — Change CI to capture the similarity score (write to file or use different mechanism).
3. **10+ new tests** — verify operator spacing for +, =, ×, and that non-operators don't get spacing.

- **Cycles budget:** 2 | **Cycles actual:** pending
- **Status:** 🔄 In progress (issue #43)

---

## Notes on "Binary Identical" Goal

True binary-identical output is extremely difficult because it depends on:
1. **Timestamps** — PDF metadata timestamps differ unless suppressed
2. **Random seeds** — some compilers use randomness
3. **Font subsetting** — the same subset algorithm must be used
4. **PDF object ordering** — exact same internal structure

In practice, we will target **semantic equivalence** first (same visual output), then work toward binary identity by matching pdflatex's specific behavior for timestamps, object ordering, and font embedding. The test corpus will be simple documents initially.
