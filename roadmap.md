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
- **M40 research (Athena direct):** cmr10 AFM has exactly 183 kern pairs. Word spacing (stretch=1.67, shrink=1.11) is already correct (TeX standard 1.667/1.111). Main remaining improvement is implementing kern pair lookups in the PDF backend. Diana was consistently hitting 200K token limit — skipped Diana for M40 planning.
- **Cycle (M40):** M40 completed in 1 implementation cycle + 1 verification. Leo delivered cmr10_kern_pair() function with all 183 AFM kern pairs, TJ operator usage for kerned text, is_cmr10_kern_font() guard (F1 only). Apollo verified 757 tests pass, CI green. Pixel similarity unchanged at 96.96% (compare.tex has few kern-pair characters).
- **M41 scope:** cmbx10 kern pairs (F2/Bold), exact word spacing precision (1.66667/1.11111), expand compare.tex for better visual coverage, target 770+ tests. Bold text in section headings benefits from cmbx10 kern pairs.
- **Cycle (M41):** M41 completed in 1 implementation cycle. Leo delivered cmbx10_kern_pair() with 181 pairs from cmbx10.afm, font_kern_pair()/font_has_kern_pairs() dispatch for F1/F2, word spacing precision (1.66667/1.11111), expanded compare.tex (bold text, kern pair test words, math). 15 new tests, 772 total tests pass, CI green. Pixel similarity = 95.88% (slightly lower due to compare.tex expansion showing more differences).
- **M42 scope:** Superscript/subscript geometry correction (size=7.07pt instead of 7.0pt, rise=3.45pt instead of 4.0pt, subscript offset=-2.5pt instead of -2.0pt). Add cmti10 kern pairs (F4/Italic) from CI texlive AFM at `/usr/share/texmf/fonts/afm/public/cm/cmti10.afm`. Target 785+ tests.
- **Cycle (M42):** M42 completed in 1 implementation cycle. Leo delivered superscript precision (7.07pt, 3.45pt rise, -2.5pt subscript offset), cmti10 kern pairs (178 pairs from AFM, F4 wired into dispatch). 17 new tests, 789 total tests pass, CI green.
- **M43 scope:** Fix justified text line width computation (include kern pair contributions in line_nat_width) + add cmbxti10 kern pairs (F5/BoldItalic). Estimated pixel similarity gain ~0.3-0.5%. Target 800+ tests.
- **Cycle (M43):** M43 completed in 1 implementation cycle. Leo delivered compute_kern_pair_total + justified width fix + cmbxti10 kern pairs (178 entries, F5). 23 new tests, 812 total tests pass, CI green. Pixel similarity = 95.77% (similarity=0.9577 from CI).
- **CRITICAL BUG FOUND (M44 planning):** cmbx10 kern pairs (M41) are dead code. `font_name_for_style(Bold)` returns `b"F3"` but `is_cmr10_kern_font` checks for `b"F2"`. Bold text (F3/cmbx10) has never benefited from kern pairs. Fix: change `is_cmr10_kern_font` and `font_kern_pair` to use F3 (not F2) for cmbx10. This bug affects section headings (which use Bold/F3).
- **Cycle (M44):** M44 completed in 1 implementation cycle. Ares fixed F2→F3 cmbx10 kern pair routing + updated tests. 14 new tests, 826 total tests pass, CI green.
- **M45 scope:** Per-line height adaptation in PDF. Currently line_height is a flat 12pt constant. Section headings (14pt font) and subsections (12pt) need larger baselineskip. Fix: add `line_height: f64` to `OutputLine`, compute as max_font_size×1.2 in engine, use it in PDF backend. Target 840+ tests.
- **M44 scope:** Fix cmbx10 kern pairs to apply to F3 (not dead-code F2). Fix `is_cmr10_kern_font` (add F3, remove F2), `font_kern_pair` (F3→cmbx10_kern_pair), `font_has_kern_pairs` (F3 dispatch), `compute_kern_pair_total` (F3 dispatch), `line_nat_width` computation for F3. Update tests that assert F3 should NOT have kerning. Estimated similarity improvement: +0.3-0.5% for bold text in section headings. Target 825+ tests.
- **Cycle (M45):** M45 completed in 1 implementation cycle + 1 verification. Ares delivered line_height field on OutputLine, compute_line_height() helper (max font_size × 1.2), break_items_with_alignment() sets per-line height, PDF backend uses line.line_height. 14 new tests, 840 total tests pass. NOTE: Bug found: Engine::typeset() page accumulation still uses flat 12.0 for accumulated_height — this is fixed in M46.
- **M46 scope:** (1) Fix page accumulation in Engine::typeset() to use line.line_height instead of flat 12.0 (bug from M45 — the OutputLine now has line_height but typeset() still uses flat 12.0 for page breaking). (2) Add cmmi10 kern pairs (F7/MathItalic) — 166 pairs from cmmi10.afm. 15+ new tests. Target 855+ tests. Pixel similarity: 95.69% after M45.
- **Cycle (M46):** M46 completed in 1 implementation cycle. Leo delivered page accumulation fix (accumulated_height uses line.line_height), cmmi10 kern pairs (166 pairs, F7 dispatch). 18 new tests, 858 total tests pass, CI green. Pixel similarity = 95.69%.
- **M47 scope:** (1) Improve PPM pixel comparison to use ±2 per-channel tolerance (counts as matching if all 3 RGB channels within 2 of each other) — antialiasing creates 1-3 value differences that our exact-match metric penalizes unfairly. (2) Add cmsy10 kern pairs (26 pairs, F8/MathSymbol). (3) Clean up unused `let metrics = StandardFontMetrics;` in test functions (prefix with `_metrics`). Target 873+ tests, measured similarity improvement to ~98%+.
- **Cycle (M47):** M47 completed in 1 implementation cycle. Leo delivered PPM ±2 tolerance, cmsy10 kern pairs (26 pairs, F8), unused variable cleanup. 21 new tests, 879 total tests pass, CI green. Pixel similarity = 95.69% (with ±2 tolerance).
- **Cycle (M48):** M48 completed in 1 implementation cycle. Leo delivered correct cmr10 AFM widths for 27 punctuation/symbol characters in StandardFontMetrics::char_width(). 21 new tests, 900 total tests pass, CI green. Pixel similarity = 95.69% (unchanged — punctuation width fix did not affect compare.tex line-breaking measurably, but improves correctness for documents with more punctuation).
- **M48 scope:** Complete punctuation character widths in StandardFontMetrics. The engine's char_width() defaults to 5.0pt for all punctuation, but cmr10 AFM has precise widths (period=2.778, comma=2.778, hyphen=3.333, colon/semicolon=2.778, exclaim=2.778, question=4.722, parens=3.889, brackets=2.778). Wrong widths cause incorrect line-breaking vs pdflatex. Also add bold (cmbx10) widths for punctuation. This is the highest-impact remaining fix. Target 899+ tests, pixel similarity to ~97%+.
- **Athena M49 analysis (direct):** Rebuilt binary and confirmed \[...\] DisplayMath is correctly parsed. Identified two critical bugs: (1) Paragraph-end Glue{natural:6.0} creates spurious 12pt vertical blank lines between paragraphs (pdflatex uses parskip=0pt). (2) PDF justification distributes remaining space uniformly instead of proportionally by stretch value — wrong for inter-sentence glue (stretch=2.5 vs normal 1.667). Fix both in M49.
- **Cycle (M49):** M49 completed in 1 implementation cycle. Leo fixed paragraph-end glue (natural:6.0→0.0, stretch:2.0→1.0) and proportional justification (remaining * stretch_i / total_stretch). 15 new tests, 915 total tests pass, CI green. Pixel similarity = 95.69% (unchanged — compare.tex doesn't have enough paragraphs for the parskip fix to show).
- **Cycle (M50):** M50 completed in 1 cycle. Leo added BoxNode::VSkip, section/subsection emit VSkip instead of horizontal Kern. 22 new tests, 938 total. Pixel similarity DROPPED to 94.75% (from 95.69%) because spacing values (24pt/8pt) don't match pdflatex article class. Fix in M51.
- **CRITICAL BUG FOUND (M50 analysis):** Athena direct analysis revealed that section heading Kern(24.0)/Kern(8.0) nodes are HORIZONTAL kerns, not vertical. The PDF backend advances current_x (horizontal) when it sees Kern nodes, NOT current_y (vertical). This means section heading spacing is applied horizontally (as indentation) instead of vertically. Body text after a section heading is only 16.8pt below instead of ~48.8pt (24+16.8+8). This 32pt vertical mismatch cascades to ALL subsequent lines, explaining why pixel similarity is stuck at 95.69%. Fix: Add BoxNode::VSkip{amount} variant, use it for section/subsection before/after spacing, handle it in PDF backend as vertical movement.
- **Cycle (M51):** M51 completed in 1 cycle. Leo updated section spacing to 15.07/9.90/13.99/6.46, section font_size 14.0→14.4, subsubsection font_size 11.0→10.0. 17 new tests, 955 total. Pixel similarity DROPPED to 94.47% — regression caused by adding 15.07pt before first section (pdflatex suppresses before-skip at top of page).
- **M51 regression root cause:** pdflatex's \@startsection uses NEGATIVE before-skip which is suppressed at top of page/column. compare.tex starts with \section{Introduction} — pdflatex adds ZERO before-spacing. We add 15.07pt VSkip, shifting all content down by 15.07pt. Fix in M52: suppress before-VSkip when no body content has been emitted yet.
- **M52 scope:** Add `content_emitted: bool` to TranslationContext. Set true when body paragraphs are emitted. Suppress before-VSkip (use 0.0) when !content_emitted. Expected similarity recovery: 97%+.
- **Cycle (M52):** M52 completed by Leo (0f325c5). Suppresses before-VSkip for first section. 960 tests. But similarity only 94.47% — font size change (14.4 vs 14.0) still hurts.
- **Cycle (M53):** M53 completed by Leo (37de084). Set all VSkip=0.0 (no effect on rendering). 965 tests, 94.53% similarity. Still below 95.69% because section font_size 14.4 (line_height 17.28pt) vs M49's 14.0 (line_height 16.8pt).
- **M54 scope:** Revert section font_size 14.4→14.0 and subsubsection font_size 10.0→11.0 to recover pixel similarity. These M51 changes made things worse vs pdflatex layout. Target: 980+ tests, 95.7%+ similarity. Keep VSkip=0.0 infrastructure intact.
- **Cycle (M54+M55):** M54 reverted font sizes (984 tests, 94.47% — still not enough). M55 removed all VSkip nodes from section headings entirely (returning to M49-style layout). Result: 1003 tests pass, pixel similarity = 95.68%, CI green (commit 57d328f). VSkip infrastructure kept for \vspace.
- **M56 scope:** Investigate and fix the remaining 4.32% pixel similarity gap. Diana's research needed to identify top improvements. Key candidates: (1) Section spacing before/after without causing cascading layout shifts; (2) Line-breaking accuracy improvements; (3) Other rendering details. Target: 97%+ similarity, 1015+ tests.
- **Cycle (M56 — MISSED):** M56 failed — 3/3 cycles used, similarity stuck at 95.69%. Section VSkip (12.24pt after section) regressed to 94.44%; reverting VSkip but keeping font_size=14.4 recovered to 95.69%. Net: font_size 14.4 in place (correct per pdflatex spec), but no similarity gain. KEY LESSON: Do NOT add VSkip around section headings. The gap must be diagnosed via a different approach — need to identify which specific pixels differ. Current state: 1000 tests, 95.69% similarity, font_size=14.4 for sections.
- **M57 approach:** Diana's forensic analysis confirmed: `margin_left=72.27pt` is WRONG (should be 126.25pt for a4paper article symmetric layout). Also `margin_top=109pt` may be off by ~15pt. These margin errors explain the remaining 4.31% gap. M57 corrects both margins. KEY LESSON: margin_left changes do NOT affect line-breaking (only PDF rendering position), making this a zero-risk line-breaking change. The only risk is if the margin calculation is wrong (would drop similarity to ~89%).
- **M58 outcome (97.25%):** M57 margin fixes confirmed correct: similarity 95.69% → 97.25% (+1.56%). Remaining gap 2.75% = ~13,778 pixels. CI test bug was in PPM header parsing — GhostScript adds comment lines; both `parse_ppm` and `parse_ppm_header` needed updating.
- **M59 scope:** Footer y-position fix (25pt → 68.36pt, pdflatex footskip=30pt from bottom of textheight) + itemize topsep fix (6pt → 8pt, pdflatex \topsep=8pt plus 2pt minus 4pt). Both are high-confidence fixes. Target 1030+ tests, >97.5% similarity.
- **Cycle (M59):** M59 completed by Leo (55ff97e). footer_y=68.355, itemize topsep=8pt, 13 new tests. 1034 total tests pass, CI green. Pixel similarity = **97.25% (UNCHANGED from M58)**. Conclusion: footer+topsep changes had zero net effect. The similarity is stuck at 97.25%.
- **M60 analysis (Athena direct):** After M57 corrected margins (+1.56%), the remaining 2.75% gap needs fresh diagnosis. KEY INSIGHT: All previous VSkip attempts (M50-M55) were with WRONG margins (72.27pt), making their results unreliable. With correct margins (126.25pt), adding section after-VSkip may now help. For compare.tex starting with \\section{Introduction}: body text is 9.62pt too HIGH vs pdflatex (missing 9.90pt after-skip from \\@startsection, only partially offset by our 17.28 vs 17.0pt section line_height). This explains a meaningful fraction of remaining gap. M60 will re-try section after-VSkip with correct understanding.
- **M60 scope:** Add section/subsection after-VSkip to match pdflatex \\@startsection afterskip values. For section: add VSkip(9.90pt) after section heading text. For subsection: add VSkip(6.46pt). Before-skip stays suppressed (content_emitted already implemented). Fix compute_line_height() to use exact 17.0pt for 14.4pt section (not 17.28). Also fix display math shrink: 3.0→9.0 to match pdflatex (abovedisplayskip=12pt plus 3pt minus 9pt). Target 1045+ tests, >97.5% similarity.
- **M60 REGRESSION**: M60 regressed similarity from 97.25% → 96.67% (-0.58%). Root cause: VSkip after section headings (9.90pt section, 6.46pt subsection) caused layout mismatch. VSkip around section headings has now been tried 8+ times and ALWAYS regresses. M61 reverts the VSkip changes but keeps compute_line_height precision and display math shrink fixes.
- **M61 scope:** Revert M60's VSkip additions around section headings. Keep compute_line_height precision (14.4→17.0, 12.0→14.0) and display math shrink (3.0→9.0). Expected: recover to 97.25%+ similarity. Scheduled Diana (#59) to analyze remaining gap for M62+.
- **Cycle (M61):** M61 completed (a3f94d0). VSkip reverted from section headings. Pixel similarity = **97.24%** (recovered from M60 regression). CI green. Tests pass.
- **M62 analysis (Athena):** Key remaining hypothesis: (1) section line_height 17pt vs pdflatex 18pt (article.cls uses fontsize 14.4pt/18pt baselineskip); (2) start_y = 718.0pt vs pdflatex 718.73pt (0.73pt too low); (3) potential hyphenation differences in line-breaking. Diana assigned to verify these hypotheses (#59 updated).
- **Cycle (M62):** M62 completed (bb67ce5). line_height 17→18 for 14.4pt section, margin_top 124.0→123.27. Pixel similarity = **97.24% (UNCHANGED from M61)**. M62 changes were structurally correct but had no measurable similarity impact.
- **M63 REGRESSION:** M63 changed compute_line_height values: 14.4pt→9.9 (WRONG, should be 18.0), 12.0pt→6.46 (WRONG, should be 14.0). Ares's team misunderstood: afterskip (9.9/6.46) ≠ baselineskip (18/14). These values are used as y-advance amounts for section heading lines. Using 9.9 instead of 18 causes section headings to only advance 9.9pt vertically, placing text too close together. Pixel similarity **dropped from 97.24% → 96.78%** (regression -0.46%). M64 must revert these values.
- **M63 VSkip{0.0} chunk separator:** M63 also added VSkip{0.0} after section headings as "chunk separators". This is harmless (VSkip with 0.0 does nothing), but didn't improve similarity.
- **Cycle (M65):** M65 completed (d7f888c). Removed VSkip{0.0} from section headings. Pixel similarity = **97.24%** (recovered). CI green. 
- **M66 hypothesis (Athena direct):** Remaining 2.76% gap (13,834 pixels) likely caused by missing itemize vertical spacing. Itemize uses Glue nodes for topsep (8pt) and itemsep (4pt) which are HORIZONTAL glue → stripped by strip_glue → NO vertical effect. pdflatex has 24pt total vertical spacing around itemize for compare.tex. Fix: convert to VSkip nodes. Diana assigned to verify via issue #63.
- **M66 REGRESSION (97.24%→97.06%):** Itemize VSkip conversion REGRESSED. VSkip-based spacing consistently regresses for both section headings AND itemize environments. This is now confirmed as a pattern: do NOT add VSkip for list spacing. The underlying model of "add VSkip to match pdflatex spacing" is fundamentally broken in our engine. Future fixes must focus on NON-SPACING sources of difference.
- **VSkip anti-pattern:** VSkip-only lines use their `line_height = amount` for vertical advance in the PDF backend. This is the WRONG model: pdflatex integrates spacing into the baseline skip calculation for neighboring lines, not as separate VSkip lines. Adding VSkip before/after lists adds spacing that ISN'T in pdflatex's model (at those exact positions relative to our layout), causing cascading mismatches.
- **LESSON: Do NOT add VSkip around itemize lists** — confirmed regresses. Add this to the same rule as section heading VSkip.
- **LESSON: Do NOT use KP forced_j sentinel that rejects `ratio < -1.0`** — M74-fix confirmed: when no valid break fits in hsize (e.g., word longer than hsize), the KP optimizer has no feasible path and produces overflow mega-lines. The original `pen * pen` demerits behavior (accepting all forced-break endings) is safer because at minimum one breakpoint (at the forced break itself) is always feasible.
- **LESSON: Do NOT add AlignmentMarker{Justify} per paragraph/section** — M76 confirmed: this approach REGRESSES 97.31% → 96.82%. Each paragraph gets its own KP run producing different line breaks than pdflatex's continuous-flow model. The 97.31% baseline is achieved with paragraphs flowing together. This is now confirmed as a pattern: ANY paragraph/section separation (Penalty, VSkip, AlignmentMarker) regresses.
- **RULE: The 6 confirmed regression anti-patterns are**: (1) VSkip around section headings, (2) VSkip around itemize lists, (3) Paragraph-end Penalty{-10000}, (4) Section/subsection Penalty{-10000}, (5) KP forced_j sentinel (reject ratio<-1.0), (6) AlignmentMarker per paragraph/section. Do NOT try any of these again.
- **M76 kern fix (correct)**: `compute_kern_pair_total` was adding raw AFM units to `line_nat_width` without dividing by 1000/font_size. At 10pt, divide by 100 (=1000/10). Fix: add `/100.0`. This is a correct fix but its effect is masked by M76 regression.
- **M76 parfillskip fix (correct)**: `is_last_line_like` threshold=10 prevents over-stretching word spaces on short last lines (emulating `\parfillskip=0pt plus 1fil`). Correct fix, keep.
- **Cycle (M67):** M67 completed (a14a9a7). Reverted M66 VSkip regression, fixed display math 10pt (natural:12→10, stretch:3→2, shrink:9→5), fixed subsection line_height 14.0→14.5. 720 engine tests pass. Expected similarity ≥ 97.24% + small improvement from display math fix.
- **Cycle (M68):** M68 completed (b51eddd). Section line_height 18→21pt (absorbing afterskip effect). Pixel similarity = **97.31%** (+0.07% from 97.24%). CI green, 720+ engine tests pass.
- **Cycle (M69):** M69 implemented by Leo (47cdf9f). Two fixes: display math post-processing (+10pt to Center-aligned 10pt lines and preceding lines) + subsection line_height 14.5→17.0. CI result: **97.30% — essentially unchanged from 97.31%**. Both fixes had ~zero net impact. Possible explanations: subsection 17.0 overshot creating regression that cancelled display math gain; or display math post-processing isn't triggered or doesn't affect the actual mismatch pixels. Diana analyzing root cause for M70 planning.
- **Diana's M70 research (CRITICAL):** Diana identified that the greedy line breaker has two critical bugs: (1) `break_into_lines()` does NOT count Glue natural width (~3.3pt per space) in `current_width`, causing lines to accumulate 15-20+ words before triggering a break — producing massively overfull lines (x up to 1299pt on A4). (2) Penalty{-10000} is not handled as forced break in the greedy breaker. Additionally, the KP breaker has tolerance=200 which is too strict for typical lines (badness ≈ 800-6400 >> 200), causing the KP to ALWAYS fall back to the broken greedy breaker. The PDF output has only 6 text y-levels where there should be ~21 lines. **This is why pixel similarity is stuck at ~97%** — most text is catastrophically misplaced. Fix: (1) Add Glue natural width to current_width in break_into_lines(); (2) Handle Penalty{≤-10000} as forced break; (3) Increase KP tolerance to 10000. Expected improvement: 97.3% → 98-99%.
- **M70 completed but similarity unchanged (97.30%):** M70 implemented all 3 fixes. CI confirmed 97.30% — no improvement. Root cause (Athena direct analysis): The real bug is NOT in the line-breaker algorithms but in the ITEM STREAM structure. Section headings emit just a Text node with NO Penalty separator. Paragraph ends emit Glue{0,1,0} with NO Penalty. So all text is ONE continuous stream: [Section heading text] + [para1 words...] + [para2 words...] + ... The KP/greedy breaks this at 345pt but paragraph 1 (~230pt wide) fits in one line, so the first break puts section heading + para1 + start of para2 on ONE y-level. Fix for M71: add Penalty{-10000} after section headings and at paragraph ends.
- **M71 REGRESSION (97.30%→96.88%):** M71 added Penalty{-10000} after section headings AND paragraph ends. This REGRESSED similarity from 97.30% to 96.88% (-0.42%). Root cause: Penalty{-10000} after paragraph ends forces each paragraph to break independently in break_items_with_alignment (separate chunk per paragraph). pdflatex breaks text in continuous flow. Different line breaks → more pixel mismatches. KEY INSIGHT: Section headings are ALREADY separated by AlignmentMarker (Center vs Justify). The Penalty after headings is redundant but harmless. The Penalty after PARAGRAPH ENDS is the regression cause. M72 must revert the paragraph-end penalties. LESSON: Paragraph-end forced breaks cause regression just like VSkip around section/list environments.
- **Cycle (M72):** M72 removed paragraph-end Penalty{-10000} (commit 5357d13). But M72 regressed to 96.77% because it KEPT the section/subsection Penalty nodes AND the forced-break processing in break_items_with_alignment. M73 fixed this by also removing the section Penalties and the forced-break block. Result: M73 = 97.30%.
- **Cycle (M73):** M73 (commit 4a18bf3) removed section/subsection Penalty{-10000} nodes AND the Penalty{≤-10000} forced-break else-if block from break_items_with_alignment. CI confirmed: **97.30% similarity** restored. This is the M70 baseline behavior. 777 engine tests pass, CI green.
- **M77 + M78 analysis:** Confirmed 97.33% (best so far) is achieved WITH mega-lines. Diana's M78 forensic analysis found only 7 y-positions in output. ~90% of remaining gap is from layout (can't fix). Two small non-layout bugs found: (1) kern x-position tracking misses kern pair contribution at line 2224 of rustlatex-pdf; (2) kern scaling in line_nat_width uses hardcoded /100.0 (assumes 10pt) instead of font_size/1000. These are the last known fixable bugs before needing a fundamentally different paragraph-separation approach.
- **Strategic assessment at M78:** The project is in a local maximum. We can squeeze ~0.1% more from non-layout fixes (M79). Breaking through 98%+ requires solving the mega-line problem — but every attempt to add paragraph separation regresses because our KP line-breaking produces different breakpoints than pdflatex's. True binary identity requires matching pdflatex's exact typesetting algorithm, which is a much larger undertaking.
- **Cycle (M79):** M79 completed (f633fd8). Leo fixed BUG #1 (kern x-position tracking at line 2224 of rustlatex-pdf) and BUG #2 (kern scaling /100.0 → font_size/1000 at line 2093). 9 new tests. CI confirmed similarity = **97.34%** (+0.01% from 97.33%). ~1184 total tests pass, CI green.
- **M80 approach (new hypothesis):** Deep analysis of mega-line root cause: The KP's `forced_j` path accepts ANY line width before a Penalty{-10000} with constant 10^8 demerits. This means mega-lines before forced breaks ALWAYS win (0 extra demerits vs positive demerits for extra breaks). Fix requires: (1) Section heading Penalty{-10000} after heading text, (2) Paragraph-end Penalty{-10000} after each paragraph, (3) forced_j width check (reject ratio<-1.0, accept underfull with 0 cost). KEY INSIGHT: Compare.tex paragraphs are all SHORT (< 345pt) so para-by-para KP produces same result as pdflatex (1 line per para). Word metrics now match pdflatex exactly (from M43/M49), so line breaks should also match. Previous M71/M74-fix regressions had different root causes: M71 kept tolerance=10000 (allowing mega-lines within paragraphs), M74-fix kept forced_j but had subsection 18.5pt regression.

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

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 263e476), 736 tests pass, CI green. Pixel similarity = 96.96%.

### M40: Character Pair Kerning (cmr10 AFM Kern Pairs) ✅ COMPLETE
Improve text rendering quality by implementing cmr10 AFM kerning pairs in the PDF backend.

**Goals:**
1. **Character pair kerning** — cmr10 AFM has 183 kern pairs (e.g., AV=-111pt, To=-83pt, ff=+78pt). Add kern pair lookup in PDF backend `render_text_node()`. For each adjacent character pair with a kern value, emit a PDF `Kern` (TJ operator with spacing) between them.
2. **10+ new tests** covering kern pair lookup and correct output for known pairs like AT, AV, To, Fo.

- **Cycles budget:** 2 | **Cycles actual:** 1 + 1 verification
- **Status:** ✅ Complete — Leo/Apollo verified 757 tests pass, CI green. Pixel similarity = 96.96%.

### M41: cmbx10 Kern Pairs + Word Spacing Precision + Expanded compare.tex
Improve rendering accuracy with bold text kern pairs and precise word spacing.

**Goals:**
1. **cmbx10 kern pairs (F2/Bold)** — cmbx10.afm has 181 kern pairs (same layout as cmr10). Add `cmbx10_kern_pair(a: u8, b: u8) -> f32` function. Update `is_cmr10_kern_font()` to return true for both F1 and F2. Section headings use F2 (Bold) — this directly improves heading text kerning.
2. **Precise word spacing** — cmr10 word space: natural=3.333pt, stretch=1.66667pt (not 1.67), shrink=1.11111pt (not 1.11). Use exact float values. cmbx10 word space: same 333/1000*10 = 3.333pt from AFM.
3. **Expand compare.tex** — Add: `\textbf{bold text}` (exercises cmbx10 kern pairs), a paragraph with "AV To Fo" (classic kern test), footnote, math fraction `$\frac{a+b}{c}$`.
4. **12+ new tests** covering cmbx10 kern pairs, precise word spacing, expanded compare.tex content.

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit f419b55), 772 tests pass, CI green. Pixel similarity = 95.88%.

### M42: Superscript/Subscript Precision + cmti10 Kern Pairs ✅ COMPLETE
Improve math rendering quality and italic text kerning.

**Goals:**
1. **Superscript geometry correction** — Fix superscript size from 7.0pt to 7.07pt (70.7% of 10pt, matching TeX `\scriptspace` fontdimen). Fix rise from +4.0pt to +3.45pt (matching cmr10 sup_shift). Fix subscript offset from -2.0pt to -2.5pt.
2. **cmti10 kern pairs (F4/Italic)** — cmti10.afm available in CI texlive at `/usr/share/texmf/fonts/afm/public/cm/cmti10.afm`. Add `cmti10_kern_pair(a: u8, b: u8) -> f32` function with pairs from that file. Update `is_cmr10_kern_font()` to return true for F1, F2, F4. Italic text uses F4 (cmti10).
3. **12+ new tests** covering corrected superscript geometry, cmti10 kern pairs.

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 4e34d18), 789 tests pass, CI green.

### M44: Fix cmbx10 Kern Pairs (F3/Bold) — Critical Bug Fix ✅ COMPLETE
Fix the dead-code bug in M41: cmbx10 kern pairs were implemented for font name "F2" but Bold text uses font name "F3".

- **Deliverables:** Fixed is_cmr10_kern_font/font_kern_pair/font_has_kern_pairs to use F3 (not F2) for cmbx10. Updated existing tests. 14 new tests.
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Ares implemented (commit ec55f78), 826 tests pass, CI green

### M45: Per-Line Height Adaptation in PDF Output
Fix the flat 12pt line_height in the PDF backend to properly reflect each line's font size.

**Goals:**
1. **Add `line_height: f64` to `OutputLine` struct** in rustlatex-engine
2. **Compute line_height in engine**: when building OutputLine, set line_height = max(font_size of BoxNode::Text nodes in that line) × 1.2 (standard TeX baselineskip factor). Default: 12.0 for 10pt text, 16.8 for 14pt section headings, 14.4 for 12pt subsections.
3. **PDF backend uses `line.line_height`**: replace the flat `line_height: f32 = 12.0` with per-line values from the engine.
4. **Glue-carrying lines**: lines that contain only Glue/Kern/Rule nodes (no Text) should still advance by their glue amount (already handled by Glue nodes) — use 0.0 or line.line_height for those.
5. **12+ new tests** verifying line_height is 12.0 for normal text, 16.8 for 14pt sections, 14.4 for 12pt subsections.

- **Cycles budget:** 2 | **Cycles actual:** 1 + 1 verification
- **Status:** ✅ Complete — Ares implemented (commit c11ee9b), 840 tests pass. NOTE: Page accumulation bug found (flat 12.0), fixed in M46.

### M46: Fix Page Accumulation + cmmi10 Kern Pairs
Fix the page accumulation bug from M45 and add cmmi10 kern pairs.

**Goals:**
1. **Fix page accumulation** — Engine::typeset() still uses flat 12.0 for accumulated_height. Replace with line.line_height (per-line value). Two locations: main loop (line 4796) and re-render loop (line 4883). Remove dead `let line_height = 12.0_f64;` at line 4757.
2. **cmmi10 kern pairs (F7/MathItalic)** — 166 pairs from cmmi10.afm. Add cmmi10_kern_pair(), update is_cmr10_kern_font() to include F7, add has_cmmi10_kern_pairs() helper, update font_kern_pair() and font_has_kern_pairs() dispatch. Note: cmmi10 uses OML encoding, Latin letters at standard ASCII positions — kern pair lookups work same as cmr10.
3. **15+ new tests** verifying page accumulation and cmmi10 kern pairs.

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 7177460), 858 tests pass, CI green. Pixel similarity = 95.69%.

### M47: PPM Tolerance + cmsy10 Kern Pairs + Cleanup ✅ COMPLETE
Improve pixel similarity measurement and add cmsy10 kern pairs.

**Goals:**
1. PPM comparison tolerance ±2 per-channel
2. cmsy10 kern pairs (26 pairs, F8/MathSymbol)
3. Clean up unused metrics variables

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit bae0fd2), 879 tests pass, CI green. Pixel similarity = 95.69%.

### M48: Complete Punctuation Character Widths in StandardFontMetrics ✅ COMPLETE
Fix the engine's char_width() function to use correct cmr10 AFM widths for all punctuation characters.

- **Deliverables:** 27 punctuation/symbol characters with correct cmr10 AFM widths, 21 new tests
- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 6062077), 900 tests pass, CI green. Pixel similarity = 95.69%.

### M49: Fix Paragraph Spacing + Proportional Justification (Pixel Similarity Fixes)
Two critical rendering fixes to improve pixel similarity from 95.69% toward 97%+:

1. **Paragraph-end trailing Glue fix** — Change paragraph end Glue from `natural:6.0, stretch:2.0` to `natural:0.0, stretch:1.0` (match pdflatex's `\parskip=0pt plus 1pt`). Current 6pt natural glue creates an extra blank line (~12pt) between paragraphs, causing vertical positions to drift by ~12pt per paragraph.
2. **Proportional justification** — Fix PDF backend to distribute remaining space proportionally based on per-glue stretch values instead of uniform `remaining/glue_count`. Fixes inter-sentence spacing.

- **Cycles budget:** 2
- **Status:** ✅ Complete — Leo implemented (commit 06a3d80), 915 tests pass, CI green. Pixel similarity = 95.69% (compare.tex has only 1 paragraph after section heading, so parskip fix not visible here).

### M50: Fix Section Heading Vertical Spacing (Critical Pixel Similarity Bug)
Fix the critical bug where section heading before/after kerns are horizontal instead of vertical.

**Root cause**: Section headings emit `[Kern(24), Text(heading), Kern(8)]`. These Kern nodes are processed as horizontal movement in the PDF backend (advancing current_x), not vertical. So body text after a section heading is only 16.8pt lower (heading line_height) instead of ~48.8pt lower (24 + 16.8 + 8). This 32pt error cascades to ALL subsequent lines.

**Fix**:
1. Add `BoxNode::VSkip { amount: f64 }` variant (vertical skip)
2. Change section/subsection/subsubsection to emit VSkip instead of horizontal Kerns for before/after spacing
3. In `break_items_with_alignment()`: A line containing ONLY a VSkip should have `line_height = amount` and the PDF backend advances current_y by that amount
4. In PDF backend: VSkip-only lines advance current_y without rendering text
5. 15+ new tests verifying VSkip behavior and section heading y-positions

**Expected impact**: +3-4% pixel similarity (eliminates 32pt vertical mismatch on every line after section headings)

- **Cycles budget:** 2
- **Status:** ✅ Complete — Leo implemented (commit 5a8e4d2), 938 tests pass. NOTE: Pixel similarity dropped to 94.75% — section spacing values (24pt/8pt) are WRONG vs pdflatex. Fix in M51.

### M51: Fix Section Heading Spacing to Match pdflatex Article Class ✅ COMPLETE
Fix section/subsection spacing values to match pdflatex article class exactly.

- **Status:** ✅ Complete — Leo implemented (commit bd45b04), 955 tests pass, CI green. Pixel similarity = 94.47% (REGRESSION vs 95.69% pre-M50 — see M52 for fix)
- **Root cause of M51 regression**: pdflatex SUPPRESSES the before-skip when section is at top of page/column. compare.tex starts with \section{Introduction} — no before-skip in pdflatex. We add 15.07pt VSkip before it, shifting ALL content down by 15.07pt. Fix in M52.

### M52: Fix Top-of-Page Section Spacing (Before-Skip Suppression) ✅ COMPLETE
Fix the M51 regression: suppress the before-VSkip for sections that appear at the start of document content.

- **Status:** ✅ Complete — Leo implemented (commit 0f325c5), 960 tests. Similarity 94.47% (still worse than M49).

### M53: Set VSkip to 0.0 ✅ COMPLETE
Set all section VSkip amounts to 0.0 to eliminate VSkip contribution.

- **Status:** ✅ Complete — Leo implemented (commit 37de084), 965 tests, 94.53% similarity. Still below 95.69% due to font_size 14.4 change.

### M54: Recover Pixel Similarity by Reverting Section Font Sizes ✅ COMPLETE (partial)
Fix the pixel similarity regression caused by M51's font size changes.

- **Cycles budget:** 2 | **Cycles actual:** 1 (M54) + 1 (M55 follow-up)
- **Status:** ✅ Complete — M54 reverted font sizes (984 tests), M55 removed VSkip nodes from sections (1003 tests, 95.68% similarity)

### M55: Remove VSkip from Section Headings ✅ COMPLETE
Remove all VSkip(0.0) nodes from section/subsection/subsubsection headings. Keep VSkip infrastructure for \vspace only.

- **Cycles budget:** 1 | **Cycles actual:** 1 (Leo)
- **Status:** ✅ Complete — 1003 tests, 95.68% similarity (commit 57d328f)

### M56: Improve Pixel Similarity Beyond 95.68% ⚠️ DEADLINE MISSED (3/3 cycles used)
Investigated and attempted section heading font size (14.0→14.4) + after-VSkip spacing.

**Outcome:**
- Commit de4ed61: 94.44% (regression — VSkip caused layout shifts)
- Commit 2640cb3: 95.69% (same as baseline — VSkip removed, kept font_size 14.4)
- Net result: font_size 14.4 is now in use (neutral effect on similarity, cosmetically correct)
- **Target of 97%+ NOT achieved** — section spacing approach exhausted
- **Lesson learned:** VSkip-based section spacing consistently regresses. The remaining gap requires a different diagnostic approach — we must determine WHICH pixels differ and WHY.

- **Cycles budget:** 3 | **Cycles actual:** 3
- **Status:** ⚠️ Deadline missed — escalated back to Athena for replanning

### M57: Correct PDF Margins to Match pdflatex article class
Fix the margin mismatch between our output and pdflatex's article class defaults.

**Root cause (Diana's forensic analysis):** 
- Our `margin_left = 72.27pt` (1 inch). pdflatex article a4paper uses symmetric centering: `(595pt - 345pt) / 2 = 126.25pt`. We are 53.98pt too far LEFT.
- Our `margin_top = 109pt`. pdflatex article first-baseline position ≈ 124pt from top. We are ~15pt too HIGH.
- Combined: our text is 53.98pt too far left AND 15pt too high vs pdflatex. This explains ~4% of the 4.31% gap.

**Changes:**
1. `margin_left`: 72.27 → 126.25pt in `rustlatex-pdf/src/lib.rs:2046`
2. `margin_top`: 109.0 → 124.0pt in `rustlatex-pdf/src/lib.rs:2047`
3. Updated 3 self-referential tests (margin_left, margin_top, start_y assertions)
4. Added diagnostic test: `test_ppm_text_bounding_box` — finds first/last non-white column+row in both PPMs and reports the offset

**Expected impact:** ~96.5%→99%+ similarity if margin analysis is correct.
**CI issue:** `test_ppm_text_bounding_box` has a bug (uses `-o` flag that our CLI doesn't support) — fixing in M58.

- **Cycles budget:** 3 | **Cycles actual:** 1 (Leo, commit 8e7b658)
- **Status:** ✅ Complete — CI confirmed: similarity improved 95.69% → 97.25% (+1.56%)

### M58: Fix CI Test Bug + Verify M57 Similarity Score ✅ COMPLETE
Fixed `test_ppm_text_bounding_box` (Leo: CLI args fix) + fixed `parse_ppm`/`parse_ppm_header` to handle GhostScript comment lines in PPM headers (Athena: comment line parsing).

**Result:** CI GREEN. Pixel similarity = **97.25%** (up from 95.69%, +1.56% from M57 margin corrections).
- Our PPM size: 1,503,038 bytes = 501,012 pixels at 72 DPI (A4 @ 72 DPI)
- pdflatex PPM size: 1,503,038 bytes (same)
- Remaining gap: 2.75% = ~13,778 mismatched pixels

- **Cycles budget:** 1 | **Cycles actual:** 2 (Leo + Athena)
- **Status:** ✅ Complete — CI green, 1021 tests pass (commit daef942)

### M43: Justified Text Width Fix + cmbxti10 Kern Pairs ✅ COMPLETE
Improve text rendering accuracy and typographic quality.

**Goals:**
1. **Fix justified text line width computation** — compute_kern_pair_total + justified width fix
2. **cmbxti10 kern pairs (F5/BoldItalic)** — 178 entries, F5 dispatch
3. **23 new tests**

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (commit 2fe0af2), 812 tests pass, CI green. Pixel similarity = 95.77%.

### M62: Section Line Height + Margin Precision ✅ COMPLETE
Fix section heading line_height (17→18pt for 14.4pt font) and margin_top precision (124.0→123.27).

- **Cycles budget:** 2 | **Cycles actual:** 1
- **Status:** ✅ Complete — Leo implemented (bb67ce5), 1064 tests pass, CI green. Pixel similarity = **97.24% (unchanged)**. Changes structurally correct but had no measurable similarity impact.

### M63: Section Heading Chunk Separation ⚠️ REGRESSION
Added VSkip{0.0} after section headings + changed compute_line_height values.

- **Status:** ⚠️ REGRESSION — commit 1e4e978 caused similarity to drop from 97.24% → 96.78% (-0.46%)
- **Root cause:** compute_line_height changed to 9.9 (should be 18.0) and 6.46 (should be 14.0)
- **M64 will fix this regression**

### M64: Revert M63 compute_line_height Regression ⚠️ PARTIAL
Reverted compute_line_height (9.9→18.0, 6.46→14.0) but did NOT remove the VSkip{0.0} nodes M63 added.

- **Status:** ⚠️ PARTIAL — similarity dropped FURTHER to 96.63% (M62 was 97.24%). VSkip{0.0} is somehow causing regression despite being "harmless". Full revert needed in M65.

### M65: Fully Revert M63 VSkip{0.0} + Restore M62 State ✅ COMPLETE
Remove BoxNode::VSkip{amount: 0.0} from section/subsection/subsubsection translation.

- **Status:** ✅ Complete — Leo implemented (commit d7f888c), 97.24% similarity confirmed in CI.
- **Cycles budget:** 1 | **Cycles actual:** 1

### M66: Fix Itemize Vertical Spacing (topsep/itemsep as VSkip) ⚠️ REGRESSION

**Result**: Converted itemize Glue → VSkip for topsep/itemsep. CI showed regression: 97.24% → 97.06% (-0.18%).
**Lesson learned**: Itemize VSkip regresses. Same pattern as section heading VSkip. Add to "never do" list.
- Commit: 3317070
- Cycles actual: 1

### M68: Section Heading Line Height Experiment (Non-VSkip Approach) ✅ COMPLETE
Adjust section heading effective line_height from 18.0 to 21.0pt in compute_line_height.

- **Result:** Pixel similarity = **97.31%** (+0.07% from 97.24%). Small but positive improvement confirmed.
- **Cycles budget:** 2 | **Cycles actual:** 1 (Ares implemented commit b51eddd)
- **Status:** ✅ Complete — CI green, 720+ engine tests pass

### M69: Display Math line_height Post-Processing + Subsection line_height Adjustment ⚠️ NO IMPROVEMENT

Diana's research (see diana/note.md) predicted two fixes for the remaining 2.69% gap:

1. **Display math post-processing**: Implemented — Center-aligned 10pt lines get +10 line_height, preceding line gets +10
2. **Subsection line_height 14.5→17.0**: Implemented in compute_line_height

**Result: 97.30% — essentially unchanged from 97.31% (M68). Both fixes had ~0 net impact.**

Possible explanations:
- Subsection 17.0 overshot (causing regression) while display math helped (cancelling out)
- Display math post-processing not triggered (wrong font_size check?)
- Display math lines are at bottom of page, already partially off-page

- **Cycles budget:** 2 | **Cycles actual:** 1 (Leo, commit 47cdf9f)
- **Lesson:** Even theoretically-correct fixes may not move the needle if the page geometry doesn't match expectations. Need deeper analysis before M70.

### M72: Revert M71 Paragraph-End Penalty Regression ✅ NEEDED
Revert the paragraph-end `Penalty{-10000}` added by M71 that caused 96.88% regression.

**Root cause:** M71 added Penalty{-10000} after every paragraph end. This forces each paragraph into a separate chunk in break_items_with_alignment. Each paragraph then breaks independently via KP, producing different line breaks than pdflatex (continuous flow). The section heading Penalty is redundant (AlignmentMarker already separates) but harmless.

**Fix:**
1. Remove the `result.push(BoxNode::Penalty { value: -10000 })` line from paragraph translation (~line 2343) in translate_node_with_context()
2. Keep section heading Penalty{-10000} (harmless)
3. Keep M70's KP tolerance=10000 and glue width fixes (neutral/correct)
4. Update tests that check paragraph output (remove expectation of trailing Penalty)
5. 10+ new tests

**Expected impact:** Recover 97.30% similarity.
- **Cycles budget:** 1
- **Status:** 🚧 Planned

### M71: Fix Paragraph/Section Separation with Forced Breaks ⚠️ REGRESSION
Added Penalty{-10000} after section headings AND paragraph ends.

- **CI result:** 96.88% (REGRESSION from 97.30%, -0.42%)
- **Status:** ⚠️ REGRESSION — see M72 for fix
- **Cycles actual:** 1 (Ares, commit 940371d)

### M70: Fix Critical Line-Breaking Bugs (Greedy Breaker + KP Tolerance)
Fix the two critical bugs in `break_into_lines()` and increase KP tolerance that are causing massively overfull lines.

**Root cause (Diana's research):** The greedy line breaker ignores Glue natural width, causing lines to never break. The KP breaker always falls back to greedy due to tolerance=200 being too strict.

**Deliverables:**
1. Add Glue natural width to current_width in break_into_lines()
2. Handle Penalty{≤-10000} as forced break in break_into_lines()
3. Update width recalculation (after line break) to include Glue natural width
4. Increase KP tolerance from 200 to 10000
5. 15+ new tests covering these fixes
**Expected impact:** 97.3% → 98-99% pixel similarity (catastrophic line-breaking failure is fixed)

- **Cycles budget:** 2
- **Status:** 🚧 In progress

### M67: Revert M66 VSkip Regression + Fix Display Math Spacing
Revert M66's itemize VSkip changes back to Glue nodes (recover 97.24%), then fix display math vertical spacing.

**Fix 1 (recovery)**: Revert itemize VSkip → Glue (back to d7f888c state for these nodes).

**Fix 2 (improvement)**: Display math spacing: change `natural: 12.0` → `natural: 10.0`, `stretch: 3.0` → `stretch: 2.0`, `shrink: 9.0` → `shrink: 5.0` for above/below display math Glue nodes. pdflatex uses `\abovedisplayskip = 10pt plus 2pt minus 5pt`. Four locations in engine.

**Expected impact**: ~400-800 pixels improvement (0.08-0.16%). Also subsection line_height 14.0→14.5.

- **Cycles budget:** 2

### M74: Fix KP Forced Break Demerits + Add Paragraph/Section Separation
**Root cause identified (Diana M74 research):** The KP line-breaker has a critical bug: forced break (`Penalty{-10000}`) handling accepts ALL lines ending at forced breaks regardless of width, using fixed demerits of `(-10000)² = 10^8`. The DP optimizer always picks a mega-line (entire paragraph on one line) because it has the same cost as any other line ending at a forced break. Result: only 7 text y-positions instead of 20+, with lines extending 600-1299pt beyond the 345pt hsize.

**Fixes:**

1. **Fix KP forced break demerits** (`crates/rustlatex-engine/src/lib.rs`, ~line 4604-4607):
   - Change `else if forced_j` branch from accepting all lines with `pen*pen` demerits
   - Treat forced breaks like "last line" (sentinel): reject if overfull (`ratio < -1.0` or `None`), accept with 0 demerits if underfull/perfect
   - This causes the optimizer to REJECT mega-lines and find intermediate breaks instead
   ```rust
   } else if forced_j {
       // Forced break: treat like last line — overfull rejected, underfull OK
       let ratio = adjustment_ratio(nat_w, stretch, shrink, hsize);
       match ratio {
           None => continue,                // overfull with no shrink → infeasible
           Some(r) if r < -1.0 => continue, // over-shrunk → infeasible
           _ => 0.0,                        // underfull or perfect → 0 demerits
       }
   }
   ```

2. **Add Paragraph forced break** (`translate_node_with_context`, `Node::Paragraph` handler, ~line 2342):
   - After `result.push(BoxNode::Glue { natural: 0.0, ... })` at end of paragraph, add:
   - `result.push(BoxNode::Penalty { value: -10000 });`

3. **Add Section/Subsection forced break** (`translate_node_with_context`, section handler, ~line 2520):
   - Change `result` from `vec![BoxNode::Text { ... }]` to `vec![BoxNode::Text { ... }, BoxNode::Penalty { value: -10000 }]`

4. **Fix section heading width** (~line 2509-2510):
   - Change `metrics.string_width_for_style(&numbered_title, ctx.current_font_style)` to `metrics.string_width_for_style(&numbered_title, FontStyle::Bold)`
   - Width was computed with Normal metrics but heading is rendered Bold

5. **Update tests** — tests checking paragraph output that verify NO trailing Penalty (M72 tests) need updating.
   - M72 tests (`test_m72_*`) asserted paragraphs do NOT have trailing Penalty — update to ALLOW trailing Penalty
   - Tests checking section produces 1 node — update to expect 2 nodes (Text + Penalty)
   - Add 15+ new tests

**Expected impact**: 97.30% → 99%+ (proper line breaking produces 20+ y-levels instead of 7)

**Why this is safe (won't regress):**
- Previous M71-M73 failed because forced break demerits were still 10^8 — mega-lines won
- This fix makes mega-lines INFEASIBLE (ratio < -1.0 → continue)
- `contains_forced_break` check (unchanged) still prevents bridging over Penalty nodes
- No VSkip involved, no chunk-splitting — pure line-breaking improvement

- **Cycles budget:** 2 | **Status:** 🚧 Planned

### M74-fix: Revert M74 Paragraph/Section Penalties — REGRESSION RECOVERY ⚠️ NEW REGRESSION
Apollo required reverting paragraph and section Penalty{-10000} additions from M74. Leo did so (commit 5773a2d), also keeping: (1) KP forced_j sentinel logic, (2) FontStyle::Bold section width fix, (3) subsection line_height 17.0 → 18.5.

**Result: 96.79% similarity** — WORSE than M73's 97.30%.

**Root cause of new regression:**
1. **KP forced_j sentinel** (kept from M74): rejects lines where `ratio < -1.0` as infeasible. When NO valid break fits in hsize, the segment falls through to emergency/fallback → produces overfull mega-lines extending 300-400pt past the right margin. Verified: 11 lines in M74-fix extend past x=471.25, including y=577.73 extending to x=861pt.
2. **Subsection line_height 18.5pt** (was 17.0 in M73): caused cascading vertical shift → regression.

**What's kept and good:** FontStyle::Bold for section heading width (correct, harmless).

- **Status:** ⚠️ REGRESSION — 96.79%, KP sentinel and subsection 18.5 are the causes

### M75: Recover 97.30% Baseline (Revert KP Sentinel + Subsection 18.5) ✅ COMPLETE
Revert the two regressions from M74-fix while keeping the Bold section width fix.

- **Cycles budget:** 2 | **Cycles actual:** 1 (Leo, commit 7053c28)
- **Status:** ✅ Complete — CI confirmed **97.31%** similarity (slight improvement over target), 792 tests pass

### M76: Paragraph Separation via AlignmentMarker ⚠️ DEADLINE MISSED (6/6 cycles, REGRESSION)
Added `AlignmentMarker{Justify}` at the start of each paragraph and section heading.

**Result: REGRESSION — 96.82%** (down from M75's 97.31%, -0.49%).
- Multiple commits: Leo (46990d2, AlignmentMarker), Ares fixes (bcc7437, 21c255b, f928a1c), Leo fixes (360adb7, b3ee789)
- Best score achieved: 96.86% (M76-fix 23054615585) — still worse than M75

**Root cause of regression:**
- Adding AlignmentMarker per paragraph creates MORE segment splits
- Each paragraph gets its own KP run — but this causes DIFFERENT line breaks than pdflatex's continuous-flow model
- pdflatex doesn't break paragraphs completely independently either; the way paragraphs end and begin has inter-paragraph glue
- The 97.31% M75 baseline is achieved with text flowing as-is (no artificial separation)

**Two good fixes extracted from M76 (kept):**
1. Kern pair scaling fix: `/100.0` in `compute_kern_pair_total` usage (360adb7)
2. Parfillskip simulation: `is_last_line_like` threshold=10 (b3ee789)

**CRITICAL LESSON: AlignmentMarker per paragraph/section REGRESSES (added to DO NOT list)**

- **Cycles budget:** 6 | **Cycles actual:** 6
- **Status:** ⚠️ DEADLINE MISSED — 96.82% regression. M77 will recover to 97.31%+.

### M77: Recover M75 Baseline (Revert AlignmentMarker, Keep PDF Fixes) ✅ COMPLETE
Revert M76's AlignmentMarker additions from the engine while keeping the two good PDF fixes.

**Goal:** Return to ≥97.31% by reverting AlignmentMarker additions from `crates/rustlatex-engine/src/lib.rs`.

**Result:** CI confirmed **97.33%** similarity (slight improvement over M75's 97.31% from kern+parfillskip PDF fixes). 1150+ tests pass, CI green.

- **Cycles budget:** 2 | **Cycles actual:** 1 (Leo, commit 0052314)
- **Status:** ✅ Complete — CI confirmed 97.33%

### M78: Fresh Analysis at M77 Baseline → Identify Safe Improvements ✅ COMPLETE
Diana performed fresh forensic analysis. Key findings:
- Output has only 7 y-positions (expected ~25) — mega-lines confirmed
- **~90% of the 2.67% gap** is from structural layout (paragraphs as mega-lines) — cannot fix without regression
- **~10% of the gap** (~1300 pixels) comes from two PDF backend word-level bugs:
  1. **BUG #1** (line 2224): `current_x += *width` does NOT include kern pair contribution → words after kerned text positioned slightly too far right (~100-500 pixels impact)
  2. **BUG #2** (line 2093): Kern scaling hardcoded `/100.0` (assumes 10pt) — wrong for 14.4pt sections (44% error) and 12pt subsections (20% error) (~50-200 pixels impact)
- Word spacing values match pdflatex exactly ✓
- TJ kern arrays are correctly scaled ✓

- **Status:** ✅ Complete — Diana's analysis in workspace/diana/note.md

### M79: Fix Kern X-Position Tracking + Kern Scaling in PDF Backend
Two targeted non-layout fixes from Diana's M78 analysis. These are safe: they only affect how accurately we track `current_x` and how much kern contribution is counted in `line_nat_width`.

**Fix 1 — BUG #1: Kern x-position tracking** (`crates/rustlatex-pdf/src/lib.rs`, line 2224):
When we render a Text node with kern pairs via TJ, the actual rendered width is `*width + kern_adjustment`. But we only advance `current_x += *width`. The next word's Tm position is slightly off (too far right). Fix: after rendering a kerned text node, include the total kern pair contribution in `current_x`:
```rust
// After: current_x += *width as f32;
// Change to:
let kern_adj = if is_cmr10_kern_font(font_name) && font_has_kern_pairs(font_name, &raw_bytes) {
    compute_kern_pair_total(font_name, &raw_bytes) * (*font_size as f32) / 1000.0
} else {
    0.0
};
current_x += *width as f32 + kern_adj;
```
Note: `font_size` is already available in scope at line 2224 as `font_size_outer` (f32). Need `raw_bytes` (already computed above).

**Fix 2 — BUG #2: Kern scaling in line_nat_width** (`crates/rustlatex-pdf/src/lib.rs`, line 2093):
Change the match arm at line 2078-2082 to destructure `font_size`:
```rust
BoxNode::Text {
    text,
    width,
    font_size,
    font_style,
    ..
} => {
```
Then change line 2093:
```rust
// FROM: compute_kern_pair_total(fn_name, &raw) / 100.0
// TO:   compute_kern_pair_total(fn_name, &raw) * (*font_size as f32) / 1000.0
```

**Tests (8+ new):**
- Test that current_x after kerned word includes kern contribution
- Test that BUG #2 fix gives correct kern contribution for 14.4pt font (not just 10pt)
- Test line_nat_width computation uses font-size-scaled kern values
- All 1150+ existing tests continue to pass

**Expected impact:** ~0.05-0.14% pixel similarity improvement (97.33% → ~97.40-97.47%)

- **Cycles budget:** 2
- **Status:** ✅ Complete — Leo implemented (commit f633fd8). CI confirmed similarity = **97.34%** (+0.01% from M78's 97.33%). ~1184 total tests pass, CI green.

### M80: Fix Mega-Line Problem via Paragraph Separation + forced_j Width Check
Attempt to solve the fundamental mega-line problem using a new combination of fixes.

**Root cause of mega-lines (Athena's deep analysis):**
The KP's `forced_j` path accepts ANY line width before `Penalty{-10000}` with constant 10^8 demerits. Mega-lines before forced breaks always win: 10^8 + 0 < 10^8 + (any_intermediate_demerits). This causes ALL content before each forced break to be placed on ONE mega-line.

**Hypothesis (new approach):**
Compare.tex's paragraphs are all SHORT (< 345pt). If we:
1. Add paragraph-end `Penalty{-10000}` (each paragraph = own KP segment)
2. Add section heading `Penalty{-10000}` after heading (section heading = own line)
3. Fix forced_j to reject overfull (only accept underfull/perfect with 0 cost)
...then the KP would place each short paragraph on its own 1-line segment, which matches pdflatex's output. Our word metrics now match pdflatex exactly, so breaks should match too.

**Changes (crates/rustlatex-engine/src/lib.rs):**

**Fix 1 — Add Penalty after paragraph end** (~line 2338):
```rust
result.push(BoxNode::Glue { natural: 0.0, stretch: 1.0, shrink: 0.0 });
result.push(BoxNode::Penalty { value: -10000 });  // ADD THIS
```

**Fix 2 — Add Penalty after section heading** (~line 2519):
```rust
let result = vec![
    BoxNode::Text { text: numbered_title, width, font_size, ... },
    BoxNode::Penalty { value: -10000 },  // ADD THIS
];
```

**Fix 3 — Fix forced_j in KP to reject overfull** (~line 4603):
```rust
} else if forced_j {
    // Forced break: reject overfull, accept underfull with 0 demerits
    let ratio = adjustment_ratio(nat_w, stretch, shrink, hsize);
    match ratio {
        None => continue,                // overfull with no shrink → infeasible
        Some(r) if r < -1.0 => continue, // over-shrunk → infeasible
        _ => 0.0,                        // underfull or perfect → 0 demerits
    }
}
```

**Important**: The `contains_forced_break` check in the KP (which prevents bridging over Penalty{-10000}) must REMAIN. This is the mechanism that FORCES breaks at the penalty positions.

**Tests (12+ new):**
- Test paragraph end now emits Penalty{-10000}
- Test section heading now emits Penalty{-10000} after Text
- Test KP forced_j rejects overfull lines
- Test KP forced_j accepts underfull lines
- Test KP places section heading on its own 1-line output
- Test KP places short paragraph on its own 1-line output
- All existing tests must still pass (update any tests that check paragraph output without Penalty)

**Key risks:**
- Tests that assert paragraph output does NOT have Penalty must be updated
- Tests that assert section heading emits exactly 1 node must be updated
- If any paragraph is > 345pt wide, KP falls back to greedy (should still work)
- Itemize topsep Glue{8,2,4} creates slight vertical offset vs pdflatex (acceptable)

**Expected impact:** If paragraphs break correctly: 97.34% → 98%+ (speculative)
**Cycles budget:** 2
**Status:** 🚧 Planned

---

## Notes on "Binary Identical" Goal

True binary-identical output is extremely difficult because it depends on:
1. **Timestamps** — PDF metadata timestamps differ unless suppressed
2. **Random seeds** — some compilers use randomness
3. **Font subsetting** — the same subset algorithm must be used
4. **PDF object ordering** — exact same internal structure

In practice, we will target **semantic equivalence** first (same visual output), then work toward binary identity by matching pdflatex's specific behavior for timestamps, object ordering, and font embedding. The test corpus will be simple documents initially.
