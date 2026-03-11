//! `rustlatex` — RustLaTex command-line interface
//!
//! Usage: rustlatex <file.tex>
//!
//! Compiles a LaTeX source file through the RustLaTex pipeline:
//! 1. Lexer: tokenize the source
//! 2. Parser: build an AST
//! 3. Engine: typeset into pages
//! 4. PDF: emit PDF output

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use rustlatex_engine::Engine;
use rustlatex_parser::Parser;
use rustlatex_pdf::PdfWriter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: rustlatex <file.tex>");
        eprintln!("Error: no input file specified.");
        process::exit(1);
    }

    let input_path = &args[1];

    // Read the input file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: cannot read '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    println!("Compiling {}...", input_path);
    println!();
    println!("--- Source ({} bytes) ---", source.len());
    println!("{}", source);
    println!("--- End of source ---");
    println!();

    // Stage 1: Lexer
    println!("[1/3] Tokenizing...");
    use rustlatex_lexer::Lexer;
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    // Exclude EndOfInput token from count
    let token_count = tokens
        .iter()
        .filter(|t| **t != rustlatex_lexer::Token::EndOfInput)
        .count();
    println!("      {} token(s) produced.", token_count);

    // Stage 2: Parser
    println!("[2/3] Parsing...");
    let mut parser = Parser::new(&source);
    let ast = parser.parse();
    println!("      AST root: {:?}", ast);

    // Stage 3: Engine + PDF
    println!("[3/3] Typesetting and generating PDF...");
    let engine = Engine::new(ast);
    let pages = engine.typeset();
    let writer = PdfWriter::new();
    let pdf = writer.write(&pages);
    println!("      {} page(s) typeset.", pages.len());

    // Derive output filename: replace .tex extension with .pdf, or append .pdf
    let input = Path::new(input_path);
    let basename = input.file_stem().unwrap_or_else(|| input.as_ref());
    let pdf_filename = format!("{}.pdf", basename.to_string_lossy());

    // Write PDF bytes to file
    match fs::write(&pdf_filename, &pdf.bytes) {
        Ok(()) => {
            println!();
            println!("PDF written to {}", pdf_filename);
        }
        Err(e) => {
            eprintln!("Error: cannot write '{}': {}", pdf_filename, e);
            process::exit(1);
        }
    }

    process::exit(0);
}
