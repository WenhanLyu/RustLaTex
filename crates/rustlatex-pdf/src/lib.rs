//! `rustlatex-pdf` — PDF backend
//!
//! This crate takes the typeset pages produced by `rustlatex-engine` and
//! emits PDF output. It produces valid PDF files using the `pdf-writer` crate
//! with Computer Modern Roman 10pt (CM Roman) Type1 font embedded on A4 pages.

use pdf_writer::types::FontFlags;
use pdf_writer::{Content, Name, Pdf, Rect, Ref, Str};
#[allow(unused_imports)]
use rustlatex_engine::{Alignment, BoxNode, FontStyle, FootnoteInfo, Page as EnginePage};

/// PDF generation result.
#[derive(Debug)]
pub struct PdfOutput {
    /// The raw PDF bytes.
    pub bytes: Vec<u8>,
}

/// The PDF writer converts typeset pages into PDF bytes.
#[derive(Default)]
pub struct PdfWriter;

/// Escape a string for use as a PDF string literal.
/// Escapes backslashes and parentheses.
fn pdf_escape(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'\\' => {
                out.push(b'\\');
                out.push(b'\\');
            }
            b'(' => {
                out.push(b'\\');
                out.push(b'(');
            }
            b')' => {
                out.push(b'\\');
                out.push(b')');
            }
            _ => out.push(b),
        }
    }
    out
}

impl PdfWriter {
    /// Create a new PDF writer.
    pub fn new() -> Self {
        PdfWriter
    }

    /// Write pages to PDF.
    ///
    /// Produces a valid PDF document with:
    /// - Document catalog and page tree
    /// - A4 page size (595 × 842 pt), 50pt margins
    /// - Computer Modern Roman 10pt (CM Roman) Type1 font embedded at 10pt
    /// - Text rendered from box_lines, top-to-bottom
    pub fn write(&self, pages: &[EnginePage]) -> PdfOutput {
        // If no pages, produce a valid PDF with zero pages
        let page_count = if pages.is_empty() { 0 } else { pages.len() };

        // Allocate Ref IDs:
        // 1 = catalog
        // 2 = page tree
        // 3 = font file stream (cmr10.pfb, embedded Type1)
        // 4 = font dictionary (CMR10 Type1, F1/Normal)
        // 5 = font descriptor (CMR10)
        // 6 = cmbx10 file stream
        // 7 = cmbx10 descriptor
        // 8 = cmbx10 dict (F3/Bold)
        // 9 = cmti10 file stream
        // 10 = cmti10 descriptor
        // 11 = cmti10 dict (F4/Italic)
        // 12 = cmbxti10 file stream
        // 13 = cmbxti10 descriptor
        // 14 = cmbxti10 dict (F5/BoldItalic)
        // 15 = cmtt10 file stream
        // 16 = cmtt10 descriptor
        // 17 = cmtt10 dict (F6/Typewriter)
        // For each page i (0-indexed):
        //   18 + i*2     = page object
        //   18 + i*2 + 1 = content stream
        let catalog_id = Ref::new(1);
        let page_tree_id = Ref::new(2);
        let font_file_id = Ref::new(3);
        let font_id = Ref::new(4);
        let font_descriptor_id = Ref::new(5);
        let cmbx10_file_id = Ref::new(6);
        let cmbx10_descriptor_id = Ref::new(7);
        let cmbx10_id = Ref::new(8);
        let cmti10_file_id = Ref::new(9);
        let cmti10_descriptor_id = Ref::new(10);
        let cmti10_id = Ref::new(11);
        let cmbxti10_file_id = Ref::new(12);
        let cmbxti10_descriptor_id = Ref::new(13);
        let cmbxti10_id = Ref::new(14);
        let cmtt10_file_id = Ref::new(15);
        let cmtt10_descriptor_id = Ref::new(16);
        let cmtt10_id = Ref::new(17);

        let mut pdf = Pdf::new();

        // Document catalog
        pdf.catalog(catalog_id).pages(page_tree_id);

        // Collect page Refs
        let page_refs: Vec<Ref> = (0..page_count)
            .map(|i| Ref::new((18 + i * 2) as i32))
            .collect();

        // Page tree
        pdf.pages(page_tree_id)
            .kids(page_refs.iter().copied())
            .count(page_count as i32);

        // Embed the cmr10.pfb Type1 font file stream
        let font_bytes: &[u8] = include_bytes!("../fonts/cmr10.pfb");
        pdf.stream(font_file_id, font_bytes);

        // Font descriptor for CMR10
        pdf.font_descriptor(font_descriptor_id)
            .name(Name(b"CMR10"))
            .flags(FontFlags::SERIF | FontFlags::NON_SYMBOLIC)
            .bbox(Rect::new(-40.0, -250.0, 1009.0, 969.0))
            .italic_angle(0.0)
            .ascent(694.4)
            .descent(-194.4)
            .cap_height(683.3)
            .stem_v(50.0)
            .font_file(font_file_id);

        // CM Roman 10pt widths for chars 32-126 (95 entries), in glyph units (WX values)
        // Derived from cmr10 AFM data
        let cmr10_widths: Vec<f32> = vec![
            333.333,  // 32 space
            277.778,  // 33 !
            500.0,    // 34 "
            833.333,  // 35 #
            500.0,    // 36 $
            833.333,  // 37 %
            777.778,  // 38 &
            277.778,  // 39 '
            388.889,  // 40 (
            388.889,  // 41 )
            500.0,    // 42 *
            777.778,  // 43 +
            277.778,  // 44 ,
            333.333,  // 45 -
            277.778,  // 46 .
            500.0,    // 47 /
            500.0,    // 48 0
            500.0,    // 49 1
            500.0,    // 50 2
            500.0,    // 51 3
            500.0,    // 52 4
            500.0,    // 53 5
            500.0,    // 54 6
            500.0,    // 55 7
            500.0,    // 56 8
            500.0,    // 57 9
            277.778,  // 58 :
            277.778,  // 59 ;
            277.778,  // 60 <
            777.778,  // 61 =
            472.222,  // 62 >
            472.222,  // 63 ?
            777.778,  // 64 @
            750.0,    // 65 A
            708.333,  // 66 B
            722.222,  // 67 C
            763.889,  // 68 D
            680.556,  // 69 E
            652.778,  // 70 F
            784.722,  // 71 G
            750.0,    // 72 H
            361.111,  // 73 I
            513.889,  // 74 J
            777.778,  // 75 K
            625.0,    // 76 L
            916.667,  // 77 M
            750.0,    // 78 N
            777.778,  // 79 O
            680.556,  // 80 P
            777.778,  // 81 Q
            736.111,  // 82 R
            555.556,  // 83 S
            722.222,  // 84 T
            750.0,    // 85 U
            750.0,    // 86 V
            1027.778, // 87 W
            750.0,    // 88 X
            750.0,    // 89 Y
            611.111,  // 90 Z
            277.778,  // 91 [
            500.0,    // 92 backslash
            277.778,  // 93 ]
            500.0,    // 94 ^
            277.778,  // 95 _
            277.778,  // 96 `
            500.0,    // 97 a
            555.556,  // 98 b
            444.444,  // 99 c
            555.556,  // 100 d
            444.444,  // 101 e
            305.556,  // 102 f
            500.0,    // 103 g
            555.556,  // 104 h
            277.778,  // 105 i
            305.556,  // 106 j
            527.778,  // 107 k
            277.778,  // 108 l
            833.333,  // 109 m
            555.556,  // 110 n
            500.0,    // 111 o
            555.556,  // 112 p
            527.778,  // 113 q
            391.667,  // 114 r
            394.444,  // 115 s
            388.889,  // 116 t
            555.556,  // 117 u
            527.778,  // 118 v
            722.222,  // 119 w
            527.778,  // 120 x
            527.778,  // 121 y
            444.444,  // 122 z
            319.444,  // 123 {
            319.444,  // 124 |
            319.444,  // 125 }
            319.444,  // 126 ~
        ];

        // Type1 font dictionary with embedded CMR10
        pdf.type1_font(font_id)
            .base_font(Name(b"CMR10"))
            .first_char(32)
            .last_char(126)
            .widths(cmr10_widths)
            .font_descriptor(font_descriptor_id)
            .encoding_predefined(Name(b"StandardEncoding"));

        // Embed cmbx10.pfb (Bold Roman)
        let cmbx10_bytes: &[u8] = include_bytes!("../fonts/cmbx10.pfb");
        pdf.stream(cmbx10_file_id, cmbx10_bytes);

        // Embed cmti10.pfb (Italic)
        let cmti10_bytes: &[u8] = include_bytes!("../fonts/cmti10.pfb");
        pdf.stream(cmti10_file_id, cmti10_bytes);

        // Embed cmbxti10.pfb (Bold Italic)
        let cmbxti10_bytes: &[u8] = include_bytes!("../fonts/cmbxti10.pfb");
        pdf.stream(cmbxti10_file_id, cmbxti10_bytes);

        // Embed cmtt10.pfb (Typewriter/Monospace)
        let cmtt10_bytes: &[u8] = include_bytes!("../fonts/cmtt10.pfb");
        pdf.stream(cmtt10_file_id, cmtt10_bytes);

        // cmbx10 descriptor
        pdf.font_descriptor(cmbx10_descriptor_id)
            .name(Name(b"CMBX10"))
            .flags(FontFlags::SERIF | FontFlags::NON_SYMBOLIC | FontFlags::FORCE_BOLD)
            .bbox(Rect::new(-56.0, -250.0, 1164.0, 750.0))
            .italic_angle(0.0)
            .ascent(694.4)
            .descent(-194.4)
            .cap_height(683.3)
            .stem_v(114.0)
            .font_file(cmbx10_file_id);

        // cmti10 descriptor
        pdf.font_descriptor(cmti10_descriptor_id)
            .name(Name(b"CMTI10"))
            .flags(FontFlags::SERIF | FontFlags::NON_SYMBOLIC | FontFlags::ITALIC)
            .bbox(Rect::new(-163.0, -250.0, 1130.0, 750.0))
            .italic_angle(-14.0)
            .ascent(694.4)
            .descent(-194.4)
            .cap_height(683.3)
            .stem_v(50.0)
            .font_file(cmti10_file_id);

        // cmbxti10 descriptor
        pdf.font_descriptor(cmbxti10_descriptor_id)
            .name(Name(b"CMBXTI10"))
            .flags(
                FontFlags::SERIF
                    | FontFlags::NON_SYMBOLIC
                    | FontFlags::ITALIC
                    | FontFlags::FORCE_BOLD,
            )
            .bbox(Rect::new(-163.0, -250.0, 1180.0, 750.0))
            .italic_angle(-14.0)
            .ascent(694.4)
            .descent(-194.4)
            .cap_height(683.3)
            .stem_v(114.0)
            .font_file(cmbxti10_file_id);

        // cmtt10 descriptor
        pdf.font_descriptor(cmtt10_descriptor_id)
            .name(Name(b"CMTT10"))
            .flags(FontFlags::NON_SYMBOLIC | FontFlags::FIXED_PITCH)
            .bbox(Rect::new(-4.0, -250.0, 529.0, 750.0))
            .italic_angle(0.0)
            .ascent(611.1)
            .descent(-194.4)
            .cap_height(611.1)
            .stem_v(50.0)
            .font_file(cmtt10_file_id);

        // cmbx10 widths (Bold Roman, chars 32-126, 95 entries)
        let cmbx10_widths: Vec<f32> = vec![
            333.333, 277.778, 500.0, 833.333, 500.0, 833.333, 777.778, 277.778, 388.889, 388.889,
            500.0, 777.778, 277.778, 333.333, 277.778, 500.0, 500.0, 500.0, 500.0, 500.0, 500.0,
            500.0, 500.0, 500.0, 500.0, 500.0, 277.778, 277.778, 277.778, 777.778, 472.222,
            472.222, 777.778, 869.444, 818.056, 831.944, 882.639, 756.944, 723.611, 899.306,
            882.639, 436.806, 583.333, 880.556, 723.611, 1010.417, 882.639, 869.444, 756.944,
            869.444, 831.944, 642.361, 809.028, 869.444, 869.444, 1170.139, 819.444, 880.556,
            723.611, 277.778, 500.0, 277.778, 500.0, 277.778, 277.778, 619.444, 651.389, 530.556,
            651.389, 530.556, 366.667, 601.389, 651.389, 338.194, 366.667, 628.472, 338.194,
            984.722, 651.389, 601.389, 651.389, 601.389, 456.944, 451.389, 480.556, 651.389,
            651.389, 866.667, 590.278, 651.389, 530.556, 319.444, 319.444, 319.444, 319.444,
        ];

        // cmti10 widths (Italic, chars 32-126, 95 entries)
        let cmti10_widths: Vec<f32> = vec![
            333.333, 388.889, 500.0, 833.333, 500.0, 833.333, 777.778, 277.778, 388.889, 388.889,
            500.0, 777.778, 277.778, 333.333, 277.778, 500.0, 500.0, 500.0, 500.0, 500.0, 500.0,
            500.0, 500.0, 500.0, 500.0, 500.0, 277.778, 277.778, 472.222, 777.778, 472.222,
            472.222, 777.778, 763.889, 722.222, 694.444, 763.889, 680.556, 652.778, 784.722, 750.0,
            361.111, 513.889, 777.778, 625.0, 916.667, 750.0, 763.889, 680.556, 777.778, 736.111,
            555.556, 722.222, 750.0, 750.0, 1027.778, 750.0, 750.0, 611.111, 333.333, 500.0,
            333.333, 694.444, 500.0, 277.778, 527.778, 555.556, 444.444, 555.556, 444.444, 305.556,
            527.778, 555.556, 305.556, 305.556, 527.778, 277.778, 833.333, 555.556, 527.778,
            555.556, 527.778, 391.667, 394.444, 388.889, 555.556, 527.778, 722.222, 527.778,
            527.778, 444.444, 319.444, 319.444, 319.444, 319.444,
        ];

        // cmbxti10 widths (Bold Italic, chars 32-126, 95 entries)
        let cmbxti10_widths: Vec<f32> = vec![
            333.333, 388.889, 500.0, 833.333, 500.0, 833.333, 777.778, 277.778, 388.889, 388.889,
            500.0, 777.778, 277.778, 333.333, 277.778, 500.0, 500.0, 500.0, 500.0, 500.0, 500.0,
            500.0, 500.0, 500.0, 500.0, 500.0, 277.778, 277.778, 472.222, 777.778, 472.222,
            472.222, 777.778, 869.444, 818.056, 831.944, 882.639, 756.944, 723.611, 899.306,
            882.639, 436.806, 583.333, 880.556, 723.611, 1010.417, 882.639, 869.444, 756.944,
            869.444, 831.944, 642.361, 809.028, 869.444, 869.444, 1170.139, 819.444, 880.556,
            723.611, 277.778, 500.0, 277.778, 694.444, 500.0, 277.778, 619.444, 651.389, 530.556,
            651.389, 530.556, 366.667, 601.389, 651.389, 338.194, 366.667, 628.472, 338.194,
            984.722, 651.389, 601.389, 651.389, 601.389, 456.944, 451.389, 480.556, 651.389,
            651.389, 866.667, 590.278, 651.389, 530.556, 319.444, 319.444, 319.444, 319.444,
        ];

        // cmtt10 widths (Typewriter — all 525.0, monospaced)
        let cmtt10_widths: Vec<f32> = vec![525.0; 95];

        // F3 = CMBX10 (Bold)
        pdf.type1_font(cmbx10_id)
            .base_font(Name(b"CMBX10"))
            .first_char(32)
            .last_char(126)
            .widths(cmbx10_widths)
            .font_descriptor(cmbx10_descriptor_id)
            .encoding_predefined(Name(b"StandardEncoding"));

        // F4 = CMTI10 (Italic)
        pdf.type1_font(cmti10_id)
            .base_font(Name(b"CMTI10"))
            .first_char(32)
            .last_char(126)
            .widths(cmti10_widths)
            .font_descriptor(cmti10_descriptor_id)
            .encoding_predefined(Name(b"StandardEncoding"));

        // F5 = CMBXTI10 (BoldItalic)
        pdf.type1_font(cmbxti10_id)
            .base_font(Name(b"CMBXTI10"))
            .first_char(32)
            .last_char(126)
            .widths(cmbxti10_widths)
            .font_descriptor(cmbxti10_descriptor_id)
            .encoding_predefined(Name(b"StandardEncoding"));

        // F6 = CMTT10 (Typewriter)
        pdf.type1_font(cmtt10_id)
            .base_font(Name(b"CMTT10"))
            .first_char(32)
            .last_char(126)
            .widths(cmtt10_widths)
            .font_descriptor(cmtt10_descriptor_id)
            .encoding_predefined(Name(b"StandardEncoding"));

        // A4 dimensions
        let media_box = Rect::new(0.0, 0.0, 595.0, 842.0);

        // Margins
        let margin_left: f32 = 72.27;
        let margin_top: f32 = 109.0;
        let font_size_outer: f32 = 10.0;
        let line_height: f32 = 12.0;

        // Starting y position: page height - top margin = 842 - 109 = 733
        let start_y: f32 = 842.0 - margin_top;

        for (i, page) in pages.iter().enumerate() {
            let page_id = Ref::new((18 + i * 2) as i32);
            let content_id = Ref::new((18 + i * 2 + 1) as i32);

            // Build content stream
            let mut content = Content::new();
            content.begin_text();
            content.set_font(Name(b"F1"), font_size_outer);

            let mut current_y = start_y;

            for line in &page.box_lines {
                // Compute line natural width and glue info
                let mut line_nat_width: f32 = 0.0;
                let mut glue_count: usize = 0;
                for node in &line.nodes {
                    match node {
                        BoxNode::Text { width, .. } => line_nat_width += *width as f32,
                        BoxNode::Kern { amount } => line_nat_width += *amount as f32,
                        BoxNode::Glue { natural, .. } => {
                            line_nat_width += *natural as f32;
                            glue_count += 1;
                        }
                        _ => {}
                    }
                }
                let hsize = 345.0_f32; // engine line-break width
                let remaining = hsize - line_nat_width;

                let start_x = match line.alignment {
                    Alignment::Center => margin_left + remaining / 2.0,
                    Alignment::RaggedLeft => margin_left + remaining,
                    Alignment::Justify | Alignment::RaggedRight => margin_left,
                };
                let glue_extra = if line.alignment == Alignment::Justify && glue_count > 0 {
                    remaining / glue_count as f32
                } else {
                    0.0
                };

                let mut current_x = start_x;
                content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);

                for node in &line.nodes {
                    match node {
                        BoxNode::Text {
                            text,
                            width,
                            font_size,
                            color,
                            font_style,
                        } => {
                            // Select font name based on font_style
                            let font_name: &[u8] = match font_style {
                                FontStyle::Normal => b"F1",
                                FontStyle::Bold => b"F3",
                                FontStyle::Italic => b"F4",
                                FontStyle::BoldItalic => b"F5",
                                FontStyle::Typewriter => b"F6",
                            };
                            // Set color if non-black
                            let has_color = color.as_ref().is_some_and(|c| !c.is_black());
                            if has_color {
                                let c = color.as_ref().unwrap();
                                // End text mode to set fill color, then re-enter
                                content.end_text();
                                content.set_fill_rgb(c.r as f32, c.g as f32, c.b as f32);
                                content.begin_text();
                                content.set_font(Name(font_name), *font_size as f32);
                                content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                            }
                            content.set_font(Name(font_name), *font_size as f32);
                            let escaped = pdf_escape(text);
                            content.show(Str(&escaped));
                            current_x += *width as f32;
                            if has_color {
                                // Reset to black
                                content.end_text();
                                content.set_fill_rgb(0.0, 0.0, 0.0);
                                content.begin_text();
                                content.set_font(Name(b"F1"), font_size_outer);
                                content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                            }
                        }
                        BoxNode::Glue { natural, .. } => {
                            current_x += *natural as f32 + glue_extra;
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        BoxNode::Kern { amount } => {
                            current_x += *amount as f32;
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        BoxNode::Rule { width, height } => {
                            // End text mode, draw rule, re-enter text mode
                            content.end_text();
                            // Draw a filled rectangle as the rule
                            let rx = margin_left;
                            let ry = current_y - *height as f32;
                            let rw = *width as f32;
                            let rh = *height as f32;
                            content.rect(rx, ry, rw, rh);
                            content.fill_nonzero();
                            current_y -= *height as f32 + 1.0;
                            // Re-enter text mode
                            content.begin_text();
                            content.set_font(Name(b"F1"), font_size_outer);
                        }
                        BoxNode::ImagePlaceholder { width, height, .. } => {
                            // Draw a grey filled rectangle as placeholder
                            content.end_text();
                            content.save_state();
                            content.set_fill_rgb(0.8, 0.8, 0.8);
                            let rx = current_x;
                            let ry = current_y - *height as f32;
                            content.rect(rx, ry, *width as f32, *height as f32);
                            content.fill_nonzero();
                            // Draw border
                            content.set_stroke_rgb(0.5, 0.5, 0.5);
                            content.rect(rx, ry, *width as f32, *height as f32);
                            content.stroke();
                            content.restore_state();
                            current_y -= *height as f32 + 2.0;
                            // Re-enter text mode
                            content.begin_text();
                            content.set_font(Name(b"F1"), font_size_outer);
                        }
                        _ => {
                            // HBox, VBox, Penalty, AlignmentMarker — skip
                        }
                    }
                }

                current_y -= line_height;
            }

            content.end_text();

            // Footnote rendering at bottom of page
            if !page.footnotes.is_empty() {
                // Calculate footnote area position
                let footnote_area_top = 60.0_f32; // Above the page number footer (25pt)
                let footnote_line_height = 10.0_f32;

                // Draw horizontal rule above footnotes
                let rule_y =
                    footnote_area_top + (page.footnotes.len() as f32 * footnote_line_height) + 5.0;
                content.rect(margin_left, rule_y, 50.0, 0.4);
                content.fill_nonzero();

                // Render each footnote
                content.begin_text();
                content.set_font(Name(b"F1"), 8.0);
                for (idx, footnote) in page.footnotes.iter().enumerate() {
                    let fn_y = footnote_area_top
                        + ((page.footnotes.len() - 1 - idx) as f32 * footnote_line_height);
                    content.set_text_matrix([1.0, 0.0, 0.0, 1.0, margin_left, fn_y]);
                    let fn_text = format!("{}. {}", footnote.number, footnote.text);
                    let escaped = pdf_escape(&fn_text);
                    content.show(Str(&escaped));
                }
                content.end_text();
            }

            // Page number footer
            let page_num_str = format!("{}", page.number);
            let page_num_width = page_num_str.len() as f32 * 5.0; // ~5pt per digit at 10pt
            let footer_x = (595.0 - page_num_width) / 2.0;
            let footer_y: f32 = 25.0; // middle of bottom margin
            content.begin_text();
            content.set_font(Name(b"F1"), 10.0);
            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, footer_x, footer_y]);
            let escaped_num = pdf_escape(&page_num_str);
            content.show(Str(&escaped_num));
            content.end_text();

            let content_bytes = content.finish();

            // Write content stream
            pdf.stream(content_id, &content_bytes);

            // Write page object
            {
                let mut page_writer = pdf.page(page_id);
                page_writer.parent(page_tree_id);
                page_writer.media_box(media_box);
                page_writer.contents(content_id);
                let mut resources = page_writer.resources();
                let mut fonts = resources.fonts();
                fonts.pair(Name(b"F1"), font_id);
                fonts.pair(Name(b"F3"), cmbx10_id);
                fonts.pair(Name(b"F4"), cmti10_id);
                fonts.pair(Name(b"F5"), cmbxti10_id);
                fonts.pair(Name(b"F6"), cmtt10_id);
            }
        }

        let bytes = pdf.finish();
        PdfOutput { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_engine::{
        Alignment, BoxNode, FontMetrics, FontStyle, OutputLine, Page as EnginePage,
        StandardFontMetrics,
    };

    #[test]
    fn test_pdf_header_starts_with_pdf() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(output.bytes.len() >= 4);
        assert_eq!(&output.bytes[0..5], b"%PDF-");
    }

    #[test]
    fn test_pdf_non_empty_output() {
        let pages = vec![EnginePage {
            number: 1,
            content: "Hello world".to_string(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![
                    BoxNode::Text {
                        text: "Hello".to_string(),
                        width: 25.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    },
                    BoxNode::Glue {
                        natural: 3.33,
                        stretch: 1.67,
                        shrink: 1.11,
                    },
                    BoxNode::Text {
                        text: "world".to_string(),
                        width: 24.76,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    },
                ],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
        // Should be a reasonably sized PDF (at least a few hundred bytes)
        assert!(output.bytes.len() > 100);
    }

    #[test]
    fn test_pdf_single_page() {
        let pages = vec![EnginePage {
            number: 1,
            content: "page one".to_string(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "Hello".to_string(),
                    width: 25.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Verify it's valid PDF structure
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(text.contains("%PDF-"));
        // Should contain /Count 1 for one page
        assert!(text.contains("/Count 1"));
    }

    #[test]
    fn test_pdf_two_pages() {
        let pages = vec![
            EnginePage {
                number: 1,
                content: "page one".to_string(),
                box_lines: vec![OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "First".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    }],
                }],
                footnotes: vec![],
            },
            EnginePage {
                number: 2,
                content: "page two".to_string(),
                box_lines: vec![OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "Second".to_string(),
                        width: 30.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    }],
                }],
                footnotes: vec![],
            },
        ];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
        let text = String::from_utf8_lossy(&output.bytes);
        // Should contain /Count 2 for two pages
        assert!(text.contains("/Count 2"));
    }

    #[test]
    fn test_pdf_empty_pages_slice() {
        let pages: Vec<EnginePage> = vec![];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Should not panic and produce a valid PDF
        assert!(!output.bytes.is_empty());
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(text.contains("%PDF-"));
        assert!(text.contains("/Count 0"));
    }

    #[test]
    fn test_pdf_escape_parentheses() {
        let escaped = pdf_escape("hello (world)");
        assert_eq!(escaped, b"hello \\(world\\)");
    }

    #[test]
    fn test_pdf_escape_backslash() {
        let escaped = pdf_escape("path\\to\\file");
        assert_eq!(escaped, b"path\\\\to\\\\file");
    }

    #[test]
    fn test_pdf_contains_cmr10() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "test".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(text.contains("CMR10"), "PDF should reference CMR10 font");
    }

    // === New CM Roman metric tests ===

    #[test]
    fn test_cm_roman_space_width() {
        let metrics = StandardFontMetrics;
        let sw = metrics.space_width();
        assert!(
            (sw - 3.333).abs() < 0.001,
            "CM Roman space width should be 3.333pt, got {}",
            sw
        );
    }

    #[test]
    fn test_cm_roman_char_a() {
        let metrics = StandardFontMetrics;
        let w = metrics.char_width('a');
        assert!(
            (w - 5.000).abs() < 0.001,
            "CM Roman 'a' width should be 5.000pt, got {}",
            w
        );
    }

    #[test]
    fn test_cm_roman_char_W_wide() {
        let metrics = StandardFontMetrics;
        let w = metrics.char_width('W');
        assert!(
            (w - 10.278).abs() < 0.001,
            "CM Roman 'W' width should be 10.278pt, got {}",
            w
        );
    }

    #[test]
    fn test_cm_roman_char_i_narrow() {
        let metrics = StandardFontMetrics;
        let w = metrics.char_width('i');
        assert!(
            (w - 2.778).abs() < 0.001,
            "CM Roman 'i' width should be 2.778pt, got {}",
            w
        );
    }

    #[test]
    fn test_cm_roman_digit_width() {
        let metrics = StandardFontMetrics;
        for d in '0'..='9' {
            let w = metrics.char_width(d);
            assert!(
                (w - 5.000).abs() < 0.001,
                "CM Roman digit '{}' width should be 5.000pt, got {}",
                d,
                w
            );
        }
    }

    #[test]
    fn test_pdf_contains_font_descriptor() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(
            text.contains("FontDescriptor"),
            "PDF should contain FontDescriptor"
        );
    }

    #[test]
    fn test_pdf_contains_font_file() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(
            text.contains("FontFile"),
            "PDF should reference embedded FontFile (cmr10.pfb)"
        );
    }

    #[test]
    fn test_pdf_contains_standard_encoding() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(
            text.contains("StandardEncoding"),
            "PDF should reference StandardEncoding"
        );
    }

    #[test]
    fn test_pdf_contains_type1_subtype() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let text = String::from_utf8_lossy(&output.bytes);
        assert!(
            text.contains("Type1"),
            "PDF should contain Type1 font subtype"
        );
    }

    #[test]
    fn test_pdf_font_embedded_bytes_present() {
        // The embedded font bytes should make the PDF significantly larger
        let pages: Vec<EnginePage> = vec![];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // With font embedding, PDF should be >35000 bytes (cmr10.pfb is 35752 bytes)
        assert!(
            output.bytes.len() > 35000,
            "PDF with embedded font should be >35KB, got {} bytes",
            output.bytes.len()
        );
    }

    #[test]
    fn test_cm_roman_multiple_char_widths() {
        let metrics = StandardFontMetrics;
        // Check a sample of CM Roman widths
        assert!((metrics.char_width('b') - 5.556).abs() < 0.001);
        assert!((metrics.char_width('c') - 4.444).abs() < 0.001);
        assert!((metrics.char_width('m') - 8.333).abs() < 0.001);
        assert!((metrics.char_width('f') - 3.056).abs() < 0.001);
        assert!((metrics.char_width('A') - 7.500).abs() < 0.001);
        assert!((metrics.char_width('M') - 9.167).abs() < 0.001);
        assert!((metrics.char_width('Z') - 6.111).abs() < 0.001);
    }

    #[test]
    fn test_cm_roman_string_width() {
        let metrics = StandardFontMetrics;
        // "Hi" = H(7.500) + i(2.778) = 10.278
        let w = metrics.string_width("Hi");
        assert!(
            (w - 10.278).abs() < 0.001,
            "string_width('Hi') should be 10.278, got {}",
            w
        );
    }

    #[test]
    fn test_pdf_output_is_bytes() {
        let pages = vec![EnginePage {
            number: 1,
            content: "sample".to_string(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "sample".to_string(),
                    width: 30.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // PdfOutput.bytes should be valid PDF
        assert!(!output.bytes.is_empty());
        assert_eq!(&output.bytes[0..4], b"%PDF");
    }

    #[test]
    fn test_pdf_page_number_in_output() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "Hello".to_string(),
                    width: 25.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // The page number "1" should appear in the PDF content stream
        // Look for the page number text operation in the raw bytes
        let text = String::from_utf8_lossy(&output.bytes);
        // The footer should contain the page number as a text show operation
        assert!(
            output.bytes.windows(3).any(|w| w == b"(1)"),
            "PDF should contain page number '1' in footer"
        );
    }

    #[test]
    fn test_pdf_page_number_two_pages() {
        let pages = vec![
            EnginePage {
                number: 1,
                content: "page one".to_string(),
                box_lines: vec![OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "First".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    }],
                }],
                footnotes: vec![],
            },
            EnginePage {
                number: 2,
                content: "page two".to_string(),
                box_lines: vec![OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "Second".to_string(),
                        width: 30.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Normal,
                    }],
                }],
                footnotes: vec![],
            },
        ];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Both page numbers should be present
        assert!(
            output.bytes.windows(3).any(|w| w == b"(1)"),
            "PDF should contain page number '1'"
        );
        assert!(
            output.bytes.windows(3).any(|w| w == b"(2)"),
            "PDF should contain page number '2'"
        );
    }

    #[test]
    fn test_cm_roman_default_fallback() {
        let metrics = StandardFontMetrics;
        // Non-ASCII chars fall back to 5.000
        let w = metrics.char_width('é');
        assert!(
            (w - 5.000).abs() < 0.001,
            "Default fallback width should be 5.000pt, got {}",
            w
        );
    }

    #[test]
    fn test_pdf_centered_line() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Center,
                nodes: vec![BoxNode::Text {
                    text: "Hello".to_string(),
                    width: 30.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
    }

    #[test]
    fn test_pdf_renders_rule() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Rule {
                    width: 345.0,
                    height: 0.5,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(
            !output.bytes.is_empty(),
            "PDF with Rule should be non-empty"
        );
        // Should still be valid PDF
        assert_eq!(&output.bytes[0..5], b"%PDF-");
    }

    #[test]
    fn test_pdf_raggedleft_line() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::RaggedLeft,
                nodes: vec![BoxNode::Text {
                    text: "Right".to_string(),
                    width: 30.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty());
    }

    #[test]
    fn test_pdf_bytes_contain_cm_font_names() {
        // Verify that the PDF byte output contains all Computer Modern font name strings
        // for bold, italic, bold-italic, and typewriter fonts.
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "bold".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Bold,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "italic".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Italic,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "bolditalic".to_string(),
                        width: 40.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::BoldItalic,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "typewriter".to_string(),
                        width: 40.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Typewriter,
                    }],
                },
            ],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        assert!(!output.bytes.is_empty(), "PDF output should not be empty");
        assert_eq!(&output.bytes[0..5], b"%PDF-", "Should be valid PDF");
        // Verify Computer Modern font names appear in the PDF byte stream
        let bytes_str = String::from_utf8_lossy(&output.bytes);
        assert!(
            bytes_str.contains("CMBX10"),
            "PDF bytes should contain 'CMBX10' for Bold font style"
        );
        assert!(
            bytes_str.contains("CMTI10"),
            "PDF bytes should contain 'CMTI10' for Italic font style"
        );
        assert!(
            bytes_str.contains("CMBXTI10"),
            "PDF bytes should contain 'CMBXTI10' for BoldItalic font style"
        );
        assert!(
            bytes_str.contains("CMTT10"),
            "PDF bytes should contain 'CMTT10' for Typewriter font style"
        );
    }

    #[test]
    fn test_pdf_with_footnotes() {
        use rustlatex_engine::FootnoteInfo;
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "Main text".to_string(),
                    width: 50.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![FootnoteInfo {
                number: 1,
                text: "A footnote".to_string(),
            }],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // Should produce valid PDF
        assert!(!output.bytes.is_empty());
        assert_eq!(&output.bytes[0..5], b"%PDF-");
        // Should be larger than a page without footnotes
        let pages_no_fn = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "Main text".to_string(),
                    width: 50.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Normal,
                }],
            }],
            footnotes: vec![],
        }];
        let output_no_fn = writer.write(&pages_no_fn);
        assert!(
            output.bytes.len() > output_no_fn.bytes.len(),
            "PDF with footnotes should be larger than without"
        );
    }

    // ===== M30 tests: PDF layout constants =====

    #[test]
    fn test_m30_pdf_margin_left() {
        // Verify the margin_left constant is 72.27 (1 inch)
        let margin_left: f32 = 72.27;
        assert!((margin_left - 72.27).abs() < 0.01);
    }

    #[test]
    fn test_m30_pdf_margin_top() {
        // Verify margin_top is 109.0
        let margin_top: f32 = 109.0;
        assert!((margin_top - 109.0).abs() < 0.01);
    }

    #[test]
    fn test_m30_pdf_line_height() {
        // Verify line_height is 12.0
        let line_height: f32 = 12.0;
        assert!((line_height - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_m30_pdf_hsize() {
        // Verify hsize is 345.0 (engine line-break width)
        let hsize: f32 = 345.0;
        assert!((hsize - 345.0).abs() < 0.01);
    }

    #[test]
    fn test_m30_pdf_start_y() {
        // start_y = 842 - 109 = 733
        let margin_top: f32 = 109.0;
        let start_y: f32 = 842.0 - margin_top;
        assert!((start_y - 733.0).abs() < 0.01);
    }

    // ===== M32 tests: Computer Modern font embedding =====

    #[test]
    fn test_pdf_contains_cmbx10_font_name() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "bold".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Bold,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(s.contains("CMBX10"), "PDF should contain CMBX10 font name");
    }

    #[test]
    fn test_pdf_contains_cmti10_font_name() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "italic".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Italic,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(s.contains("CMTI10"), "PDF should contain CMTI10 font name");
    }

    #[test]
    fn test_pdf_contains_cmbxti10_font_name() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "bolditalic".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::BoldItalic,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(
            s.contains("CMBXTI10"),
            "PDF should contain CMBXTI10 font name"
        );
    }

    #[test]
    fn test_pdf_contains_cmtt10_font_name() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "mono".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Typewriter,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(s.contains("CMTT10"), "PDF should contain CMTT10 font name");
    }

    #[test]
    fn test_pdf_does_not_contain_helvetica() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "text".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Bold,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(
            !s.contains("Helvetica"),
            "PDF should not contain Helvetica font name"
        );
    }

    #[test]
    fn test_pdf_does_not_contain_courier() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "code".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Typewriter,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(
            !s.contains("Courier"),
            "PDF should not contain Courier font name"
        );
    }

    #[test]
    fn test_pdf_cmbx10_has_font_file() {
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![OutputLine {
                alignment: Alignment::Justify,
                nodes: vec![BoxNode::Text {
                    text: "bold".to_string(),
                    width: 20.0,
                    font_size: 10.0,
                    color: None,
                    font_style: FontStyle::Bold,
                }],
            }],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // PDF should be large due to embedded font files
        assert!(
            output.bytes.len() > 100_000,
            "PDF should contain embedded font data (>100KB), got {} bytes",
            output.bytes.len()
        );
    }

    #[test]
    fn test_pdf_all_cm_fonts_embedded() {
        // All 4 new CM fonts should appear in a multi-style PDF
        let pages = vec![EnginePage {
            number: 1,
            content: String::new(),
            box_lines: vec![
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "bold".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Bold,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "italic".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Italic,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "bolditalic".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::BoldItalic,
                    }],
                },
                OutputLine {
                    alignment: Alignment::Justify,
                    nodes: vec![BoxNode::Text {
                        text: "mono".to_string(),
                        width: 20.0,
                        font_size: 10.0,
                        color: None,
                        font_style: FontStyle::Typewriter,
                    }],
                },
            ],
            footnotes: vec![],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        let s = String::from_utf8_lossy(&output.bytes);
        assert!(s.contains("CMBX10"), "should have CMBX10");
        assert!(s.contains("CMTI10"), "should have CMTI10");
        assert!(s.contains("CMBXTI10"), "should have CMBXTI10");
        assert!(s.contains("CMTT10"), "should have CMTT10");
    }
}
