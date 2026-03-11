//! `rustlatex-lexer` — LaTeX tokenizer
//!
//! This crate implements the first stage of the LaTeX compilation pipeline:
//! tokenizing raw LaTeX source into a stream of tokens. It handles TeX's
//! category code mechanism, control sequences, grouping, math mode, and more.

/// A token produced by the LaTeX lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A control sequence such as `\section` or `\textbf`.
    ControlSequence(String),
    /// A single character with its category.
    Character(char, Category),
    /// End of input.
    EndOfInput,
    /// A parameter token: `#1`–`#9` yields `Parameter(1)`–`Parameter(9)`,
    /// `##` yields `Parameter(0)`.
    Parameter(u8),
    /// An active character (catcode 13), e.g. `~`.
    Active(char),
    /// Implicit paragraph break (double newline / blank line).
    Par,
    /// A space token (catcode 10 or single end-of-line).
    Space,
}

/// TeX category codes assigned to characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Category 0: escape character (usually `\`)
    Escape,
    /// Category 1: begin group (usually `{`)
    BeginGroup,
    /// Category 2: end group (usually `}`)
    EndGroup,
    /// Category 3: math shift (usually `$`)
    MathShift,
    /// Category 4: alignment tab (usually `&`)
    AlignmentTab,
    /// Category 5: end of line (usually `\n`)
    EndOfLine,
    /// Category 6: parameter (usually `#`)
    Parameter,
    /// Category 7: superscript (usually `^`)
    Superscript,
    /// Category 8: subscript (usually `_`)
    Subscript,
    /// Category 9: ignored character
    Ignored,
    /// Category 10: space (usually space and tab)
    Space,
    /// Category 11: letter (a-z, A-Z)
    Letter,
    /// Category 12: other (anything else)
    Other,
    /// Category 13: active character (usually `~`)
    Active,
    /// Category 14: comment (usually `%`)
    Comment,
    /// Category 15: invalid character
    Invalid,
}

/// A table mapping each of the 256 byte values to a [`Category`].
///
/// Initialized with TeX's default category code assignments.
pub struct CatcodeTable {
    table: [Category; 256],
}

impl CatcodeTable {
    /// Create a new catcode table with TeX default assignments.
    pub fn new() -> Self {
        let mut t = CatcodeTable {
            table: [Category::Other; 256],
        };
        // Set TeX defaults for ASCII range
        t.set('\\', Category::Escape);
        t.set('{', Category::BeginGroup);
        t.set('}', Category::EndGroup);
        t.set('$', Category::MathShift);
        t.set('&', Category::AlignmentTab);
        t.set('\n', Category::EndOfLine);
        t.set('#', Category::Parameter);
        t.set('^', Category::Superscript);
        t.set('_', Category::Subscript);
        t.set('\x00', Category::Ignored);
        t.set(' ', Category::Space);
        t.set('\t', Category::Space);
        for c in b'a'..=b'z' {
            t.set(c as char, Category::Letter);
        }
        for c in b'A'..=b'Z' {
            t.set(c as char, Category::Letter);
        }
        t.set('~', Category::Active);
        t.set('%', Category::Comment);
        t.set('\x7f', Category::Invalid);
        t
    }

    /// Look up the category for a character.
    pub fn get(&self, ch: char) -> Category {
        if (ch as usize) < 256 {
            self.table[ch as usize]
        } else {
            Category::Letter // Unicode letters default to Letter
        }
    }

    /// Set the category for a character.
    pub fn set(&mut self, ch: char, cat: Category) {
        if (ch as usize) < 256 {
            self.table[ch as usize] = cat;
        }
    }
}

impl Default for CatcodeTable {
    fn default() -> Self {
        Self::new()
    }
}

/// The lexer tokenizes a LaTeX source string into a sequence of [`Token`]s.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    /// The catcode table consulted for every character.
    pub catcodes: CatcodeTable,
    /// Whether we are logically at the start of a line (for `\par` detection).
    at_line_start: bool,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            catcodes: CatcodeTable::new(),
            at_line_start: true,
        }
    }

    /// Return the default TeX category for a character (static, backward-compat).
    pub fn category(ch: char) -> Category {
        match ch {
            '\\' => Category::Escape,
            '{' => Category::BeginGroup,
            '}' => Category::EndGroup,
            '$' => Category::MathShift,
            '&' => Category::AlignmentTab,
            '\n' => Category::EndOfLine,
            '#' => Category::Parameter,
            '^' => Category::Superscript,
            '_' => Category::Subscript,
            '\x00' => Category::Ignored,
            ' ' | '\t' => Category::Space,
            'a'..='z' | 'A'..='Z' => Category::Letter,
            '~' => Category::Active,
            '%' => Category::Comment,
            '\x7f' => Category::Invalid,
            _ => Category::Other,
        }
    }

    /// Instance method: look up a character's category in the current table.
    pub fn cat(&self, ch: char) -> Category {
        self.catcodes.get(ch)
    }

    /// Peek at the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Advance and return the current character.
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    /// Skip characters while they have the Space catcode.
    fn skip_spaces(&mut self) {
        while let Some(c) = self.peek() {
            if self.cat(c) == Category::Space {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read the next token from the input.
    pub fn next_token(&mut self) -> Token {
        loop {
            let ch = match self.peek() {
                None => return Token::EndOfInput,
                Some(c) => c,
            };

            let cat = self.cat(ch);

            match cat {
                Category::Ignored => {
                    self.advance();
                    // loop again
                }

                Category::Invalid => {
                    self.advance();
                    // ignore invalid characters, loop again
                }

                Category::Comment => {
                    // Skip past '%'
                    self.advance();
                    // Skip until '\n' or EOF
                    while let Some(c) = self.peek() {
                        if self.cat(c) == Category::EndOfLine {
                            break;
                        }
                        self.advance();
                    }
                    // If we stopped on '\n', consume it
                    if let Some(c) = self.peek() {
                        if self.cat(c) == Category::EndOfLine {
                            self.advance();
                        }
                    }
                    // Skip leading spaces on the next line
                    self.skip_spaces();
                    // loop to get next token
                }

                Category::Space => {
                    self.advance();
                    // Consume all consecutive spaces
                    self.skip_spaces();
                    self.at_line_start = false;
                    return Token::Space;
                }

                Category::EndOfLine => {
                    self.advance();
                    if self.at_line_start {
                        // Blank line → \par
                        // Consume any further blank lines / spaces
                        self.skip_blank_lines();
                        return Token::Par;
                    }
                    // Single newline → space
                    self.at_line_start = true;
                    // Skip any trailing spaces on the next line
                    self.skip_spaces();
                    return Token::Space;
                }

                Category::Escape => {
                    self.advance(); // consume '\'
                    self.at_line_start = false;
                    match self.peek() {
                        None => return Token::ControlSequence(String::new()),
                        Some(c) if self.cat(c) == Category::Letter => {
                            let mut name = String::new();
                            while let Some(c2) = self.peek() {
                                if self.cat(c2) == Category::Letter {
                                    name.push(c2);
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            // Skip trailing spaces after a word control sequence
                            self.skip_spaces();
                            return Token::ControlSequence(name);
                        }
                        Some(c) => {
                            self.advance();
                            return Token::ControlSequence(c.to_string());
                        }
                    }
                }

                Category::Parameter => {
                    self.advance(); // consume '#'
                    self.at_line_start = false;
                    match self.peek() {
                        Some('#') if self.cat('#') == Category::Parameter => {
                            self.advance();
                            return Token::Parameter(0);
                        }
                        Some(c) if c.is_ascii_digit() && c != '0' => {
                            self.advance();
                            return Token::Parameter(c as u8 - b'0');
                        }
                        _ => return Token::Parameter(0),
                    }
                }

                Category::Active => {
                    self.advance();
                    self.at_line_start = false;
                    return Token::Active(ch);
                }

                Category::BeginGroup
                | Category::EndGroup
                | Category::MathShift
                | Category::AlignmentTab
                | Category::Superscript
                | Category::Subscript
                | Category::Letter
                | Category::Other => {
                    self.advance();
                    self.at_line_start = false;
                    return Token::Character(ch, cat);
                }
            }
        }
    }

    /// Skip blank lines (newlines possibly separated by spaces).
    fn skip_blank_lines(&mut self) {
        loop {
            match self.peek() {
                Some(c) if self.cat(c) == Category::Space => {
                    self.advance();
                }
                Some(c) if self.cat(c) == Category::EndOfLine => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Collect all tokens into a vector.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::EndOfInput {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_sequence() {
        let mut lexer = Lexer::new(r"\hello world");
        assert_eq!(
            lexer.next_token(),
            Token::ControlSequence("hello".to_string())
        );
        // trailing spaces after \hello are consumed; next is 'w'
        assert_eq!(lexer.next_token(), Token::Character('w', Category::Letter));
    }

    #[test]
    fn test_character_categories() {
        assert_eq!(Lexer::category('{'), Category::BeginGroup);
        assert_eq!(Lexer::category('}'), Category::EndGroup);
        assert_eq!(Lexer::category('$'), Category::MathShift);
        assert_eq!(Lexer::category('a'), Category::Letter);
        assert_eq!(Lexer::category(' '), Category::Space);
    }

    #[test]
    fn test_comment_skipping() {
        // comment eats the newline and leading spaces on next line
        let mut lexer = Lexer::new("a% this is a comment\n  b");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        // '%' starts comment; comment consumes rest of line + newline + leading spaces
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_tokenize_simple_document() {
        let src = r"\documentclass{article}";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens[0],
            Token::ControlSequence("documentclass".to_string())
        );
        assert_eq!(tokens[1], Token::Character('{', Category::BeginGroup));
    }

    #[test]
    fn test_end_of_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    // ------ New tests for production-quality lexer ------

    #[test]
    fn test_parameter_1_through_9() {
        for digit in 1u8..=9 {
            let src = format!("#{}", digit);
            let mut lexer = Lexer::new(&src);
            assert_eq!(lexer.next_token(), Token::Parameter(digit));
            assert_eq!(lexer.next_token(), Token::EndOfInput);
        }
    }

    #[test]
    fn test_double_hash() {
        let mut lexer = Lexer::new("##");
        assert_eq!(lexer.next_token(), Token::Parameter(0));
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_active_tilde() {
        let mut lexer = Lexer::new("~");
        assert_eq!(lexer.next_token(), Token::Active('~'));
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_double_newline_par() {
        let mut lexer = Lexer::new("a\n\nb");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        // first '\n' is a space (single newline mid-line)
        assert_eq!(lexer.next_token(), Token::Space);
        // at_line_start is true after newline-space; next '\n' at line start → Par
        assert_eq!(lexer.next_token(), Token::Par);
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_single_newline_becomes_space() {
        let mut lexer = Lexer::new("a\nb");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Space);
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_control_word_eats_trailing_spaces() {
        let mut lexer = Lexer::new(r"\hello   world");
        assert_eq!(
            lexer.next_token(),
            Token::ControlSequence("hello".to_string())
        );
        // trailing spaces consumed by control word
        assert_eq!(lexer.next_token(), Token::Character('w', Category::Letter));
    }

    #[test]
    fn test_single_char_control_sequence() {
        let mut lexer = Lexer::new(r"\.");
        assert_eq!(lexer.next_token(), Token::ControlSequence(".".to_string()));
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_backslash_space_control_sequence() {
        let mut lexer = Lexer::new(r"\ ");
        assert_eq!(lexer.next_token(), Token::ControlSequence(" ".to_string()));
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_empty_group() {
        let mut lexer = Lexer::new("{}");
        assert_eq!(
            lexer.next_token(),
            Token::Character('{', Category::BeginGroup)
        );
        assert_eq!(
            lexer.next_token(),
            Token::Character('}', Category::EndGroup)
        );
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_group_with_content() {
        let mut lexer = Lexer::new("{hello}");
        assert_eq!(
            lexer.next_token(),
            Token::Character('{', Category::BeginGroup)
        );
        assert_eq!(lexer.next_token(), Token::Character('h', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Character('e', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Character('l', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Character('l', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Character('o', Category::Letter));
        assert_eq!(
            lexer.next_token(),
            Token::Character('}', Category::EndGroup)
        );
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_space_token() {
        let mut lexer = Lexer::new("a   b");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Space);
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_catcode_table_defaults() {
        let t = CatcodeTable::new();
        assert_eq!(t.get('\\'), Category::Escape);
        assert_eq!(t.get('{'), Category::BeginGroup);
        assert_eq!(t.get('}'), Category::EndGroup);
        assert_eq!(t.get('$'), Category::MathShift);
        assert_eq!(t.get('&'), Category::AlignmentTab);
        assert_eq!(t.get('\n'), Category::EndOfLine);
        assert_eq!(t.get('#'), Category::Parameter);
        assert_eq!(t.get('^'), Category::Superscript);
        assert_eq!(t.get('_'), Category::Subscript);
        assert_eq!(t.get('\x00'), Category::Ignored);
        assert_eq!(t.get(' '), Category::Space);
        assert_eq!(t.get('\t'), Category::Space);
        assert_eq!(t.get('a'), Category::Letter);
        assert_eq!(t.get('Z'), Category::Letter);
        assert_eq!(t.get('~'), Category::Active);
        assert_eq!(t.get('%'), Category::Comment);
        assert_eq!(t.get('\x7f'), Category::Invalid);
        assert_eq!(t.get('1'), Category::Other);
    }

    #[test]
    fn test_catcode_table_set() {
        let mut t = CatcodeTable::new();
        t.set('X', Category::Escape);
        assert_eq!(t.get('X'), Category::Escape);
    }

    #[test]
    fn test_ignored_character() {
        let mut lexer = Lexer::new("a\x00b");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        // null byte is ignored
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_invalid_character_skipped() {
        let mut lexer = Lexer::new("a\x7fb");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        // DEL is invalid, skipped
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_cat_instance_method() {
        let lexer = Lexer::new("");
        assert_eq!(lexer.cat('a'), Category::Letter);
        assert_eq!(lexer.cat('{'), Category::BeginGroup);
    }

    #[test]
    fn test_empty_control_sequence_at_eof() {
        let mut lexer = Lexer::new("\\");
        assert_eq!(lexer.next_token(), Token::ControlSequence(String::new()));
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_comment_at_end_of_input() {
        let mut lexer = Lexer::new("a% comment");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        // comment at EOF
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }

    #[test]
    fn test_par_from_leading_newline() {
        // file starts with blank line(s) → Par
        let mut lexer = Lexer::new("\n\na");
        // at_line_start is true, first '\n' at line start → Par... actually
        // first \n at line_start → the logic should be:
        // at_line_start=true, see EndOfLine -> skip_blank_lines -> Par
        assert_eq!(lexer.next_token(), Token::Par);
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
    }

    #[test]
    fn test_math_shift() {
        let mut lexer = Lexer::new("$x$");
        assert_eq!(
            lexer.next_token(),
            Token::Character('$', Category::MathShift)
        );
        assert_eq!(lexer.next_token(), Token::Character('x', Category::Letter));
        assert_eq!(
            lexer.next_token(),
            Token::Character('$', Category::MathShift)
        );
    }

    #[test]
    fn test_subscript_superscript() {
        let mut lexer = Lexer::new("a_1^2");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        assert_eq!(
            lexer.next_token(),
            Token::Character('_', Category::Subscript)
        );
        assert_eq!(lexer.next_token(), Token::Character('1', Category::Other));
        assert_eq!(
            lexer.next_token(),
            Token::Character('^', Category::Superscript)
        );
        assert_eq!(lexer.next_token(), Token::Character('2', Category::Other));
    }

    #[test]
    fn test_alignment_tab() {
        let mut lexer = Lexer::new("a&b");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        assert_eq!(
            lexer.next_token(),
            Token::Character('&', Category::AlignmentTab)
        );
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_tokenize_full() {
        let mut lexer = Lexer::new(r"\cmd{x}");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::ControlSequence("cmd".to_string()));
        assert_eq!(tokens[1], Token::Character('{', Category::BeginGroup));
        assert_eq!(tokens[2], Token::Character('x', Category::Letter));
        assert_eq!(tokens[3], Token::Character('}', Category::EndGroup));
        assert_eq!(tokens[4], Token::EndOfInput);
    }
}
