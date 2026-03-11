//! `rustlatex-pdf` — PDF backend
//!
//! This crate takes the typeset pages produced by `rustlatex-engine` and
//! emits PDF output. It produces valid PDF files using the `pdf-writer` crate
//! with Computer Modern Roman 10pt (CM Roman) Type1 font embedded on A4 pages.

use pdf_writer::types::FontFlags;
use pdf_writer::{Content, Name, Pdf, Rect, Ref, Str};
use rustlatex_engine::{BoxNode, Page as EnginePage};

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
        // 4 = font dictionary (CMR10 Type1)
        // 5 = font descriptor
        // For each page i (0-indexed):
        //   6 + i*2     = page object
        //   6 + i*2 + 1 = content stream
        let catalog_id = Ref::new(1);
        let page_tree_id = Ref::new(2);
        let font_file_id = Ref::new(3);
        let font_id = Ref::new(4);
        let font_descriptor_id = Ref::new(5);

        let mut pdf = Pdf::new();

        // Document catalog
        pdf.catalog(catalog_id).pages(page_tree_id);

        // Collect page Refs
        let page_refs: Vec<Ref> = (0..page_count)
            .map(|i| Ref::new((6 + i * 2) as i32))
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

        // A4 dimensions
        let media_box = Rect::new(0.0, 0.0, 595.0, 842.0);

        // Margins
        let margin_left: f32 = 50.0;
        let margin_top: f32 = 50.0;
        let font_size: f32 = 10.0;
        let line_height: f32 = 14.0;

        // Starting y position: page height - top margin = 842 - 50 = 792
        let start_y: f32 = 842.0 - margin_top;

        for (i, page) in pages.iter().enumerate() {
            let page_id = Ref::new((6 + i * 2) as i32);
            let content_id = Ref::new((6 + i * 2 + 1) as i32);

            // Build content stream
            let mut content = Content::new();
            content.begin_text();
            content.set_font(Name(b"F1"), font_size);

            let mut current_y = start_y;

            for line in &page.box_lines {
                let mut current_x = margin_left;

                // Position at the start of this line
                content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);

                for node in line {
                    match node {
                        BoxNode::Text { text, width } => {
                            // Move to current_x position using Td relative positioning
                            // We already set the text matrix, so use show
                            let escaped = pdf_escape(text);
                            content.show(Str(&escaped));
                            current_x += *width as f32;
                        }
                        BoxNode::Glue { natural, .. } => {
                            // Advance x by natural width - emit a space
                            // Use text matrix repositioning for the next text node
                            current_x += *natural as f32;
                            // Re-position for the next text element
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        BoxNode::Kern { amount } => {
                            current_x += *amount as f32;
                            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, current_x, current_y]);
                        }
                        _ => {
                            // HBox, VBox, Penalty — skip for now
                        }
                    }
                }

                current_y -= line_height;
            }

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
                resources.fonts().pair(Name(b"F1"), font_id);
            }
        }

        let bytes = pdf.finish();
        PdfOutput { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlatex_engine::{BoxNode, FontMetrics, Page as EnginePage, StandardFontMetrics};

    #[test]
    fn test_pdf_header_starts_with_pdf() {
        let pages = vec![EnginePage {
            number: 1,
            content: "test".to_string(),
            box_lines: vec![],
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
            box_lines: vec![vec![
                BoxNode::Text {
                    text: "Hello".to_string(),
                    width: 25.0,
                },
                BoxNode::Glue {
                    natural: 3.33,
                    stretch: 1.67,
                    shrink: 1.11,
                },
                BoxNode::Text {
                    text: "world".to_string(),
                    width: 24.76,
                },
            ]],
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
            box_lines: vec![vec![BoxNode::Text {
                text: "Hello".to_string(),
                width: 25.0,
            }]],
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
                box_lines: vec![vec![BoxNode::Text {
                    text: "First".to_string(),
                    width: 20.0,
                }]],
            },
            EnginePage {
                number: 2,
                content: "page two".to_string(),
                box_lines: vec![vec![BoxNode::Text {
                    text: "Second".to_string(),
                    width: 30.0,
                }]],
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
            box_lines: vec![vec![BoxNode::Text {
                text: "test".to_string(),
                width: 20.0,
            }]],
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
            box_lines: vec![vec![BoxNode::Text {
                text: "sample".to_string(),
                width: 30.0,
            }]],
        }];
        let writer = PdfWriter::new();
        let output = writer.write(&pages);
        // PdfOutput.bytes should be valid PDF
        assert!(!output.bytes.is_empty());
        assert_eq!(&output.bytes[0..4], b"%PDF");
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
}
